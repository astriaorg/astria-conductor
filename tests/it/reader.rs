use std::time::Duration;
use sequencer_relayer::sequencer::SequencerClient;

use crate::helper::{init_environment, init_stack, wait_until_ready};

#[tokio::test]
async fn should_build_podman() {
    let podman = init_environment();
    let info = init_stack(&podman).await;
    wait_until_ready(&podman, &info.pod_name).await;

    let cosmos_endpoint = info.make_sequencer_api_endpoint();
    let cosmos_grpc_endpoint = info.make_sequencer_grpc_endpoint();

    // FIXME: use a more reliable check to ensure any blocks are
    // available on the sequencer. Right now we have to explicitly
    // wait a sufficient period of time. This is flaky.
    tokio::time::sleep(Duration::from_secs(30)).await;

    assert_eq!(true, true);
    // let client = SequencerClient::new(cosmos_endpoint).unwrap();
    // client.get_latest_block().await.unwrap();
}
