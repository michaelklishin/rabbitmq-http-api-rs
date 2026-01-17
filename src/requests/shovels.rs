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

use crate::commons::MessageTransferAcknowledgementMode;
use crate::error::ConversionError;
use crate::requests::parameters::RuntimeParameterDefinition;
use crate::responses::{RuntimeParameter, Shovel};
use serde::Serialize;
use serde_json::{Map, Value, json};

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
    pub reconnect_delay: Option<u32>,

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

        if params.source.predeclared {
            value.insert("src-predeclared".to_owned(), json!(true));
        }
        if params.destination.predeclared {
            value.insert("dest-predeclared".to_owned(), json!(true));
        }

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
    pub reconnect_delay: Option<u32>,

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
    /// AMQP 1.0 [address](https://www.rabbitmq.com/docs/amqp#addresses) to consume from (a queue name or topic pattern)
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
    /// AMQP 1.0 [address](https://www.rabbitmq.com/docs/amqp#addresses) to publish to (a queue name or topic pattern)
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

/// Type-safe AMQP 0-9-1 shovel source endpoint configuration.
///
/// This enum enforces that a source is either a queue or an exchange at compile time,
/// preventing invalid configurations where both or neither are specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amqp091ShovelSourceEndpoint<'a> {
    /// Consume messages from a queue
    Queue {
        uri: &'a str,
        queue: &'a str,
        predeclared: bool,
    },
    /// Consume messages from an exchange (creates a temporary queue bound to it)
    Exchange {
        uri: &'a str,
        exchange: &'a str,
        routing_key: Option<&'a str>,
        predeclared: bool,
    },
}

impl<'a> Amqp091ShovelSourceEndpoint<'a> {
    pub fn queue(uri: &'a str, queue: &'a str) -> Self {
        Self::Queue {
            uri,
            queue,
            predeclared: false,
        }
    }

    pub fn predeclared_queue(uri: &'a str, queue: &'a str) -> Self {
        Self::Queue {
            uri,
            queue,
            predeclared: true,
        }
    }

    pub fn exchange(uri: &'a str, exchange: &'a str, routing_key: Option<&'a str>) -> Self {
        Self::Exchange {
            uri,
            exchange,
            routing_key,
            predeclared: false,
        }
    }

    pub fn predeclared_exchange(
        uri: &'a str,
        exchange: &'a str,
        routing_key: Option<&'a str>,
    ) -> Self {
        Self::Exchange {
            uri,
            exchange,
            routing_key,
            predeclared: true,
        }
    }

    pub fn uri(&self) -> &'a str {
        match self {
            Self::Queue { uri, .. } => uri,
            Self::Exchange { uri, .. } => uri,
        }
    }

    pub fn is_predeclared(&self) -> bool {
        match self {
            Self::Queue { predeclared, .. } => *predeclared,
            Self::Exchange { predeclared, .. } => *predeclared,
        }
    }
}

impl<'a> From<Amqp091ShovelSourceEndpoint<'a>> for Amqp091ShovelSourceParams<'a> {
    fn from(endpoint: Amqp091ShovelSourceEndpoint<'a>) -> Self {
        match endpoint {
            Amqp091ShovelSourceEndpoint::Queue {
                uri,
                queue,
                predeclared,
            } => Self {
                source_uri: uri,
                source_queue: Some(queue),
                source_exchange: None,
                source_exchange_routing_key: None,
                predeclared,
            },
            Amqp091ShovelSourceEndpoint::Exchange {
                uri,
                exchange,
                routing_key,
                predeclared,
            } => Self {
                source_uri: uri,
                source_queue: None,
                source_exchange: Some(exchange),
                source_exchange_routing_key: routing_key,
                predeclared,
            },
        }
    }
}

/// Type-safe AMQP 0-9-1 shovel destination endpoint configuration.
///
/// This enum enforces that a destination is either a queue or an exchange at compile time,
/// preventing invalid configurations where both or neither are specified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Amqp091ShovelDestinationEndpoint<'a> {
    /// Publish messages to a queue
    Queue {
        uri: &'a str,
        queue: &'a str,
        predeclared: bool,
    },
    /// Publish messages to an exchange
    Exchange {
        uri: &'a str,
        exchange: &'a str,
        routing_key: Option<&'a str>,
        predeclared: bool,
    },
}

impl<'a> Amqp091ShovelDestinationEndpoint<'a> {
    pub fn queue(uri: &'a str, queue: &'a str) -> Self {
        Self::Queue {
            uri,
            queue,
            predeclared: false,
        }
    }

