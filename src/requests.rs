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

use crate::commons::{ExchangeType, QueueType};
use crate::responses::{ExchangeInfo, QueueInfo, VirtualHost};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

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

/// Represents [queue](https://rabbitmq.com/docs/queues/) properties used at declaration time.
///
/// **Prefer the constructor functions** rather than manual instantiation.
///
/// ```rust
/// use rabbitmq_http_client::requests::QueueParams;
///
/// // Prefer type-specific constructors
/// let qq = QueueParams::new_quorum_queue("critical-orders", None);
/// let sq = QueueParams::new_stream("high-volume-logs", None);
/// let cq = QueueParams::new_durable_classic_queue("simple-tasks", None);
/// ```
#[derive(Serialize, Debug)]
pub struct QueueParams<'a> {
    /// Queue name: must be no longer than 255 bytes in length
    pub name: &'a str,
    /// Queue type: determines replication, durability, and performance characteristics
    #[serde(skip_serializing)]
    pub queue_type: QueueType,
    /// Queue durability: durable queues survive broker restarts, transient are discarded on node boot
    pub durable: bool,
    /// Auto-delete: queue is deleted when the last consumer disconnects ([auto-delete queues](https://rabbitmq.com/docs/queues/#temporary-queues))
    pub auto_delete: bool,
    /// [Exclusive queues](https://rabbitmq.com/docs/queues/#temporary-queues) cannot be declared
    /// over the HTTP API due to the semantics of this property and the short-lived transactional
    /// nature of the HTTP API.
    pub exclusive: bool,
    /// [Optional queue arguments](https://rabbitmq.com/docs/queues/#optional-arguments): TTL, dead letter exchange, length limits, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: XArguments,
}

impl<'a> QueueParams<'a> {
    /// Returns declaration request parameters for a [quorum queue](https://rabbitmq.com/docs/quorum-queues/).
    ///
    /// Quorum queues provide strong consistency guarantees through replication and are always durable.
    /// Use for critical workloads where message safety is paramount. Cannot be transient or exclusive.
    pub fn new_quorum_queue(name: &'a str, optional_args: XArguments) -> Self {
        let typ = QueueType::Quorum;
        let args = Self::combined_args(optional_args, &typ);
        Self {
            name,
            queue_type: QueueType::Quorum,
            durable: true,
            auto_delete: false,
            exclusive: false,
            arguments: args,
        }
    }

    /// Returns declaration request parameters for a [stream](https://rabbitmq.com/docs/streams/).
    ///
    /// Streams are append-only message logs optimized for high throughput and parallel processing.
    /// Use for scenarios requiring message replay, high volume ingestion, or fan-out patterns.
    pub fn new_stream(name: &'a str, optional_args: XArguments) -> Self {
        let typ = QueueType::Stream;
        let args = Self::combined_args(optional_args, &typ);
        Self {
            name,
            queue_type: QueueType::Stream,
            durable: true,
            auto_delete: false,
            exclusive: false,
            arguments: args,
        }
    }

    /// Returns declaration request parameters for a durable [classic queue](https://rabbitmq.com/docs/queues/).
    ///
    /// Classic queues are the traditional RabbitMQ queue type, suitable for straightforward
    /// messaging scenarios. Limited to single-node operation but supports all RabbitMQ features.
    pub fn new_durable_classic_queue(name: &'a str, optional_args: XArguments) -> Self {
        let typ = QueueType::Classic;
        let args = Self::combined_args(optional_args, &typ);
        Self {
            name,
            queue_type: QueueType::Classic,
            durable: true,
            auto_delete: false,
            exclusive: false,
            arguments: args,
        }
    }

    /// Returns declaration request parameters for a transient, auto-delete [classic queue](https://rabbitmq.com/docs/queues/).
    ///
    /// Classic queues are the traditional RabbitMQ queue type, suitable for straightforward
    /// messaging scenarios. Limited to single-node operation but supports all RabbitMQ features.
    pub fn new_transient_autodelete(name: &'a str, optional_args: XArguments) -> Self {
        let typ = QueueType::Classic;
        let args = Self::combined_args(optional_args, &typ);
        Self {
            name,
            queue_type: QueueType::Classic,
            durable: false,
            auto_delete: true,
            exclusive: false,
            arguments: args,
        }
    }

    /// For when you want to control every queue property.
    pub fn new(
        name: &'a str,
        queue_type: QueueType,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        let args = Self::combined_args(optional_args, &queue_type);
        Self {
            name,
            queue_type,
            durable,
            auto_delete,
            exclusive: false,
            arguments: args,
        }
    }

    /// Combines user-provided arguments with an argument that represents queue type.
    pub fn combined_args(optional_args: XArguments, queue_type: &QueueType) -> XArguments {
        let mut result = Map::<String, Value>::new();
        result.insert("x-queue-type".to_owned(), json!(queue_type));

        if let Some(mut val) = optional_args {
            result.append(&mut val)
        }

        Some(result)
    }

    pub fn with_message_ttl(mut self, millis: u64) -> Self {
        self.add_argument("x-message-ttl".to_owned(), json!(millis));
        self
    }

