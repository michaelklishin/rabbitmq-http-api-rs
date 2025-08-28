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

use crate::commons::{ExchangeType, MessageTransferAcknowledgementMode, PolicyTarget, QueueType};
use crate::responses;
use crate::responses::{Policy, PolicyDefinition as PolDef};
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
}

/// Represents a bulk user delete operation.
/// Used by [`crate::api::Client`] and [`crate::blocking_api::Client`]'s functions
/// that delete multiple users in a single operation.
#[derive(Serialize, Deserialize)]
pub struct BulkUserDelete<'a> {
    #[serde(borrow, rename = "users")]
    pub usernames: Vec<&'a str>,
}

/// Runtime parameter value map.
///
/// Contains the actual configuration data for a runtime parameter.
/// The structure depends on the component type (federation, shovel, etc.).
pub type RuntimeParameterValue = Map<String, Value>;

/// Represents a [runtime parameter](https://rabbitmq.com/docs/parameters/).
///
/// Runtime parameters are key-value pairs that configure plugin behavior at runtime.
/// The `component` field identifies the plugin (e.g., "federation-upstream", "shovel"),
/// while `name` is the parameter identifier within that component's namespace.
///
/// Common components include:
/// * "federation-upstream": Federation plugin upstreams
/// * "shovel": Dynamic shovel configurations
/// * "mqtt": MQTT plugin parameters
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuntimeParameterDefinition<'a> {
    pub name: &'a str,
    pub vhost: &'a str,
    pub component: &'a str,
    pub value: RuntimeParameterValue,
}

/// Represents a [global runtime parameter](https://rabbitmq.com/docs/parameters/).
///
/// Global parameters apply to the entire RabbitMQ node rather than a specific virtual host.
/// Used for cluster-wide configuration settings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalRuntimeParameterDefinition<'a> {
    /// Parameter name
    pub name: &'a str,
    /// Parameter value (structure depends on the parameter type)
    pub value: RuntimeParameterValue,
}

/// Policy definition map containing the actual policy rules.
///
/// Contains key-value pairs that define the policy behavior, such as:
/// * "ha-mode": "all" (high availability settings)
/// * "message-ttl": 60000 (message time-to-live in milliseconds)
/// * "max-length": 1000 (maximum queue length)
pub type PolicyDefinition = Map<String, Value>;

impl From<PolDef> for PolicyDefinition {
    fn from(policy: responses::PolicyDefinition) -> Self {
        match policy.0 {
            None => {
                let empty: Map<String, Value> = Map::new();
                empty
            }
            Some(value) => value,
        }
    }
}

/// Represents a [policy](https://rabbitmq.com/docs/parameters/#policies).
///
/// Policies apply configuration to queues and exchanges whose names match a regex pattern.
/// When multiple policies match the same resource, the one with the highest priority wins.
///
/// # Pattern Examples
/// * `"^amq\\."` - Matches resources starting with "amq."
/// * `".*"` - Matches all resources  
/// * `"orders\\.(urgent|normal)"` - Matches "orders.urgent" or "orders.normal"
/// * `"temp_.*"` - Matches resources starting with "temp_"
///
/// # Priority
/// Higher numbers have higher priority. If two policies match the same resource,
/// the policy with the higher priority value takes precedence. Typical range: 0-999.
#[derive(Serialize, Debug)]
pub struct PolicyParams<'a> {
    pub vhost: &'a str,
    pub name: &'a str,
    /// Regular expression pattern to match queue/exchange names
    pub pattern: &'a str,
    #[serde(rename(serialize = "apply-to"))]
    pub apply_to: PolicyTarget,
    /// Priority when multiple policies match (higher wins)
    pub priority: i32,
    pub definition: PolicyDefinition,
}

impl<'a> From<&'a Policy> for PolicyParams<'a> {
    fn from(policy: &'a Policy) -> Self {
        PolicyParams {
            vhost: &policy.vhost,
            name: &policy.name,
            pattern: &policy.pattern,
            apply_to: policy.apply_to.clone(),
            priority: policy.priority as i32, // Converting i16 to i32
            definition: policy.definition.clone().into(),
        }
    }
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

/// Controls when federation resources (temporary queues/exchanges) are cleaned up.
///
/// Federation creates temporary resources on the downstream cluster. This enum controls
/// when these resources are removed to prevent resource leaks.
#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FederationResourceCleanupMode {
    /// Default cleanup behavior: resources are cleaned up when the federation link is stopped
    #[default]
    Default,
    /// Never clean up federation resources automatically (manual cleanup required)
    Never,
}

