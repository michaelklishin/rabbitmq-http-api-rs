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
use rabbitmq_http_client::{
    blocking_api::Client, error::Error as APIClientError, requests::ExchangeParams,
};
use serde_json::{json, Map, Value};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

use rabbitmq_http_client::commons::ExchangeType;

#[test]
fn test_blocking_declare_a_durable_fanout_exchange() {
    test_declare_a_durable_exchange_of_type("rust.tests.fanout.1", ExchangeType::Fanout);
}

#[test]
fn test_blocking_declare_a_durable_topic_exchange() {
    test_declare_a_durable_exchange_of_type("rust.tests.topic.1", ExchangeType::Topic);
}

#[test]
fn test_blocking_declare_a_durable_direct_exchange() {
    test_declare_a_durable_exchange_of_type("rust.tests.direct.1", ExchangeType::Direct);
}

#[test]
fn test_blocking_declare_a_durable_headers_exchange() {
    test_declare_a_durable_exchange_of_type("rust.tests.headers.1", ExchangeType::Headers);
}

#[test]
fn test_blocking_declare_a_durable_local_random_exchange() {
    test_declare_a_durable_exchange_of_type("rust.tests.local-rnd.1", ExchangeType::LocalRandom);
}

#[test]
fn test_blocking_declare_a_durable_custom_exchange_type() {
    // This is a core type that's not in the AMQP 0-9-1 spec,
    // using it requiring additional plugins on the node
    test_declare_a_durable_exchange_of_type(
        "rust.tests.local-rnd.2",
        ExchangeType::Plugin("x-local-random".to_owned()),
    );
}

fn test_declare_a_durable_exchange_of_type(name: &str, typ: ExchangeType) {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";

    let _ = rc.delete_exchange(vhost, name, false);

    let result1 = rc.get_exchange_info(vhost, name);
    assert!(result1.is_err());

    let mut map = Map::<String, Value>::new();
    map.insert("x-alternate-exchange".to_owned(), json!("amq.fanout"));
    let optional_args = Some(map);
    let params = match typ {
        ExchangeType::Fanout => ExchangeParams::durable_fanout(name, optional_args),
        ExchangeType::Topic => ExchangeParams::durable_topic(name, optional_args),
        ExchangeType::Direct => ExchangeParams::durable_direct(name, optional_args),
        ExchangeType::Headers => ExchangeParams::durable_headers(name, optional_args),
        ExchangeType::LocalRandom => ExchangeParams::durable_local_random(name, optional_args),
        ExchangeType::Plugin(custom_type) => {
            ExchangeParams::plugin(name, custom_type, false, false, optional_args)
        }
        // the consistent hashing and other exchanges are intentionally ignored
        // in these tests
        _ => ExchangeParams::durable_fanout(name, optional_args),
    };
    let result2 = rc.declare_exchange(vhost, &params);
    assert!(result2.is_ok());

    let _ = rc.delete_exchange(vhost, name, false);
}

#[test]
fn test_blocking_delete_exchange() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.cq.10";

    let _ = rc.delete_exchange(vhost, name, false);

    let result1 = rc.get_exchange_info(vhost, name);
    assert!(result1.is_err());

    let params = ExchangeParams::durable_fanout(name, None);
    let result2 = rc.declare_exchange(vhost, &params);
    assert!(result2.is_ok());

    let _ = rc.delete_exchange(vhost, name, false);
    let result3 = rc.get_exchange_info(vhost, name);
    assert!(result3.is_err());
    assert!(matches!(result3.unwrap_err(), APIClientError::NotFound));
}

#[test]
fn test_blocking_list_all_exchanges() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_exchanges();
    assert!(result1.is_ok());
}

#[test]
fn test_blocking_list_exchanges_in_a_virtual_host() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_exchanges_in("/");
    assert!(result1.is_ok(), "list_exchanges_in returned {:?}", result1);
}