    pub fn with_queue_ttl(mut self, millis: u64) -> Self {
        self.add_argument("x-expires".to_owned(), json!(millis));
        self
    }

    pub fn with_max_length(mut self, max_length: u64) -> Self {
        self.add_argument("x-max-length".to_owned(), json!(max_length));
        self
    }

    pub fn with_max_length_bytes(mut self, max_length_in_bytes: u64) -> Self {
        self.add_argument("x-max-length-bytes".to_owned(), json!(max_length_in_bytes));
        self
    }

    pub fn with_dead_letter_exchange(mut self, exchange: &str) -> Self {
        self.add_argument("x-dead-letter-exchange".to_owned(), json!(exchange));
        self
    }

    pub fn with_dead_letter_routing_key(mut self, routing_key: &str) -> Self {
        self.add_argument("x-dead-letter-routing-key".to_owned(), json!(routing_key));
        self
    }

    pub fn with_argument(mut self, key: String, value: Value) -> Self {
        self.add_argument(key, value);
        self
    }

    fn add_argument(&mut self, key: String, value: Value) {
        self.arguments
            .get_or_insert_with(Default::default)
            .insert(key, value);
    }
}

impl<'a> From<&'a QueueInfo> for QueueParams<'a> {
    fn from(queue: &'a QueueInfo) -> Self {
        let queue_type = QueueType::from(queue.queue_type.as_str());
        Self {
            name: &queue.name,
            queue_type,
            durable: queue.durable,
            auto_delete: queue.auto_delete,
            exclusive: queue.exclusive,
            arguments: if queue.arguments.is_empty() {
                None
            } else {
                Some(queue.arguments.0.clone())
            },
        }
    }
}

/// [Stream](https://rabbitmq.com/docs/streams/) properties used at declaration time
#[derive(Serialize, Debug)]
pub struct StreamParams<'a> {
    /// The name of the stream to declare.
    /// Must be no longer than 255 bytes in length.
    pub name: &'a str,
    /// Stream retention time in RabbitMQ duration format.
    /// Examples: "7D" (7 days), "1h30m" (1.5 hours), "300s" (5 minutes).
    /// Use "0" or empty string to disable expiration.
    pub expiration: &'a str,
    /// Maximum stream size in bytes. When exceeded, oldest segments are removed.
    /// Typical values range from 1GB to 100GB depending on use case.
    pub max_length_bytes: Option<u64>,
    /// Maximum size of individual stream segments in bytes.
    /// Defaults to 500MB if not specified. Smaller segments allow for more
    /// granular retention but may impact performance.
    pub max_segment_length_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: XArguments,
}

impl<'a> StreamParams<'a> {
    /// Returns basic stream parameters with just name and expiration.
    ///
    /// Use this for simple streams where default size limits are acceptable.
    /// For streams with specific size or performance requirements, use the more
    /// specific constructor methods.
    pub fn new(name: &'a str, expiration: &'a str) -> Self {
        Self {
            name,
            expiration,
            max_length_bytes: None,
            max_segment_length_bytes: None,
            arguments: None,
        }
    }

    /// Returns stream parameters with size limits.
    ///
    /// Use this when you need to control stream storage usage. The stream will
    /// automatically remove old segments when the total size exceeds max_length_bytes.
    /// This is essential for streams with high message volumes to prevent unbounded growth.
    pub fn with_expiration_and_length_limit(
        name: &'a str,
        expiration: &'a str,
        max_length_bytes: u64,
    ) -> Self {
        Self {
            name,
            expiration,
            max_length_bytes: Some(max_length_bytes),
            max_segment_length_bytes: None,
            arguments: None,
        }
    }

    pub fn with_max_length_bytes(mut self, bytes: u64) -> Self {
        self.max_length_bytes = Some(bytes);
        self
    }

    pub fn with_max_segment_length_bytes(mut self, bytes: u64) -> Self {
        self.max_segment_length_bytes = Some(bytes);
        self
    }

    pub fn with_argument(mut self, key: String, value: Value) -> Self {
        self.arguments
            .get_or_insert_with(Default::default)
            .insert(key, value);
        self
    }
}

/// [Exchange](https://rabbitmq.com/docs/exchanges/) properties used at declaration time.
///
/// Exchanges route messages to queues based on their type and routing rules.
#[derive(Debug, Serialize)]
pub struct ExchangeParams<'a> {
    #[serde(skip_serializing)]
    pub name: &'a str,
    #[serde(rename(serialize = "type"))]
    pub exchange_type: ExchangeType,
    /// Exchange durability: durable exchanges survive broker restarts, transient are discarded on node boot
    pub durable: bool,
    /// Auto-delete: exchange is deleted when the last queue is unbound from it
    pub auto_delete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: XArguments,
}

impl<'a> ExchangeParams<'a> {
    /// Returns declaration request parameters for a durable exchange of any type.
    ///
    /// Durable exchanges survive broker restarts and are not auto-deleted.
    /// Use this when you need a durable exchange with custom type and arguments.
    pub fn durable(name: &'a str, exchange_type: ExchangeType, optional_args: XArguments) -> Self {
        Self::new(name, exchange_type, true, false, optional_args)
    }

