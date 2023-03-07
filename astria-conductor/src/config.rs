use serde::{Deserialize, Serialize};

/// The global configuration for the driver and its components.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// URL of the Celestia Node
    pub celestia_node_url: String,

    /// Namespace that we want to work in
    pub namespace_id: String,

    /// Address of the RPC server for execution
    pub rpc_address: String,
}
