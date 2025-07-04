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
use std::fmt;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all(serialize = "lowercase", deserialize = "PascalCase"))]
pub enum SupportedProtocol {
    /// Represents the inter-node and CLI tool communication protocol
    /// (a.k.a. the Erlang distributions protocol)
    Clustering,
    /// Represents both AMQP 1.0 and AMQP 0-9-1 because they share a listener
    #[serde(rename = "amqp")]
    AMQP,
    /// Represents both AMQP 1.0 with TLS enabled and AMQP 0-9-1 with TLS enabled
    #[serde(rename = "amqp/ssl")]
    AMQPWithTLS,
    /// Represents the RabbitMQ Stream protocol
    #[serde(rename = "stream")]
    Stream,
    /// Represents the RabbitMQ Stream protocol with TLS enabled
    #[serde(rename = "stream/ssl")]
    StreamWithTLS,
    /// Represents both MQTTv5 and MQTTv3
    #[serde(rename = "mqtt")]
    MQTT,
    /// Represents both MQTTv5 and MQTTv3 with TLS enabled
    #[serde(rename = "mqtt/ssl")]
    MQTTWithTLS,
    /// Represents STOMP 1.0 through 1.2
    #[serde(rename = "stomp")]
    STOMP,
    /// Represents STOMP 1.0 through 1.2 with TLS enabled
    #[serde(rename = "stomp/ssl")]
    STOMPWithTLS,
    /// Represents AMQP 1.0 over WebSockets
    #[serde(rename = "http/web-amqp")]
    AMQPOverWebSockets,
    /// Represents AMQP 1.0 over WebSockets with TLS enabled
    #[serde(rename = "https/web-amqp")]
    AMQPOverWebSocketsWithTLS,
    /// Represents both MQTTv5 and MQTTv3 over WebSockets
    #[serde(rename = "http/web-mqtt")]
    MQTTOverWebSockets,
    /// Represents both MQTTv5 and MQTTv3 over WebSockets with TLS enabled
    #[serde(rename = "https/web-mqtt")]
    MQTTOverWebSocketsWithTLS,
    /// Represents STOMP 1.0 through 1.2 over WebSockets
    #[serde(rename = "http/web-stomp")]
    STOMPOverWebsockets,
    /// Represents STOMP 1.0 through 1.2 over WebSockets with TLS enabled
    #[serde(rename = "https/web-stomp")]
    STOMPOverWebsocketsWithTLS,
    /// Represents an HTTP endpoint for Prometheus scraping
    #[serde(rename = "http/prometheus")]
    Prometheus,
    /// Represents an HTTP endpoint for Prometheus scraping with TLS enabled
    #[serde(rename = "https/prometheus")]
    PrometheusWithTLS,

    /// Represents an HTTP API endpoint
    #[serde(rename = "http")]
    HTTP,
    /// Represents an HTTP API endpoint with TLS enabled
    #[serde(rename = "https")]
    HTTPWithTLS,
    /// All other protocols, e.g. those coming from 3rd party plugins
    Other(String),
}

pub const SUPPORTED_PROTOCOL_CLUSTERING: &str = "clustering";

pub const SUPPORTED_PROTOCOL_AMQP: &str = "amqp";
pub const SUPPORTED_PROTOCOL_AMQP_WITH_TLS: &str = "amqps";

pub const SUPPORTED_PROTOCOL_STREAM: &str = "stream";
pub const SUPPORTED_PROTOCOL_STREAM_WITH_TLS: &str = "stream/ssl";

pub const SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS: &str = "http/web-amqp";
pub const SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS_WITH_TLS: &str = "https/web-amqp";

pub const SUPPORTED_PROTOCOL_MQTT: &str = "mqtt";
pub const SUPPORTED_PROTOCOL_MQTT_WITH_TLS: &str = "mqtt/ssl";
pub const SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS: &str = "http/web-mqtt";
pub const SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS_WITH_TLS: &str = "https/web-mqtt";

