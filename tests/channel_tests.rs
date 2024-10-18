use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_channels() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_channels();
    assert!(result1.is_ok(), "list_channels returned {:?}", result1);
}

#[test]
fn test_list_virtual_host_channels() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let result1 = rc.list_channels_in(vh_name);
    assert!(result1.is_ok(), "list_channels_in returned {:?}", result1);
}