    /// Instantiates a [`ExchangeParams`] of a [fanout exchange](https://www.rabbitmq.com/docs/exchanges).
    pub fn fanout(
        name: &'a str,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        Self::new(
            name,
            ExchangeType::Fanout,
            durable,
            auto_delete,
            optional_args,
        )
    }

    /// Instantiates a [`ExchangeParams`] of a durable [fanout exchange](https://www.rabbitmq.com/docs/exchanges).
    pub fn durable_fanout(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Fanout, true, false, optional_args)
    }

    /// Returns declaration request parameters for a [topic exchange](https://rabbitmq.com/docs/exchanges/).
    ///
    /// Topic exchanges route messages based on pattern matching with routing keys.
    /// See [tutorials 3-5](https://rabbitmq.com/docs/getstarted/) for routing patterns and binding examples.
    pub fn topic(
        name: &'a str,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        Self::new(
            name,
            ExchangeType::Topic,
            durable,
            auto_delete,
            optional_args,
        )
    }

    /// Returns declaration request parameters for a durable [topic exchange](https://rabbitmq.com/docs/exchanges/).
    ///
    /// A durable topic exchange that survives broker restarts.
    /// See [tutorials 3-5](https://rabbitmq.com/docs/getstarted/) for routing patterns and binding examples.
    pub fn durable_topic(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Topic, true, false, optional_args)
    }

    /// Instantiates a [`ExchangeParams`] of a [direct exchange](https://www.rabbitmq.com/docs/exchanges).
    pub fn direct(
        name: &'a str,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        Self::new(
            name,
            ExchangeType::Direct,
            durable,
            auto_delete,
            optional_args,
        )
    }

    /// Instantiates a [`ExchangeParams`] of a durable [direct exchange](https://www.rabbitmq.com/docs/exchanges).
    pub fn durable_direct(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Direct, true, false, optional_args)
    }

    /// Returns declaration request parameters for a headers exchange.
    ///
    /// Headers exchanges route messages based on header attributes rather than routing keys.
    pub fn headers(
        name: &'a str,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        Self::new(
            name,
            ExchangeType::Headers,
            durable,
            auto_delete,
            optional_args,
        )
    }

    /// Returns declaration request parameters for a durable headers exchange.
    ///
    /// A durable headers exchange that survives broker restarts.
    pub fn durable_headers(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Headers, true, false, optional_args)
    }

    /// Returns declaration request parameters for a [local-random exchange](https://www.rabbitmq.com/docs/local-random-exchange).
    ///
    /// A plugin-provided exchange type that randomly selects one queue from
    /// the bound queues for each message. Useful for simple load balancing scenarios.
    pub fn local_random(
        name: &'a str,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        Self::new(
            name,
            ExchangeType::LocalRandom,
            durable,
            auto_delete,
            optional_args,
        )
    }

    /// Returns declaration request parameters for a durable [local-random exchange](https://www.rabbitmq.com/docs/local-random-exchange).
    ///
    /// A durable local-random exchange that survives broker restarts.
    pub fn durable_local_random(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::LocalRandom, true, false, optional_args)
    }

    /// Returns declaration request parameters for a custom plugin-provided exchange type.
    ///
    /// Use this for exchange types provided by RabbitMQ plugins that are not
    /// directly supported by this client.
    pub fn plugin(
        name: &'a str,
        exchange_type: String,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        Self::new(
            name,
            ExchangeType::Plugin(exchange_type),
            durable,
            auto_delete,
            optional_args,
        )
    }

    pub fn new(
        name: &'a str,
        exchange_type: ExchangeType,
        durable: bool,
        auto_delete: bool,
        optional_args: XArguments,
    ) -> Self {
        Self {
            name,
            exchange_type,
            durable,
            auto_delete,
            arguments: optional_args,
        }
    }

    pub fn with_argument(mut self, key: String, value: Value) -> Self {
        self.arguments
            .get_or_insert_with(Default::default)
            .insert(key, value);
        self
    }
}

impl<'a> From<&'a ExchangeInfo> for ExchangeParams<'a> {
    fn from(exchange: &'a ExchangeInfo) -> Self {
        Self {
            name: &exchange.name,
            exchange_type: ExchangeType::from(exchange.exchange_type.as_str()),
            durable: exchange.durable,
            auto_delete: exchange.auto_delete,
            arguments: if exchange.arguments.is_empty() {
                None
            } else {
                Some(exchange.arguments.0.clone())
            },
        }
    }
}

/// Represents a bulk user delete operation.
/// Used by [`crate::api::Client`] and [`crate::blocking_api::Client`]'s functions
/// that delete multiple users in a single operation.
#[derive(Serialize, Deserialize)]
pub struct BulkUserDelete<'a> {
    #[serde(borrow, rename = "users")]
    pub usernames: Vec<&'a str>,
}

pub mod parameters;
pub use parameters::{
    GlobalRuntimeParameterDefinition, RuntimeParameterDefinition, RuntimeParameterValue,
};

pub mod policies;
pub use policies::{PolicyDefinition, PolicyParams};

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