    pub fn predeclared_queue(uri: &'a str, queue: &'a str) -> Self {
        Self::Queue {
            uri,
            queue,
            predeclared: true,
        }
    }

    pub fn exchange(uri: &'a str, exchange: &'a str, routing_key: Option<&'a str>) -> Self {
        Self::Exchange {
            uri,
            exchange,
            routing_key,
            predeclared: false,
        }
    }

    pub fn predeclared_exchange(
        uri: &'a str,
        exchange: &'a str,
        routing_key: Option<&'a str>,
    ) -> Self {
        Self::Exchange {
            uri,
            exchange,
            routing_key,
            predeclared: true,
        }
    }

    pub fn uri(&self) -> &'a str {
        match self {
            Self::Queue { uri, .. } => uri,
            Self::Exchange { uri, .. } => uri,
        }
    }

    pub fn is_predeclared(&self) -> bool {
        match self {
            Self::Queue { predeclared, .. } => *predeclared,
            Self::Exchange { predeclared, .. } => *predeclared,
        }
    }
}

impl<'a> From<Amqp091ShovelDestinationEndpoint<'a>> for Amqp091ShovelDestinationParams<'a> {
    fn from(endpoint: Amqp091ShovelDestinationEndpoint<'a>) -> Self {
        match endpoint {
            Amqp091ShovelDestinationEndpoint::Queue {
                uri,
                queue,
                predeclared,
            } => Self {
                destination_uri: uri,
                destination_queue: Some(queue),
                destination_exchange: None,
                destination_exchange_routing_key: None,
                predeclared,
            },
            Amqp091ShovelDestinationEndpoint::Exchange {
                uri,
                exchange,
                routing_key,
                predeclared,
            } => Self {
                destination_uri: uri,
                destination_queue: None,
                destination_exchange: Some(exchange),
                destination_exchange_routing_key: routing_key,
                predeclared,
            },
        }
    }
}

/// Owned version of [`Amqp091ShovelSourceEndpoint`] for cases where owned strings are needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedAmqp091ShovelSourceEndpoint {
    Queue {
        uri: String,
        queue: String,
        predeclared: bool,
    },
    Exchange {
        uri: String,
        exchange: String,
        routing_key: Option<String>,
        predeclared: bool,
    },
}

impl OwnedAmqp091ShovelSourceEndpoint {
    pub fn queue(uri: impl Into<String>, queue: impl Into<String>) -> Self {
        Self::Queue {
            uri: uri.into(),
            queue: queue.into(),
            predeclared: false,
        }
    }

    pub fn predeclared_queue(uri: impl Into<String>, queue: impl Into<String>) -> Self {
        Self::Queue {
            uri: uri.into(),
            queue: queue.into(),
            predeclared: true,
        }
    }

    pub fn exchange(
        uri: impl Into<String>,
        exchange: impl Into<String>,
        routing_key: Option<String>,
    ) -> Self {
        Self::Exchange {
            uri: uri.into(),
            exchange: exchange.into(),
            routing_key,
            predeclared: false,
        }
    }

    pub fn predeclared_exchange(
        uri: impl Into<String>,
        exchange: impl Into<String>,
        routing_key: Option<String>,
    ) -> Self {
        Self::Exchange {
            uri: uri.into(),
            exchange: exchange.into(),
            routing_key,
            predeclared: true,
        }
    }

    pub fn uri(&self) -> &str {
        match self {
            Self::Queue { uri, .. } => uri,
            Self::Exchange { uri, .. } => uri,
        }
    }

    pub fn is_predeclared(&self) -> bool {
        match self {
            Self::Queue { predeclared, .. } => *predeclared,
            Self::Exchange { predeclared, .. } => *predeclared,
        }
    }

    pub fn as_ref(&self) -> Amqp091ShovelSourceEndpoint<'_> {
        match self {
            Self::Queue {
                uri,
                queue,
                predeclared,
            } => Amqp091ShovelSourceEndpoint::Queue {
                uri,
                queue,
                predeclared: *predeclared,
            },
            Self::Exchange {
                uri,
                exchange,
                routing_key,
                predeclared,
            } => Amqp091ShovelSourceEndpoint::Exchange {
                uri,
                exchange,
                routing_key: routing_key.as_deref(),
                predeclared: *predeclared,
            },
        }
    }
}

/// Owned version of [`Amqp091ShovelDestinationEndpoint`] for cases where owned strings are needed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnedAmqp091ShovelDestinationEndpoint {
    Queue {
        uri: String,
        queue: String,
        predeclared: bool,
    },
    Exchange {
        uri: String,
        exchange: String,
        routing_key: Option<String>,
        predeclared: bool,
    },
}

