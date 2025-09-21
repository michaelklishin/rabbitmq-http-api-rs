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

use crate::commons::ExchangeType;
use crate::requests::XArguments;
use crate::responses::ExchangeInfo;
use serde::Serialize;
use serde_json::Value;

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
