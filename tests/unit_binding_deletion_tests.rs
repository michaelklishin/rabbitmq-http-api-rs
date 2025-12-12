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

use rabbitmq_http_client::commons::BindingDestinationType;
use rabbitmq_http_client::requests::BindingDeletionParams;
use rabbitmq_http_client::responses::{BindingInfo, XArguments};
use serde_json::{Map, json};

#[test]
fn test_binding_deletion_params_from_binding_info_queue() {
    let binding = BindingInfo {
        vhost: "/".to_string(),
        source: "amq.direct".to_string(),
        destination: "my-queue".to_string(),
        destination_type: BindingDestinationType::Queue,
        routing_key: "my-key".to_string(),
        arguments: XArguments(Map::new()),
        properties_key: Some("my-key".to_string()),
    };

    let params = BindingDeletionParams::from_binding_info(&binding);

    assert_eq!(params.virtual_host, "/");
    assert_eq!(params.source, "amq.direct");
    assert_eq!(params.destination, "my-queue");
    assert_eq!(params.destination_type, BindingDestinationType::Queue);
    assert_eq!(params.routing_key, "my-key");
    assert!(params.arguments.is_none());
}

#[test]
fn test_binding_deletion_params_from_binding_info_exchange() {
    let binding = BindingInfo {
        vhost: "test-vhost".to_string(),
        source: "source-exchange".to_string(),
        destination: "dest-exchange".to_string(),
        destination_type: BindingDestinationType::Exchange,
        routing_key: "".to_string(),
        arguments: XArguments(Map::new()),
        properties_key: Some("~".to_string()),
    };

    let params = BindingDeletionParams::from_binding_info(&binding);

    assert_eq!(params.virtual_host, "test-vhost");
    assert_eq!(params.source, "source-exchange");
    assert_eq!(params.destination, "dest-exchange");
    assert_eq!(params.destination_type, BindingDestinationType::Exchange);
    assert_eq!(params.routing_key, "");
    assert!(params.arguments.is_none());
}

#[test]
fn test_binding_deletion_params_from_binding_info_with_arguments() {
    let mut args_map = Map::new();
    args_map.insert("x-match".to_string(), json!("all"));
    args_map.insert("header1".to_string(), json!("value1"));

    let binding = BindingInfo {
        vhost: "/".to_string(),
        source: "amq.headers".to_string(),
        destination: "headers-queue".to_string(),
        destination_type: BindingDestinationType::Queue,
        routing_key: "".to_string(),
        arguments: XArguments(args_map.clone()),
        properties_key: Some("~".to_string()),
    };

    let params = BindingDeletionParams::from_binding_info(&binding);

    assert!(params.arguments.is_some());
    let args = params.arguments.unwrap();
    assert_eq!(args.get("x-match"), Some(&json!("all")));
    assert_eq!(args.get("header1"), Some(&json!("value1")));
}

#[test]
fn test_binding_deletion_params_from_trait() {
    let binding = BindingInfo {
        vhost: "/".to_string(),
        source: "amq.direct".to_string(),
        destination: "my-queue".to_string(),
        destination_type: BindingDestinationType::Queue,
        routing_key: "my-key".to_string(),
        arguments: XArguments(Map::new()),
        properties_key: Some("my-key".to_string()),
    };

    let params: BindingDeletionParams = (&binding).into();

    assert_eq!(params.virtual_host, "/");
    assert_eq!(params.source, "amq.direct");
    assert_eq!(params.destination, "my-queue");
}
