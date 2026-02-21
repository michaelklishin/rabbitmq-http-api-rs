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

use proptest::prelude::*;
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
        "applications": []
    })
}

fn arb_semver() -> impl Strategy<Value = String> {
    (1u32..30, 0u32..20, 0u32..50).prop_map(|(a, b, c)| format!("{a}.{b}.{c}"))
}

fn arb_otp_release() -> impl Strategy<Value = String> {
    (24u32..30).prop_map(|v| v.to_string())
}

fn arb_erlang_full_version() -> impl Strategy<Value = String> {
    (24u32..30, 12u32..16, 0u32..10, prop::bool::ANY).prop_map(
        |(otp, erts_major, erts_minor, jit)| {
            let jit_tag = if jit { " [jit]" } else { "" };
            format!("Erlang/OTP {otp} [erts-{erts_major}.{erts_minor}] [source] [64-bit]{jit_tag}")
        },
    )
}

fn arb_crypto_lib_version() -> impl Strategy<Value = String> {
    prop_oneof![
        (1u32..4, 0u32..10, 0u32..10)
            .prop_map(|(a, b, c)| format!("OpenSSL {a}.{b}.{c} 1 Jan 2025")),
        (3u32..4, 0u32..10, 0u32..10).prop_map(|(a, b, c)| format!("LibreSSL {a}.{b}.{c}")),
    ]
}

proptest! {
    #[test]
    fn prop_cluster_node_preserves_version_fields(
        rmq_version in arb_semver(),
        erlang_version in arb_otp_release(),
        erlang_full in arb_erlang_full_version(),
        crypto_lib in arb_crypto_lib_version(),
    ) {
        let mut json = minimal_node_json();
        let obj = json.as_object_mut().unwrap();
        obj.insert("rabbitmq_version".into(), json!(rmq_version));
        obj.insert("erlang_version".into(), json!(erlang_version));
        obj.insert("erlang_full_version".into(), json!(erlang_full));
        obj.insert("crypto_lib_version".into(), json!(crypto_lib));

        let node: ClusterNode = from_value(json).unwrap();
        prop_assert_eq!(node.rabbitmq_version.as_deref(), Some(rmq_version.as_str()));
        prop_assert_eq!(node.erlang_version.as_deref(), Some(erlang_version.as_str()));
        prop_assert_eq!(node.erlang_full_version.as_deref(), Some(erlang_full.as_str()));
        prop_assert_eq!(node.crypto_lib_version.as_deref(), Some(crypto_lib.as_str()));
    }

    #[test]
    fn prop_cluster_node_without_version_fields_always_none(
        uptime in 1u32..1_000_000,
        processors in 1u32..256,
    ) {
        let mut json = minimal_node_json();
        let obj = json.as_object_mut().unwrap();
        obj.insert("uptime".into(), json!(uptime));
        obj.insert("processors".into(), json!(processors));

        let node: ClusterNode = from_value(json).unwrap();
        prop_assert!(node.rabbitmq_version.is_none());
        prop_assert!(node.erlang_version.is_none());
        prop_assert!(node.erlang_full_version.is_none());
        prop_assert!(node.crypto_lib_version.is_none());
    }
}
