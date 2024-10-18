use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_connections();
    assert!(result1.is_ok(), "list_connections returned {:?}", result1);
}

#[test]
fn test_list_user_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_user_connections(USERNAME);
    assert!(
        result1.is_ok(),
        "list_user_connections returned {:?}",
        result1
    );
}

#[test]
fn test_list_virtual_host_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let result1 = rc.list_connections_in(vh_name);
    assert!(
        result1.is_ok(),
        "list_connections_in returned {:?}",
        result1
    );
}