pub const SUPPORTED_PROTOCOL_STOMP: &str = "stomp";
pub const SUPPORTED_PROTOCOL_STOMP_WITH_TLS: &str = "stomp/ssl";
pub const SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS: &str = "http/stomp-mqtt";
pub const SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS_WITH_TLS: &str = "https/stomp-mqtt";

pub const SUPPORTED_PROTOCOL_PROMETHEUS: &str = "http/prometheus";
pub const SUPPORTED_PROTOCOL_PROMETHEUS_WITH_TLS: &str = "https/prometheus";

pub const SUPPORTED_PROTOCOL_HTTP: &str = "http";
pub const SUPPORTED_PROTOCOL_HTTP_WITH_TLS: &str = "https";

impl From<&str> for SupportedProtocol {
    fn from(value: &str) -> Self {
        match value {
            SUPPORTED_PROTOCOL_CLUSTERING => SupportedProtocol::Clustering,
            SUPPORTED_PROTOCOL_AMQP => SupportedProtocol::AMQP,
            SUPPORTED_PROTOCOL_AMQP_WITH_TLS => SupportedProtocol::AMQPWithTLS,
            SUPPORTED_PROTOCOL_STREAM => SupportedProtocol::Stream,
            SUPPORTED_PROTOCOL_STREAM_WITH_TLS => SupportedProtocol::StreamWithTLS,
            SUPPORTED_PROTOCOL_MQTT => SupportedProtocol::MQTT,
            SUPPORTED_PROTOCOL_MQTT_WITH_TLS => SupportedProtocol::MQTTWithTLS,
            SUPPORTED_PROTOCOL_STOMP => SupportedProtocol::STOMP,
            SUPPORTED_PROTOCOL_STOMP_WITH_TLS => SupportedProtocol::STOMPWithTLS,
            SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS => SupportedProtocol::AMQPOverWebSockets,
            SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS_WITH_TLS => {
                SupportedProtocol::AMQPOverWebSockets
            }
            SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS => SupportedProtocol::MQTTOverWebSockets,
            SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS_WITH_TLS => {
                SupportedProtocol::MQTTOverWebSocketsWithTLS
            }
            SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS => SupportedProtocol::STOMPOverWebsockets,
            SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS_WITH_TLS => {
                SupportedProtocol::STOMPOverWebsocketsWithTLS
            }
            SUPPORTED_PROTOCOL_PROMETHEUS => SupportedProtocol::Prometheus,
            SUPPORTED_PROTOCOL_PROMETHEUS_WITH_TLS => SupportedProtocol::PrometheusWithTLS,
            SUPPORTED_PROTOCOL_HTTP => SupportedProtocol::HTTP,
            SUPPORTED_PROTOCOL_HTTP_WITH_TLS => SupportedProtocol::HTTPWithTLS,
            other => SupportedProtocol::Other(other.to_owned()),
        }
    }
}

impl From<String> for SupportedProtocol {
    fn from(value: String) -> Self {
        SupportedProtocol::from(value.as_str())
    }
}

