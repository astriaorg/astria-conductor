use color_eyre::eyre::{eyre, Result};
use sequencer_relayer::proto::SequencerMsg;
use sequencer_relayer::sequencer_block::{
    cosmos_tx_body_to_sequencer_msgs, get_namespace, parse_cosmos_tx, Namespace, SequencerBlock,
};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task,
};

use crate::config::Config;
use crate::execution_client::ExecutionRpcClient;

pub(crate) type JoinHandle = task::JoinHandle<Result<()>>;

/// The channel for sending commands to the executor task.
pub(crate) type Sender = UnboundedSender<ExecutorCommand>;
/// The channel the executor task uses to listen for commands.
type Receiver = UnboundedReceiver<ExecutorCommand>;

/// spawns a executor task and returns a tuple with the task's join handle
/// and the channel for sending commands to this executor
pub(crate) async fn spawn(conf: &Config) -> Result<(JoinHandle, Sender)> {
    log::info!("Spawning executor task.");
    let (mut executor, executor_tx) = Executor::new(
        &conf.execution_rpc_url,
        get_namespace(conf.chain_id.as_bytes()),
    )
    .await?;
    let join_handle = task::spawn(async move { executor.run().await });
    log::info!("Spawned executor task.");
    Ok((join_handle, executor_tx))
}

#[derive(Debug)]
pub(crate) enum ExecutorCommand {
    /// Command for when a block is received
    BlockReceived {
        block: Box<SequencerBlock>,
    },

    Shutdown,
}

//#[allow(dead_code)] // TODO - remove after developing
struct Executor {
    /// Channel on which executor commands are received.
    cmd_rx: Receiver,
    /// The execution rpc client that we use to send messages to the execution service
    execution_rpc_client: ExecutionRpcClient,
    /// Namespace ID
    namespace: Namespace,
}

impl Executor {
    /// Creates a new Executor instance and returns a command sender and an alert receiver.
    async fn new(rpc_address: &str, namespace: Namespace) -> Result<(Self, Sender)> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let execution_rpc_client = ExecutionRpcClient::new(rpc_address).await?;
        Ok((
            Self {
                cmd_rx,
                execution_rpc_client,
                namespace,
            },
            cmd_tx,
        ))
    }

    async fn run(&mut self) -> Result<()> {
        log::info!("Starting executor event loop.");

        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                ExecutorCommand::BlockReceived { block } => {
                    log::info!("ExecutorCommand::BlockReceived {:#?}", block);
                    self.execute_block(*block).await?;
                }
                ExecutorCommand::Shutdown => {
                    log::info!("Shutting down executor event loop.");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Uses RPC to send block to execution service
    async fn execute_block(&mut self, block: SequencerBlock) -> Result<()> {
        let header = Header {
            block_hash: block.block_hash.0,
        };

        // get transactions for our namespace
        let Some(txs) = block.rollup_txs.get(&self.namespace) else {
            return Err(eyre!("sequencer block did not contains txs for namespace"));
        };

        // parse cosmos sequencer transactions into rollup transactions
        // by converting them to SequencerMsgs and extracting the `data` field
        let txs = txs
            .iter()
            .filter_map(|tx| {
                let body = parse_cosmos_tx(&tx.transaction).ok()?;
                let msgs: Vec<SequencerMsg> = cosmos_tx_body_to_sequencer_msgs(body).ok()?;
                if msgs.len() > 1 {
                    // this should not happen and is a bug in the sequencer relayer
                    return None;
                }
                let Some(msg) = msgs.first() else {
                return None;
            };
                Some(msg.data.clone())
            })
            .collect::<Vec<_>>();

        self.execution_rpc_client
            .call_do_block(header.to_bytes()?, txs)
            .await?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Header {
    block_hash: Vec<u8>,
}

impl Header {
    fn to_bytes(&self) -> Result<Vec<u8>> {
        // TODO: don't use json, use our own serializer (or protobuf for now?)
        let string = serde_json::to_string(self).map_err(|e| eyre!(e))?;
        Ok(string.into_bytes())
    }
}