impl From<&str> for FederationResourceCleanupMode {
    fn from(value: &str) -> Self {
        match value {
            "default" => FederationResourceCleanupMode::Default,
            "never" => FederationResourceCleanupMode::Never,
            _ => FederationResourceCleanupMode::default(),
        }
    }
}

impl From<String> for FederationResourceCleanupMode {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

/// [Runtime parameter](https://www.rabbitmq.com/docs/parameters) component name used by federation upstreams.
///
/// This constant is used internally when creating [`RuntimeParameterDefinition`]
/// instances for federation upstreams.
pub const FEDERATION_UPSTREAM_COMPONENT: &str = "federation-upstream";

/// Parameters specific to [queue federation](https://www.rabbitmq.com/docs/federated-queues).
pub struct QueueFederationParams<'a> {
    /// Name of the upstream queue to federate from (None uses the same name as downstream)
    pub queue: Option<&'a str>,
    /// Consumer tag for the federation link (None uses auto-generated tag)
    pub consumer_tag: Option<&'a str>,
}

impl<'a> QueueFederationParams<'a> {
    /// Returns queue federation parameters with a specific upstream queue name.
    ///
    /// Use this when the upstream queue has a different name than the downstream queue.
    pub fn new(queue: &'a str) -> Self {
        Self {
            queue: Some(queue),
            consumer_tag: None,
        }
    }

    /// Returns queue federation parameters with both queue name and consumer tag.
    ///
    /// Use this when you need to specify both the upstream queue name and a custom
    /// consumer tag for identification and management purposes.
    pub fn new_with_consumer_tag(queue: &'a str, consumer_tag: &'a str) -> Self {
        Self {
            queue: Some(queue),
            consumer_tag: Some(consumer_tag),
        }
    }
}

/// Parameters specific to [exchange federation](https://www.rabbitmq.com/docs/federated-exchanges).
pub struct ExchangeFederationParams<'a> {
    /// Name of the upstream exchange to federate from (None uses the same name as downstream)
    pub exchange: Option<&'a str>,
    /// Maximum hops for federation chains to prevent infinite loops
    pub max_hops: Option<u8>,
    /// Queue type for the temporary federation queue
    pub queue_type: QueueType,
    /// Time-to-live for the federation queue in milliseconds
    pub ttl: Option<u32>,
    /// Message TTL for federated messages in milliseconds
    pub message_ttl: Option<u32>,
    /// When to clean up temporary federation resources
    pub resource_cleanup_mode: FederationResourceCleanupMode,
}

impl ExchangeFederationParams<'_> {
    /// Returns exchange federation parameters with the specified queue type.
    ///
    /// The queue type determines the characteristics of the temporary queue used
    /// for the federation link. Use quorum queues for durability or classic for simplicity.
    pub fn new(queue_type: QueueType) -> Self {
        Self {
            exchange: None,
            max_hops: None,
            queue_type,
            ttl: None,
            message_ttl: None,
            resource_cleanup_mode: FederationResourceCleanupMode::default(),
        }
    }
}

/// Matches the default used by the federation plugin.
const DEFAULT_FEDERATION_PREFETCH: u16 = 1000;
/// Matches the default used by the federation plugin.
const DEFAULT_FEDERATION_RECONNECT_DELAY: u16 = 5;

/// Represents a set of parameters that define a federation upstream
/// and a number of the federation type-specific (exchange, queue) parameters
/// that are associated with an upstream.
///
/// A federation upstream is declared as a runtime parameter,
/// therefore this type implements a conversion that is used
/// by [`crate::api::Client#declare_federation_upstream`] and [`crate::blocking_api::Client#declare_federation_upstream`]
pub struct FederationUpstreamParams<'a> {
    pub name: &'a str,
    pub vhost: &'a str,
    pub uri: &'a str,
    pub reconnect_delay: u16,
    pub trust_user_id: bool,
    pub prefetch_count: u16,
    pub ack_mode: MessageTransferAcknowledgementMode,
    pub bind_using_nowait: bool,

    pub queue_federation: Option<QueueFederationParams<'a>>,
    pub exchange_federation: Option<ExchangeFederationParams<'a>>,
}

impl<'a> FederationUpstreamParams<'a> {
    /// Creates a federation upstream that will be used for [queue federation](https://www.rabbitmq.com/docs/federated-queues).
    pub fn new_queue_federation_upstream(
        vhost: &'a str,
        name: &'a str,
        uri: &'a str,
        params: QueueFederationParams<'a>,
    ) -> Self {
        Self {
            vhost,
            name,
            uri,
            ack_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
            reconnect_delay: DEFAULT_FEDERATION_RECONNECT_DELAY,
            trust_user_id: false,
            prefetch_count: DEFAULT_FEDERATION_PREFETCH,
            bind_using_nowait: false,
            exchange_federation: None,
            queue_federation: Some(params),
        }
    }

