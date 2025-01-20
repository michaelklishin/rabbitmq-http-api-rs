// Copyright (C) 2023-2025 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use rabbitmq_http_client::{blocking_api::Client, commons::QueueType, requests::QueueParams};
use serde_json::{json, Map, Value};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_blocking_declare_and_redeclare_a_classic_queue() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.cq.69373293479827";

    let _ = rc.delete_queue(vhost, name, false);

    let result1 = rc.get_queue_info(vhost, name);
    assert!(result1.is_err());

    let mut map = Map::<String, Value>::new();
    map.insert("x-max-length".to_owned(), json!(10_000));
    // note: x-queue-type will be injected by QueueParams::new_durable_classic_queue
    let optional_args = Some(map);
    let params = QueueParams::new_durable_classic_queue(name, optional_args.clone());
    let result2 = rc.declare_queue(vhost, &params);
    assert!(result2.is_ok(), "declare_queue returned {:?}", result2);

    let params2 = QueueParams::new(name, QueueType::Classic, true, false, optional_args.clone());
    let result3 = rc.declare_queue(vhost, &params2);
    assert!(result3.is_ok(), "declare_queue returned {:?}", result3);

    let _ = rc.delete_queue(vhost, name, false);
}

#[test]
fn test_blocking_declare_a_quorum_queue() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.qq.182374982374";

    let _ = rc.delete_queue(vhost, name, false);

    let result1 = rc.get_queue_info(vhost, name);
    assert!(result1.is_err());

    let mut map = Map::<String, Value>::new();
    map.insert("x-max-length".to_owned(), json!(10_000));
    let optional_args = Some(map);
    let params = QueueParams::new_quorum_queue(name, optional_args);
    let result2 = rc.declare_queue(vhost, &params);
    assert!(result2.is_ok(), "declare_queue returned {:?}", result2);

    let _ = rc.delete_queue(vhost, name, false);
}

#[test]
fn test_blocking_declare_a_stream() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.qq.927348926347988623";

    let _ = rc.delete_queue(vhost, name, false);

    let result1 = rc.get_queue_info(vhost, name);
    assert!(result1.is_err());

    let mut map = Map::<String, Value>::new();
    map.insert("x-max-length-bytes".to_owned(), json!(10_000_000));
    let optional_args = Some(map);
    let params = QueueParams::new_stream(name, optional_args);
    let result2 = rc.declare_queue(vhost, &params);
    assert!(result2.is_ok(), "declare_queue returned {:?}", result2);

    let _ = rc.delete_queue(vhost, name, false);
}

#[test]
fn test_blocking_delete_queue() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.cq.982734982364982364896";

    let _ = rc.delete_queue(vhost, name, false);

    let result1 = rc.get_queue_info(vhost, name);
    assert!(result1.is_err());

    let params = QueueParams::new_durable_classic_queue(name, None);
    let result2 = rc.declare_queue(vhost, &params);
    assert!(result2.is_ok(), "declare_queue returned {:?}", result2);

    rc.delete_queue(vhost, name, false).unwrap();
    let result3 = rc.get_queue_info(vhost, name);
    assert!(result3.is_err());
}

#[test]
fn test_blocking_list_all_queues() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";

    let params = QueueParams::new_durable_classic_queue("rust.tests.cq.23487866", None);
    let result1 = rc.declare_queue(vh_name, &params);
    assert!(result1.is_ok(), "declare_queue returned {:?}", result1);

    test_helpers::await_queue_metric_emission();

    let result2 = rc.list_queues();
    assert!(result2.is_ok(), "list_queues returned {:?}", result2);

    rc.delete_queue(vh_name, params.name, false).unwrap();
}

#[test]
fn test_blocking_list_queues_in_a_virtual_host() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";

    let params = QueueParams::new_durable_classic_queue("rust.tests.cq.64692734867", None);
    let result1 = rc.declare_queue(vh_name, &params);
    assert!(result1.is_ok(), "declare_queue returned {:?}", result1);

    test_helpers::await_queue_metric_emission();

    let result2 = rc.list_queues_in(vh_name);
    assert!(result2.is_ok(), "list_queues_in returned {:?}", result2);

    rc.delete_queue(vh_name, params.name, false).unwrap();
}
