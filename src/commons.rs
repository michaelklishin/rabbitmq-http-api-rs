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
use std::str::FromStr;

use serde::{Deserialize, Serialize};
#[cfg(feature = "zeroize")]
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Configuration for HTTP request retry behavior.
///
/// This controls how many times HTTP requests will be retried and the delay between attempts.
/// By default, no retries are performed to maintain backward compatibility.
#[derive(Debug, Clone, PartialEq)]
pub struct RetrySettings {
    /// Maximum number of retry attempts (not including the initial request).
    /// A value of 0 means "no retries will be performed", 1 means one retry attempt, etc.
    pub max_attempts: u32,
    /// A fixed delay between retry attempts in milliseconds.
    pub delay_ms: u64,
}

impl Default for RetrySettings {
    fn default() -> Self {
        Self {
            // No retries by default for backward compatibility
            max_attempts: 0,
            delay_ms: 1000,
        }
    }
}

/// Default page size used by the RabbitMQ HTTP API.
pub const DEFAULT_PAGE_SIZE: usize = 100;

/// Maximum page size allowed by the RabbitMQ HTTP API.
pub const MAX_PAGE_SIZE: usize = 500;

/// Pagination parameters for list endpoints.
///
/// The RabbitMQ HTTP API supports pagination with `page` (1-indexed) and `page_size` parameters.
/// Default page size is 100, maximum is 500. Construction methods clamp `page_size` to
/// [`MAX_PAGE_SIZE`] if it exceeds the limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[must_use]
pub struct PaginationParams {
    pub page: Option<usize>,
    pub page_size: Option<usize>,
}

impl PaginationParams {
    /// Creates pagination parameters with the given page number and page size.
    ///
    /// Values exceeding [`MAX_PAGE_SIZE`] (500) are clamped to the maximum.
    pub fn new(page: usize, page_size: usize) -> Self {
        Self {
            page: Some(page),
            page_size: Some(page_size.min(MAX_PAGE_SIZE)),
        }
    }

    /// Creates pagination parameters for the first page with the default page size (100).
    pub fn first_page_default() -> Self {
        Self::new(1, DEFAULT_PAGE_SIZE)
    }

    /// Creates pagination parameters for the first page with a custom page size.
    ///
    /// Values exceeding [`MAX_PAGE_SIZE`] (500) are clamped to the maximum.
    pub fn first_page(page_size: usize) -> Self {
        Self::new(1, page_size)
    }

    pub fn to_query_string(&self) -> Option<String> {
        match (self.page, self.page_size) {
            (Some(page), Some(size)) => Some(format!("page={}&page_size={}", page, size)),
            (Some(page), None) => Some(format!("page={}", page)),
            (None, Some(size)) => Some(format!("page_size={}", size)),
            (None, None) => None,
        }
    }

    /// Returns pagination parameters for the next page, preserving the current page size.
    ///
    /// Returns `None` if no page is set (pagination not initialized).
    pub fn next_page(&self) -> Option<Self> {
        self.page.map(|p| Self {
            page: Some(p + 1),
            page_size: self.page_size,
        })
    }

    /// Returns the current page number, or `None` if pagination is not configured.
    pub fn current_page(&self) -> Option<usize> {
        self.page
    }

    /// Returns the page size, or `None` if not explicitly set.
    pub fn page_size(&self) -> Option<usize> {
        self.page_size
    }

    /// Returns true if results indicate this is the last page of data.
    ///
    /// Returns `true` if the results slice is empty or contains fewer items than
    /// the page size, indicating no more data is available.
    ///
    /// This is a helper for implementing pagination loops:
    /// ```ignore
    /// let mut params = PaginationParams::first_page_default();
    /// loop {
    ///     let results = client.list_queues_paged(&params)?;
    ///     // process results...
    ///     if params.is_last_page(&results) {
    ///         break;
    ///     }
    ///     params = params.next_page().unwrap();
    /// }
    /// ```
    pub fn is_last_page<T>(&self, results: &[T]) -> bool {
        let effective_page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE).max(1);
        results.is_empty() || results.len() < effective_page_size
    }
}

pub type Username = String;
pub type VirtualHostName = String;
pub type PermissionPattern = String;

pub type ChannelId = u32;

/// A password wrapper that securely zeroizes its contents on drop.
///
/// Use this type as the password parameter when constructing a
/// [`crate::api::Client`] or [`crate::blocking_api::Client`] to ensure
/// that credentials are cleared from memory when no longer needed.
///
/// `Password` implements [`Display`] transparently, so it can be used
/// anywhere a `Display` type is expected, including the client constructors.
///
/// ```rust
/// use rabbitmq_http_client::commons::Password;
///
/// let password = Password::from("s3cret");
/// assert_eq!(format!("{}", password), "s3cret");
/// // password is zeroized when dropped
/// ```
#[cfg(feature = "zeroize")]
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Password(String);

