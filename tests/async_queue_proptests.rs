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

mod test_helpers;

use crate::test_helpers::{PASSWORD, USERNAME, async_await_metric_emission, endpoint};
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rabbitmq_http_client::{api::Client, commons::QueueType, requests::QueueParams};
use serde_json::{Map, Value, json};
use tokio::runtime::Runtime;

fn arb_queue_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.[a-zA-Z0-9_-]{8,20}").unwrap()
}

fn arb_classic_queue_params()
-> impl Strategy<Value = (String, bool, bool, Option<Map<String, Value>>)> {
    (
        arb_queue_name(),
        any::<bool>(), // durable
        any::<bool>(), // auto_delete
        arb_optional_args(),
    )
}

fn arb_quorum_queue_params() -> impl Strategy<Value = (String, Option<Map<String, Value>>)> {
    (arb_queue_name(), arb_optional_args())
}

fn arb_stream_params() -> impl Strategy<Value = (String, u64)> {
    (arb_queue_name(), arb_max_length_bytes())
}

fn arb_message_ttl() -> impl Strategy<Value = u64> {
    1000u64..3600000u64
}

fn arb_max_length() -> impl Strategy<Value = u64> {
    100u64..1000000u64
}

fn arb_max_length_bytes() -> impl Strategy<Value = u64> {
    1024u64..100_000_000u64
}