impl From<SupportedProtocol> for String {
    fn from(value: SupportedProtocol) -> String {
        match value {
            SupportedProtocol::Clustering => SUPPORTED_PROTOCOL_CLUSTERING.to_owned(),
            SupportedProtocol::AMQP => SUPPORTED_PROTOCOL_AMQP.to_owned(),
            SupportedProtocol::AMQPWithTLS => SUPPORTED_PROTOCOL_AMQP_WITH_TLS.to_owned(),
            SupportedProtocol::Stream => SUPPORTED_PROTOCOL_STREAM.to_owned(),
            SupportedProtocol::StreamWithTLS => SUPPORTED_PROTOCOL_STREAM_WITH_TLS.to_owned(),
            SupportedProtocol::MQTT => SUPPORTED_PROTOCOL_MQTT.to_owned(),
            SupportedProtocol::MQTTWithTLS => SUPPORTED_PROTOCOL_MQTT_WITH_TLS.to_owned(),
            SupportedProtocol::STOMP => SUPPORTED_PROTOCOL_STOMP.to_owned(),
            SupportedProtocol::STOMPWithTLS => SUPPORTED_PROTOCOL_STOMP_WITH_TLS.to_owned(),
            SupportedProtocol::AMQPOverWebSockets => {
                SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS.to_owned()
            }
            SupportedProtocol::AMQPOverWebSocketsWithTLS => {
                SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS_WITH_TLS.to_owned()
            }
            SupportedProtocol::MQTTOverWebSockets => {
                SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS.to_owned()
            }
            SupportedProtocol::MQTTOverWebSocketsWithTLS => {
                SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS_WITH_TLS.to_owned()
            }
            SupportedProtocol::STOMPOverWebsockets => {
                SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS.to_owned()
            }
            SupportedProtocol::STOMPOverWebsocketsWithTLS => {
                SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS_WITH_TLS.to_owned()
            }
            SupportedProtocol::Prometheus => SUPPORTED_PROTOCOL_PROMETHEUS.to_owned(),
            SupportedProtocol::PrometheusWithTLS => {
                SUPPORTED_PROTOCOL_PROMETHEUS_WITH_TLS.to_owned()
            }
            SupportedProtocol::HTTP => SUPPORTED_PROTOCOL_HTTP.to_owned(),
            SupportedProtocol::HTTPWithTLS => SUPPORTED_PROTOCOL_HTTP_WITH_TLS.to_owned(),
            SupportedProtocol::Other(s) => s,
        }
    }
}

impl From<&SupportedProtocol> for String {
    fn from(value: &SupportedProtocol) -> Self {
        match value {
            SupportedProtocol::Clustering => SUPPORTED_PROTOCOL_CLUSTERING.to_owned(),
            SupportedProtocol::AMQP => SUPPORTED_PROTOCOL_AMQP.to_owned(),
            SupportedProtocol::AMQPWithTLS => SUPPORTED_PROTOCOL_AMQP_WITH_TLS.to_owned(),
            SupportedProtocol::Stream => SUPPORTED_PROTOCOL_STREAM.to_owned(),
            SupportedProtocol::StreamWithTLS => SUPPORTED_PROTOCOL_STREAM_WITH_TLS.to_owned(),
            SupportedProtocol::MQTT => SUPPORTED_PROTOCOL_MQTT.to_owned(),
            SupportedProtocol::MQTTWithTLS => SUPPORTED_PROTOCOL_MQTT_WITH_TLS.to_owned(),
            SupportedProtocol::STOMP => SUPPORTED_PROTOCOL_STOMP.to_owned(),
            SupportedProtocol::STOMPWithTLS => SUPPORTED_PROTOCOL_STOMP_WITH_TLS.to_owned(),
            SupportedProtocol::AMQPOverWebSockets => {
                SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS.to_owned()
            }
            SupportedProtocol::AMQPOverWebSocketsWithTLS => {
                SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS_WITH_TLS.to_owned()
            }
            SupportedProtocol::MQTTOverWebSockets => {
                SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS.to_owned()
            }
            SupportedProtocol::MQTTOverWebSocketsWithTLS => {
                SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS_WITH_TLS.to_owned()
            }
            SupportedProtocol::STOMPOverWebsockets => {
                SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS.to_owned()
            }
            SupportedProtocol::STOMPOverWebsocketsWithTLS => {
                SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS_WITH_TLS.to_owned()
            }
            SupportedProtocol::Prometheus => SUPPORTED_PROTOCOL_PROMETHEUS.to_owned(),
            SupportedProtocol::PrometheusWithTLS => {
                SUPPORTED_PROTOCOL_PROMETHEUS_WITH_TLS.to_owned()
            }
            SupportedProtocol::HTTP => SUPPORTED_PROTOCOL_HTTP.to_owned(),
            SupportedProtocol::HTTPWithTLS => SUPPORTED_PROTOCOL_HTTP_WITH_TLS.to_owned(),
            SupportedProtocol::Other(s) => (*s).clone(),
        }
    }
}

impl fmt::Display for SupportedProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let proto: String = self.into();
        write!(f, "{proto}")
    }
}

