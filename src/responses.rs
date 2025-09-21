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

//! Types in this module are used to represent API responses, such as [`QueueDefinition`], [`PolicyDefinition`],
//! [`User`], [`VirtualHost`], [`Shovel`] or [`FederationLink`].

use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::commons::{PolicyTarget, QueueType, Username, VirtualHostName};
use crate::formatting::*;
use serde::{
    Deserialize, Serialize,
    de::{MapAccess, Visitor},
};
use serde_json::Map;

#[cfg(feature = "tabled")]
use std::borrow::Cow;
#[cfg(feature = "tabled")]
use tabled::Tabled;

pub mod federation;
pub use federation::{FederationLink, FederationType, FederationUpstream};

pub mod shovel;
pub use shovel::{Shovel, ShovelPublishingState, ShovelState, ShovelType};

pub mod feature_flags;
pub use feature_flags::{FeatureFlag, FeatureFlagList, FeatureFlagStability, FeatureFlagState};

pub mod deprecations;
pub use deprecations::{DeprecatedFeature, DeprecatedFeatureList, DeprecationPhase};

pub mod health_checks;
pub use health_checks::{
    ClusterAlarmCheckDetails, HealthCheckFailureDetails, NoActivePortListenerDetails,
    NoActiveProtocolListenerDetails41AndLater, NoActiveProtocolListenerDetailsPre41,
    QuorumCriticalityCheckDetails, QuorumEndangeredQueue, ResourceAlarm,
};

pub mod parameters;
pub use parameters::{
    GlobalRuntimeParameter, GlobalRuntimeParameterValue, RuntimeParameter, RuntimeParameterValue,
    RuntimeParameterWithoutVirtualHost,
};

pub mod policies;
pub use policies::{Policy, PolicyDefinition, PolicyWithoutVirtualHost};

pub mod definitions;
pub use definitions::{
    BindingDefinition, BindingDefinitionWithoutVirtualHost, BindingInfo,
    BindingInfoWithoutVirtualHost, ClusterDefinitionSet, ExchangeDefinition,
    ExchangeDefinitionWithoutVirtualHost, ExchangeInfo, ExchangeInfoWithoutVirtualHost,
    NamedPolicyTargetObject, OptionalArgumentSourceOps, QueueDefinition,
    QueueDefinitionWithoutVirtualHost, QueueOps, VirtualHostDefinitionSet, XArguments,
};

pub mod tanzu;
pub use tanzu::{
    HostnamePortPairs, MessagingProtocol, OperatingMode, SchemaDefinitionSyncState,
    SchemaDefinitionSyncStatus, WarmStandbyReplicationInVirtualHost,
    WarmStandbyReplicationLinkStateOnDownstream, WarmStandbyReplicationState,
    WarmStandbyReplicationStateOnUpstream, WarmStandbyReplicationStatus,
};

pub mod vhosts;
pub use vhosts::{EnforcedLimits, VirtualHost, VirtualHostLimits, VirtualHostMetadata};

pub mod connections;
pub use connections::{
    ClientCapabilities, ClientProperties, Connection, ConnectionDetails, UserConnection,
};

pub mod channels;
pub use channels::{Channel, ChannelDetails, ChannelState};

pub mod cluster;
pub use cluster::{
    AuthenticationAttemptStatistics, ChurnRates, ClusterIdentity, ClusterNode, ClusterTags,
    Listener, NodeList, NodeMemoryBreakdown, NodeMemoryFootprint, NodeMemoryTotals, Overview,
};

pub mod permissions;
pub use permissions::{Permissions, TopicPermission};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct TagList(pub Vec<String>);

impl TagList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.iter().any(|s| s == key)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, String> {
        self.0.iter_mut()
    }
}

impl Deref for TagList {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for TagList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PluginList(pub Vec<String>);

impl PluginList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.iter().any(|s| s == key)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, String> {
        self.0.iter_mut()
    }
}