    /// Creates a federation upstream that will be used for [exchange federation](https://www.rabbitmq.com/docs/federated-exchanges).
    pub fn new_exchange_federation_upstream(
        vhost: &'a str,
        name: &'a str,
        uri: &'a str,
        params: ExchangeFederationParams<'a>,
    ) -> Self {
        Self {
            vhost,
            name,
            uri,
            ack_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
            reconnect_delay: DEFAULT_FEDERATION_RECONNECT_DELAY,
            trust_user_id: false,
            prefetch_count: DEFAULT_FEDERATION_PREFETCH,
            bind_using_nowait: false,
            queue_federation: None,
            exchange_federation: Some(params),
        }
    }
}

impl<'a> From<FederationUpstreamParams<'a>> for RuntimeParameterDefinition<'a> {
    fn from(params: FederationUpstreamParams<'a>) -> Self {
        let mut value = Map::new();

        value.insert("uri".to_owned(), json!(params.uri));
        value.insert("prefetch-count".to_owned(), json!(params.prefetch_count));
        value.insert("trust-user-id".to_owned(), json!(params.trust_user_id));
        value.insert("reconnect-delay".to_owned(), json!(params.reconnect_delay));
        value.insert("ack-mode".to_owned(), json!(params.ack_mode));

        if let Some(qf) = params.queue_federation {
            value.insert("queue".to_owned(), json!(qf.queue));
            if let Some(val) = qf.consumer_tag {
                value.insert("consumer-tag".to_owned(), json!(val));
            }
        }

        if let Some(ef) = params.exchange_federation {
            value.insert("queue-type".to_owned(), json!(ef.queue_type));
            if let Some(val) = ef.exchange {
                value.insert("exchange".to_owned(), json!(val));
            };

            if let Some(val) = ef.max_hops {
                value.insert("max-hops".to_owned(), json!(val));
            }
            if let Some(val) = ef.ttl {
                value.insert("expires".to_owned(), json!(val));
            }
            if let Some(val) = ef.message_ttl {
                value.insert("message-ttl".to_owned(), json!(val));
            }
        }

        Self {
            name: params.name,
            vhost: params.vhost,
            component: FEDERATION_UPSTREAM_COMPONENT,
            value,
        }
    }
}

/// [Runtime parameter](https://www.rabbitmq.com/docs/parameters) component used by dynamic shovels.
///
/// This constant is used internally when creating [`RuntimeParameterDefinition`]
/// instances for dynamic shovels.
pub const SHOVEL_COMPONENT: &str = "shovel";

/// Parameters for a shovel definition that will use AMQP 0-9-1 for both source and destination
/// connections (operations).
#[derive(Serialize)]
pub struct Amqp091ShovelParams<'a> {
    /// Shovel name (must be unique within the virtual host)
    pub name: &'a str,
    /// Virtual host where the shovel will be created
    pub vhost: &'a str,

    /// Message acknowledgment mode for reliability
    pub acknowledgement_mode: MessageTransferAcknowledgementMode,
    /// Delay in seconds before reconnecting after connection failure
    pub reconnect_delay: Option<u16>,

    /// Source endpoint configuration
    pub source: Amqp091ShovelSourceParams<'a>,
    /// Destination endpoint configuration
    pub destination: Amqp091ShovelDestinationParams<'a>,
}

impl<'a> From<Amqp091ShovelParams<'a>> for RuntimeParameterDefinition<'a> {
    fn from(params: Amqp091ShovelParams<'a>) -> Self {
        let mut value = Map::new();

        value.insert("src-protocol".to_owned(), json!("amqp091"));
        value.insert("dest-protocol".to_owned(), json!("amqp091"));

        value.insert("src-uri".to_owned(), json!(params.source.source_uri));
        if let Some(sq) = params.source.source_queue {
            value.insert("src-queue".to_owned(), json!(sq));
        }
        if let Some(sx) = params.source.source_exchange {
            value.insert("src-exchange".to_owned(), json!(sx));
        }
        if let Some(sxrk) = params.source.source_exchange_routing_key {
            value.insert("src-exchange-key".to_owned(), json!(sxrk));
        }

        value.insert(
            "dest-uri".to_owned(),
            json!(params.destination.destination_uri),
        );
        value.insert("ack-mode".to_owned(), json!(params.acknowledgement_mode));

        if let Some(dq) = params.destination.destination_queue {
            value.insert("dest-queue".to_owned(), json!(dq));
        }
        if let Some(dx) = params.destination.destination_exchange {
            value.insert("dest-exchange".to_owned(), json!(dx));
        }
        if let Some(dxrk) = params.destination.destination_exchange_routing_key {
            value.insert("dest-exchange-key".to_owned(), json!(dxrk));
        }

        value.insert(
            "src-predeclared".to_owned(),
            json!(params.source.predeclared),
        );
        value.insert(
            "dest-predeclared".to_owned(),
            json!(params.destination.predeclared),
        );

        if let Some(val) = params.reconnect_delay {
            value.insert("reconnect-delay".to_owned(), json!(val));
        }

        Self {
            name: params.name,
            vhost: params.vhost,
            component: SHOVEL_COMPONENT,
            value,
        }
    }
}

