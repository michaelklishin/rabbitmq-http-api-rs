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
use crate::commons::{ExchangeType, MessageTransferAcknowledgementMode, PolicyTarget, QueueType};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Properties of a [virtual host](https://rabbitmq.com/docs/vhosts/) to be created or updated.
#[derive(Serialize)]
pub struct VirtualHostParams<'a> {
    /// Virtual host name
    pub name: &'a str,
    /// Optional description, e.g. what purpose does this virtual host serve?
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    // A list of virtual host tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<&'a str>>,
    //
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_queue_type: Option<QueueType>,
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

/// Represents resource usage a limit to be enforced
/// on a [virtual host](https://rabbitmq.com/docs/vhosts/) or a user.
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
#[derive(Serialize)]
pub struct UserParams<'a> {
    /// Username
    pub name: &'a str,
    /// Hashed and salted password of the user.
    /// Use functions in [`crate::password_hashing`] instead of [manually salting and hashing values](https://rabbitmq.com/docs/passwords/#computing-password-hash).
    pub password_hash: &'a str,
    /// A comma-separate list of user tags
    pub tags: &'a str,
}

pub type XArguments = Option<Map<String, Value>>;

/// [Queue](https://rabbitmq.com/docs/queues/) properties used at declaration time.
/// Prefer constructor functions, they correctly put [`QueueType`] to the optional
/// argument map.
#[derive(Serialize, Debug)]
pub struct QueueParams<'a> {
    /// The name of the queue to declare.
    /// Must be no longer than 255 bytes in length.
    pub name: &'a str,
    /// The type of the queue to declare, such as
    /// [quorum](https://rabbitmq.com/docs/quorum-queues.html), classic, or [stream](https://rabbitmq.com/streams/)
    #[serde(skip_serializing)]
    pub queue_type: QueueType,
    /// [Queue durability](https://rabbitmq.com/docs/queues/#durability)
    pub durable: bool,
    /// Should this queue be an [auto-delete](https://rabbitmq.com/docs/queues/#temporary-queues) one?
    pub auto_delete: bool,
    /// Should this queue be an [exclusive](https://rabbitmq.com/docs/queues/#temporary-queues) one?
    pub exclusive: bool,
    /// [Optional queue arguments](https://rabbitmq.com/docs/queues/#optional-arguments)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: XArguments,
}

impl<'a> QueueParams<'a> {
    /// Instantiates a [`QueueParams`] of a [quorum queue](https://rabbitmq.com/docs/quorum-queues/).
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

    /// Instantiates a [`QueueParams`] of a [stream](https://rabbitmq.com/docs/streams/).
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

    /// Instantiates a [`QueueParams`] of a classic [durable queue](https://rabbitmq.com/docs/queues/).
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
    pub expiration: &'a str,
    pub max_length_bytes: Option<u64>,
    pub max_segment_length_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: XArguments,
}

impl<'a> StreamParams<'a> {
    pub fn new(name: &'a str, expiration: &'a str) -> Self {
        Self {
            name,
            expiration,
            max_length_bytes: None,
            max_segment_length_bytes: None,
            arguments: None,
        }
    }

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

/// Exchange properties used at queue declaration time
#[derive(Debug, Serialize)]
pub struct ExchangeParams<'a> {
    #[serde(skip_serializing)]
    pub name: &'a str,
    #[serde(rename(serialize = "type"))]
    pub exchange_type: ExchangeType,
    pub durable: bool,
    pub auto_delete: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: XArguments,
}

impl<'a> ExchangeParams<'a> {
    pub fn durable(name: &'a str, exchange_type: ExchangeType, optional_args: XArguments) -> Self {
        Self::new(name, exchange_type, true, false, optional_args)
    }

    /// Instantiates a [`ExchangeParams`] of a [fanout exchange](https://rabbitmq.com/docs/tutorials/tutorial-three-python/).
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

    /// Instantiates a [`ExchangeParams`] of a durable [fanout exchange](https://rabbitmq.com/docs/tutorials/tutorial-three-python/).
    pub fn durable_fanout(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Fanout, true, false, optional_args)
    }

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

    /// Instantiates a [`ExchangeParams`] of a durable [fanout exchange](https://rabbitmq.com/docs/tutorials/tutorial-five-python/).
    pub fn durable_topic(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Topic, true, false, optional_args)
    }

    /// Instantiates a [`ExchangeParams`] of a [direct exchange](https://rabbitmq.com/docs/tutorials/tutorial-four-python/).
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

    /// Instantiates a [`ExchangeParams`] of a durable [direct exchange](https://rabbitmq.com/docs/tutorials/tutorial-four-python/).
    pub fn durable_direct(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Direct, true, false, optional_args)
    }

    /// Instantiates a [`ExchangeParams`] of a headers exchange
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

    /// Instantiates a [`ExchangeParams`] of a durable headers exchange
    pub fn durable_headers(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::Headers, true, false, optional_args)
    }

    /// Instantiates a [`ExchangeParams`] of a headers exchange
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

    /// Instantiates a [`ExchangeParams`] of a durable headers exchange
    pub fn durable_local_random(name: &'a str, optional_args: XArguments) -> Self {
        Self::new(name, ExchangeType::LocalRandom, true, false, optional_args)
    }

    /// Instantiates a [`ExchangeParams`] of a custom (plugin-provided) type
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

#[derive(Serialize, Deserialize)]
pub struct BulkUserDelete<'a> {
    #[serde(borrow, rename = "users")]
    pub usernames: Vec<&'a str>,
}

pub type RuntimeParameterValue = Map<String, Value>;

