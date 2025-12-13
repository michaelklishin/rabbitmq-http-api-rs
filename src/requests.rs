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

//! Types in this module represent requests a RabbitMQ HTTP API client
//! may need to perform, such as [`UserParams`], [`VirtualHostParams`], [`QueueParams`], [`QueueFederationParams`] or [`RuntimeParameterDefinition`].
//!
//! Most types provide constructor functions for common scenarios.

use serde::Serialize;
use serde_json::{Map, Value};

// Re-export OverflowBehavior from commons for backwards compatibility
pub use crate::commons::OverflowBehavior;

pub mod exchanges;
pub use exchanges::{ExchangeParams, OwnedExchangeParams};

pub mod parameters;
pub use parameters::{
    GlobalRuntimeParameterDefinition, RuntimeParameterDefinition, RuntimeParameterValue,
};

pub mod policies;
pub use policies::{OwnedPolicyParams, PolicyDefinition, PolicyDefinitionBuilder, PolicyParams};

pub mod federation;

pub use federation::{
    DEFAULT_FEDERATION_PREFETCH, DEFAULT_FEDERATION_RECONNECT_DELAY, ExchangeFederationParams,
    FEDERATION_UPSTREAM_COMPONENT, FederationResourceCleanupMode, FederationUpstreamParams,
    OwnedExchangeFederationParams, OwnedFederationUpstreamParams, OwnedQueueFederationParams,
    QueueFederationParams,
};

pub mod shovels;

pub use shovels::{
    Amqp10ShovelDestinationParams, Amqp10ShovelParams, Amqp10ShovelSourceParams,
    Amqp091ShovelDestinationEndpoint, Amqp091ShovelDestinationParams, Amqp091ShovelParams,
    Amqp091ShovelSourceEndpoint, Amqp091ShovelSourceParams, MessageProperties,
    OwnedAmqp091ShovelDestinationEndpoint, OwnedAmqp091ShovelSourceEndpoint, SHOVEL_COMPONENT,
};

pub mod queues_and_streams;
pub use queues_and_streams::{OwnedQueueParams, QueueParams, StreamParams};

pub mod users;
pub use users::{BulkUserDelete, OwnedUserParams, UserParams};

pub mod permissions;
pub use permissions::{Permissions, TopicPermissions};

pub mod vhosts;
pub use vhosts::{VirtualHostParams, VirtualHostParamsBuilder};

pub mod bindings;
pub use bindings::BindingDeletionParams;

pub mod limits;
pub use limits::EnforcedLimitParams;

/// Optional arguments map ("x-arguments") for queue and exchange declarations.
///
/// Used to pass RabbitMQ-specific arguments when declaring queues or exchanges.
/// Common arguments include TTL settings, dead letter exchanges, queue length limits, etc.
/// See the [RabbitMQ documentation](https://rabbitmq.com/docs/queues/#optional-arguments) for details.
///
/// # Examples
///
/// ```rust
/// use serde_json::{Map, Value, json};
/// use rabbitmq_http_client::requests::XArguments;
///
/// // Queue with message TTL and dead letter exchange
/// let mut args = Map::new();
/// args.insert("x-message-ttl".to_string(), json!(60000)); // 60 seconds
/// args.insert("x-dead-letter-exchange".to_string(), json!("dlx"));
/// let queue_args: XArguments = Some(args);
///
/// // No additional arguments
/// let no_args: XArguments = None;
/// ```
pub type XArguments = Option<Map<String, Value>>;

/// A builder for constructing [`XArguments`] with a fluent API.
///
/// This builder provides convenient methods for common queue and exchange arguments,
/// eliminating the need for manual JSON construction.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::XArgumentsBuilder;
///
/// let args = XArgumentsBuilder::new()
///     .max_length(10_000)
///     .message_ttl(60_000)
///     .dead_letter_exchange("dlx")
///     .dead_letter_routing_key("dlx.routing.key")
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct XArgumentsBuilder {
    inner: Map<String, Value>,
}

