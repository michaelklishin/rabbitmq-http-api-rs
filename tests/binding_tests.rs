extern crate rabbitmq_http_client;
use rabbitmq_http_client::{
    blocking::Client,
    commons::BindingDestinationType,
    requests::{ExchangeParams, QueueParams},
};

extern crate common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_all_bindings() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let cq = "rust.cq.durable.1";
    let fanout = "amq.fanout";

    let result1 = rc.declare_queue(vh_name, &QueueParams::new_durable_classic_queue(cq, None));
    assert!(result1.is_ok());

    let result2 = rc.bind_queue(vh_name, cq, fanout, None, None);
    assert!(result2.is_ok());

    let result3 = rc.list_bindings();
    assert!(result3.is_ok());
    let vec = result3.unwrap();
    assert!(vec
        .iter()
        .find(|b| b.destination == cq && b.source == fanout)
        .is_some());

    let result4 = rc.list_bindings_in(vh_name);
    assert!(result4.is_ok());
    let vec = result4.unwrap();
    assert!(vec
        .iter()
        .find(|vh| vh.vhost == vh_name && vh.source == fanout)
        .is_some());

    let _ = rc.delete_queue(vh_name, cq);
}

#[test]
fn test_list_only_queue_bindings() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let cq = "rust.cq.durable.2";
    let fanout = "amq.fanout";

    let result1 = rc.declare_queue(vh_name, &QueueParams::new_durable_classic_queue(cq, None));
    assert!(result1.is_ok());

    let result2 = rc.bind_queue(vh_name, cq, fanout, None, None);
    assert!(result2.is_ok());

    let result3 = rc.list_queue_bindings(vh_name, cq);
    assert!(result3.is_ok());
    let vec = result3.unwrap();
    assert!(vec
        .iter()
        .find(|b| b.destination_type == BindingDestinationType::Queue
            && b.vhost == vh_name
            && b.destination == cq
            && b.source == fanout)
        .is_some());

    let _ = rc.delete_queue(vh_name, cq);
}

#[test]
fn test_list_only_exchange_bindings() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let cq = "rust.cq.durable.3";
    let fanout1 = "amq.fanout";
    let fanout2 = "rust.x.fanout";

    let result1 = rc.declare_exchange(
        vh_name,
        &ExchangeParams::fanout(fanout2, false, false, None),
    );
    assert!(result1.is_ok());

    let result2 = rc.bind_exchange(vh_name, fanout1, fanout2, None, None);
    assert!(result2.is_ok());

    let result3 = rc.declare_queue(vh_name, &QueueParams::new_durable_classic_queue(cq, None));
    assert!(result3.is_ok());

    let result4 = rc.bind_queue(vh_name, cq, fanout1, None, None);
    assert!(result4.is_ok());

    let result5 = rc.list_exchange_bindings_with_source(vh_name, fanout2);
    assert!(result5.is_ok());
    let vec = result5.unwrap();
    assert!(vec
        .iter()
        .find(|b| b.destination_type == BindingDestinationType::Queue)
        .is_none());
    assert!(vec
        .iter()
        .find(|b| b.destination_type == BindingDestinationType::Exchange
            && b.vhost == vh_name
            && b.destination == fanout1
            && b.source == fanout2)
        .is_some());

    let result6 = rc.list_exchange_bindings_with_destination(vh_name, fanout1);
    assert!(result6.is_ok());
    let vec = result6.unwrap();
    assert!(vec
        .iter()
        .find(|b| b.destination_type == BindingDestinationType::Queue)
        .is_none());
    assert!(vec
        .iter()
        .find(|b| b.destination_type == BindingDestinationType::Exchange
            && b.vhost == vh_name
            && b.destination == fanout1
            && b.source == fanout2)
        .is_some());

    let _ = rc.delete_queue(vh_name, cq);
    let _ = rc.delete_exchange(vh_name, fanout2);
}
