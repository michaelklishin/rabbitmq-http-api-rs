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

use rabbitmq_http_client::responses::{
    Listener, Overview, has_tls_enabled, human_friendly_protocol_name,
};

#[test]
fn test_unit_human_friendly_protocol_name_amqp() {
    assert_eq!(human_friendly_protocol_name("amqp"), "AMQP 1.0 and 0-9-1");
}

#[test]
fn test_unit_human_friendly_protocol_name_amqp_ssl() {
    assert_eq!(
        human_friendly_protocol_name("amqp/ssl"),
        "AMQP 1.0 and 0-9-1 with TLS"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_mqtt() {
    assert_eq!(human_friendly_protocol_name("mqtt"), "MQTT");
}

#[test]
fn test_unit_human_friendly_protocol_name_mqtt_ssl() {
    assert_eq!(human_friendly_protocol_name("mqtt/ssl"), "MQTT with TLS");
}

#[test]
fn test_unit_human_friendly_protocol_name_stomp() {
    assert_eq!(human_friendly_protocol_name("stomp"), "STOMP");
}

#[test]
fn test_unit_human_friendly_protocol_name_stomp_ssl() {
    assert_eq!(human_friendly_protocol_name("stomp/ssl"), "STOMP with TLS");
}

#[test]
fn test_unit_human_friendly_protocol_name_stream() {
    assert_eq!(human_friendly_protocol_name("stream"), "Stream Protocol");
}

#[test]
fn test_unit_human_friendly_protocol_name_stream_ssl() {
    assert_eq!(
        human_friendly_protocol_name("stream/ssl"),
        "Stream Protocol with TLS"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_web_mqtt() {
    assert_eq!(
        human_friendly_protocol_name("http/web-mqtt"),
        "MQTT over WebSockets"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_web_mqtt_ssl() {
    assert_eq!(
        human_friendly_protocol_name("https/web-mqtt"),
        "MQTT over WebSockets with TLS"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_web_stomp() {
    assert_eq!(
        human_friendly_protocol_name("http/web-stomp"),
        "STOMP over WebSockets"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_web_stomp_ssl() {
    assert_eq!(
        human_friendly_protocol_name("https/web-stomp"),
        "STOMP over WebSockets with TLS"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_web_amqp() {
    assert_eq!(
        human_friendly_protocol_name("http/web-amqp"),
        "AMQP 1.0 over WebSockets"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_web_amqp_ssl() {
    assert_eq!(
        human_friendly_protocol_name("https/web-amqp"),
        "AMQP 1.0 over WebSockets with TLS"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_prometheus() {
    assert_eq!(
        human_friendly_protocol_name("http/prometheus"),
        "Prometheus"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_prometheus_ssl() {
    assert_eq!(
        human_friendly_protocol_name("https/prometheus"),
        "Prometheus with TLS"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_http() {
    assert_eq!(human_friendly_protocol_name("http"), "HTTP API");
}

#[test]
fn test_unit_human_friendly_protocol_name_https() {
    assert_eq!(human_friendly_protocol_name("https"), "HTTP API with TLS");
}

#[test]
fn test_unit_human_friendly_protocol_name_clustering() {
    assert_eq!(
        human_friendly_protocol_name("clustering"),
        "Inter-node and CLI Tool Communication"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_unknown_protocol() {
    assert_eq!(
        human_friendly_protocol_name("custom-protocol"),
        "custom-protocol"
    );
}

#[test]
fn test_unit_human_friendly_protocol_name_empty_string() {
    assert_eq!(human_friendly_protocol_name(""), "");
}

#[test]
fn test_unit_has_tls_enabled_for_empty_string() {
    assert!(!has_tls_enabled(""));
}

#[test]
fn test_unit_tls_protocols_have_tls_in_friendly_name() {
    let tls_protocols = [
        "amqp/ssl",
        "mqtt/ssl",
        "stomp/ssl",
        "stream/ssl",
        "https/web-mqtt",
        "https/web-stomp",
        "https/web-amqp",
        "https/prometheus",
        "https",
    ];

    for protocol in &tls_protocols {
        assert!(
            human_friendly_protocol_name(protocol).contains("TLS"),
            "friendly name for {} should contain 'TLS'",
            protocol
        );
    }
}

#[test]
fn test_unit_has_tls_enabled_for_tls_protocols() {
    let tls_protocols = [
        "amqp/ssl",
        "mqtt/ssl",
        "stomp/ssl",
        "stream/ssl",
        "https/web-mqtt",
        "https/web-stomp",
        "https/web-amqp",
        "https/prometheus",
        "https",
    ];

    for protocol in &tls_protocols {
        assert!(
            has_tls_enabled(protocol),
            "{} should have TLS enabled",
            protocol
        );
    }
}

#[test]
fn test_unit_has_tls_enabled_for_plaintext_protocols() {
    let plaintext_protocols = [
        "amqp",
        "mqtt",
        "stomp",
        "stream",
        "http/web-mqtt",
        "http/web-stomp",
        "http/web-amqp",
        "http/prometheus",
        "http",
        "clustering",
    ];

    for protocol in &plaintext_protocols {
        assert!(
            !has_tls_enabled(protocol),
            "{} should not have TLS enabled",
            protocol
        );
    }
}

#[test]
fn test_unit_has_tls_enabled_for_unknown_protocol() {
    assert!(!has_tls_enabled("custom-protocol"));
}

#[test]
fn test_unit_listener_deserializes_with_ip_address_rename() {
    let json = r#"{
        "node": "rabbit@node1",
        "protocol": "amqp",
        "port": 5672,
        "ip_address": "0.0.0.0"
    }"#;

    let listener: Listener = serde_json::from_str(json).unwrap();
    assert_eq!(listener.node, "rabbit@node1");
    assert_eq!(listener.protocol, "amqp");
    assert_eq!(listener.port, 5672);
    assert_eq!(listener.interface, "0.0.0.0");
}

#[test]
fn test_unit_listener_human_friendly_name() {
    let listener = Listener {
        node: "rabbit@node1".to_string(),
        protocol: "amqp".to_string(),
        port: 5672,
        interface: "0.0.0.0".to_string(),
    };

    assert_eq!(listener.human_friendly_name(), "AMQP 1.0 and 0-9-1");
}

#[test]
fn test_unit_listener_has_tls_enabled() {
    let tls_listener = Listener {
        node: "rabbit@node1".to_string(),
        protocol: "amqp/ssl".to_string(),
        port: 5671,
        interface: "0.0.0.0".to_string(),
    };

    let plaintext_listener = Listener {
        node: "rabbit@node1".to_string(),
        protocol: "amqp".to_string(),
        port: 5672,
        interface: "0.0.0.0".to_string(),
    };

    assert!(tls_listener.has_tls_enabled());
    assert!(!plaintext_listener.has_tls_enabled());
}

#[test]
fn test_unit_overview_listeners_defaults_to_empty() {
    let overview = Overview::default();
    assert!(overview.listeners.is_empty());
}

#[test]
fn test_unit_overview_deserializes_with_listeners() {
    let json = r#"{
        "cluster_name": "rabbit@node1",
        "node": "rabbit@node1",
        "erlang_full_version": "Erlang/OTP 27 [jit]",
        "erlang_version": "27.0",
        "rabbitmq_version": "4.1.0",
        "product_name": "RabbitMQ",
        "product_version": "4.1.0",
        "statistics_db_event_queue": 0,
        "churn_rates": {
            "connection_created": 0,
            "connection_closed": 0,
            "queue_declared": 0,
            "queue_created": 0,
            "queue_deleted": 0,
            "channel_created": 0,
            "channel_closed": 0
        },
        "queue_totals": {},
        "object_totals": {},
        "message_stats": {},
        "listeners": [
            {
                "node": "rabbit@node1",
                "protocol": "amqp",
                "port": 5672,
                "ip_address": "0.0.0.0"
            },
            {
                "node": "rabbit@node1",
                "protocol": "amqp/ssl",
                "port": 5671,
                "ip_address": "::"
            }
        ]
    }"#;

    let overview: Overview = serde_json::from_str(json).unwrap();
    assert_eq!(overview.listeners.len(), 2);

    assert_eq!(overview.listeners[0].node, "rabbit@node1");
    assert_eq!(overview.listeners[0].protocol, "amqp");
    assert_eq!(overview.listeners[0].port, 5672);
    assert_eq!(overview.listeners[0].interface, "0.0.0.0");

    assert_eq!(overview.listeners[1].node, "rabbit@node1");
    assert_eq!(overview.listeners[1].protocol, "amqp/ssl");
    assert_eq!(overview.listeners[1].port, 5671);
    assert_eq!(overview.listeners[1].interface, "::");
}

#[test]
fn test_unit_overview_deserializes_without_listeners() {
    let json = r#"{
        "cluster_name": "rabbit@node1",
        "node": "rabbit@node1",
        "erlang_full_version": "Erlang/OTP 27",
        "erlang_version": "27.0",
        "rabbitmq_version": "4.1.0",
        "product_name": "RabbitMQ",
        "product_version": "4.1.0",
        "statistics_db_event_queue": 0,
        "churn_rates": {
            "connection_created": 0,
            "connection_closed": 0,
            "queue_declared": 0,
            "queue_created": 0,
            "queue_deleted": 0,
            "channel_created": 0,
            "channel_closed": 0
        },
        "queue_totals": {},
        "object_totals": {},
        "message_stats": {}
    }"#;

    let overview: Overview = serde_json::from_str(json).unwrap();
    assert!(overview.listeners.is_empty());
}
