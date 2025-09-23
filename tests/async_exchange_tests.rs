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
use rabbitmq_http_client::{api::Client, error::Error as APIClientError, requests::ExchangeParams};
use serde_json::{Map, Value, json};

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, async_testing_against_3_13_x, endpoint};

use rabbitmq_http_client::commons::ExchangeType;

#[tokio::test]
async fn test_async_declare_a_durable_fanout_exchange() {
    test_async_declare_a_durable_exchange_of_type("rust.tests.fanout.1", ExchangeType::Fanout)
        .await;
}

#[tokio::test]
async fn test_async_declare_a_durable_topic_exchange() {
    test_async_declare_a_durable_exchange_of_type("rust.tests.topic.1", ExchangeType::Topic).await;
}

#[tokio::test]
async fn test_async_declare_a_durable_direct_exchange() {
    test_async_declare_a_durable_exchange_of_type("rust.tests.direct.1", ExchangeType::Direct)
        .await;
}

#[tokio::test]
async fn test_async_declare_a_durable_headers_exchange() {
    test_async_declare_a_durable_exchange_of_type("rust.tests.headers.1", ExchangeType::Headers)
        .await;
}

#[tokio::test]
async fn test_async_declare_a_durable_local_random_exchange() {
    if async_testing_against_3_13_x().await {
        return;
    }

    test_async_declare_a_durable_exchange_of_type(
        "rust.tests.local-rnd.1",
        ExchangeType::LocalRandom,
    )
    .await;
}

#[tokio::test]
async fn test_async_declare_a_durable_custom_exchange_type() {
    if async_testing_against_3_13_x().await {
        return;
    }

    // This is a core type that's not in the AMQP 0-9-1 spec,
    // using it requiring additional plugins on the node
    test_async_declare_a_durable_exchange_of_type(
        "rust.tests.local-rnd.2",
        ExchangeType::Plugin("x-local-random".to_owned()),
    )
    .await;
}

async fn test_async_declare_a_durable_exchange_of_type(name: &str, typ: ExchangeType) {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";

    let _ = rc.delete_exchange(vhost, name, false).await;

    let result1 = rc.get_exchange_info(vhost, name).await;
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
    let result2 = rc.declare_exchange(vhost, &params).await;
    assert!(result2.is_ok());

    let _ = rc.delete_exchange(vhost, name, false).await;
}

#[tokio::test]
async fn test_async_delete_exchange() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.cq.10";

    // delete it in case it exists from a previous failed run
    let _ = rc.delete_exchange(vhost, name, true).await;

    let result1 = rc.get_exchange_info(vhost, name).await;
    assert!(result1.is_err());

    let params = ExchangeParams::durable_fanout(name, None);
    let result2 = rc.declare_exchange(vhost, &params).await;
    assert!(result2.is_ok());

    // now delete it for real
    let _ = rc.delete_exchange(vhost, name, false).await;

    // idempotent delete should succeed
    let _ = rc.delete_exchange(vhost, name, true).await;

    // non-idempotent delete should fail
    assert!(rc.delete_exchange(vhost, name, false).await.is_err());

    let result3 = rc.get_exchange_info(vhost, name).await;
    assert!(result3.is_err());
    assert!(matches!(result3.unwrap_err(), APIClientError::NotFound));
}

#[tokio::test]
async fn test_async_list_all_exchanges() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_exchanges().await;
    assert!(result1.is_ok());
}

#[tokio::test]
async fn test_async_list_exchanges_in_a_virtual_host() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_exchanges_in("/").await;
    assert!(result1.is_ok(), "list_exchanges_in returned {result1:?}");
}