impl OwnedAmqp091ShovelDestinationEndpoint {
    pub fn queue(uri: impl Into<String>, queue: impl Into<String>) -> Self {
        Self::Queue {
            uri: uri.into(),
            queue: queue.into(),
            predeclared: false,
        }
    }

    pub fn predeclared_queue(uri: impl Into<String>, queue: impl Into<String>) -> Self {
        Self::Queue {
            uri: uri.into(),
            queue: queue.into(),
            predeclared: true,
        }
    }

    pub fn exchange(
        uri: impl Into<String>,
        exchange: impl Into<String>,
        routing_key: Option<String>,
    ) -> Self {
        Self::Exchange {
            uri: uri.into(),
            exchange: exchange.into(),
            routing_key,
            predeclared: false,
        }
    }

    pub fn predeclared_exchange(
        uri: impl Into<String>,
        exchange: impl Into<String>,
        routing_key: Option<String>,
    ) -> Self {
        Self::Exchange {
            uri: uri.into(),
            exchange: exchange.into(),
            routing_key,
            predeclared: true,
        }
    }

    pub fn uri(&self) -> &str {
        match self {
            Self::Queue { uri, .. } => uri,
            Self::Exchange { uri, .. } => uri,
        }
    }

    pub fn is_predeclared(&self) -> bool {
        match self {
            Self::Queue { predeclared, .. } => *predeclared,
            Self::Exchange { predeclared, .. } => *predeclared,
        }
    }

    pub fn as_ref(&self) -> Amqp091ShovelDestinationEndpoint<'_> {
        match self {
            Self::Queue {
                uri,
                queue,
                predeclared,
            } => Amqp091ShovelDestinationEndpoint::Queue {
                uri,
                queue,
                predeclared: *predeclared,
            },
            Self::Exchange {
                uri,
                exchange,
                routing_key,
                predeclared,
            } => Amqp091ShovelDestinationEndpoint::Exchange {
                uri,
                exchange,
                routing_key: routing_key.as_deref(),
                predeclared: *predeclared,
            },
        }
    }
}

/// Used to specify custom properties that should be applied to messages
/// when they are re-published by shovels.
pub type MessageProperties = Map<String, Value>;

/// A helper type used for conversion between shovels and runtime parameters
/// (the end goal is usually modifying a shovel).
#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct OwnedShovelParams {
    pub name: String,
    pub vhost: String,
    pub source_protocol: String,
    pub destination_protocol: String,
    pub acknowledgement_mode: MessageTransferAcknowledgementMode,
    pub reconnect_delay: Option<u32>,

    pub source_uri: String,
    pub source_queue: Option<String>,
    pub source_exchange: Option<String>,
    pub source_exchange_routing_key: Option<String>,
    pub source_address: Option<String>,
    pub source_predeclared: Option<bool>,

    pub destination_uri: String,
    pub destination_queue: Option<String>,
    pub destination_exchange: Option<String>,
    pub destination_exchange_routing_key: Option<String>,
    pub destination_address: Option<String>,
    pub destination_predeclared: Option<bool>,
}

impl OwnedShovelParams {
    /// Returns a copy with the source URI replaced.
    pub fn with_source_uri(mut self, uri: impl Into<String>) -> Self {
        self.source_uri = uri.into();
        self
    }

    /// Returns a copy with the destination URI replaced.
    pub fn with_destination_uri(mut self, uri: impl Into<String>) -> Self {
        self.destination_uri = uri.into();
        self
    }
}

impl TryFrom<RuntimeParameter> for OwnedShovelParams {
    type Error = ConversionError;

    fn try_from(param: RuntimeParameter) -> Result<Self, Self::Error> {
        let values = &param.value.0;

        let source_protocol = values
            .get("src-protocol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingProperty {
                argument: "src-protocol".to_string(),
            })?
            .to_string();

