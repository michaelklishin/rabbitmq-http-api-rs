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
mod test_helpers;

use rabbitmq_http_client::responses::Overview;
use serde_json::from_str;

#[test]
fn test_unit_overview_with_crypto_lib_version() {
    let json = r#"{
        "cluster_name": "rabbit@node1",
        "node": "rabbit@node1",
        "erlang_full_version": "Erlang/OTP 27 [erts-15.2] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit]",
        "erlang_version": "27",
        "rabbitmq_version": "4.2.4",
        "crypto_lib_version": "OpenSSL 3.4.1 11 Feb 2025",
        "product_name": "RabbitMQ",
        "product_version": "4.2.4",
        "statistics_db_event_queue": 0,
        "churn_rates": {},
        "queue_totals": {},
        "object_totals": {},
        "message_stats": {},
        "listeners": []
    }"#;

    let overview: Overview = from_str(json).unwrap();
    assert_eq!(
        overview.crypto_lib_version.as_deref(),
        Some("OpenSSL 3.4.1 11 Feb 2025")
    );
}

// older RabbitMQ versions do not include this field
#[test]
fn test_unit_overview_without_crypto_lib_version() {
    let json = r#"{
        "cluster_name": "rabbit@node1",
        "node": "rabbit@node1",
        "erlang_full_version": "Erlang/OTP 26 [erts-14.2.5]",
        "erlang_version": "26",
        "rabbitmq_version": "4.1.0",
        "product_name": "RabbitMQ",
        "product_version": "4.1.0",
        "statistics_db_event_queue": 0,
        "churn_rates": {},
        "queue_totals": {},
        "object_totals": {},
        "message_stats": {},
        "listeners": []
    }"#;

    let overview: Overview = from_str(json).unwrap();
    assert!(overview.crypto_lib_version.is_none());
}

#[test]
fn test_unit_overview_crypto_lib_version_libressl() {
    let json = r#"{
        "cluster_name": "rabbit@node1",
        "node": "rabbit@node1",
        "erlang_full_version": "Erlang/OTP 27 [erts-15.2]",
        "erlang_version": "27",
        "rabbitmq_version": "4.2.4",
        "crypto_lib_version": "LibreSSL 3.9.2",
        "product_name": "RabbitMQ",
        "product_version": "4.2.4",
        "statistics_db_event_queue": 0,
        "churn_rates": {},
        "queue_totals": {},
        "object_totals": {},
        "message_stats": {},
        "listeners": []
    }"#;

    let overview: Overview = from_str(json).unwrap();
    assert_eq!(
        overview.crypto_lib_version.as_deref(),
        Some("LibreSSL 3.9.2")
    );
}
