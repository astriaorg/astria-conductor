use color_eyre::eyre::Result;
use rs_cnc::{CelestiaNodeClient, NamespacedDataResponse};
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task,
};

use crate::config::Config;
use crate::{driver, executor};

pub(crate) type JoinHandle = task::JoinHandle<Result<()>>;

/// The channel for sending commands to the reader task.
pub(crate) type Sender = UnboundedSender<ReaderCommand>;
/// The channel the reader task uses to listen for commands.
type Receiver = UnboundedReceiver<ReaderCommand>;

/// spawns a reader task and returns a tuple with the task's join handle
/// and the channel for sending commands to this reader
pub(crate) fn spawn(
    conf: &Config,
    driver_tx: driver::Sender,
    executor_tx: executor::Sender,
) -> Result<(JoinHandle, Sender)> {
    log::info!("Spawning reader task.");
    let (mut reader, reader_tx) = Reader::new(conf, driver_tx, executor_tx)?;
    let join_handle = task::spawn(async move { reader.run().await });
    log::info!("Spawned reader task.");
    Ok((join_handle, reader_tx))
}

#[derive(Debug)]
#[allow(dead_code)] // TODO - remove after developing
pub(crate) enum ReaderCommand {
    /// Get new blocks
    GetNewBlocks,

    Shutdown,
}

#[allow(dead_code)] // TODO - remove after developing
struct Reader {
    /// Channel on which reader commands are sent.
    cmd_tx: Sender,
    /// Channel on which reader commands are received.
    cmd_rx: Receiver,
    /// Channel on which the reader sends commands to the driver.
    driver_tx: driver::Sender,

    /// The channel used to send messages to the executor task.
    executor_tx: executor::Sender,

    /// The client used to communicate with Celestia.
    celestia_node_client: CelestiaNodeClient,

    /// Namespace ID
    namespace_id: String,

    /// Keep track of the last block height fetched
    last_block_height: u64,
}

impl Reader {
    /// Creates a new Reader instance and returns a command sender and an alert receiver.
    fn new(
        conf: &Config,
        driver_tx: driver::Sender,
        executor_tx: executor::Sender,
    ) -> Result<(Self, Sender)> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let celestia_node_client = CelestiaNodeClient::new(conf.celestia_node_url.to_owned())?;
        Ok((
            Self {
                cmd_tx: cmd_tx.clone(),
                cmd_rx,
                driver_tx,
                executor_tx,
                celestia_node_client,
                namespace_id: conf.namespace_id.to_owned(),
                last_block_height: 0,
            },
            cmd_tx,
        ))
    }

    async fn run(&mut self) -> Result<()> {
        log::info!("Starting reader event loop.");

        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                ReaderCommand::GetNewBlocks => {
                    self.get_new_blocks().await?;
                }
                ReaderCommand::Shutdown => {
                    log::info!("Shutting down reader event loop.");
                    break;
                }
            }
        }

        Ok(())
    }

    /// This function is responsible for fetching all the latest blocks
    async fn get_new_blocks(&mut self) -> Result<()> {
        log::info!("ReaderCommand::GetNewBlocks");

        // get most recent block
        let res = self
            .celestia_node_client
            // NOTE - requesting w/ height of 0 gives us the last block. this isn't documented.
            .namespaced_data(&self.namespace_id, 0)
            .await;

        match res {
            Ok(namespaced_data) => {
                if let Some(height) = namespaced_data.height {
                    // get blocks between current height and last height received and send to executor
                    for h in (self.last_block_height + 1)..(height) {
                        let block = self.get_block(h).await?;
                        self.process_block(block).await?;
                    }
                    // process the most recent block, which is actually the first one we requested above
                    self.process_block(namespaced_data).await?;
                }
            }
            Err(e) => {
                // just log the error for now.
                // any blocks that weren't fetched will be handled in the next cycle
                log::error!("{}", e.to_string());
            }
        }

        Ok(())
    }

    /// Gets an individual block
    async fn get_block(&mut self, height: u64) -> Result<NamespacedDataResponse> {
        let res = self
            .celestia_node_client
            .namespaced_data(&self.namespace_id, height)
            .await?;

        Ok(res)
    }

    /// Processes an individual block
    async fn process_block(&mut self, block: NamespacedDataResponse) -> Result<()> {
        self.last_block_height = block.height.unwrap();
        self.executor_tx
            .send(executor::ExecutorCommand::BlockReceived { block })?;

        Ok(())
    }
}
