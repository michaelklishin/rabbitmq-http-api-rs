use rabbitmq_http_client::{blocking::Client, requests::VirtualHostParams};

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_consumers() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_consumers();
    assert!(result1.is_ok(), "list_consumers returned {:?}", result1);
}

#[test]
fn test_list_vhost_consumers() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_vhost_consumers");
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let result2 = rc.list_consumers_in(vh_params.name);
    assert!(result2.is_ok(), "list_consumers_in returned {:?}", result2);
}
