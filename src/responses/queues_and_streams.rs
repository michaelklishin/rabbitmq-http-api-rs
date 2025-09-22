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

use std::fmt;

use crate::commons::{PolicyTarget, QueueType, VirtualHostName};
use crate::responses::{
    ConnectionDetails, NodeList, Rate,
    cluster::GarbageCollectionDetails,
    definitions::{NamedPolicyTargetObject, QueueOps, XArguments},
    policies::Policy,
};
use serde::Deserialize;

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct StreamPublisher {
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub connection_details: ConnectionDetails,
    pub queue: NameAndVirtualHost,
    pub reference: String,
    pub publisher_id: u32,
    pub published: u64,
    pub confirmed: u64,
    pub errored: u64,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct StreamConsumer {
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub connection_details: ConnectionDetails,
    pub queue: NameAndVirtualHost,
    pub subscription_id: u32,
    pub credits: u64,
    pub consumed: u64,
    pub offset_lag: u64,
    pub offset: u64,
    #[cfg_attr(feature = "tabled", tabled(display = "display_arg_table"))]
    pub properties: XArguments,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct NameAndVirtualHost {
    pub name: String,
    #[serde(rename(deserialize = "vhost"))]
    pub vhost: VirtualHostName,
}

impl fmt::Display for NameAndVirtualHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "'{}' in virtual host '{}'", self.name, self.vhost)
    }
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct QueueInfo {
    pub name: String,
    pub vhost: VirtualHostName,
    #[serde(rename(deserialize = "type"))]
    pub queue_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub exclusive: bool,
    #[cfg_attr(feature = "tabled", tabled(display = "display_arg_table"))]
    pub arguments: XArguments,

    #[serde(default = "undefined")]
    pub node: String,
    #[serde(default)]
    pub state: String,
    // only quorum queues and streams will have this
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub leader: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub members: Option<NodeList>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub online: Option<NodeList>,

    #[serde(default)]
    pub memory: u64,
    #[serde(rename(deserialize = "consumers"))]
    #[serde(default)]
    pub consumer_count: u16,
    #[serde(default)]
    pub consumer_utilisation: f32,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub exclusive_consumer_tag: Option<String>,

    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub policy: Option<String>,

    #[serde(default)]
    pub message_bytes: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_persistent: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_ram: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_ready: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_unacknowledged: u64,

    #[serde(rename(deserialize = "messages"))]
    #[serde(default)]
    pub message_count: u64,
    #[serde(rename(deserialize = "messages_persistent"))]
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub on_disk_message_count: u64,
    #[serde(rename(deserialize = "messages_ram"))]
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub in_memory_message_count: u64,
    #[serde(rename(deserialize = "messages_unacknowledged"))]
    #[serde(default)]
    pub unacknowledged_message_count: u64,
}

/// Represents detailed queue information with extended metrics and garbage collection details.
/// This is an enhanced version of `QueueInfo` that includes additional fields from the detailed queues endpoint.
#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct DetailedQueueInfo {
    pub name: String,
    pub vhost: VirtualHostName,
    #[serde(rename(deserialize = "type"))]
    pub queue_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub exclusive: bool,
    #[cfg_attr(feature = "tabled", tabled(display = "display_arg_table"))]
    pub arguments: XArguments,

    #[serde(default = "undefined")]
    pub node: String,
    #[serde(default)]
    pub state: String,
    // only quorum queues and streams will have this
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub leader: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub members: Option<NodeList>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub online: Option<NodeList>,

    #[serde(default)]
    pub memory: u64,
    #[serde(rename(deserialize = "consumers"))]
    #[serde(default)]
    pub consumer_count: u16,
    #[serde(default)]
    pub consumer_utilisation: f32,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub exclusive_consumer_tag: Option<String>,

    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub policy: Option<String>,

    #[serde(default)]
    pub message_bytes: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_persistent: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_ram: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_ready: u64,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub message_bytes_unacknowledged: u64,

    #[serde(rename(deserialize = "messages"))]
    #[serde(default)]
    pub message_count: u64,
    #[serde(rename(deserialize = "messages_persistent"))]
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub on_disk_message_count: u64,
    #[serde(rename(deserialize = "messages_ram"))]
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub in_memory_message_count: u64,
    #[serde(rename(deserialize = "messages_unacknowledged"))]
    #[serde(default)]
    pub unacknowledged_message_count: u64,

    // Additional detailed fields
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub garbage_collection: Option<GarbageCollectionDetails>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_batch_size: Option<u32>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_batch_size_avg: Option<f64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_batch_size_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_file_handle_open_attempt_avg_time: Option<f64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_file_handle_open_attempt_avg_time_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_read_avg_time: Option<f64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_read_avg_time_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_read_bytes: Option<u64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_read_bytes_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_read_count: Option<u64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_read_count_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_reopen_count: Option<u64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_reopen_count_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_seek_avg_time: Option<f64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_seek_avg_time_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_seek_count: Option<u64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_seek_count_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_sync_avg_time: Option<f64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_sync_avg_time_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_sync_count: Option<u64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_sync_count_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_write_avg_time: Option<f64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_write_avg_time_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_write_bytes: Option<u64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_write_bytes_details: Option<Rate>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_write_count: Option<u64>,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub io_write_count_details: Option<Rate>,
}

impl QueueOps for DetailedQueueInfo {
    fn name(&self) -> &str {
        &self.name
    }

    fn queue_type(&self) -> QueueType {
        QueueType::from(self.queue_type.as_str())
    }

    fn policy_target_type(&self) -> PolicyTarget {
        PolicyTarget::from(self.queue_type())
    }

    fn x_arguments(&self) -> &XArguments {
        &self.arguments
    }
}

impl NamedPolicyTargetObject for DetailedQueueInfo {
    fn vhost(&self) -> String {
        self.vhost.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn policy_target(&self) -> PolicyTarget {
        self.policy_target_type()
    }

    fn does_match(&self, policy: &Policy) -> bool {
        policy.does_match_object(self)
    }
}

impl QueueOps for QueueInfo {
    fn name(&self) -> &str {
        &self.name
    }

    fn queue_type(&self) -> QueueType {
        QueueType::from(self.queue_type.as_str())
    }

    fn policy_target_type(&self) -> PolicyTarget {
        PolicyTarget::from(self.queue_type())
    }

    fn x_arguments(&self) -> &XArguments {
        &self.arguments
    }
}

impl NamedPolicyTargetObject for QueueInfo {
    fn vhost(&self) -> String {
        self.vhost.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn policy_target(&self) -> PolicyTarget {
        self.policy_target_type()
    }

    fn does_match(&self, policy: &Policy) -> bool {
        policy.does_match_object(self)
    }
}

fn undefined() -> String {
    "?".to_string()
}