/// Exchange types. Most variants are for exchange types included with modern RabbitMQ distributions.
/// For custom types provided by 3rd party plugins, use the `Plugin(String)` variant.
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all(serialize = "lowercase", deserialize = "PascalCase"))]
pub enum ExchangeType {
    /// Fanout exchange
    Fanout,
    /// Topic exchange
    Topic,
    /// Direct exchange
    Direct,
    /// Headers exchange
    Headers,
    /// Consistent hashing (consistent hash) exchange
    #[serde(rename = "x-consistent-hash")]
    ConsistentHashing,
    /// Modulus hash, ships with the 'rabbitmq-sharding' plugin
    #[serde(rename = "x-modulus-hash")]
    ModulusHash,
    /// Random exchange
    #[serde(rename = "x-random")]
    Random,
    /// Local random exchange
    #[serde(rename = "x-local-random")]
    LocalRandom,
    /// JMS topic exchange
    #[serde(rename = "x-jms-topic")]
    JmsTopic,
    /// Recent history exchange
    #[serde(rename = "x-recent-history")]
    RecentHistory,
    /// x-delayed-message exchange
    #[serde(rename = "x-delayed-message")]
    DelayedMessage,
    /// x-message-deduplication
    #[serde(rename = "x-message-deduplication")]
    MessageDeduplication,
    /// Other types
    #[serde(untagged)]
    Plugin(String),
}

pub const X_ARGUMENT_KEY_X_QUEUE_TYPE: &str = "x-queue-type";

pub const EXCHANGE_TYPE_FANOUT: &str = "fanout";
pub const EXCHANGE_TYPE_TOPIC: &str = "topic";
pub const EXCHANGE_TYPE_DIRECT: &str = "direct";
pub const EXCHANGE_TYPE_HEADERS: &str = "headers";
pub const EXCHANGE_TYPE_CONSISTENT_HASHING: &str = "x-consistent-hash";
pub const EXCHANGE_TYPE_MODULUS_HASH: &str = "x-modulus-hash";
pub const EXCHANGE_TYPE_RANDOM: &str = "x-random";
pub const EXCHANGE_TYPE_JMS_TOPIC: &str = "x-jms-topic";
pub const EXCHANGE_TYPE_LOCAL_RANDOM: &str = "x-local-random";
pub const EXCHANGE_TYPE_RECENT_HISTORY: &str = "x-recent-history";
pub const EXCHANGE_TYPE_DELAYED_MESSAGE: &str = "x-delayed-message";
pub const EXCHANGE_TYPE_MESSAGE_DEDUPLICATION: &str = "x-message-deduplication";

impl From<&str> for ExchangeType {
    fn from(value: &str) -> Self {
        match value {
            EXCHANGE_TYPE_FANOUT => ExchangeType::Fanout,
            EXCHANGE_TYPE_TOPIC => ExchangeType::Topic,
            EXCHANGE_TYPE_DIRECT => ExchangeType::Direct,
            EXCHANGE_TYPE_HEADERS => ExchangeType::Headers,
            EXCHANGE_TYPE_CONSISTENT_HASHING => ExchangeType::ConsistentHashing,
            EXCHANGE_TYPE_MODULUS_HASH => ExchangeType::ModulusHash,
            EXCHANGE_TYPE_RANDOM => ExchangeType::Random,
            EXCHANGE_TYPE_LOCAL_RANDOM => ExchangeType::LocalRandom,
            EXCHANGE_TYPE_JMS_TOPIC => ExchangeType::JmsTopic,
            EXCHANGE_TYPE_RECENT_HISTORY => ExchangeType::RecentHistory,
            EXCHANGE_TYPE_DELAYED_MESSAGE => ExchangeType::DelayedMessage,
            EXCHANGE_TYPE_MESSAGE_DEDUPLICATION => ExchangeType::MessageDeduplication,
            other => ExchangeType::Plugin(other.to_owned()),
        }
    }
}

impl From<String> for ExchangeType {
    fn from(value: String) -> Self {
        ExchangeType::from(value.as_str())
    }
}

