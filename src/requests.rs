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
pub use policies::{PolicyDefinition, PolicyParams};

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
pub use vhosts::VirtualHostParams;

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