/// AMQP 0-9-1 shovel source settings.
#[derive(Serialize)]
pub struct Amqp091ShovelSourceParams<'a> {
    /// AMQP URI of the source broker
    pub source_uri: &'a str,

    /// Source queue name (used with queue_source constructors)
    pub source_queue: Option<&'a str>,

    /// Source exchange name (used with exchange_source constructors)
    pub source_exchange: Option<&'a str>,
    /// Routing key for exchange sources (filters messages)
    pub source_exchange_routing_key: Option<&'a str>,

    /// Whether the source topology is predeclared (already exists)
    pub predeclared: bool,
}

impl<'a> Amqp091ShovelSourceParams<'a> {
    /// Creates request parameters for a shovel that uses a queue as source.
    pub fn queue_source(source_uri: &'a str, source_queue: &'a str) -> Self {
        Self {
            source_uri,
            source_queue: Some(source_queue),

            source_exchange: None,
            source_exchange_routing_key: None,

            predeclared: false,
        }
    }

    /// Creates request parameters for a shovel that uses an exchange as source
    /// (which means declaring a queue that will be bound to source exchange).
    pub fn exchange_source(
        source_uri: &'a str,
        source_exchange: &'a str,
        source_exchange_routing_key: Option<&'a str>,
    ) -> Self {
        Self {
            source_uri,
            source_exchange: Some(source_exchange),
            source_exchange_routing_key,

            source_queue: None,

            predeclared: false,
        }
    }

    /// Creates request parameters for a shovel that uses a pre-declared queue as source.
    /// Such shovels will not try to declare their source queue.
    pub fn predeclared_queue_source(source_uri: &'a str, source_queue: &'a str) -> Self {
        Self {
            source_uri,
            source_queue: Some(source_queue),

            source_exchange: None,
            source_exchange_routing_key: None,

            predeclared: true,
        }
    }

    /// Creates request parameters for a shovel that uses a pre-declared exchange as source.
    /// Such shovels will not try to declare their source exchange.
    pub fn predeclared_exchange_source(
        source_uri: &'a str,
        source_exchange: &'a str,
        source_exchange_routing_key: Option<&'a str>,
    ) -> Self {
        Self {
            source_uri,
            source_exchange: Some(source_exchange),
            source_exchange_routing_key,

            source_queue: None,

            predeclared: true,
        }
    }
}

/// AMQP 0-9-1 shovel destination settings.
#[derive(Serialize)]
pub struct Amqp091ShovelDestinationParams<'a> {
    /// AMQP URI of the destination broker
    pub destination_uri: &'a str,

    /// Destination queue name (used with queue_destination constructors)
    pub destination_queue: Option<&'a str>,
    /// Destination exchange name (used with exchange_destination constructors)
    pub destination_exchange: Option<&'a str>,
    /// Routing key for exchange destinations
    pub destination_exchange_routing_key: Option<&'a str>,

    /// Whether the destination topology is predeclared (already exists)
    pub predeclared: bool,
}

impl<'a> Amqp091ShovelDestinationParams<'a> {
    /// Creates request parameters for a shovel that uses a queue as destination.
    pub fn queue_destination(destination_uri: &'a str, destination_queue: &'a str) -> Self {
        Self {
            destination_uri,
            destination_queue: Some(destination_queue),

            destination_exchange: None,
            destination_exchange_routing_key: None,

            predeclared: false,
        }
    }

    /// Creates request parameters for a shovel that uses an exchange as destination.
    pub fn exchange_destination(
        destination_uri: &'a str,
        destination_exchange: &'a str,
        destination_exchange_routing_key: Option<&'a str>,
    ) -> Self {
        Self {
            destination_uri,
            destination_exchange: Some(destination_exchange),
            destination_exchange_routing_key,

            destination_queue: None,

            predeclared: false,
        }
    }