#[cfg(feature = "zeroize")]
impl Password {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[cfg(feature = "zeroize")]
impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Password([REDACTED])")
    }
}

#[cfg(feature = "zeroize")]
impl Display for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "zeroize")]
impl From<String> for Password {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(feature = "zeroize")]
impl From<&str> for Password {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
#[serde(rename_all(serialize = "lowercase", deserialize = "PascalCase"))]
pub enum SupportedProtocol {
    /// Represents the inter-node and CLI tool communication protocol
    /// (a.k.a. the Erlang distribution protocol)
    Clustering,
    /// Represents both AMQP 1.0 and AMQP 0-9-1 because they share a listener.
    /// Some API endpoints (e.g. auth attempts) return "amqp091" or "amqp10" specifically.
    #[serde(rename = "amqp", alias = "amqp091", alias = "amqp10")]
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
                SupportedProtocol::AMQPOverWebSocketsWithTLS
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
        Self::from(value.as_str())
    }
}

impl FromStr for SupportedProtocol {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for SupportedProtocol {
    fn as_ref(&self) -> &str {
        match self {
            SupportedProtocol::Clustering => SUPPORTED_PROTOCOL_CLUSTERING,
            SupportedProtocol::AMQP => SUPPORTED_PROTOCOL_AMQP,
            SupportedProtocol::AMQPWithTLS => SUPPORTED_PROTOCOL_AMQP_WITH_TLS,
            SupportedProtocol::Stream => SUPPORTED_PROTOCOL_STREAM,
            SupportedProtocol::StreamWithTLS => SUPPORTED_PROTOCOL_STREAM_WITH_TLS,
            SupportedProtocol::MQTT => SUPPORTED_PROTOCOL_MQTT,
            SupportedProtocol::MQTTWithTLS => SUPPORTED_PROTOCOL_MQTT_WITH_TLS,
            SupportedProtocol::STOMP => SUPPORTED_PROTOCOL_STOMP,
            SupportedProtocol::STOMPWithTLS => SUPPORTED_PROTOCOL_STOMP_WITH_TLS,
            SupportedProtocol::AMQPOverWebSockets => SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS,
            SupportedProtocol::AMQPOverWebSocketsWithTLS => {
                SUPPORTED_PROTOCOL_AMQP_OVER_WEBSOCKETS_WITH_TLS
            }
            SupportedProtocol::MQTTOverWebSockets => SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS,
            SupportedProtocol::MQTTOverWebSocketsWithTLS => {
                SUPPORTED_PROTOCOL_MQTT_OVER_WEBSOCKETS_WITH_TLS
            }
            SupportedProtocol::STOMPOverWebsockets => SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS,
            SupportedProtocol::STOMPOverWebsocketsWithTLS => {
                SUPPORTED_PROTOCOL_STOMP_OVER_WEBSOCKETS_WITH_TLS
            }
            SupportedProtocol::Prometheus => SUPPORTED_PROTOCOL_PROMETHEUS,
            SupportedProtocol::PrometheusWithTLS => SUPPORTED_PROTOCOL_PROMETHEUS_WITH_TLS,
            SupportedProtocol::HTTP => SUPPORTED_PROTOCOL_HTTP,
            SupportedProtocol::HTTPWithTLS => SUPPORTED_PROTOCOL_HTTP_WITH_TLS,
            SupportedProtocol::Other(s) => s.as_str(),
        }
    }
}

impl From<SupportedProtocol> for String {
    fn from(value: SupportedProtocol) -> String {
        match value {
            SupportedProtocol::Other(s) => s,
            other => other.as_ref().to_owned(),
        }
    }
}

impl From<&SupportedProtocol> for String {
    fn from(value: &SupportedProtocol) -> Self {
        value.clone().into()
    }
}

impl fmt::Display for SupportedProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
    }
}

/// Exchange types. Most variants are for exchange types included with modern RabbitMQ distributions.
/// For custom types provided by 3rd party plugins, use the `Plugin(String)` variant.
#[derive(Eq, PartialEq, Serialize, Deserialize, Debug, Clone, Hash)]
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
pub const X_ARGUMENT_KEY_X_OVERFLOW: &str = "x-overflow";

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
        Self::from(value.as_str())
    }
}

