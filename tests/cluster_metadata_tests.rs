extern crate rabbitmq_http_client;
use rabbitmq_http_client::blocking::Client;

extern crate common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_get_cluster_name() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let result = rc.get_cluster_name();
    assert!(result.is_ok());
    let meta = result.unwrap();
    assert!(meta.name.starts_with("rabbit"))
}

#[test]
fn test_set_cluster_name() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.get_cluster_name();
    assert!(result1.is_ok());
    let meta1 = result1.unwrap();
    assert!(meta1.name.starts_with("rabbit"));

    let result2 = rc.set_cluster_name("rusty");
    assert!(result2.is_ok());

    let result3 = rc.get_cluster_name();
    assert!(result3.is_ok());
    let meta3 = result3.unwrap();
    assert!(meta3.name == String::from("rusty"));

    let _ = rc.set_cluster_name(&meta1.name);
}
