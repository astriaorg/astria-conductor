use clap::{Arg, Command};

pub fn parse_args() -> clap::ArgMatches {
    Command::new("astria-conductor")
        .version("0.1")
        .about(
            "A cli to read and write blocks from and to different sources. Uses the Actor model.",
        )
        .arg(
            Arg::new("url")
                .short('u')
                .long("url")
                .help("URL of the data layer server.")
                .required(true),
        )
        .arg(
            Arg::new("namespace_id")
                .short('n')
                .long("namespace_id")
                .help("Namespace ID as a string; the hex encoding of a [u8; 8]")
                .required(true),
        )
        .arg(
            Arg::new("rpc_address")
                .short('r')
                .long("rpc_address")
                .help("Address of the execution RPC server.")
                .required(true),
        )
        .get_matches()
}
