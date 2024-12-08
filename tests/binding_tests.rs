// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
use rabbitmq_http_client::{
    blocking_api::Client,
    commons::BindingDestinationType,
    requests::{ExchangeParams, QueueParams},
};

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_all_bindings() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let cq = "rust.cq.durable.1";
    let fanout = "amq.fanout";

    let result1 = rc.declare_queue(vh_name, &QueueParams::new_durable_classic_queue(cq, None));
    assert!(result1.is_ok(), "declare_queue returned {:?}", result1);

    let result2 = rc.bind_queue(vh_name, cq, fanout, None, None);
    assert!(result2.is_ok(), "bind_queue returned {:?}", result2);

    let result3 = rc.list_bindings();
    assert!(result3.is_ok(), "list_bindings returned {:?}", result3);
    let vec = result3.unwrap();
    assert!(vec
        .iter()
        .any(|b| b.destination == cq && b.source == fanout));

    let result4 = rc.list_bindings_in(vh_name);
    assert!(result4.is_ok(), "list_bindings_in returned {:?}", result4);
    let vec = result4.unwrap();
    assert!(vec
        .iter()
        .any(|vh| vh.vhost == vh_name && vh.source == fanout));

    let _ = rc.delete_queue(vh_name, cq);
}

#[test]
fn test_list_only_queue_bindings() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let cq = "rust.cq.durable.2";
    let fanout = "amq.fanout";

    let result1 = rc.declare_queue(vh_name, &QueueParams::new_durable_classic_queue(cq, None));
    assert!(result1.is_ok(), "declare_queue returned {:?}", result1);

    let result2 = rc.bind_queue(vh_name, cq, fanout, None, None);
    assert!(result2.is_ok(), "bind_queue returned {:?}", result2);

    let result3 = rc.list_queue_bindings(vh_name, cq);
    assert!(
        result3.is_ok(),
        "list_queue_bindings returned {:?}",
        result3
    );
    let vec = result3.unwrap();
    assert!(vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Queue
            && b.vhost == vh_name
            && b.destination == cq
            && b.source == fanout));

    let _ = rc.delete_queue(vh_name, cq);
}

#[test]
fn test_list_only_exchange_bindings() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let cq = "rust.cq.durable.3";
    let fanout1 = "amq.fanout";
    let fanout2 = "rust.x.fanout";

    let result1 = rc.declare_exchange(
        vh_name,
        &ExchangeParams::fanout(fanout2, false, false, None),
    );
    assert!(result1.is_ok(), "declare_exchange returned {:?}", result1);

    let result2 = rc.bind_exchange(vh_name, fanout1, fanout2, None, None);
    assert!(result2.is_ok(), "bind_exchange returned {:?}", result2);

    let result3 = rc.declare_queue(vh_name, &QueueParams::new_durable_classic_queue(cq, None));
    assert!(result3.is_ok(), "declare_queue returned {:?}", result3);

    let result4 = rc.bind_queue(vh_name, cq, fanout1, None, None);
    assert!(result4.is_ok(), "bind_queue returned {:?}", result4);

    let result5 = rc.list_exchange_bindings_with_source(vh_name, fanout2);
    assert!(
        result5.is_ok(),
        "list_exchange_bindings_with_source returned {:?}",
        result5
    );
    let vec = result5.unwrap();
    assert!(!vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Queue));
    assert!(vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Exchange
            && b.vhost == vh_name
            && b.destination == fanout1
            && b.source == fanout2));

    let result6 = rc.list_exchange_bindings_with_destination(vh_name, fanout1);
    assert!(
        result6.is_ok(),
        "list_exchange_bindings_with_destination returned {:?}",
        result6
    );
    let vec = result6.unwrap();
    assert!(!vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Queue));
    assert!(vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Exchange
            && b.vhost == vh_name
            && b.destination == fanout1
            && b.source == fanout2));

    let _ = rc.delete_queue(vh_name, cq);
    let _ = rc.delete_exchange(vh_name, fanout2);
}

#[test]
fn test_delete_queue_bindings() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let cq = "rust.cq.delete_queue_binding";
    let fanout = "amq.fanout";

    let result1 = rc.declare_queue(vh_name, &QueueParams::new_durable_classic_queue(cq, None));
    assert!(result1.is_ok(), "declare_queue returned {:?}", result1);

    let result2 = rc.bind_queue(vh_name, cq, fanout, Some("foo"), None);
    assert!(result2.is_ok(), "bind_queue returned {:?}", result2);

    let result3 = rc.list_queue_bindings(vh_name, cq);
    assert!(
        result3.is_ok(),
        "list_queue_bindings returned {:?}",
        result3
    );
    let vec = result3.unwrap();
    assert!(vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Queue
            && b.vhost == vh_name
            && b.destination == cq
            && b.source == fanout));

    let m: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let result4 = rc.delete_binding(
        vh_name,
        fanout,
        cq,
        BindingDestinationType::Queue,
        "foo",
        Some(m),
    );
    assert!(result4.is_ok(), "delete_binding returned {:?}", result4);

    let result5 = rc.list_queue_bindings(vh_name, cq);
    assert!(
        result5.is_ok(),
        "list_queue_bindings returned {:?}",
        result5
    );
    let vec = result5.unwrap();
    assert!(!vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Queue
            && b.vhost == vh_name
            && b.destination == cq
            && b.source == fanout));

    let _ = rc.delete_queue(vh_name, cq);
}

#[test]
fn test_delete_exchange_bindings() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let fanout = "amq.fanout";
    let direct = "amq.direct";

    let result2 = rc.bind_exchange(vh_name, direct, fanout, Some("foo"), None);
    assert!(result2.is_ok(), "bind_queue returned {:?}", result2);

    let result3 = rc.list_exchange_bindings_with_destination(vh_name, direct);
    assert!(
        result3.is_ok(),
        "list_exchange_bindings_with_destination returned {:?}",
        result3
    );
    let vec = result3.unwrap();
    assert!(vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Exchange
            && b.vhost == vh_name
            && b.destination == direct
            && b.source == fanout));

    let m: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
    let result4 = rc.delete_binding(
        vh_name,
        fanout,
        direct,
        BindingDestinationType::Exchange,
        "foo",
        Some(m),
    );
    assert!(result4.is_ok(), "delete_binding returned {:?}", result4);

    let result5 = rc.list_exchange_bindings_with_destination(vh_name, direct);
    assert!(
        result5.is_ok(),
        "list_exchange_bindings_with_destination returned {:?}",
        result5
    );
    let vec = result5.unwrap();
    assert!(!vec
        .iter()
        .any(|b| b.destination_type == BindingDestinationType::Exchange
            && b.vhost == vh_name
            && b.destination == direct
            && b.source == fanout));
}
