use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_nodes() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_nodes();

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().any(|n| n.name.starts_with("rabbit@")))
}

#[test]
fn test_get_node_info() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().unwrap();
    let name = nodes.first().unwrap().name.clone();
    let node = &rc.get_node_info(&name).unwrap();

    assert!(node.processors >= 1);
    assert!(node.uptime >= 1);
    assert!(node.total_erlang_processes >= 1);
}