fn arb_optional_args() -> impl Strategy<Value = Option<Map<String, Value>>> {
    prop_oneof![
        Just(None),
        arb_message_ttl().prop_map(|ttl| {
            let mut map = Map::new();
            map.insert("x-message-ttl".to_string(), json!(ttl));
            Some(map)
        }),
        arb_max_length().prop_map(|len| {
            let mut map = Map::new();
            map.insert("x-max-length".to_string(), json!(len));
            Some(map)
        }),
        arb_max_length_bytes().prop_map(|bytes| {
            let mut map = Map::new();
            map.insert("x-max-length-bytes".to_string(), json!(bytes));
            Some(map)
        }),
        (arb_message_ttl(), arb_max_length()).prop_map(|(ttl, len)| {
            let mut map = Map::new();
            map.insert("x-message-ttl".to_string(), json!(ttl));
            map.insert("x-max-length".to_string(), json!(len));
            Some(map)
        }),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prop_async_durable_client_named_classic_queue(
        (name, durable, auto_delete, optional_args) in arb_classic_queue_params()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);
            let vhost = "/";

            let _ = client.delete_queue(vhost, &name, true).await;

            let params = QueueParams::new(&name, QueueType::Classic, durable, auto_delete, optional_args);
            let result1 = client.declare_queue(vhost, &params).await;
            prop_assert!(result1.is_ok(), "Failed to declare classic queue: {result1:?}");

            async_await_metric_emission(20).await;

            let result2 = client.list_queues().await;
            prop_assert!(result2.is_ok(), "Failed to list queues: {result2:?}");

            let queues = result2.unwrap();
            let found_queue = queues.iter().find(|q| q.name == name);
            prop_assert!(found_queue.is_some(), "list_queues did not include the declared queue: {}", name);

            let queue = found_queue.unwrap();
            prop_assert_eq!(&queue.queue_type, "classic");
            prop_assert_eq!(queue.durable, durable);
            prop_assert_eq!(queue.auto_delete, auto_delete);

            let _ = client.delete_queue(vhost, &name, true).await;
            Ok(())
        })?;
    }

    #[test]
    fn prop_async_durable_client_named_quorum_queue(
        (name, optional_args) in arb_quorum_queue_params()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);
            let vhost = "/";

            let _ = client.delete_queue(vhost, &name, true).await;

            let params = QueueParams::new_quorum_queue(&name, optional_args);
            let result1 = client.declare_queue(vhost, &params).await;
            prop_assert!(result1.is_ok(), "Failed to declare quorum queue: {result1:?}");

            async_await_metric_emission(20).await;

            let result2 = client.list_queues_in(vhost).await;
            prop_assert!(result2.is_ok(), "Failed to list queues in vhost: {result2:?}");

            let queues = result2.unwrap();
            let found_queue = queues.iter().find(|q| q.name == name);
            prop_assert!(found_queue.is_some(), "list_queues did not include the declared queue: {}", name);

            let queue = found_queue.unwrap();
            prop_assert_eq!(&queue.queue_type, "quorum");
            prop_assert_eq!(queue.durable, true);
            prop_assert_eq!(queue.auto_delete, false);

            let _ = client.delete_queue(vhost, &name, true).await;
            Ok(())
        })?;
    }

    #[test]
    fn prop_async_stream_essential_ops(
        (name, max_length_bytes) in arb_stream_params()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);
            let vhost = "/";

            let _ = client.delete_queue(vhost, &name, true).await;

            let mut map = Map::new();
            map.insert("x-max-length-bytes".to_string(), json!(max_length_bytes));
            let optional_args = Some(map);

            let params = QueueParams::new_stream(&name, optional_args);
            let result1 = client.declare_queue(vhost, &params).await;
            prop_assert!(result1.is_ok(), "Failed to declare stream: {result1:?}");

            async_await_metric_emission(20).await;

            let result2 = client.list_queues_with_details().await;
            prop_assert!(result2.is_ok(), "list_queues_with_details did not include the declared queue: {result2:?}");

            let queues = result2.unwrap();
            let found_queue = queues.iter().find(|q| q.name == name);
            prop_assert!(found_queue.is_some(), "list_queues_with_details did not include the declared stream: {}", name);

            let queue = found_queue.unwrap();
            prop_assert_eq!(&queue.queue_type, "stream");
            prop_assert_eq!(queue.durable, true);
            prop_assert_eq!(queue.auto_delete, false);

            let _ = client.delete_queue(vhost, &name, true).await;
            Ok(())
        })?;
    }

    #[test]
    fn prop_async_transient_autodelete_classic_queue(
        name in arb_queue_name(),
        optional_args in arb_optional_args()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);
            let vhost = "/";

            let _ = client.delete_queue(vhost, &name, true).await;

            let params = QueueParams::new_transient_autodelete(&name, optional_args);
            let result1 = client.declare_queue(vhost, &params).await;
            prop_assert!(result1.is_ok(), "Failed to declare transient auto-delete queue: {result1:?}");

            async_await_metric_emission(20).await;

            let result2 = client.get_queue_info(vhost, &name).await;
            prop_assert!(result2.is_ok(), "Failed to get queue info: {result2:?}");

            let queue = result2.unwrap();
            prop_assert_eq!(queue.name, name.clone());
            prop_assert_eq!(queue.vhost, vhost);
            prop_assert_eq!(&queue.queue_type, "classic");
            prop_assert_eq!(queue.durable, false);
            prop_assert_eq!(queue.auto_delete, true);

            let _ = client.delete_queue(vhost, &name, true).await;
            Ok(())
        })?;
    }

    #[test]
    fn prop_async_list_queues_consistency(
        names in prop::collection::vec(arb_queue_name(), 1..3)
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);
            let vhost = "/";

            for name in &names {
                let _ = client.delete_queue(vhost, name, true).await;
            }

            for name in &names {
                let params = QueueParams::new_durable_classic_queue(name, None);
                let result1 = client.declare_queue(vhost, &params).await;
                prop_assert!(result1.is_ok(), "Failed to declare queue {}: {result1:?}", name);
            }

            async_await_metric_emission(20).await;

            let result2 = client.list_queues().await;
            prop_assert!(result2.is_ok(), "Failed to list all queues: {result2:?}");

            let result3 = client.list_queues_in(vhost).await;
            prop_assert!(result3.is_ok(), "Failed to list queues in vhost: {result3:?}");

            let all_queues = result2.unwrap();
            let vhost_queues = result3.unwrap();

            for name in &names {
                let found_in_all = all_queues.iter().any(|q| q.name == *name);
                let found_in_vhost = vhost_queues.iter().any(|q| q.name == *name);

                prop_assert!(found_in_all, "list_queues did not include the declared queue {}", name);
                prop_assert!(found_in_vhost, "list_queues_in did not include the declared queue {}", name);
            }

            for name in &names {
                let _ = client.delete_queue(vhost, name, true).await;
            }
            Ok(())
        })?;
    }
}