/// Represents a [runtime parameter](https://rabbitmq.com/docs/parameters/).
#[derive(Serialize, Deserialize)]
pub struct RuntimeParameterDefinition<'a> {
    pub name: &'a str,
    pub vhost: &'a str,
    pub component: &'a str,
    pub value: RuntimeParameterValue,
}

pub type PolicyDefinition = Option<Map<String, Value>>;

/// Represents a [policy](https://rabbitmq.com/docs/parameters/#policies).
#[derive(Serialize)]
pub struct PolicyParams<'a> {
    pub vhost: &'a str,
    pub name: &'a str,
    pub pattern: &'a str,
    #[serde(rename(serialize = "apply-to"))]
    pub apply_to: PolicyTarget,
    pub priority: i32,
    pub definition: PolicyDefinition,
}

/// Represents a user's [permission in a particular virtual host](https://rabbitmq.com/docs/access-control/).
#[derive(Serialize)]
pub struct Permissions<'a> {
    pub user: &'a str,
    pub vhost: &'a str,
    pub configure: &'a str,
    pub read: &'a str,
    pub write: &'a str,
}

#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FederationResourceCleanupMode {
    #[default]
    Default,
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

pub const FEDERATION_UPSTREAM_COMPONENT: &str = "federation-upstream";

/// Represents a set of queue federation parameters
/// that are associated with an upstream.
pub struct QueueFederationParams<'a> {
    pub queue: Option<&'a str>,
    pub consumer_tag: Option<&'a str>,
}

impl<'a> QueueFederationParams<'a> {
    pub fn new(queue: &'a str) -> Self {
        Self {
            queue: Some(queue),
            consumer_tag: None,
        }
    }

    pub fn new_with_consumer_tag(queue: &'a str, consumer_tag: &'a str) -> Self {
        Self {
            queue: Some(queue),
            consumer_tag: Some(consumer_tag),
        }
    }
}

/// Represents a set of exchange federation parameters
/// that are associated with an upstream.
pub struct ExchangeFederationParams<'a> {
    pub exchange: Option<&'a str>,
    pub max_hops: Option<u8>,
    pub queue_type: QueueType,
    pub ttl: Option<u32>,
    pub message_ttl: Option<u32>,
    pub resource_cleanup_mode: FederationResourceCleanupMode,
}

impl ExchangeFederationParams<'_> {
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
            component: SHOVEL_COMPONENT,
            value,
        }
    }
}

pub const SHOVEL_COMPONENT: &str = "shovel";

/// Represents a dynamic AMQP 0-9-1 shovel definition.
#[derive(Serialize)]
pub struct Amqp091ShovelParams<'a> {
    pub name: &'a str,
    pub vhost: &'a str,

    pub acknowledgement_mode: MessageTransferAcknowledgementMode,
    pub reconnect_delay: Option<u16>,

    pub source: Amqp091ShovelSourceParams<'a>,
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

#[derive(Serialize)]
pub struct Amqp091ShovelSourceParams<'a> {
    pub source_uri: &'a str,

    pub source_queue: Option<&'a str>,

    pub source_exchange: Option<&'a str>,
    pub source_exchange_routing_key: Option<&'a str>,

    pub predeclared: bool,
}

impl<'a> Amqp091ShovelSourceParams<'a> {
    pub fn queue_source(source_uri: &'a str, source_queue: &'a str) -> Self {
        Self {
            source_uri,
            source_queue: Some(source_queue),

            source_exchange: None,
            source_exchange_routing_key: None,

            predeclared: false,
        }
    }

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

    pub fn predeclared_queue_source(source_uri: &'a str, source_queue: &'a str) -> Self {
        Self {
            source_uri,
            source_queue: Some(source_queue),

            source_exchange: None,
            source_exchange_routing_key: None,

            predeclared: true,
        }
    }

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

#[derive(Serialize)]
pub struct Amqp091ShovelDestinationParams<'a> {
    pub destination_uri: &'a str,

    pub destination_queue: Option<&'a str>,
    pub destination_exchange: Option<&'a str>,
    pub destination_exchange_routing_key: Option<&'a str>,

    pub predeclared: bool,
}

impl<'a> Amqp091ShovelDestinationParams<'a> {
    pub fn queue_destination(destination_uri: &'a str, destination_queue: &'a str) -> Self {
        Self {
            destination_uri,
            destination_queue: Some(destination_queue),

            destination_exchange: None,
            destination_exchange_routing_key: None,

            predeclared: false,
        }
    }

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

/// Represents a dynamic shovel definition.
#[derive(Serialize)]
pub struct Amqp10ShovelParams<'a> {
    pub name: &'a str,
    pub vhost: &'a str,

    pub acknowledgement_mode: MessageTransferAcknowledgementMode,
    pub reconnect_delay: Option<u16>,

    pub source: Amqp10ShovelSourceParams<'a>,
    pub destination: Amqp10ShovelDestinationParams<'a>,
}

#[derive(Serialize)]
pub struct Amqp10ShovelSourceParams<'a> {
    pub source_uri: &'a str,
    pub source_address: &'a str,
}

impl<'a> Amqp10ShovelSourceParams<'a> {
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

#[derive(Serialize)]
pub struct Amqp10ShovelDestinationParams<'a> {
    pub destination_uri: &'a str,
    pub destination_address: &'a str,
}

impl<'a> Amqp10ShovelDestinationParams<'a> {
    pub fn new(uri: &'a str, address: &'a str) -> Self {
        Self {
            destination_uri: uri,
            destination_address: address,
        }
    }
}

pub type MessageProperties = Map<String, Value>;

#[derive(Serialize, Default)]
pub struct EmptyPayload;

impl EmptyPayload {
    pub fn new() -> Self {
        Self
    }
}