impl XArgumentsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds an argument using the type-safe [`OptionalQueueArgument`] enum.
    pub fn with_argument(mut self, arg: OptionalQueueArgument) -> Self {
        let (key, value) = arg.to_key_value();
        self.inner.insert(key, value);
        self
    }

    pub fn message_ttl(self, millis: u64) -> Self {
        self.with_argument(OptionalQueueArgument::MessageTtl(millis))
    }

    pub fn queue_ttl(self, millis: u64) -> Self {
        self.with_argument(OptionalQueueArgument::QueueTtl(millis))
    }

    pub fn max_length(self, max: u64) -> Self {
        self.with_argument(OptionalQueueArgument::MaxLength(max))
    }

    pub fn max_length_bytes(self, max_bytes: u64) -> Self {
        self.with_argument(OptionalQueueArgument::MaxLengthBytes(max_bytes))
    }

    pub fn dead_letter_exchange(self, exchange: &str) -> Self {
        self.with_argument(OptionalQueueArgument::DeadLetterExchange(
            exchange.to_owned(),
        ))
    }

    pub fn dead_letter_routing_key(self, routing_key: &str) -> Self {
        self.with_argument(OptionalQueueArgument::DeadLetterRoutingKey(
            routing_key.to_owned(),
        ))
    }

    pub fn overflow_drop_head(self) -> Self {
        self.with_argument(OptionalQueueArgument::Overflow(OverflowBehavior::DropHead))
    }

    pub fn overflow_reject_publish(self) -> Self {
        self.with_argument(OptionalQueueArgument::Overflow(
            OverflowBehavior::RejectPublish,
        ))
    }

    pub fn overflow_reject_publish_dlx(self) -> Self {
        self.with_argument(OptionalQueueArgument::Overflow(
            OverflowBehavior::RejectPublishDlx,
        ))
    }

    pub fn max_priority(self, max: u8) -> Self {
        self.with_argument(OptionalQueueArgument::MaxPriority(max))
    }

    pub fn quorum_initial_group_size(self, size: u32) -> Self {
        self.with_argument(OptionalQueueArgument::QuorumInitialGroupSize(size))
    }

    pub fn quorum_target_group_size(self, size: u32) -> Self {
        self.with_argument(OptionalQueueArgument::QuorumTargetGroupSize(size))
    }

    pub fn delivery_limit(self, limit: u64) -> Self {
        self.with_argument(OptionalQueueArgument::DeliveryLimit(limit))
    }

    pub fn single_active_consumer(self, enabled: bool) -> Self {
        self.with_argument(OptionalQueueArgument::SingleActiveConsumer(enabled))
    }

    pub fn queue_leader_locator(self, locator: QueueLeaderLocator) -> Self {
        self.with_argument(OptionalQueueArgument::QueueLeaderLocator(locator))
    }

    pub fn dead_letter_strategy(self, strategy: DeadLetterStrategy) -> Self {
        self.with_argument(OptionalQueueArgument::DeadLetterStrategy(strategy))
    }

    pub fn initial_cluster_size(self, size: u32) -> Self {
        self.with_argument(OptionalQueueArgument::InitialClusterSize(size))
    }

    pub fn max_age(self, age: &str) -> Self {
        self.with_argument(OptionalQueueArgument::MaxAge(age.to_owned()))
    }

    pub fn stream_max_segment_size_bytes(self, bytes: u64) -> Self {
        self.with_argument(OptionalQueueArgument::StreamMaxSegmentSizeBytes(bytes))
    }

    pub fn stream_filter_size_bytes(self, bytes: u8) -> Self {
        self.with_argument(OptionalQueueArgument::StreamFilterSizeBytes(bytes))
    }

    pub fn custom(self, key: &str, value: Value) -> Self {
        self.with_argument(OptionalQueueArgument::Custom(key.to_owned(), value))
    }

    /// Returns `None` if no arguments were added.
    pub fn build(self) -> XArguments {
        if self.inner.is_empty() {
            None
        } else {
            Some(self.inner)
        }
    }
}

