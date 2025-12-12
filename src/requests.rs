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

pub mod exchanges;
pub use exchanges::ExchangeParams;

pub mod parameters;
pub use parameters::{
    GlobalRuntimeParameterDefinition, RuntimeParameterDefinition, RuntimeParameterValue,
};

pub mod policies;
pub use policies::{PolicyDefinition, PolicyDefinitionBuilder, PolicyParams};

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
    Amqp091ShovelDestinationParams, Amqp091ShovelParams, Amqp091ShovelSourceParams,
    MessageProperties, SHOVEL_COMPONENT,
};

pub mod queues_and_streams;
pub use queues_and_streams::{QueueParams, StreamParams};

pub mod users;
pub use users::{BulkUserDelete, UserParams};

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

    pub fn message_ttl(mut self, millis: u64) -> Self {
        self.inner
            .insert("x-message-ttl".to_owned(), Value::from(millis));
        self
    }

    pub fn queue_ttl(mut self, millis: u64) -> Self {
        self.inner
            .insert("x-expires".to_owned(), Value::from(millis));
        self
    }

    pub fn max_length(mut self, max: u64) -> Self {
        self.inner
            .insert("x-max-length".to_owned(), Value::from(max));
        self
    }

    pub fn max_length_bytes(mut self, max_bytes: u64) -> Self {
        self.inner
            .insert("x-max-length-bytes".to_owned(), Value::from(max_bytes));
        self
    }

    pub fn dead_letter_exchange(mut self, exchange: &str) -> Self {
        self.inner
            .insert("x-dead-letter-exchange".to_owned(), Value::from(exchange));
        self
    }

    pub fn dead_letter_routing_key(mut self, routing_key: &str) -> Self {
        self.inner.insert(
            "x-dead-letter-routing-key".to_owned(),
            Value::from(routing_key),
        );
        self
    }

    pub fn overflow_drop_head(mut self) -> Self {
        self.inner
            .insert("x-overflow".to_owned(), Value::from("drop-head"));
        self
    }

    pub fn overflow_reject_publish(mut self) -> Self {
        self.inner
            .insert("x-overflow".to_owned(), Value::from("reject-publish"));
        self
    }

    pub fn overflow_reject_publish_dlx(mut self) -> Self {
        self.inner
            .insert("x-overflow".to_owned(), Value::from("reject-publish-dlx"));
        self
    }

    pub fn max_priority(mut self, max: u8) -> Self {
        self.inner
            .insert("x-max-priority".to_owned(), Value::from(max));
        self
    }

    pub fn quorum_initial_group_size(mut self, size: u32) -> Self {
        self.inner
            .insert("x-quorum-initial-group-size".to_owned(), Value::from(size));
        self
    }

    pub fn delivery_limit(mut self, limit: u64) -> Self {
        self.inner
            .insert("x-delivery-limit".to_owned(), Value::from(limit));
        self
    }

    pub fn single_active_consumer(mut self, enabled: bool) -> Self {
        self.inner
            .insert("x-single-active-consumer".to_owned(), Value::from(enabled));
        self
    }

    pub fn custom(mut self, key: &str, value: Value) -> Self {
        self.inner.insert(key.to_owned(), value);
        self
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
