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

use crate::commons::{PolicyTarget, QueueType, SupportedProtocol, Username, VirtualHostName};
use crate::formatting::*;
use crate::utils::{percentage, percentage_as_text};
use serde::{
    Deserialize, Serialize,
    de::{MapAccess, Visitor},
};
use serde_aux::prelude::*;
use serde_json::Map;

#[cfg(feature = "tabled")]
use std::borrow::Cow;
use std::fmt::Formatter;
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

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct NodeList(Vec<String>);

impl fmt::Display for NodeList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_vertical_list_without_bullets(f, &self.0)
    }
}

impl NodeList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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

impl Deref for NodeList {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for NodeList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for NodeList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct NodeMemoryFootprint {
    #[serde(rename = "memory")]
    pub breakdown: NodeMemoryBreakdown,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct NodeMemoryTotals {
    pub rss: u64,
    pub allocated: u64,
    #[serde(rename = "erlang")]
    pub used_by_runtime: u64,
}

impl NodeMemoryTotals {
    /// Returns the greatest value between the totals computed
    /// using different mechanisms (RSS, runtime allocator metrics)
    pub fn max(&self) -> u64 {
        std::cmp::max(std::cmp::max(self.used_by_runtime, self.rss), self.rss)
    }
}

impl fmt::Display for NodeMemoryTotals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "RSS: {}", self.rss)?;
        writeln!(f, "allocated: {}", self.allocated)?;
        writeln!(f, "used by the runtime: {}", self.used_by_runtime)?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct NodeMemoryBreakdown {
    pub connection_readers: u64,
    pub connection_writers: u64,
    pub connection_channels: u64,
    pub connection_other: u64,
    #[serde(rename = "queue_procs")]
    pub classic_queue_procs: u64,
    pub quorum_queue_procs: u64,
    pub stream_queue_procs: u64,
    pub stream_queue_replica_reader_procs: u64,
    pub stream_queue_coordinator_procs: u64,
    pub plugins: u64,
    pub metadata_store: u64,
    #[serde(rename = "other_proc")]
    pub other_procs: u64,
    pub metrics: u64,
    #[serde(rename = "mgmt_db")]
    pub management_db: u64,
    pub mnesia: u64,
    #[serde(rename = "quorum_ets")]
    pub quorum_queue_ets_tables: u64,
    #[serde(rename = "metadata_store_ets")]
    pub metadata_store_ets_tables: u64,
    #[serde(rename = "other_ets")]
    pub other_ets_tables: u64,
    #[serde(rename = "binary")]
    pub binary_heap: u64,
    #[serde(rename = "msg_index")]
    pub message_indices: u64,
    pub code: u64,
    #[serde(rename = "atom")]
    pub atom_table: u64,
    pub other_system: u64,
    #[serde(rename = "allocated_unused")]
    pub allocated_but_unused: u64,
    #[serde(rename = "reserved_unallocated")]
    pub reserved_but_unallocated: u64,
    #[serde(rename = "strategy")]
    pub calculation_strategy: String,
    pub total: NodeMemoryTotals,
}

macro_rules! percentage_fn {
    ($fn_name:ident, $field:ident) => {
        pub fn $fn_name(&mut self) -> f64 {
            percentage(self.$field, self.grand_total())
        }
    };
}

macro_rules! percentage_as_text_fn {
    ($fn_name:ident, $field:ident) => {
        pub fn $fn_name(&mut self) -> String {
            percentage_as_text(self.$field, self.grand_total())
        }
    };
}

#[allow(dead_code)]
impl NodeMemoryBreakdown {
    /// Returns the greatest value between the totals computed
    /// using different mechanisms (RSS, runtime allocator metrics)
    pub fn grand_total(&self) -> u64 {
        self.total.max()
    }

    percentage_fn!(connection_readers_percentage, connection_readers);
    percentage_as_text_fn!(connection_readers_percentage_as_text, connection_readers);
    percentage_fn!(connection_writers_percentage, connection_writers);
    percentage_as_text_fn!(connection_writers_percentage_as_text, connection_writers);
    percentage_fn!(connection_channels_percentage, connection_channels);
    percentage_as_text_fn!(connection_channels_percentage_as_text, connection_channels);
    percentage_fn!(connection_other_percentage, connection_other);
    percentage_as_text_fn!(connection_other_percentage_as_text, connection_other);

