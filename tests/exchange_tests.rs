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
use rabbitmq_http_client::{blocking::Client, requests::ExchangeParams};
use serde_json::{json, Map, Value};

mod common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_declare_a_fanout_exchange() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.fanout.1";

    let _ = rc.delete_exchange(vhost, name);

    let result1 = rc.get_exchange_info(vhost, name);
    assert!(result1.is_err());

    let mut map = Map::<String, Value>::new();
    map.insert("x-alternate-exchange".to_owned(), json!("amq.fanout"));
    let optional_args = Some(map);
    let params = ExchangeParams::durable_fanout(name, optional_args);
    let result2 = rc.declare_exchange(vhost, &params);
    assert!(result2.is_ok());

    let _ = rc.delete_exchange(vhost, name);
}

#[test]
fn test_delete_exchange() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.cq.10";

    let _ = rc.delete_exchange(vhost, name);

    let result1 = rc.get_exchange_info(vhost, name);
    assert!(result1.is_err());

    let params = ExchangeParams::durable_fanout(name, None);
    let result2 = rc.declare_exchange(vhost, &params);
    assert!(result2.is_ok());

    let _ = rc.delete_exchange(vhost, name);
    let result3 = rc.get_exchange_info(vhost, name);
    assert!(result3.is_err());
}

#[test]
fn test_list_all_exchanges() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_exchanges();
    assert!(result1.is_ok());
}

#[test]
fn test_list_exchanges_in_a_virtual_host() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_exchanges_in("/");
    assert!(result1.is_ok(), "list_exchanges_in returned {:?}", result1);
}
