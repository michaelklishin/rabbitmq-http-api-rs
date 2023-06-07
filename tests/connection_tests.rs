extern crate rabbitmq_http_client;
use rabbitmq_http_client::blocking::Client;

extern crate common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_connections() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_connections();
    assert!(result1.is_ok(), "list_connections returned {:?}", result1);
}

#[test]
fn test_list_user_connections() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_user_connections(USERNAME);
    assert!(
        result1.is_ok(),
        "list_user_connections returned {:?}",
        result1
    );
}
