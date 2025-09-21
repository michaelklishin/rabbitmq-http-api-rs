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

use crate::commons::QueueType;
use crate::requests::XArguments;
use crate::responses::QueueInfo;
use serde::Serialize;
use serde_json::{Map, Value, json};

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