impl From<ExchangeType> for String {
    fn from(value: ExchangeType) -> String {
        match value {
            ExchangeType::Fanout => EXCHANGE_TYPE_FANOUT.to_owned(),
            ExchangeType::Topic => EXCHANGE_TYPE_TOPIC.to_owned(),
            ExchangeType::Direct => EXCHANGE_TYPE_DIRECT.to_owned(),
            ExchangeType::Headers => EXCHANGE_TYPE_HEADERS.to_owned(),
            ExchangeType::ConsistentHashing => EXCHANGE_TYPE_CONSISTENT_HASHING.to_owned(),
            ExchangeType::ModulusHash => EXCHANGE_TYPE_MODULUS_HASH.to_owned(),
            ExchangeType::Random => EXCHANGE_TYPE_RANDOM.to_owned(),
            ExchangeType::LocalRandom => EXCHANGE_TYPE_LOCAL_RANDOM.to_owned(),
            ExchangeType::JmsTopic => EXCHANGE_TYPE_JMS_TOPIC.to_owned(),
            ExchangeType::RecentHistory => EXCHANGE_TYPE_RECENT_HISTORY.to_owned(),
            ExchangeType::DelayedMessage => EXCHANGE_TYPE_DELAYED_MESSAGE.to_owned(),
            ExchangeType::MessageDeduplication => EXCHANGE_TYPE_MESSAGE_DEDUPLICATION.to_owned(),
            ExchangeType::Plugin(exchange_type) => exchange_type,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all(serialize = "lowercase", deserialize = "PascalCase"))]
pub enum QueueType {
    #[default]
    Classic,
    Quorum,
    Stream,
    // Tanzu RabbitMQ-specific
    Delayed,
    // A type this client is not aware of
    #[serde(untagged)]
    Unsupported(String),
}

impl Display for QueueType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueueType::Classic => write!(f, "classic"),
            QueueType::Quorum => write!(f, "quorum"),
            QueueType::Stream => write!(f, "stream"),
            QueueType::Delayed => write!(f, "delayed"),
            QueueType::Unsupported(s) => write!(f, "{s}"),
        }
    }
}

impl From<&str> for QueueType {
    fn from(value: &str) -> Self {
        let val = value.to_ascii_lowercase();
        match val.as_str() {
            "classic" => QueueType::Classic,
            "quorum" => QueueType::Quorum,
            "stream" => QueueType::Stream,
            "delayed" => QueueType::Delayed,
            _ => QueueType::Unsupported(value.to_owned()),
        }
    }
}

impl From<String> for QueueType {
    fn from(value: String) -> Self {
        let val = value.to_ascii_lowercase();
        match val.as_str() {
            "classic" => QueueType::Classic,
            "quorum" => QueueType::Quorum,
            "stream" => QueueType::Stream,
            _ => QueueType::Unsupported(value),
        }
    }
}

impl From<QueueType> for String {
    fn from(value: QueueType) -> Self {
        match value {
            QueueType::Classic => "classic".to_owned(),
            QueueType::Quorum => "quorum".to_owned(),
            QueueType::Stream => "stream".to_owned(),
            QueueType::Delayed => "delayed".to_owned(),
            QueueType::Unsupported(val) => val.to_owned(),
        }
    }
}

/// Binding destination can be either a queue or another exchange
/// (in the case of [exchange-to-exchange bindings](https://rabbitmq.com/docs/e2e/)).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BindingDestinationType {
    Queue,
    Exchange,
}

impl fmt::Display for BindingDestinationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BindingDestinationType::Queue => write!(f, "queue")?,
            BindingDestinationType::Exchange => write!(f, "exchange")?,
        };

        Ok(())
    }
}

impl BindingDestinationType {
    pub fn path_appreviation(&self) -> String {
        match *self {
            BindingDestinationType::Queue => "q".to_owned(),
            BindingDestinationType::Exchange => "e".to_owned(),
        }
    }
}

impl From<&str> for BindingDestinationType {
    fn from(value: &str) -> Self {
        match value {
            "queue" => BindingDestinationType::Queue,
            "exchange" => BindingDestinationType::Exchange,
            _ => BindingDestinationType::Queue,
        }
    }
}

