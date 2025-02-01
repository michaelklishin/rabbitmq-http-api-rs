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
use crate::commons::{ExchangeType, PolicyTarget, QueueType};
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
pub struct RuntimeParameterDefinition {
    pub name: String,
    pub vhost: String,
    pub component: String,
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

pub type MessageProperties = Map<String, Value>;

#[derive(Serialize, Default)]
pub struct EmptyPayload;

impl EmptyPayload {
    pub fn new() -> Self {
        Self
    }
}