    percentage_fn!(classic_queue_procs_percentage, classic_queue_procs);
    percentage_as_text_fn!(classic_queue_procs_percentage_as_text, classic_queue_procs);
    percentage_fn!(quorum_queue_procs_percentage, quorum_queue_procs);
    percentage_as_text_fn!(quorum_queue_procs_percentage_as_text, quorum_queue_procs);
    percentage_fn!(stream_queue_procs_percentage, stream_queue_procs);
    percentage_as_text_fn!(stream_queue_procs_percentage_as_text, stream_queue_procs);
    percentage_fn!(
        stream_queue_replica_reader_procs_percentage,
        stream_queue_replica_reader_procs
    );
    percentage_as_text_fn!(
        stream_queue_replica_reader_procs_percentage_as_text,
        stream_queue_replica_reader_procs
    );
    percentage_fn!(
        stream_queue_coordinator_procs_percentage,
        stream_queue_coordinator_procs
    );
    percentage_as_text_fn!(
        stream_queue_coordinator_procs_percentage_as_text,
        stream_queue_coordinator_procs
    );

    percentage_fn!(plugins_percentage, plugins);
    percentage_as_text_fn!(plugins_percentage_as_text, plugins);

    percentage_fn!(metadata_store_percentage, metadata_store);
    percentage_as_text_fn!(metadata_store_percentage_as_text, metadata_store);

    percentage_fn!(other_procs_percentage, other_procs);
    percentage_as_text_fn!(other_procs_percentage_as_text, other_procs);

    percentage_fn!(metrics_percentage, metrics);
    percentage_as_text_fn!(metrics_percentage_as_text, metrics);

    percentage_fn!(management_db_percentage, management_db);
    percentage_as_text_fn!(management_db_percentage_as_text, management_db);

    percentage_fn!(mnesia_percentage, mnesia);
    percentage_as_text_fn!(mnesia_percentage_as_text, mnesia);

    percentage_fn!(quorum_queue_ets_tables_percentage, quorum_queue_ets_tables);
    percentage_as_text_fn!(
        quorum_queue_ets_tables_percentage_as_text,
        quorum_queue_ets_tables
    );

    percentage_fn!(
        metadata_store_ets_tables_percentage,
        metadata_store_ets_tables
    );
    percentage_as_text_fn!(
        metadata_store_ets_tables_percentage_as_text,
        metadata_store_ets_tables
    );

    percentage_fn!(other_ets_tables_percentage, other_ets_tables);
    percentage_as_text_fn!(other_ets_tables_percentage_as_text, other_ets_tables);

    percentage_fn!(binary_heap_percentage, binary_heap);
    percentage_as_text_fn!(binary_heap_percentage_as_text, binary_heap);

    percentage_fn!(message_indices_percentage, message_indices);
    percentage_as_text_fn!(message_indices_percentage_as_text, message_indices);

    percentage_fn!(code_percentage, code);
    percentage_as_text_fn!(code_percentage_as_text, code);

    percentage_fn!(atom_table_percentage, atom_table);
    percentage_as_text_fn!(atom_table_percentage_as_text, atom_table);

    percentage_fn!(other_system_percentage, other_system);
    percentage_as_text_fn!(other_system_percentage_as_text, other_system);

    percentage_fn!(allocated_but_unused_percentage, allocated_but_unused);
    percentage_as_text_fn!(
        allocated_but_unused_percentage_as_text,
        allocated_but_unused
    );

    percentage_fn!(
        reserved_but_unallocated_percentage,
        reserved_but_unallocated
    );
    percentage_as_text_fn!(
        reserved_but_unallocated_percentage_as_text,
        reserved_but_unallocated
    );
}

impl fmt::Display for NodeMemoryBreakdown {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = [
            ("Connection readers".to_owned(), self.connection_readers),
            ("Connection writers".to_owned(), self.connection_writers),
            ("AMQP 0-9-1 channels".to_owned(), self.connection_channels),
            (
                "Other connection processes".to_owned(),
                self.connection_other,
            ),
            (
                "Classic queue replica processes".to_owned(),
                self.classic_queue_procs,
            ),
            (
                "Quorum queue replica processes".to_owned(),
                self.quorum_queue_procs,
            ),
            (
                "Stream replica processes".to_owned(),
                self.stream_queue_procs,
            ),
            (
                "Stream replica reader processes".to_owned(),
                self.stream_queue_replica_reader_procs,
            ),
            (
                "Stream coordinator processes".to_owned(),
                self.stream_queue_coordinator_procs,
            ),
            ("Plugins".to_owned(), self.plugins),
            ("Metadata store".to_owned(), self.metadata_store),
            ("Other processes:".to_owned(), self.other_procs),
            ("Metrics".to_owned(), self.metrics),
            ("Management stats database".to_owned(), self.management_db),
            ("Mnesia".to_owned(), self.mnesia),
            (
                "Quorum queue ETS tables".to_owned(),
                self.quorum_queue_ets_tables,
            ),
            (
                "Metadata store ETS tables".to_owned(),
                self.metadata_store_ets_tables,
            ),
            ("Other ETS tables".to_owned(), self.other_ets_tables),
            ("Binary heap".to_owned(), self.binary_heap),
            ("Message indices".to_owned(), self.message_indices),
            ("Code modules".to_owned(), self.code),
            ("Atom table".to_owned(), self.atom_table),
            ("Other system footprint".to_owned(), self.other_system),
            ("Allocated but unused".to_owned(), self.allocated_but_unused),
            (
                "Reserved but unallocated".to_owned(),
                self.reserved_but_unallocated,
            ),
        ];