impl From<String> for BindingDestinationType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "queue" => BindingDestinationType::Queue,
            "exchange" => BindingDestinationType::Exchange,
            _ => BindingDestinationType::Queue,
        }
    }
}

impl From<BindingDestinationType> for String {
    fn from(value: BindingDestinationType) -> Self {
        match value {
            BindingDestinationType::Queue => "queue".to_owned(),
            BindingDestinationType::Exchange => "exchange".to_owned(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PolicyTarget {
    Queues,
    ClassicQueues,
    QuorumQueues,
    Streams,
    Exchanges,
    All,
}

impl From<QueueType> for PolicyTarget {
    fn from(value: QueueType) -> Self {
        match value {
            QueueType::Classic => PolicyTarget::ClassicQueues,
            QueueType::Quorum => PolicyTarget::QuorumQueues,
            QueueType::Stream => PolicyTarget::Streams,
            QueueType::Delayed => PolicyTarget::Queues,
            QueueType::Unsupported(_) => PolicyTarget::Queues,
        }
    }
}

impl PolicyTarget {
    // Returns true if this policy target includes the target category.
    // For example, [`PolicyTarget::Queue`] matches [`PolicyTarget::ClassicQueues`], [`PolicyTarget::QuorumQueues`],
    // [`PolicyTarget::Stream`], [`PolicyTarget::Queue`] but not [`PolicyTarget::Exchanges`].
    pub fn does_apply_to(&self, other: PolicyTarget) -> bool {
        match (self, other) {
            (PolicyTarget::All, _) => true,
            (_, PolicyTarget::All) => true,
            // queues includes
            (PolicyTarget::Queues, PolicyTarget::Queues) => true,
            (PolicyTarget::Queues, PolicyTarget::ClassicQueues) => true,
            (PolicyTarget::Queues, PolicyTarget::QuorumQueues) => true,
            // streams are included into "queues"
            (PolicyTarget::Queues, PolicyTarget::Streams) => true,
            (PolicyTarget::ClassicQueues, PolicyTarget::ClassicQueues) => true,
            (PolicyTarget::QuorumQueues, PolicyTarget::QuorumQueues) => true,
            (PolicyTarget::Streams, PolicyTarget::Streams) => true,
            (PolicyTarget::Exchanges, PolicyTarget::Exchanges) => true,
            _ => false,
        }
    }
}

impl fmt::Display for PolicyTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))?;

        Ok(())
    }
}

impl From<&str> for PolicyTarget {
    fn from(value: &str) -> Self {
        match value {
            "queues" => PolicyTarget::Queues,
            "queue" => PolicyTarget::Queues,
            "classic_queues" => PolicyTarget::ClassicQueues,
            "classic_queue" => PolicyTarget::ClassicQueues,
            "quorum_queues" => PolicyTarget::QuorumQueues,
            "quorum_queue" => PolicyTarget::QuorumQueues,
            "streams" => PolicyTarget::Streams,
            "stream" => PolicyTarget::Streams,
            "exchanges" => PolicyTarget::Exchanges,
            "exchange" => PolicyTarget::Exchanges,
            "all" => PolicyTarget::All,
            _ => PolicyTarget::Queues,
        }
    }
}

impl From<String> for PolicyTarget {
    fn from(value: String) -> Self {
        match value.as_str() {
            "queues" => PolicyTarget::Queues,
            "queue" => PolicyTarget::Queues,
            "classic_queues" => PolicyTarget::ClassicQueues,
            "classic_queue" => PolicyTarget::ClassicQueues,
            "quorum_queues" => PolicyTarget::QuorumQueues,
            "quorum_queue" => PolicyTarget::QuorumQueues,
            "streams" => PolicyTarget::Streams,
            "stream" => PolicyTarget::Streams,
            "exchanges" => PolicyTarget::Exchanges,
            "exchange" => PolicyTarget::Exchanges,
            "all" => PolicyTarget::All,
            _ => PolicyTarget::Queues,
        }
    }
}

