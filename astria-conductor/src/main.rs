use clap::Parser;
use std::time::Duration;

use astria_rpc::RpcClient;
use tokio::{signal, time};

use crate::alert::Alert;
use crate::cli::Cli;
use crate::conf::Conf;
use crate::driver::DriverCommand;
use crate::error::*;

pub mod alert;
mod cli;
pub mod conf;
mod driver;
mod error;
mod executor;
mod logger;
mod reader;

#[tokio::main]
async fn main() -> Result<()> {
    // logs
    logger::initialize();

    // parse cli args
    let args = Cli::parse();

    // configuration
    let conf = Conf::new(
        args.url,
        args.namespace_id,
        args.rpc_address,
    );
    log::info!("Using node at {}", conf.celestia_node_url);

    // TODO - handle error properly
    // TODO - actually implement. this is just poc.
    let mut execution_rpc_client = RpcClient::new(&conf.rpc_address).await.expect("uh oh");
    let fake_header: Vec<u8> = vec![0, 1, 255];
    let fake_tx: Vec<Vec<u8>> = vec![vec![0, 1, 255], vec![1, 2, 3], vec![1, 0, 1, 1]];
    execution_rpc_client.do_block(fake_header, fake_tx).await.expect("uh oh do block");

    // spawn our driver
    let (mut driver_handle, mut alert_rx) = driver::spawn(conf)?;

    // NOTE - this will most likely be replaced by an RPC server that will receive gossip
    //  messages from the sequencer
    let mut interval = time::interval(Duration::from_secs(3));

    let mut run = true;
    while run {
        tokio::select! {
            // handle alerts from the driver
            Some(alert) = alert_rx.recv() => {
                match alert {
                    Alert::DriverError(error_string) => {
                        println!("error: {}", error_string);
                        run = false;
                    }
                    Alert::BlockReceived{block_height} => {
                        println!("block received at {}", block_height);
                    }
                }
            }
            // request new blocks every X seconds
            _ = interval.tick() => {
                driver_handle.tx.send(DriverCommand::GetNewBlocks)?;
            }
            // shutdown properly on ctrl-c
            _ = signal::ctrl_c() => {
                driver_handle.shutdown().await?;
            }
        }
        if !run {
            break;
        }
    }

    Ok(())
}
