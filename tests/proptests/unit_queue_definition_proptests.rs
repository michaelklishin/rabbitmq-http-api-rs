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

use proptest::prelude::*;
use rabbitmq_http_client::responses::{OptionalArgumentSourceOps, QueueDefinition, XArguments};
use serde_json::{Map, json};

#[test]
fn test_without_quorum_queue_incompatible_keys_removes_specific_keys() {
    let mut arguments = Map::new();
    arguments.insert("x-queue-type".to_string(), json!("quorum"));
    arguments.insert("x-queue-mode".to_string(), json!("lazy"));
    arguments.insert("x-max-priority".to_string(), json!(10));
    arguments.insert("x-message-ttl".to_string(), json!(60000));

    let original_def = QueueDefinition {
        name: "test-queue".to_string(),
        vhost: "/".to_string(),
        durable: true,
        auto_delete: false,
        arguments: XArguments(arguments),
    };

    let filtered_def = original_def.without_quorum_queue_incompatible_keys();

    assert!(filtered_def.arguments.contains_key("x-queue-type"));
    assert!(filtered_def.arguments.contains_key("x-message-ttl"));
    assert!(!filtered_def.arguments.contains_key("x-queue-mode"));
    assert!(!filtered_def.arguments.contains_key("x-max-priority"));
    assert_eq!(filtered_def.name, original_def.name);
    assert_eq!(filtered_def.vhost, original_def.vhost);
    assert_eq!(filtered_def.durable, original_def.durable);
    assert_eq!(filtered_def.auto_delete, original_def.auto_delete);
}

proptest! {
    #[test]
    fn prop_without_quorum_queue_incompatible_keys_preserves_compatible(
        name in "[a-zA-Z0-9._-]+",
        vhost in "[a-zA-Z0-9/_-]+",
        durable in any::<bool>(),
        auto_delete in any::<bool>()
    ) {
        let mut arguments = Map::new();
        arguments.insert("x-queue-type".to_string(), json!("quorum"));
        arguments.insert("x-message-ttl".to_string(), json!(60000));
        arguments.insert("x-expires".to_string(), json!(120000));

        let original_def = QueueDefinition {
            name: name.clone(),
            vhost: vhost.clone(),
            durable,
            auto_delete,
            arguments: XArguments(arguments),
        };

        let filtered_def = original_def.without_quorum_queue_incompatible_keys();

        prop_assert_eq!(filtered_def.name, original_def.name);
        prop_assert_eq!(filtered_def.vhost, original_def.vhost);
        prop_assert_eq!(filtered_def.durable, original_def.durable);
        prop_assert_eq!(filtered_def.auto_delete, original_def.auto_delete);
        prop_assert_eq!(filtered_def.arguments.len(), original_def.arguments.len());

        prop_assert!(filtered_def.arguments.contains_key("x-queue-type"));
        prop_assert!(filtered_def.arguments.contains_key("x-message-ttl"));
        prop_assert!(filtered_def.arguments.contains_key("x-expires"));
    }

}

#[test]
fn test_without_quorum_queue_incompatible_keys_idempotent() {
    let mut arguments = Map::new();
    arguments.insert("x-queue-type".to_string(), json!("quorum"));
    arguments.insert("x-queue-mode".to_string(), json!("lazy"));
    arguments.insert("x-max-priority".to_string(), json!(10));
    arguments.insert("x-message-ttl".to_string(), json!(60000));

    let original_def = QueueDefinition {
        name: "test-queue".to_string(),
        vhost: "/".to_string(),
        durable: true,
        auto_delete: false,
        arguments: XArguments(arguments),
    };

    let filtered_once = original_def.without_quorum_queue_incompatible_keys();
    let filtered_twice = filtered_once.without_quorum_queue_incompatible_keys();

    assert_eq!(filtered_once.name, filtered_twice.name);
    assert_eq!(filtered_once.vhost, filtered_twice.vhost);
    assert_eq!(filtered_once.durable, filtered_twice.durable);
    assert_eq!(filtered_once.auto_delete, filtered_twice.auto_delete);
    assert_eq!(
        filtered_once.arguments.len(),
        filtered_twice.arguments.len()
    );

    assert!(filtered_once.arguments.contains_key("x-queue-type"));
    assert!(filtered_once.arguments.contains_key("x-message-ttl"));
    assert!(!filtered_once.arguments.contains_key("x-queue-mode"));
    assert!(!filtered_once.arguments.contains_key("x-max-priority"));

    assert!(filtered_twice.arguments.contains_key("x-queue-type"));
    assert!(filtered_twice.arguments.contains_key("x-message-ttl"));
    assert!(!filtered_twice.arguments.contains_key("x-queue-mode"));
    assert!(!filtered_twice.arguments.contains_key("x-max-priority"));
}
