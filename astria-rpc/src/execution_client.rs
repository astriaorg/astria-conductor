use execution::{
    execution_service_client::ExecutionServiceClient, DoBlockRequest, DoBlockResponse,
};
use tonic::transport::Channel;

pub mod execution {
    tonic::include_proto!("execution");
}

/// Represents an RpcClient
pub struct ExecutionRpcClient {
    /// The actual rpc client
    client: ExecutionServiceClient<Channel>,
}

impl ExecutionRpcClient {
    /// Creates a new RPC Client
    ///
    /// # Arguments
    ///
    /// * `address` - The address of the RPC server that we want to communicate with.
    pub async fn new(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let client = ExecutionServiceClient::connect(address.to_owned()).await?;
        Ok(ExecutionRpcClient { client })
    }

    /// Calls remote procedure DoBlock
    ///
    /// # Arguments
    ///
    /// * `header` - Header of the block
    /// * `transactions` - List of transactions
    pub async fn call_do_block(
        &mut self,
        header: Vec<u8>,
        transactions: Vec<Vec<u8>>,
    ) -> Result<DoBlockResponse, Box<dyn std::error::Error>> {
        let request = DoBlockRequest {
            header,
            transactions,
        };
        let response = self.client.do_block(request).await?.into_inner();
        Ok(response)
    }
}
