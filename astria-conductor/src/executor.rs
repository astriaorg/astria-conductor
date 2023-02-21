use rs_cnc::NamespacedDataResponse;
use tokio::{
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
    task,
};

use crate::conf::Conf;
use crate::{driver, error::*};

pub(crate) type JoinHandle = task::JoinHandle<Result<()>>;

/// The channel for sending commands to the executor task.
pub(crate) type Sender = UnboundedSender<ExecutorCommand>;
/// The channel the executor task uses to listen for commands.
type Receiver = UnboundedReceiver<ExecutorCommand>;

/// spawns a executor task and returns a tuple with the task's join handle
/// and the channel for sending commands to this executor
pub(crate) fn spawn(conf: &Conf, driver_tx: driver::Sender) -> Result<(JoinHandle, Sender)> {
    log::info!("Spawning executor task.");
    let (mut executor, executor_tx) = Executor::new(conf, driver_tx)?;
    let join_handle = task::spawn(async move { executor.run().await });
    log::info!("Spawned executor task.");
    Ok((join_handle, executor_tx))
}

#[allow(dead_code)] // TODO - remove after developing
#[derive(Debug)]
pub(crate) enum ExecutorCommand {
    /// Command for when a block is received
    BlockReceived {
        // FIXME - this will probably not be a NamespacedDataResponse ultimately
        block: NamespacedDataResponse,
    },

    Shutdown,
}

#[allow(dead_code)] // TODO - remove after developing
struct Executor {
    /// Channel on which executor commands are received.
    cmd_rx: Receiver,
    /// Channel on which the executor sends commands to the driver.
    driver_tx: driver::Sender,
}

impl Executor {
    /// Creates a new Executor instance and returns a command sender and an alert receiver.
    fn new(_conf: &Conf, driver_tx: driver::Sender) -> Result<(Self, Sender)> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        Ok((Self { cmd_rx, driver_tx }, cmd_tx))
    }

    async fn run(&mut self) -> Result<()> {
        log::info!("Starting executor event loop.");

        while let Some(cmd) = self.cmd_rx.recv().await {
            match cmd {
                ExecutorCommand::BlockReceived { block } => {
                    log::info!("ExecutorCommand::BlockReceived {:#?}", block);
                    self.execute_block(block).await?;
                }
                ExecutorCommand::Shutdown => {
                    log::info!("Shutting down executor event loop.");
                    break;
                }
            }
        }

        Ok(())
    }

    /// Uses abci to submit blocks to an evm
    async fn execute_block(&mut self, _block: NamespacedDataResponse) -> Result<()> {
        Ok(())
    }
}