/// A type-safe queue argument that can be converted to [`XArguments`].
///
/// This enum provides compile-time type safety for common queue arguments,
/// preventing typos in argument names and ensuring correct value types.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::OptionalQueueArgument;
///
/// let args: Vec<OptionalQueueArgument> = vec![
///     OptionalQueueArgument::MessageTtl(60_000),
///     OptionalQueueArgument::MaxLength(10_000),
///     OptionalQueueArgument::DeadLetterExchange("dlx".to_string()),
/// ];
///
/// // Convert to XArguments for use with queue declaration
/// let x_args = OptionalQueueArgument::to_x_arguments(args);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum OptionalQueueArgument {
    /// Message TTL in milliseconds (x-message-ttl)
    MessageTtl(u64),
    /// Queue TTL/expiration in milliseconds (x-expires)
    QueueTtl(u64),
    /// Maximum number of messages in the queue (x-max-length)
    MaxLength(u64),
    /// Maximum total size of messages in bytes (x-max-length-bytes)
    MaxLengthBytes(u64),
    /// Dead letter exchange name (x-dead-letter-exchange)
    DeadLetterExchange(String),
    /// Dead letter routing key (x-dead-letter-routing-key)
    DeadLetterRoutingKey(String),
    /// Dead letter strategy (x-dead-letter-strategy)
    DeadLetterStrategy(DeadLetterStrategy),
    /// Overflow behavior (x-overflow)
    Overflow(OverflowBehavior),
    /// Maximum priority for priority queues (x-max-priority, 1-255)
    MaxPriority(u8),
    /// Initial quorum group size (x-quorum-initial-group-size)
    QuorumInitialGroupSize(u32),
    /// Target quorum group size (x-quorum-target-group-size)
    QuorumTargetGroupSize(u32),
    /// Queue leader locator strategy (x-queue-leader-locator)
    QueueLeaderLocator(QueueLeaderLocator),
    /// Delivery limit before dead-lettering (x-delivery-limit)
    DeliveryLimit(u64),
    /// Enable single active consumer mode (x-single-active-consumer)
    SingleActiveConsumer(bool),
    /// Initial cluster size (x-initial-cluster-size)
    InitialClusterSize(u32),
    /// Maximum age for retention, e.g. "7D" (x-max-age)
    MaxAge(String),
    /// Maximum segment size in bytes (x-stream-max-segment-size-bytes)
    StreamMaxSegmentSizeBytes(u64),
    /// Bloom filter size in bytes, from 16 to 255 (x-stream-filter-size-bytes)
    StreamFilterSizeBytes(u8),
    /// Custom argument with key and value
    Custom(String, Value),
}

