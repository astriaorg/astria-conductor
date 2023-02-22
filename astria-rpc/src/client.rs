use execution::{DoBlockRequest, DoBlockResponse, execution_service_client::ExecutionServiceClient};
use tonic::transport::Channel;

pub mod execution {
    tonic::include_proto!("execution");
}

pub struct RpcClient {
    client: ExecutionServiceClient<Channel>,
}

impl RpcClient {
    pub async fn new(address: &str) -> Result<Self, Box<dyn std::error::Error>> {
        println!("address: {}", address);
        let client = ExecutionServiceClient::connect(address.to_owned()).await?;
        Ok(RpcClient { client })
    }

    pub async fn do_block(&mut self, header: Vec<u8>, transactions: Vec<Vec<u8>>) -> Result<DoBlockResponse, Box<dyn std::error::Error>> {
        let request = DoBlockRequest {
            header,
            transactions,
        };
        let response = self.client.do_block(request).await?.into_inner();
        Ok(response)
    }
}
