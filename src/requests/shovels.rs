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
use crate::requests::parameters::RuntimeParameterDefinition;
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

/// Used to specify custom properties that should be applied to messages
/// when they are re-published by shovels.
pub type MessageProperties = Map<String, Value>;