impl FromStr for ExchangeType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for ExchangeType {
    fn as_ref(&self) -> &str {
        match self {
            ExchangeType::Fanout => EXCHANGE_TYPE_FANOUT,
            ExchangeType::Topic => EXCHANGE_TYPE_TOPIC,
            ExchangeType::Direct => EXCHANGE_TYPE_DIRECT,
            ExchangeType::Headers => EXCHANGE_TYPE_HEADERS,
            ExchangeType::ConsistentHashing => EXCHANGE_TYPE_CONSISTENT_HASHING,
            ExchangeType::ModulusHash => EXCHANGE_TYPE_MODULUS_HASH,
            ExchangeType::Random => EXCHANGE_TYPE_RANDOM,
            ExchangeType::LocalRandom => EXCHANGE_TYPE_LOCAL_RANDOM,
            ExchangeType::JmsTopic => EXCHANGE_TYPE_JMS_TOPIC,
            ExchangeType::RecentHistory => EXCHANGE_TYPE_RECENT_HISTORY,
            ExchangeType::DelayedMessage => EXCHANGE_TYPE_DELAYED_MESSAGE,
            ExchangeType::MessageDeduplication => EXCHANGE_TYPE_MESSAGE_DEDUPLICATION,
            ExchangeType::Plugin(s) => s,
        }
    }
}

impl From<ExchangeType> for String {
    fn from(value: ExchangeType) -> String {
        match value {
            ExchangeType::Plugin(s) => s,
            other => other.as_ref().to_owned(),
        }
    }
}

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone, Default, Hash)]
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
        Self::from(value.as_str())
    }
}

impl FromStr for QueueType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for QueueType {
    fn as_ref(&self) -> &str {
        match self {
            QueueType::Classic => "classic",
            QueueType::Quorum => "quorum",
            QueueType::Stream => "stream",
            QueueType::Delayed => "delayed",
            QueueType::Unsupported(s) => s.as_str(),
        }
    }
}

impl From<QueueType> for String {
    fn from(value: QueueType) -> Self {
        match value {
            QueueType::Unsupported(s) => s,
            other => other.as_ref().to_owned(),
        }
    }
}

/// Binding destination can be either a queue or another exchange
/// (in the case of [exchange-to-exchange bindings](https://rabbitmq.com/docs/e2e/)).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
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
    pub fn path_abbreviation(&self) -> String {
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
        Self::from(value.as_str())
    }
}

impl FromStr for BindingDestinationType {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

/// Represents whether a binding resource is a source or destination in binding operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingVertex {
    Source,
    Destination,
}

impl AsRef<str> for BindingVertex {
    fn as_ref(&self) -> &str {
        match self {
            Self::Source => "source",
            Self::Destination => "destination",
        }
    }
}

