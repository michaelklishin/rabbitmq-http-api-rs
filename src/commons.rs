use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Formatter;

/// Exchange types. Most variants are for exchange types included with modern RabbitMQ distributions.
/// For custom types provided by 3rd party plugins, use the `Plugin(String)` variant.
#[derive(Serialize, Debug, PartialEq, Eq)]
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
    ConsistentHashing,
    /// Modulus hash, ships with the 'rabbitmq-sharding' plugin
    ModulusHash,
    /// Random exchange
    Random,
    /// JMS topic exchange
    JmsTopic,
    /// Recent history exchange
    RecentHistory,
    /// All other x-* exchange types, for example, those provided by plugins
    Plugin(String),
}

const EXCHANGE_TYPE_FANOUT: &str = "fanout";
const EXCHANGE_TYPE_TOPIC: &str = "topic";
const EXCHANGE_TYPE_DIRECT: &str = "direct";
const EXCHANGE_TYPE_HEADERS: &str = "headers";
const EXCHANGE_TYPE_CONSISTENT_HASHING: &str = "x-consistent-hash";
const EXCHANGE_TYPE_MODULUS_HASH: &str = "x-modulus-hash";
const EXCHANGE_TYPE_RANDOM: &str = "x-random";
const EXCHANGE_TYPE_JMS_TOPIC: &str = "x-jms-topic";
const EXCHANGE_TYPE_RECENT_HISTORY: &str = "x-recent-history";

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
            EXCHANGE_TYPE_JMS_TOPIC => ExchangeType::JmsTopic,
            EXCHANGE_TYPE_RECENT_HISTORY => ExchangeType::RecentHistory,
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
            ExchangeType::JmsTopic => EXCHANGE_TYPE_JMS_TOPIC.to_owned(),
            ExchangeType::RecentHistory => EXCHANGE_TYPE_RECENT_HISTORY.to_owned(),
            ExchangeType::Plugin(exchange_type) => exchange_type,
        }
    }
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all(serialize = "lowercase", deserialize = "PascalCase"))]
pub enum QueueType {
    Classic,
    Quorum,
    Stream,
}

impl From<&str> for QueueType {
    fn from(value: &str) -> Self {
        match value {
            "classic" => QueueType::Classic,
            "quorum" => QueueType::Quorum,
            "stream" => QueueType::Stream,
            _ => QueueType::Classic,
        }
    }
}

impl From<String> for QueueType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "classic" => QueueType::Classic,
            "quorum" => QueueType::Quorum,
            "stream" => QueueType::Stream,
            _ => QueueType::Classic,
        }
    }
}

impl From<QueueType> for String {
    fn from(value: QueueType) -> Self {
        match value {
            QueueType::Classic => "classic".to_owned(),
            QueueType::Quorum => "quorum".to_owned(),
            QueueType::Stream => "stream".to_owned(),
        }
    }
}

/// Binding destination can be either a queue or another exchange
/// (in the case of [exchange-to-exchange bindings](https://rabbitmq.com/e2e.html)).
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BindingDestinationType {
    Queue,
    Exchange,
}

impl fmt::Display for BindingDestinationType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyTarget {
    Queues,
    ClassicQueues,
    QuorumQueues,
    Streams,
    Exchanges,
    All,
}

impl fmt::Display for PolicyTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))?;

        Ok(())
    }
}

impl From<&str> for PolicyTarget {
    fn from(value: &str) -> Self {
        match value {
            "queues" => PolicyTarget::Queues,
            "classic_queues" => PolicyTarget::ClassicQueues,
            "quorum_queues" => PolicyTarget::QuorumQueues,
            "streams" => PolicyTarget::Streams,
            "exchanges" => PolicyTarget::Exchanges,
            "all" => PolicyTarget::All,
            _ => PolicyTarget::Queues,
        }
    }
}

impl From<String> for PolicyTarget {
    fn from(value: String) -> Self {
        match value.as_str() {
            "queues" => PolicyTarget::Queues,
            "classic_queues" => PolicyTarget::ClassicQueues,
            "quorum_queues" => PolicyTarget::QuorumQueues,
            "streams" => PolicyTarget::Streams,
            "exchanges" => PolicyTarget::Exchanges,
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

#[allow(clippy::to_string_trait_impl)]
impl ToString for VirtualHostLimitTarget {
    fn to_string(&self) -> String {
        match self {
            VirtualHostLimitTarget::MaxConnections => "max-connections".to_owned(),
            VirtualHostLimitTarget::MaxQueues => "max-queues".to_owned(),
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
        match value {
            VirtualHostLimitTarget::MaxConnections => "max-connections".to_owned(),
            VirtualHostLimitTarget::MaxQueues => "max-queues".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged, rename_all = "kebab-case")]
pub enum UserLimitTarget {
    MaxConnections,
    MaxChannels,
}

#[allow(clippy::to_string_trait_impl)]
impl ToString for UserLimitTarget {
    fn to_string(&self) -> String {
        match self {
            UserLimitTarget::MaxConnections => "max-connections".to_owned(),
            UserLimitTarget::MaxChannels => "max-channels".to_owned(),
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
        match value {
            UserLimitTarget::MaxConnections => "max-connections".to_owned(),
            UserLimitTarget::MaxChannels => "max-channels".to_owned(),
        }
    }
}