impl From<PolicyTarget> for String {
    fn from(value: PolicyTarget) -> Self {
        match value {
            PolicyTarget::Queues => "queues".to_owned(),
            PolicyTarget::ClassicQueues => "classic_queues".to_owned(),
            PolicyTarget::QuorumQueues => "quorum_queues".to_owned(),
            PolicyTarget::Streams => "streams".to_owned(),
            PolicyTarget::Exchanges => "exchanges".to_owned(),
            PolicyTarget::All => "all".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum VirtualHostLimitTarget {
    MaxConnections,
    MaxQueues,
}

impl AsRef<str> for VirtualHostLimitTarget {
    fn as_ref(&self) -> &str {
        match self {
            VirtualHostLimitTarget::MaxConnections => "max-connections",
            VirtualHostLimitTarget::MaxQueues => "max-queues",
        }
    }
}

impl From<&str> for VirtualHostLimitTarget {
    fn from(value: &str) -> Self {
        match value {
            "max-connections" => VirtualHostLimitTarget::MaxConnections,
            "max-queues" => VirtualHostLimitTarget::MaxQueues,
            _ => VirtualHostLimitTarget::MaxConnections,
        }
    }
}

impl From<String> for VirtualHostLimitTarget {
    fn from(value: String) -> Self {
        match value.as_str() {
            "max-connections" => VirtualHostLimitTarget::MaxConnections,
            "max-queues" => VirtualHostLimitTarget::MaxQueues,
            _ => VirtualHostLimitTarget::MaxConnections,
        }
    }
}

impl From<VirtualHostLimitTarget> for String {
    fn from(value: VirtualHostLimitTarget) -> Self {
        value.as_ref().to_string()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged, rename_all = "kebab-case")]
pub enum UserLimitTarget {
    MaxConnections,
    MaxChannels,
}

impl AsRef<str> for UserLimitTarget {
    fn as_ref(&self) -> &str {
        match self {
            UserLimitTarget::MaxConnections => "max-connections",
            UserLimitTarget::MaxChannels => "max-channels",
        }
    }
}

impl From<&str> for UserLimitTarget {
    fn from(value: &str) -> Self {
        match value {
            "max-connections" => UserLimitTarget::MaxConnections,
            "max-channels" => UserLimitTarget::MaxChannels,
            _ => UserLimitTarget::MaxConnections,
        }
    }
}

impl From<String> for UserLimitTarget {
    fn from(value: String) -> Self {
        match value.as_str() {
            "max-connections" => UserLimitTarget::MaxConnections,
            "max-channels" => UserLimitTarget::MaxChannels,
            _ => UserLimitTarget::MaxConnections,
        }
    }
}

impl From<UserLimitTarget> for String {
    fn from(value: UserLimitTarget) -> Self {
        value.as_ref().to_string()
    }
}

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum MessageTransferAcknowledgementMode {
    #[serde(rename = "no-ack")]
    Immediate,
    #[serde(rename = "on-publish")]
    WhenPublished,
    #[default]
    #[serde(rename = "on-confirm")]
    WhenConfirmed,
}

impl From<&str> for MessageTransferAcknowledgementMode {
    fn from(value: &str) -> Self {
        match value {
            "no-ack" => MessageTransferAcknowledgementMode::Immediate,
            "on-publish" => MessageTransferAcknowledgementMode::WhenPublished,
            "on-confirm" => MessageTransferAcknowledgementMode::WhenConfirmed,
            _ => MessageTransferAcknowledgementMode::default(),
        }
    }
}

impl From<String> for MessageTransferAcknowledgementMode {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for MessageTransferAcknowledgementMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageTransferAcknowledgementMode::Immediate => write!(f, "no-ack"),
            MessageTransferAcknowledgementMode::WhenPublished => write!(f, "on-publish"),
            MessageTransferAcknowledgementMode::WhenConfirmed => write!(f, "on-confirm"),
        }
    }
}

impl From<MessageTransferAcknowledgementMode> for String {
    fn from(value: MessageTransferAcknowledgementMode) -> Self {
        value.to_string()
    }
}