impl OptionalQueueArgument {
    fn to_key_value(&self) -> (String, Value) {
        match self {
            OptionalQueueArgument::MessageTtl(v) => ("x-message-ttl".to_owned(), Value::from(*v)),
            OptionalQueueArgument::QueueTtl(v) => ("x-expires".to_owned(), Value::from(*v)),
            OptionalQueueArgument::MaxLength(v) => ("x-max-length".to_owned(), Value::from(*v)),
            OptionalQueueArgument::MaxLengthBytes(v) => {
                ("x-max-length-bytes".to_owned(), Value::from(*v))
            }
            OptionalQueueArgument::DeadLetterExchange(v) => {
                ("x-dead-letter-exchange".to_owned(), Value::from(v.as_str()))
            }
            OptionalQueueArgument::DeadLetterRoutingKey(v) => (
                "x-dead-letter-routing-key".to_owned(),
                Value::from(v.as_str()),
            ),
            OptionalQueueArgument::DeadLetterStrategy(v) => {
                let s: &str = (*v).into();
                ("x-dead-letter-strategy".to_owned(), Value::from(s))
            }
            OptionalQueueArgument::Overflow(v) => {
                let s: &str = (*v).into();
                ("x-overflow".to_owned(), Value::from(s))
            }
            OptionalQueueArgument::MaxPriority(v) => ("x-max-priority".to_owned(), Value::from(*v)),
            OptionalQueueArgument::QuorumInitialGroupSize(v) => {
                ("x-quorum-initial-group-size".to_owned(), Value::from(*v))
            }
            OptionalQueueArgument::QuorumTargetGroupSize(v) => {
                ("x-quorum-target-group-size".to_owned(), Value::from(*v))
            }
            OptionalQueueArgument::QueueLeaderLocator(v) => {
                let s: &str = (*v).into();
                ("x-queue-leader-locator".to_owned(), Value::from(s))
            }
            OptionalQueueArgument::DeliveryLimit(v) => {
                ("x-delivery-limit".to_owned(), Value::from(*v))
            }
            OptionalQueueArgument::SingleActiveConsumer(v) => {
                ("x-single-active-consumer".to_owned(), Value::from(*v))
            }
            OptionalQueueArgument::InitialClusterSize(v) => {
                ("x-initial-cluster-size".to_owned(), Value::from(*v))
            }
            OptionalQueueArgument::MaxAge(v) => ("x-max-age".to_owned(), Value::from(v.as_str())),
            OptionalQueueArgument::StreamMaxSegmentSizeBytes(v) => (
                "x-stream-max-segment-size-bytes".to_owned(),
                Value::from(*v),
            ),
            OptionalQueueArgument::StreamFilterSizeBytes(v) => {
                ("x-stream-filter-size-bytes".to_owned(), Value::from(*v))
            }
            OptionalQueueArgument::Custom(k, v) => (k.clone(), v.clone()),
        }
    }

    /// Converts an iterator of OptionalQueueArguments to XArguments.
    pub fn to_x_arguments(args: impl IntoIterator<Item = OptionalQueueArgument>) -> XArguments {
        let mut map = Map::new();
        for arg in args {
            let (key, value) = arg.to_key_value();
            map.insert(key, value);
        }
        if map.is_empty() { None } else { Some(map) }
    }
}

/// Dead letter strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeadLetterStrategy {
    /// Messages are re-published (default)
    AtMostOnce,
    /// Messages are dead-lettered using at-least-once guarantees
    AtLeastOnce,
}

impl From<DeadLetterStrategy> for &str {
    fn from(value: DeadLetterStrategy) -> Self {
        match value {
            DeadLetterStrategy::AtMostOnce => "at-most-once",
            DeadLetterStrategy::AtLeastOnce => "at-least-once",
        }
    }
}

impl From<&str> for DeadLetterStrategy {
    fn from(s: &str) -> Self {
        match s {
            "at-least-once" => DeadLetterStrategy::AtLeastOnce,
            _ => DeadLetterStrategy::AtMostOnce, // default
        }
    }
}

/// Queue leader locator strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueueLeaderLocator {
    /// Picks the node with the least number of queue leaders (default)
    Balanced,
    /// Picks the node the client is connected to
    ClientLocal,
}

impl From<QueueLeaderLocator> for &str {
    fn from(value: QueueLeaderLocator) -> Self {
        match value {
            QueueLeaderLocator::Balanced => "balanced",
            QueueLeaderLocator::ClientLocal => "client-local",
        }
    }
}

impl From<&str> for QueueLeaderLocator {
    fn from(s: &str) -> Self {
        match s {
            "client-local" => QueueLeaderLocator::ClientLocal,
            _ => QueueLeaderLocator::Balanced, // default
        }
    }
}

/// Empty payload struct for API requests that don't require a body.
///
/// Some RabbitMQ API endpoints require a POST request but don't need any request body.
/// Use this type for such endpoints to provide a valid serializable payload.
#[derive(Serialize, Default)]
pub struct EmptyPayload;

impl EmptyPayload {
    /// Returns a new empty payload instance.
    pub fn new() -> Self {
        Self
    }
}
