use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_health_check_cluster_wide_alarms() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_cluster_wide_alarms();
    assert!(result1.is_ok());
}

#[test]
fn test_health_check_local_alarms() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_local_alarms();
    assert!(result1.is_ok());
}

#[test]
fn test_health_check_node_is_quorum_critical() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_if_node_is_quorum_critical();
    assert!(result1.is_ok());
}