impl Deref for PluginList {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PluginList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for PluginList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

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

/// Represents a number of key OAuth 2 configuration settings.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct OAuthConfiguration {
    pub oauth_enabled: bool,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub oauth_client_id: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub oauth_provider_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct UserLimits {
    #[serde(rename(deserialize = "user"))]
    pub username: Username,
    #[serde(rename(deserialize = "value"))]
    pub limits: EnforcedLimits,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct User {
    pub name: Username,
    pub tags: TagList,
    pub password_hash: String,
}

impl User {
    pub fn with_name(&self, name: String) -> Self {
        Self {
            name,
            tags: self.tags.clone(),
            password_hash: self.password_hash.clone(),
        }
    }

    pub fn with_tags(&self, tags: TagList) -> Self {
        Self {
            name: self.name.clone(),
            tags,
            password_hash: self.password_hash.clone(),
        }
    }

    pub fn with_password_hash(&self, password_hash: String) -> Self {
        Self {
            name: self.name.clone(),
            tags: self.tags.clone(),
            password_hash,
        }
    }
}

/// Represents the currently authenticated user details returned by the `GET /api/whoami` endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct CurrentUser {
    pub name: Username,
    pub tags: TagList,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Consumer {
    pub consumer_tag: String,
    pub active: bool,
    #[serde(rename(deserialize = "ack_required"))]
    pub manual_ack: bool,
    pub prefetch_count: u32,
    pub exclusive: bool,
    pub arguments: XArguments,
    #[serde(rename(deserialize = "consumer_timeout"))]
    pub delivery_ack_timeout: u64,
    pub queue: NameAndVirtualHost,

    #[serde(deserialize_with = "deserialize_object_that_may_be_empty")]
    pub channel_details: Option<ChannelDetails>,
}

#[cfg(feature = "tabled")]
impl Tabled for Consumer {
    const LENGTH: usize = 9;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        let mut fds: Vec<Cow<'static, str>> = Vec::with_capacity(Self::LENGTH);
        let qinfo = &self.queue;
        fds.push(Cow::Owned(qinfo.vhost.clone()));
        fds.push(Cow::Owned(qinfo.name.clone()));
        fds.push(Cow::Owned(self.consumer_tag.clone()));
        fds.push(Cow::Owned(self.manual_ack.to_string()));
        fds.push(Cow::Owned(self.prefetch_count.to_string()));
        fds.push(Cow::Owned(self.active.to_string()));
        fds.push(Cow::Owned(self.exclusive.to_string()));
        fds.push(Cow::Owned(self.arguments.to_string()));
        fds.push(Cow::Owned(self.delivery_ack_timeout.to_string()));

        fds
    }

    fn headers() -> Vec<Cow<'static, str>> {
        let mut hds: Vec<Cow<'static, str>> = Vec::with_capacity(Self::LENGTH);
        hds.push(Cow::Borrowed("vhost"));
        hds.push(Cow::Borrowed("queue"));
        hds.push(Cow::Borrowed("consumer_tag"));
        hds.push(Cow::Borrowed("manual_ack"));
        hds.push(Cow::Borrowed("prefetch_count"));
        hds.push(Cow::Borrowed("active"));
        hds.push(Cow::Borrowed("exclusive"));
        hds.push(Cow::Borrowed("arguments"));
        hds.push(Cow::Borrowed("delivery_ack_timeout"));

        hds
    }
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

/// Garbage collection details for queue processes
#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct GarbageCollectionDetails {
    pub fullsweep_after: u32,
    pub max_heap_size: u32,
    pub min_bin_vheap_size: u32,
    pub min_heap_size: u32,
    pub minor_gcs: u32,
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

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct GetMessage {
    pub payload_bytes: u32,
    pub redelivered: bool,
    pub exchange: String,
    pub routing_key: String,
    pub message_count: u32,
    #[serde(deserialize_with = "deserialize_message_properties")]
    pub properties: MessageProperties,
    pub payload: String,
    pub payload_encoding: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct MessageList(pub Vec<GetMessage>);

impl MessageList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for MessageList {
    type Target = Vec<GetMessage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MessageList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for MessageList {
    type Item = GetMessage;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[allow(clippy::partialeq_ne_impl)]
impl PartialEq for MessageList {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }

    fn ne(&self, other: &Self) -> bool {
        self.0.ne(&other.0)
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct MessageRouted {
    pub routed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct MessageProperties(pub Map<String, serde_json::Value>);

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Rate {
    pub rate: f64,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct ObjectTotals {
    pub connections: u64,
    pub channels: u64,
    pub queues: u64,
    pub exchanges: u64,
    pub consumers: u64,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct QueueTotals {
    pub messages: u64,
    #[serde(rename = "messages_ready", default)]
    pub messages_ready_for_delivery: u64,
    #[serde(rename = "messages_unacknowledged", default)]
    pub messages_delivered_but_unacknowledged_by_consumers: u64,
    pub messages_details: Rate,
    #[serde(rename = "messages_ready_details", default)]
    pub messages_ready_for_delivery_details: Rate,
    #[serde(rename = "messages_unacknowledged_details", default)]
    pub messages_delivered_but_unacknowledged_by_consumers_details: Rate,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct MessageStats {
    /// Consumer delivery rate plus polling (via 'basic.get') rate
    #[serde(rename = "deliver_get_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub delivery_details: Option<Rate>,
    #[serde(rename = "publish_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub publishing_details: Option<Rate>,

    #[serde(rename = "deliver_no_ack_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub delivery_with_automatic_acknowledgement_details: Option<Rate>,
    #[serde(rename = "redeliver_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub redelivery_details: Option<Rate>,

    #[serde(rename = "confirm_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub publisher_confirmation_details: Option<Rate>,
    #[serde(rename = "ack_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub consumer_acknowledgement_details: Option<Rate>,

    #[serde(rename = "drop_unroutable_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub unroutable_dropped_message_details: Option<Rate>,
    #[serde(rename = "return_unroutable_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub unroutable_returned_message_details: Option<Rate>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct TagMap(pub Map<String, serde_json::Value>);

//
// Implementation
//

fn undefined() -> String {
    "?".to_string()
}

fn deserialize_map_or_seq<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    struct MapVisitor<T> {
        default: T,
    }

    impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for MapVisitor<T> {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map")
        }

        fn visit_seq<A>(self, _seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            // Treat a sequence as the default for the type.
            Ok(self.default)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let deserializer = serde::de::value::MapAccessDeserializer::new(map);
            let m = Deserialize::deserialize(deserializer)?;
            Ok(m)
        }
    }

    deserializer.deserialize_any(MapVisitor {
        default: T::default(),
    })
}

fn deserialize_message_properties<'de, D>(deserializer: D) -> Result<MessageProperties, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_map_or_seq::<MessageProperties, D>(deserializer)
}

pub fn deserialize_object_that_may_be_empty<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(
        untagged,
        deny_unknown_fields,
        expecting = "object, empty object or null"
    )]
    enum Helper<T> {
        Data(T),
        Empty {},
        Null,
    }
    match Helper::deserialize(deserializer) {
        Ok(Helper::Data(data)) => Ok(Some(data)),
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}
