//! The driver is the top-level coordinator that runs and manages all the components
//! necessary for this reader/validator.

use color_eyre::eyre::{eyre, Result};
use futures::future::{poll_fn, FutureExt};
use log::{error, info};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

// use std::future::{Future};
use std::pin::Pin;
// use std::task::{Context, Poll, Waker};

use crate::executor::ExecutorCommand;
use crate::reader::ReaderCommand;
use crate::{alert::AlertSender, config::Config, executor, reader};

/// The channel through which the user can send commands to the driver.
pub(crate) type Sender = UnboundedSender<DriverCommand>;
/// The channel on which the driver listens for commands from the user.
pub(crate) type Receiver = UnboundedReceiver<DriverCommand>;

/// The type of commands that the driver can receive.
#[derive(Debug)]
pub(crate) enum DriverCommand {
    /// Get new blocks
    GetNewBlocks,
    /// Gracefully shuts down the driver and its components.
    Shutdown,
}

#[allow(dead_code)] // TODO - remove after developing
pub(crate) struct Driver {
    pub(crate) cmd_tx: Sender,

    /// The channel on which other components in the driver sends the driver messages.
    cmd_rx: Receiver,

    /// The channel used to send messages to the reader task.
    reader_tx: reader::Sender,
    reader_join_handle: reader::JoinHandle,

    /// The channel used to send messages to the executor task.
    executor_tx: executor::Sender,
    executor_join_handle: executor::JoinHandle,

    /// The channel on which the driver and tasks in the driver can post alerts
    /// to the consumer of the driver.
    alert_tx: AlertSender,

    /// The global configuration
    conf: Config,
}

impl Driver {
    pub(crate) async fn new(conf: Config, alert_tx: AlertSender) -> Result<Self> {
        let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
        let (executor_join_handle, executor_tx) = executor::spawn(&conf).await?;
        let (reader_join_handle, reader_tx) =
            reader::spawn(&conf, cmd_tx.clone(), executor_tx.clone()).await?;

        Ok(Self {
            cmd_tx: cmd_tx.clone(),
            cmd_rx,
            reader_tx,
            reader_join_handle,
            executor_tx,
            executor_join_handle,
            alert_tx,
            conf,
        })
    }

    /// Runs the Driver event loop.
    pub(crate) async fn run(&mut self) -> Result<()> {
        info!("Starting driver event loop.");
        while let Some(cmd) = self.cmd_rx.recv().await {
            // TODO: these are kind of janky, we might want to move to a polling-based architecture
            if let Some(Ok(res)) = poll_fn(|cx| {
                Pin::new(&mut self.reader_join_handle)
                    .as_mut()
                    .poll_unpin(cx)
            })
            .now_or_never()
            {
                error!("Reader task exited unexpectedly.");
                return res;
            }

            if let Some(Ok(res)) = poll_fn(|cx| {
                Pin::new(&mut self.executor_join_handle)
                    .as_mut()
                    .poll_unpin(cx)
            })
            .now_or_never()
            {
                error!("Executor task exited unexpectedly.");
                return res;
            }

            match cmd {
                DriverCommand::Shutdown => {
                    self.shutdown()?;
                    break;
                }
                DriverCommand::GetNewBlocks => {
                    self.reader_tx
                        .send(ReaderCommand::GetNewBlocks)
                        .map_err(|e| eyre!("reader rx channel closed: {}", e))?;
                }
            }
        }
        error!("Driver event loop exited unexpectedly.");
        Ok(())
    }

    /// Sends shutdown commands to the other actors.
    fn shutdown(&mut self) -> Result<()> {
        // TODO: mutex to prevent double shutdown
        info!("Shutting down driver.");
        self.reader_tx.send(ReaderCommand::Shutdown)?;
        self.executor_tx.send(ExecutorCommand::Shutdown)?;
        Ok(())
    }
}

// impl Future for Driver {
//     type Output = Result<()>;

//     fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
//         if let Poll::Ready(Ok(res)) = self.reader_join_handle.poll_unpin(cx) {
//             error!("Reader task exited unexpectedly.");
//             return Poll::Ready(Ok(()));
//         }

//         if let Poll::Ready(Ok(res)) = self.executor_join_handle.poll_unpin(cx) {
//             error!("Executor task exited unexpectedly.");
//             return Poll::Ready(Ok(()));
//         }

//         match self.cmd_rx.poll_recv(cx) {
//             Poll::Ready(Some(cmd)) => match cmd {
//                 DriverCommand::Shutdown => {
//                     self.shutdown_actors()?;
//                     return Poll::Ready(Ok(()));
//                 }
//                 DriverCommand::GetNewBlocks => {
//                     self.reader_tx
//                         .send(ReaderCommand::GetNewBlocks)
//                         .map_err(|e| eyre!("reader rx channel closed: {}", e))?;
//                 }
//             },
//             Poll::Ready(None) => {
//                 return Poll::Ready(Err(eyre!("Driver event loop exited unexpectedly.")));
//             }
//             Poll::Pending => {}
//         }

//         self.waker = Some(cx.waker().clone());
//         Poll::Pending
//     }
// }
