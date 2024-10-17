use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_leader_rebalancing() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.rebalance_queue_leaders();
    assert!(
        result1.is_ok(),
        "rebalance_queue_leaders returned {:?}",
        result1
    );
}