        let destination_protocol = values
            .get("dest-protocol")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingProperty {
                argument: "dest-protocol".to_string(),
            })?
            .to_string();

        let source_uri = values
            .get("src-uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingProperty {
                argument: "src-uri".to_string(),
            })?
            .to_string();

        let destination_uri = values
            .get("dest-uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingProperty {
                argument: "dest-uri".to_string(),
            })?
            .to_string();

        let acknowledgement_mode = values
            .get("ack-mode")
            .and_then(|v| v.as_str())
            .map(MessageTransferAcknowledgementMode::from)
            .unwrap_or_default();

        let reconnect_delay = values
            .get("reconnect-delay")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let source_queue = values
            .get("src-queue")
            .and_then(|v| v.as_str())
            .map(String::from);

        let source_exchange = values
            .get("src-exchange")
            .and_then(|v| v.as_str())
            .map(String::from);

        let source_exchange_routing_key = values
            .get("src-exchange-key")
            .and_then(|v| v.as_str())
            .map(String::from);

        let source_address = values
            .get("src-address")
            .and_then(|v| v.as_str())
            .map(String::from);

        let source_predeclared = values.get("src-predeclared").and_then(|v| v.as_bool());

        let destination_queue = values
            .get("dest-queue")
            .and_then(|v| v.as_str())
            .map(String::from);

        let destination_exchange = values
            .get("dest-exchange")
            .and_then(|v| v.as_str())
            .map(String::from);

        let destination_exchange_routing_key = values
            .get("dest-exchange-key")
            .and_then(|v| v.as_str())
            .map(String::from);

        let destination_address = values
            .get("dest-address")
            .and_then(|v| v.as_str())
            .map(String::from);

        let destination_predeclared = values.get("dest-predeclared").and_then(|v| v.as_bool());

        Ok(OwnedShovelParams {
            name: param.name,
            vhost: param.vhost.to_string(),
            source_protocol,
            destination_protocol,
            acknowledgement_mode,
            reconnect_delay,
            source_uri,
            source_queue,
            source_exchange,
            source_exchange_routing_key,
            source_address,
            source_predeclared,
            destination_uri,
            destination_queue,
            destination_exchange,
            destination_exchange_routing_key,
            destination_address,
            destination_predeclared,
        })
    }
}

impl<'a> From<&'a OwnedShovelParams> for RuntimeParameterDefinition<'a> {
    fn from(params: &'a OwnedShovelParams) -> Self {
        let mut value = Map::new();

        value.insert("src-protocol".to_owned(), json!(params.source_protocol));
        value.insert(
            "dest-protocol".to_owned(),
            json!(params.destination_protocol),
        );
        value.insert("src-uri".to_owned(), json!(params.source_uri));
        value.insert("dest-uri".to_owned(), json!(params.destination_uri));
        value.insert("ack-mode".to_owned(), json!(params.acknowledgement_mode));

        if let Some(delay) = params.reconnect_delay {
            value.insert("reconnect-delay".to_owned(), json!(delay));
        }

        if let Some(queue) = &params.source_queue {
            value.insert("src-queue".to_owned(), json!(queue));
        }
        if let Some(exchange) = &params.source_exchange {
            value.insert("src-exchange".to_owned(), json!(exchange));
        }
        if let Some(key) = &params.source_exchange_routing_key {
            value.insert("src-exchange-key".to_owned(), json!(key));
        }
        if let Some(address) = &params.source_address {
            value.insert("src-address".to_owned(), json!(address));
        }
        if let Some(predeclared) = params.source_predeclared {
            value.insert("src-predeclared".to_owned(), json!(predeclared));
        }

        if let Some(queue) = &params.destination_queue {
            value.insert("dest-queue".to_owned(), json!(queue));
        }
        if let Some(exchange) = &params.destination_exchange {
            value.insert("dest-exchange".to_owned(), json!(exchange));
        }
        if let Some(key) = &params.destination_exchange_routing_key {
            value.insert("dest-exchange-key".to_owned(), json!(key));
        }
        if let Some(address) = &params.destination_address {
            value.insert("dest-address".to_owned(), json!(address));
        }
        if let Some(predeclared) = params.destination_predeclared {
            value.insert("dest-predeclared".to_owned(), json!(predeclared));
        }

        Self {
            name: &params.name,
            vhost: &params.vhost,
            component: SHOVEL_COMPONENT,
            value,
        }
    }
}

impl From<Shovel> for OwnedShovelParams {
    fn from(shovel: Shovel) -> Self {
        Self {
            name: shovel.name,
            vhost: shovel.vhost.unwrap_or_default(),
            source_protocol: shovel
                .source_protocol
                .map(|p| p.to_string())
                .unwrap_or_default(),
            destination_protocol: shovel
                .destination_protocol
                .map(|p| p.to_string())
                .unwrap_or_default(),
            acknowledgement_mode: MessageTransferAcknowledgementMode::default(),
            reconnect_delay: None,
            source_uri: shovel.source_uri.unwrap_or_default(),
            source_queue: shovel.source,
            source_exchange: None,
            source_exchange_routing_key: None,
            source_address: shovel.source_address,
            source_predeclared: None,
            destination_uri: shovel.destination_uri.unwrap_or_default(),
            destination_queue: shovel.destination,
            destination_exchange: None,
            destination_exchange_routing_key: None,
            destination_address: shovel.destination_address,
            destination_predeclared: None,
        }
    }
}
