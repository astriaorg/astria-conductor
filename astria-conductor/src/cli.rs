use clap::Parser;

#[derive(Parser)]
pub(crate) struct Cli {
    #[arg(short = 'u', long = "url", help = "URL of the data layer server.")]
    pub url: String,

    #[arg(
        short = 'n',
        long = "namespace_id",
        help = "Namespace ID as a string; the hex encoding of a [u8; 8]"
    )]
    pub namespace_id: String,

    #[arg(
        short = 'r',
        long = "rpc_address",
        help = "Address of the execution RPC server."
    )]
    pub rpc_address: String,
}
