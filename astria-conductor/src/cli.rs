use clap::Parser;

#[derive(Parser)]
pub(crate) struct Cli {
    /// URL of the data layer server.
    #[arg(long = "url")]
    pub(crate) url: String,

    /// Namespace ID as a string; the hex encoding of a [u8; 8]
    #[arg(long = "namespace-id")]
    pub(crate) namespace_id: String,

    /// Address of the execution RPC server.
    #[arg(long = "rpc-address")]
    pub(crate) rpc_address: String,
}