    /// Creates request parameters for a shovel that uses a pre-declared queue as destination.
    /// Such shovels will not try to declare their destination queue.
    pub fn predeclared_queue_destination(
        destination_uri: &'a str,
        destination_queue: &'a str,
    ) -> Self {
        Self {
            destination_uri,
            destination_queue: Some(destination_queue),

            destination_exchange: None,
            destination_exchange_routing_key: None,

            predeclared: true,
        }
    }

    /// Creates request parameters for a shovel that uses a pre-declared exchange as destination.
    /// Such shovels will not try to declare their destination exchange.
    pub fn predeclared_exchange_destination(
        destination_uri: &'a str,
        destination_exchange: &'a str,
        destination_exchange_routing_key: Option<&'a str>,
    ) -> Self {
        Self {
            destination_uri,
            destination_exchange: Some(destination_exchange),
            destination_exchange_routing_key,

            destination_queue: None,

            predeclared: true,
        }
    }
}

/// Parameters for a shovel definition that will use AMQP 1.0 for both source and destination
/// connections (operations).
#[derive(Serialize)]
pub struct Amqp10ShovelParams<'a> {
    /// Shovel name (must be unique within the virtual host)
    pub name: &'a str,
    /// Virtual host where the shovel will be created
    pub vhost: &'a str,

    /// Message acknowledgment mode for reliability
    pub acknowledgement_mode: MessageTransferAcknowledgementMode,
    /// Delay in seconds before reconnecting after connection failure
    pub reconnect_delay: Option<u16>,

    /// Source endpoint configuration
    pub source: Amqp10ShovelSourceParams<'a>,
    /// Destination endpoint configuration
    pub destination: Amqp10ShovelDestinationParams<'a>,
}

/// AMQP 1.0 shovel source settings.
#[derive(Serialize)]
pub struct Amqp10ShovelSourceParams<'a> {
    /// AMQP 1.0 URI of the source broker
    pub source_uri: &'a str,
    /// AMQP 1.0 address to consume from (queue name or topic pattern)
    pub source_address: &'a str,
}

impl<'a> Amqp10ShovelSourceParams<'a> {
    /// The address parameter specifies what to consume from on the source broker.
    /// This could be a queue name, topic pattern, or other address format supported by the broker.
    pub fn new(uri: &'a str, address: &'a str) -> Self {
        Self {
            source_uri: uri,
            source_address: address,
        }
    }
}

impl<'a> From<Amqp10ShovelParams<'a>> for RuntimeParameterDefinition<'a> {
    fn from(params: Amqp10ShovelParams<'a>) -> Self {
        let mut value = Map::new();

        value.insert("src-protocol".to_owned(), json!("amqp10"));
        value.insert("dest-protocol".to_owned(), json!("amqp10"));

        value.insert("src-uri".to_owned(), json!(params.source.source_uri));
        value.insert(
            "src-address".to_owned(),
            json!(params.source.source_address),
        );

        value.insert(
            "dest-uri".to_owned(),
            json!(params.destination.destination_uri),
        );
        value.insert(
            "dest-address".to_owned(),
            json!(params.destination.destination_address),
        );

        value.insert("ack-mode".to_owned(), json!(params.acknowledgement_mode));
        if let Some(val) = params.reconnect_delay {
            value.insert("reconnect-delay".to_owned(), json!(val));
        }

        Self {
            name: params.name,
            vhost: params.vhost,
            component: SHOVEL_COMPONENT,
            value,
        }
    }
}

/// AMQP 1.0 shovel destination settings.
///
/// AMQP 1.0 uses [address-based](https://www.rabbitmq.com/docs/amqp#addresses) routing for message destinations.
/// The address typically corresponds to a queue name or topic pattern.
#[derive(Serialize)]
pub struct Amqp10ShovelDestinationParams<'a> {
    /// AMQP 1.0 URI of the destination broker
    pub destination_uri: &'a str,
    /// AMQP 1.0 [address](https://www.rabbitmq.com/docs/amqp#addresses) to publish to (queue name or topic pattern)
    pub destination_address: &'a str,
}

impl<'a> Amqp10ShovelDestinationParams<'a> {
    /// The address parameter specifies where to publish messages on the destination broker.
    /// This could be a queue name, topic pattern, or other address format supported by the broker.
    pub fn new(uri: &'a str, address: &'a str) -> Self {
        Self {
            destination_uri: uri,
            destination_address: address,
        }
    }
}

/// Used to specify custom properties that should be applied to messages
/// when they are re-published by shovels.
pub type MessageProperties = Map<String, Value>;

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
