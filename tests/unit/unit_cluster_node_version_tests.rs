// Copyright (C) 2023-2026 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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

use rabbitmq_http_client::responses::ClusterNode;
use serde_json::{Value, from_value, json};

fn minimal_node_json() -> Value {
    json!({
        "name": "rabbit@hostname",
        "uptime": 60000,
        "run_queue": 0,
        "processors": 8,
        "os_pid": "1234",
        "fd_total": 1048576,
        "proc_total": 1048576,
        "mem_limit": 6843490508u64,
        "mem_alarm": false,
        "disk_free_limit": 50000000,
        "disk_free_alarm": false,
        "rates_mode": "basic",
        "enabled_plugins": ["rabbitmq_management"],
        "being_drained": false,
        "applications": [
            {"name": "rabbit", "description": "RabbitMQ", "version": "4.2.4"}
        ]
    })
}

#[test]
fn test_unit_cluster_node_with_all_version_fields() {
    let mut json = minimal_node_json();
    let obj = json.as_object_mut().unwrap();
    obj.insert("rabbitmq_version".into(), json!("4.2.4"));
    obj.insert("erlang_version".into(), json!("27"));
    obj.insert(
        "erlang_full_version".into(),
        json!("Erlang/OTP 27 [erts-15.2] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit]"),
    );
    obj.insert(
        "crypto_lib_version".into(),
        json!("OpenSSL 3.4.1 11 Feb 2025"),
    );

    let node: ClusterNode = from_value(json).unwrap();
    assert_eq!(node.rabbitmq_version.as_deref(), Some("4.2.4"));
    assert_eq!(node.erlang_version.as_deref(), Some("27"));
    assert_eq!(
        node.erlang_full_version.as_deref(),
        Some(
            "Erlang/OTP 27 [erts-15.2] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit]"
        )
    );
    assert_eq!(
        node.crypto_lib_version.as_deref(),
        Some("OpenSSL 3.4.1 11 Feb 2025")
    );
}

// older RabbitMQ versions do not include these fields
#[test]
fn test_unit_cluster_node_without_version_fields() {
    let json = minimal_node_json();

    let node: ClusterNode = from_value(json).unwrap();
    assert!(node.rabbitmq_version.is_none());
    assert!(node.erlang_version.is_none());
    assert!(node.erlang_full_version.is_none());
    assert!(node.crypto_lib_version.is_none());
}

#[test]
fn test_unit_cluster_node_with_partial_version_fields() {
    let mut json = minimal_node_json();
    let obj = json.as_object_mut().unwrap();
    obj.insert("rabbitmq_version".into(), json!("4.2.4"));
    obj.insert("erlang_version".into(), json!("27"));

    let node: ClusterNode = from_value(json).unwrap();
    assert_eq!(node.rabbitmq_version.as_deref(), Some("4.2.4"));
    assert_eq!(node.erlang_version.as_deref(), Some("27"));
    assert!(node.erlang_full_version.is_none());
    assert!(node.crypto_lib_version.is_none());
}

#[test]
fn test_unit_cluster_node_existing_fields_unaffected() {
    let mut json = minimal_node_json();
    let obj = json.as_object_mut().unwrap();
    obj.insert("rabbitmq_version".into(), json!("4.2.4"));

    let node: ClusterNode = from_value(json).unwrap();
    assert_eq!(node.name, "rabbit@hostname");
    assert_eq!(node.uptime, 60000);
    assert_eq!(node.processors, 8);
    assert_eq!(node.os_pid, 1234);
    assert!(!node.has_memory_alarm_in_effect);
    assert!(!node.has_free_disk_space_alarm_in_effect);
    assert!(!node.being_drained);
    assert_eq!(node.rates_mode, "basic");
}

#[test]
fn test_unit_cluster_node_rabbitmq_version_uses_field_when_present() {
    let mut json = minimal_node_json();
    let obj = json.as_object_mut().unwrap();
    obj.insert("rabbitmq_version".into(), json!("4.2.4"));

    let node: ClusterNode = from_value(json).unwrap();
    assert_eq!(node.rabbitmq_version(), "4.2.4");
}

// on older nodes, the field is absent and the method falls back to the rabbit OTP application
#[test]
fn test_unit_cluster_node_rabbitmq_version_falls_back_to_otp_app() {
    let json = minimal_node_json();

    let node: ClusterNode = from_value(json).unwrap();
    assert!(node.rabbitmq_version.is_none());
    assert_eq!(node.rabbitmq_version(), "4.2.4");
}
