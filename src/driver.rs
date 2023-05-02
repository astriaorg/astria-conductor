//! The driver is the top-level coordinator that runs and manages all the components
//! necessary for this reader/validator.

use color_eyre::eyre::{eyre, Result};
use futures::{
    future::{poll_fn, FutureExt},
    StreamExt,
};
use log::{debug, info};
use sequencer_relayer::sequencer_block::SequencerBlock;
use tokio::{
    select,
    sync::mpsc::{self, UnboundedReceiver, UnboundedSender},
};

use std::pin::Pin;
use std::sync::Mutex;

use crate::alert::Alert;
use crate::executor::ExecutorCommand;
use crate::network::{Event as NetworkEvent, GossipNetwork};
use crate::reader::ReaderCommand;
use crate::{alert::AlertSender, config::Config, executor, reader};

/// The channel through which the user can send commands to the driver.
pub(crate) type Sender = UnboundedSender<DriverCommand>;
/// The channel on which the driver listens for commands from the user.
pub(crate) type Receiver = UnboundedReceiver<DriverCommand>;

/// The type of commands that the driver can receive.
#[derive(Debug)]
pub enum DriverCommand {
    /// Get new blocks
    GetNewBlocks,
    /// Gracefully shuts down the driver and its components.
    Shutdown,
}

pub struct Driver {
    pub cmd_tx: Sender,

    /// The channel on which other components in the driver sends the driver messages.
    cmd_rx: Receiver,

    /// The channel used to send messages to the reader task.
    reader_tx: reader::Sender,
    reader_join_handle: reader::JoinHandle,

    /// The channel used to send messages to the executor task.
    executor_tx: executor::Sender,
    executor_join_handle: executor::JoinHandle,

    network: GossipNetwork,

    alert_tx: AlertSender,

    is_shutdown: Mutex<bool>,
}

impl Driver {
    pub async fn new(conf: Config, alert_tx: AlertSender) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (executor_join_handle, executor_tx) = executor::spawn(&conf, alert_tx.clone()).await?;
        let (reader_join_handle, reader_tx) = reader::spawn(&conf, executor_tx.clone()).await?;

        Ok(Self {
            cmd_tx: cmd_tx.clone(),
            cmd_rx,
            reader_tx,
            reader_join_handle,
            executor_tx,
            executor_join_handle,
            network: GossipNetwork::new()?,
            alert_tx,
            is_shutdown: Mutex::new(false),
        })
    }

    /// Runs the Driver event loop.
    pub async fn run(&mut self) -> Result<()> {
        info!("Starting driver event loop.");
        loop {
            select! {
                res = self.network.0.next() => {
                    if let Some(res) = res {
                        self.handle_network_event(res)?;
                    }
                },
                cmd = self.cmd_rx.recv() => {
                    if let Some(cmd) = cmd {
                        self.handle_driver_command(cmd)?;
                    } else {
                        info!("Driver command channel closed.");
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_network_event(&self, event: NetworkEvent) -> Result<()> {
        match event {
            NetworkEvent::NewListenAddr(addr) => {
                info!("listening on {}", addr);
            }
            NetworkEvent::Message(msg) => {
                debug!("received gossip message: {:?}", msg);
                let block = SequencerBlock::from_bytes(&msg.data)?;
                self.executor_tx
                    .send(ExecutorCommand::BlockReceivedGossip {
                        block: Box::new(block),
                    })?;
            }
            _ => debug!("received network event: {:?}", event),
        }

        Ok(())
    }

    fn handle_driver_command(&mut self, cmd: DriverCommand) -> Result<()> {
        // TODO: these are kind of janky, we might want to move to a polling-based architecture
        if let Some(Ok(res)) = poll_fn(|cx| {
            Pin::new(&mut self.reader_join_handle)
                .as_mut()
                .poll_unpin(cx)
        })
        .now_or_never()
        {
            self.alert_tx.send(Alert::DriverError(eyre!(
                "Reader task exited unexpectedly."
            )))?;
            return res;
        }

        if let Some(Ok(res)) = poll_fn(|cx| {
            Pin::new(&mut self.executor_join_handle)
                .as_mut()
                .poll_unpin(cx)
        })
        .now_or_never()
        {
            self.alert_tx.send(Alert::DriverError(eyre!(
                "Executor task exited unexpectedly."
            )))?;
            return res;
        }

        match cmd {
            DriverCommand::Shutdown => {
                self.shutdown()?;
            }
            DriverCommand::GetNewBlocks => {
                self.reader_tx
                    .send(ReaderCommand::GetNewBlocks)
                    .map_err(|e| eyre!("reader rx channel closed: {}", e))?;
            }
        }

        Ok(())
    }

    /// Sends shutdown commands to the other actors.
    fn shutdown(&mut self) -> Result<()> {
        let mut is_shutdown = self.is_shutdown.lock().unwrap();
        if *is_shutdown {
            return Ok(());
        }
        *is_shutdown = true;

        info!("Shutting down driver.");
        self.reader_tx.send(ReaderCommand::Shutdown)?;
        self.executor_tx.send(ExecutorCommand::Shutdown)?;
        Ok(())
    }
}