        for (k, v) in data {
            writeln!(f, "{k}: {v}")?;
        }

        Ok(())
    }
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

/// Represents a report on the authentication attempts made to a specific node.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct AuthenticationAttemptStatistics {
    pub protocol: SupportedProtocol,
    #[serde(rename = "auth_attempts")]
    pub all_attempt_count: u64,
    #[serde(rename = "auth_attempts_failed")]
    pub failure_count: u64,
    #[serde(rename = "auth_attempts_succeeded")]
    pub success_count: u64,
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

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct ClusterNode {
    pub name: String,
    pub uptime: u32,
    pub run_queue: u32,
    pub processors: u32,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub os_pid: u32,
    pub fd_total: u32,
    #[serde(rename(deserialize = "proc_total"))]
    pub total_erlang_processes: u32,
    #[serde(rename(deserialize = "mem_limit"))]
    pub memory_high_watermark: u64,
    #[serde(rename(deserialize = "mem_alarm"))]
    pub has_memory_alarm_in_effect: bool,
    #[serde(rename(deserialize = "disk_free_limit"))]
    pub free_disk_space_low_watermark: u64,
    #[serde(rename(deserialize = "disk_free_alarm"))]
    pub has_free_disk_space_alarm_in_effect: bool,
    pub rates_mode: String,
    pub enabled_plugins: PluginList,
    pub being_drained: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ClusterIdentity {
    pub name: String,
}

impl fmt::Display for ClusterIdentity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl From<String> for ClusterIdentity {
    fn from(name: String) -> Self {
        Self { name }
    }
}

impl<'a> From<&'a str> for ClusterIdentity {
    fn from(name: &'a str) -> Self {
        Self {
            name: name.to_string(),
        }
    }
}

impl AsRef<str> for ClusterIdentity {
    fn as_ref(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ClusterTags(pub Map<String, serde_json::Value>);

impl From<GlobalRuntimeParameterValue> for ClusterTags {
    fn from(value: GlobalRuntimeParameterValue) -> Self {
        ClusterTags(value.0.as_object().unwrap().clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Permissions {
    pub user: Username,
    pub vhost: VirtualHostName,
    pub configure: String,
    pub read: String,
    pub write: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct TopicPermission {
    pub user: Username,
    pub vhost: VirtualHostName,
    pub exchange: String,
    pub read: String,
    pub write: String,
}

impl Permissions {
    pub fn with_username(&self, username: &str) -> Self {
        Permissions {
            user: username.to_owned(),
            vhost: self.vhost.clone(),
            configure: self.configure.clone(),
            read: self.read.clone(),
            write: self.write.clone(),
        }
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

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct ChurnRates {
    pub connection_created: u32,
    pub connection_closed: u32,
    pub queue_declared: u32,
    pub queue_created: u32,
    pub queue_deleted: u32,
    pub channel_created: u32,
    pub channel_closed: u32,
}

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

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Listener {
    node: String,
    protocol: String,
    port: u32,
    #[serde(rename(deserialize = "ip_address"))]
    interface: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct TagMap(pub Map<String, serde_json::Value>);

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Overview {
    pub cluster_name: String,
    pub node: String,

    pub erlang_full_version: String,
    pub erlang_version: String,
    pub rabbitmq_version: String,
    pub product_name: String,
    pub product_version: String,

    // these two won't be available in 3.13.x
    #[cfg_attr(feature = "tabled", tabled(display = "display_tag_map_option"))]
    pub cluster_tags: Option<TagMap>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_tag_map_option"))]
    pub node_tags: Option<TagMap>,

    pub statistics_db_event_queue: u64,
    pub churn_rates: ChurnRates,

    pub queue_totals: QueueTotals,
    pub object_totals: ObjectTotals,
    pub message_stats: MessageStats,
}

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
