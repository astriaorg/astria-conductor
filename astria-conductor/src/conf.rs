/// The global configuration for the driver and its components.
#[allow(dead_code)] // TODO - remove after developing
pub struct Conf {
    /// URL of the Celestia Node
    pub celestia_node_url: String,

    /// Namespace that we want to work in
    pub namespace_id: String,

    /// Address of the RPC server for execution
    pub rpc_address: String,
}

impl Conf {
    pub fn new(celestia_node_url: String, namespace_id: String, rpc_address: String) -> Self {
        Self {
            namespace_id,
            celestia_node_url,
            rpc_address,
        }
    }
}