impl AsRef<str> for BindingDestinationType {
    fn as_ref(&self) -> &str {
        match self {
            BindingDestinationType::Queue => "queue",
            BindingDestinationType::Exchange => "exchange",
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

#[derive(Eq, PartialEq, Debug, Deserialize, Serialize, Clone, Copy)]
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

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum OverflowBehavior {
    DropHead,
    RejectPublish,
    RejectPublishDlx,
}

pub const OVERFLOW_REJECT_PUBLISH: &str = "reject-publish";
pub const OVERFLOW_REJECT_PUBLISH_DLX: &str = "reject-publish-dlx";
pub const OVERFLOW_DROP_HEAD: &str = "drop-head";

impl From<OverflowBehavior> for &str {
    fn from(value: OverflowBehavior) -> Self {
        match value {
            OverflowBehavior::DropHead => OVERFLOW_DROP_HEAD,
            OverflowBehavior::RejectPublish => OVERFLOW_REJECT_PUBLISH,
            OverflowBehavior::RejectPublishDlx => OVERFLOW_REJECT_PUBLISH_DLX,
        }
    }
}

impl From<&str> for OverflowBehavior {
    fn from(s: &str) -> Self {
        match s {
            OVERFLOW_DROP_HEAD => OverflowBehavior::DropHead,
            OVERFLOW_REJECT_PUBLISH => OverflowBehavior::RejectPublish,
            OVERFLOW_REJECT_PUBLISH_DLX => OverflowBehavior::RejectPublishDlx,
            _ => OverflowBehavior::DropHead,
        }
    }
}

impl From<String> for OverflowBehavior {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl FromStr for OverflowBehavior {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<OverflowBehavior> for String {
    fn from(value: OverflowBehavior) -> Self {
        let s: &str = value.into();
        s.to_owned()
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
        write!(f, "{}", Into::<String>::into(*self))?;

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
        Self::from(value.as_str())
    }
}

impl FromStr for PolicyTarget {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for PolicyTarget {
    fn as_ref(&self) -> &str {
        match self {
            PolicyTarget::Queues => "queues",
            PolicyTarget::ClassicQueues => "classic_queues",
            PolicyTarget::QuorumQueues => "quorum_queues",
            PolicyTarget::Streams => "streams",
            PolicyTarget::Exchanges => "exchanges",
            PolicyTarget::All => "all",
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
        Self::from(value.as_str())
    }
}

impl FromStr for VirtualHostLimitTarget {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
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
        Self::from(value.as_str())
    }
}

impl FromStr for UserLimitTarget {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl From<UserLimitTarget> for String {
    fn from(value: UserLimitTarget) -> Self {
        value.as_ref().to_string()
    }
}

/// TLS peer verification modes used by RabbitMQ.
/// See [TLS Support Guide](https://www.rabbitmq.com/docs/ssl#peer-verification) to learn more.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum TlsPeerVerificationMode {
    #[default]
    /// Enables peer certificate chain verification
    Enabled,
    /// Disables peer verification: no certificate chain validation is performed
    Disabled,
}

pub const TLS_PEER_VERIFICATION_KEY: &str = "verify";

pub const TLS_PEER_VERIFICATION_VERIFY_PEER: &str = "verify_peer";
pub const TLS_PEER_VERIFICATION_VERIFY_NONE: &str = "verify_none";

impl From<&str> for TlsPeerVerificationMode {
    fn from(value: &str) -> Self {
        match value {
            TLS_PEER_VERIFICATION_VERIFY_PEER => TlsPeerVerificationMode::Enabled,
            TLS_PEER_VERIFICATION_VERIFY_NONE => TlsPeerVerificationMode::Disabled,
            _ => TlsPeerVerificationMode::Enabled,
        }
    }
}

impl From<String> for TlsPeerVerificationMode {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl FromStr for TlsPeerVerificationMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for TlsPeerVerificationMode {
    fn as_ref(&self) -> &str {
        match self {
            TlsPeerVerificationMode::Enabled => TLS_PEER_VERIFICATION_VERIFY_PEER,
            TlsPeerVerificationMode::Disabled => TLS_PEER_VERIFICATION_VERIFY_NONE,
        }
    }
}

impl From<TlsPeerVerificationMode> for String {
    fn from(value: TlsPeerVerificationMode) -> Self {
        match value {
            TlsPeerVerificationMode::Enabled => TLS_PEER_VERIFICATION_VERIFY_PEER.to_owned(),
            TlsPeerVerificationMode::Disabled => TLS_PEER_VERIFICATION_VERIFY_NONE.to_owned(),
        }
    }
}

impl From<&TlsPeerVerificationMode> for String {
    fn from(value: &TlsPeerVerificationMode) -> Self {
        value.clone().into()
    }
}

impl fmt::Display for TlsPeerVerificationMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from(self))
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

impl FromStr for MessageTransferAcknowledgementMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for MessageTransferAcknowledgementMode {
    fn as_ref(&self) -> &str {
        match self {
            MessageTransferAcknowledgementMode::Immediate => "no-ack",
            MessageTransferAcknowledgementMode::WhenPublished => "on-publish",
            MessageTransferAcknowledgementMode::WhenConfirmed => "on-confirm",
        }
    }
}

impl Display for MessageTransferAcknowledgementMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

/// Federation links can use multiple channels or reuse a single channel.
/// This is an advanced setting.
#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum ChannelUseMode {
    #[default]
    /// Federation links will use multiple channels for commands and message transfer
    Multiple,
    /// Federation links will reuse a single channel for both commands and message transfer
    Single,
}

impl From<&str> for ChannelUseMode {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "multiple" => ChannelUseMode::Multiple,
            "single" => ChannelUseMode::Single,
            _ => ChannelUseMode::default(),
        }
    }
}

impl From<String> for ChannelUseMode {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl FromStr for ChannelUseMode {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(s))
    }
}

impl AsRef<str> for ChannelUseMode {
    fn as_ref(&self) -> &str {
        match self {
            ChannelUseMode::Multiple => "multiple",
            ChannelUseMode::Single => "single",
        }
    }
}

impl Display for ChannelUseMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelUseMode::Multiple => write!(f, "multiple"),
            ChannelUseMode::Single => write!(f, "single"),
        }
    }
}

impl From<ChannelUseMode> for String {
    fn from(value: ChannelUseMode) -> Self {
        value.to_string()
    }
}
