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

use crate::commons::QueueType;
use crate::responses::VirtualHost;
use serde::{Deserialize, Serialize};
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

/// Properties of a [virtual host](https://rabbitmq.com/docs/vhosts/) to be created or updated.
///
/// Virtual hosts provide logical separation within a RabbitMQ instance, similar to
/// namespaces. Each virtual host has its own set of exchanges, queues, bindings, and permissions.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::{VirtualHostParams, QueueType};
///
/// // Basic virtual host
/// let basic = VirtualHostParams::named("production");
///
/// // Virtual host with description and settings
/// let configured = VirtualHostParams {
///     name: "app-staging",
///     description: Some("Staging environment for application testing"),
///     tags: Some(vec!["staging", "testing"]),
///     default_queue_type: Some(QueueType::Quorum),
///     tracing: true,
/// };
/// ```
#[derive(Serialize)]
pub struct VirtualHostParams<'a> {
    /// Virtual host name (must be unique within the RabbitMQ instance)
    pub name: &'a str,
    /// Optional description explaining the virtual host's purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    /// List of tags for organizing and categorizing virtual hosts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<&'a str>>,
    /// Default queue type for new queues in this virtual host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_queue_type: Option<QueueType>,
    /// Enable message tracing for debugging and monitoring
    pub tracing: bool,
}

impl<'a> VirtualHostParams<'a> {
    pub fn named(name: &'a str) -> Self {
        VirtualHostParams {
            name,
            description: None,
            tags: None,
            default_queue_type: None,
            tracing: false,
        }
    }
}

/// Represents a resource usage limit to be enforced on a [virtual host](https://rabbitmq.com/docs/vhosts/) or a user.
///
/// Can enforce limits on connections, queues, or other resources depending on the limit type.
/// The `kind` parameter specifies what type of resource to limit, while `value` sets the maximum allowed.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::EnforcedLimitParams;
///
/// // A limit of 100 connections
/// let max_connections = EnforcedLimitParams::new("max-connections", 100);
///
/// // A limit of 50 queues
/// let max_queues = EnforcedLimitParams::new("max-queues", 50);
/// ```
#[derive(Serialize)]
pub struct EnforcedLimitParams<T> {
    pub kind: T,
    pub value: i64,
}

impl<T> EnforcedLimitParams<T> {
    pub fn new(kind: T, value: i64) -> Self {
        EnforcedLimitParams { kind, value }
    }
}

/// Properties of a [user](https://rabbitmq.com/docs/access-control/#user-management) to be created or updated.
///
/// Use the functions in [`crate::password_hashing`] to generate
/// [salted password hashes](https://rabbitmq.com/docs/passwords/#computing-password-hash).
#[derive(Serialize)]
pub struct UserParams<'a> {
    /// Username (must be unique within the RabbitMQ cluster)
    pub name: &'a str,
    /// Pre-hashed and salted password, see [RabbitMQ doc guide on passwords](https://www.rabbitmq.com/docs/passwords) to learn more
    /// Use [`crate::password_hashing`] functions to generate secure hashes.
    pub password_hash: &'a str,
    /// Comma-separated list of user tags (e.g., "administrator", "monitoring", "management")
    pub tags: &'a str,
}

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

/// Represents a bulk user delete operation.
/// Used by [`crate::api::Client`] and [`crate::blocking_api::Client`]'s functions
/// that delete multiple users in a single operation.
#[derive(Serialize, Deserialize)]
pub struct BulkUserDelete<'a> {
    #[serde(borrow, rename = "users")]
    pub usernames: Vec<&'a str>,
}

/// Represents a user's [permission in a particular virtual host](https://rabbitmq.com/docs/access-control/).
///
/// Permissions are defined using regular expression patterns that match resource names.
///
/// Use ".*" to grant full access, or "" to deny access for a group of operations
#[derive(Serialize, Debug)]
pub struct Permissions<'a> {
    pub user: &'a str,
    pub vhost: &'a str,
    /// Regex pattern for resources user can configure (create/delete)
    pub configure: &'a str,
    /// Regex pattern for resources user can read from
    pub read: &'a str,
    /// Regex pattern for resources user can write to
    pub write: &'a str,
}

/// Represents a user's [topic permission in a particular virtual host](https://www.rabbitmq.com/docs/access-control#topic-authorisation).
///
/// Topic permissions are defined using regular expression patterns that match exchange names.
#[derive(Serialize, Debug)]
pub struct TopicPermissions<'a> {
    pub user: &'a str,
    pub vhost: &'a str,
    /// Regex pattern for the topics the user can publish to
    pub write: &'a str,
    /// Regex pattern for the topics the user can consume from (subscribe to)
    pub read: &'a str,
    /// The topic exchange these permissions apply to
    pub exchange: &'a str,
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

impl<'a> From<&'a VirtualHost> for VirtualHostParams<'a> {
    fn from(vhost: &'a VirtualHost) -> Self {
        Self {
            name: &vhost.name,
            description: vhost.description.as_deref(),
            tags: vhost
                .tags
                .as_ref()
                .map(|tags| tags.iter().map(|s| s.as_str()).collect()),
            default_queue_type: vhost
                .default_queue_type
                .as_ref()
                .map(|s| QueueType::from(s.as_str())),
            // this is an inherently transient setting
            tracing: false,
        }
    }
}
