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
use std::ops::{Deref, DerefMut};

use crate::commons::SupportedProtocol;
use crate::formatting::*;
use crate::responses::{PluginList, parameters::GlobalRuntimeParameterValue, permissions::TagMap};
use crate::utils::{percentage, percentage_as_text};
use serde::{Deserialize, Deserializer};
use serde_aux::prelude::*;
use serde_json::Map;

#[cfg(feature = "tabled")]
use tabled::Tabled;

fn deserialize_memory_breakdown<'de, D>(
    deserializer: D,
) -> Result<Option<NodeMemoryBreakdown>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;

    match value {
        Value::String(s) if s == "not_available" => Ok(None),
        _ => {
            let breakdown = NodeMemoryBreakdown::deserialize(value).map_err(D::Error::custom)?;
            Ok(Some(breakdown))
        }
    }
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
    #[serde(rename = "memory", deserialize_with = "deserialize_memory_breakdown")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub breakdown: Option<NodeMemoryBreakdown>,
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
        std::cmp::max(
            std::cmp::max(self.used_by_runtime, self.rss),
            self.allocated,
        )
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
    // can be negative due to a bug in RabbitMQ versions before 4.2.3
    #[serde(rename = "other_ets")]
    pub other_ets_tables: i64,
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

    pub fn other_ets_tables_percentage(&mut self) -> f64 {
        percentage(self.other_ets_tables.max(0) as u64, self.grand_total())
    }
    pub fn other_ets_tables_percentage_as_text(&mut self) -> String {
        percentage_as_text(self.other_ets_tables.max(0) as u64, self.grand_total())
    }

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
        writeln!(f, "Connection readers: {}", self.connection_readers)?;
        writeln!(f, "Connection writers: {}", self.connection_writers)?;
        writeln!(f, "AMQP 0-9-1 channels: {}", self.connection_channels)?;
        writeln!(f, "Other connection processes: {}", self.connection_other)?;
        writeln!(
            f,
            "Classic queue replica processes: {}",
            self.classic_queue_procs
        )?;
        writeln!(
            f,
            "Quorum queue replica processes: {}",
            self.quorum_queue_procs
        )?;
        writeln!(f, "Stream replica processes: {}", self.stream_queue_procs)?;
        writeln!(
            f,
            "Stream replica reader processes: {}",
            self.stream_queue_replica_reader_procs
        )?;
        writeln!(
            f,
            "Stream coordinator processes: {}",
            self.stream_queue_coordinator_procs
        )?;
        writeln!(f, "Plugins: {}", self.plugins)?;
        writeln!(f, "Metadata store: {}", self.metadata_store)?;
        writeln!(f, "Other processes: {}", self.other_procs)?;
        writeln!(f, "Metrics: {}", self.metrics)?;
        writeln!(f, "Management stats database: {}", self.management_db)?;
        writeln!(f, "Mnesia: {}", self.mnesia)?;
        writeln!(
            f,
            "Quorum queue ETS tables: {}",
            self.quorum_queue_ets_tables
        )?;
        writeln!(
            f,
            "Metadata store ETS tables: {}",
            self.metadata_store_ets_tables
        )?;
        writeln!(f, "Other ETS tables: {}", self.other_ets_tables)?;
        writeln!(f, "Binary heap: {}", self.binary_heap)?;
        writeln!(f, "Message indices: {}", self.message_indices)?;
        writeln!(f, "Code modules: {}", self.code)?;
        writeln!(f, "Atom table: {}", self.atom_table)?;
        writeln!(f, "Other system footprint: {}", self.other_system)?;
        writeln!(f, "Allocated but unused: {}", self.allocated_but_unused)?;
        writeln!(
            f,
            "Reserved but unallocated: {}",
            self.reserved_but_unallocated
        )?;

        Ok(())
    }
}

/// An OTP application running on a RabbitMQ node.
///
/// Each node reports a list of running OTP applications via `GET /api/nodes`.
/// The `rabbit` application's version corresponds to the node's RabbitMQ version.
#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct OtpApplication {
    pub name: String,
    pub description: String,
    pub version: String,
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
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub applications: Vec<OtpApplication>,

    // available since RabbitMQ 4.2.4
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub rabbitmq_version: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub erlang_version: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub erlang_full_version: Option<String>,
    #[serde(default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub crypto_lib_version: Option<String>,
}

impl ClusterNode {
    /// Returns the RabbitMQ version reported by this node.
    ///
    /// On RabbitMQ 4.2.4+, uses the `rabbitmq_version` field from the API response.
    /// On older versions, falls back to the `rabbit` OTP application version.
    pub fn rabbitmq_version(&self) -> &str {
        if let Some(ref v) = self.rabbitmq_version {
            return v;
        }
        self.applications
            .iter()
            .find(|app| app.name == "rabbit")
            .map(|app| app.version.as_str())
            .expect("rabbit application must be present on a responding node")
    }
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ClusterIdentity {
    pub name: String,
}

impl fmt::Display for ClusterIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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

impl TryFrom<GlobalRuntimeParameterValue> for ClusterTags {
    type Error = crate::error::ConversionError;

    fn try_from(value: GlobalRuntimeParameterValue) -> Result<Self, Self::Error> {
        value
            .0
            .as_object()
            .map(|obj| ClusterTags(obj.clone()))
            .ok_or_else(|| crate::error::ConversionError::InvalidType {
                expected: "JSON object".to_string(),
            })
    }
}

/// Represents a report on the authentication attempts made to a specific node.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct AuthenticationAttemptStatistics {
    #[cfg_attr(feature = "tabled", tabled(rename = "Protocol"))]
    pub protocol: SupportedProtocol,
    #[serde(rename = "auth_attempts")]
    #[cfg_attr(feature = "tabled", tabled(rename = "Number of attempts"))]
    pub all_attempt_count: u64,
    #[serde(rename = "auth_attempts_failed")]
    #[cfg_attr(feature = "tabled", tabled(rename = "Failed"))]
    pub failure_count: u64,
    #[serde(rename = "auth_attempts_succeeded")]
    #[cfg_attr(feature = "tabled", tabled(rename = "Successful"))]
    pub success_count: u64,
}

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

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Listener {
    pub node: String,
    pub protocol: String,
    pub port: u32,
    #[serde(rename(deserialize = "ip_address"))]
    pub interface: String,
}

impl Listener {
    pub fn human_friendly_name(&self) -> &str {
        human_friendly_protocol_name(&self.protocol)
    }

    pub fn has_tls_enabled(&self) -> bool {
        has_tls_enabled(&self.protocol)
    }
}

pub fn human_friendly_protocol_name(protocol: &str) -> &str {
    match protocol {
        "amqp" => "AMQP 1.0 and 0-9-1",
        "amqp/ssl" => "AMQP 1.0 and 0-9-1 with TLS",
        "mqtt" => "MQTT",
        "mqtt/ssl" => "MQTT with TLS",
        "stomp" => "STOMP",
        "stomp/ssl" => "STOMP with TLS",
        "stream" => "Stream Protocol",
        "stream/ssl" => "Stream Protocol with TLS",
        "http/web-mqtt" => "MQTT over WebSockets",
        "https/web-mqtt" => "MQTT over WebSockets with TLS",
        "http/web-stomp" => "STOMP over WebSockets",
        "https/web-stomp" => "STOMP over WebSockets with TLS",
        "http/web-amqp" => "AMQP 1.0 over WebSockets",
        "https/web-amqp" => "AMQP 1.0 over WebSockets with TLS",
        "http/prometheus" => "Prometheus",
        "https/prometheus" => "Prometheus with TLS",
        "http" => "HTTP API",
        "https" => "HTTP API with TLS",
        "clustering" => "Inter-node and CLI Tool Communication",
        other => other,
    }
}

pub fn has_tls_enabled(protocol: &str) -> bool {
    matches!(
        protocol,
        "amqp/ssl"
            | "mqtt/ssl"
            | "stomp/ssl"
            | "stream/ssl"
            | "https/web-mqtt"
            | "https/web-stomp"
            | "https/web-amqp"
            | "https/prometheus"
            | "https"
    )
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Overview {
    pub cluster_name: String,
    pub node: String,

    pub erlang_full_version: String,
    pub erlang_version: String,
    pub rabbitmq_version: String,
    // available since RabbitMQ 4.2.4
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub crypto_lib_version: Option<String>,
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

    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub listeners: Vec<Listener>,
}

impl Overview {
    pub fn has_jit_enabled(&self) -> bool {
        self.erlang_full_version.contains("[jit]")
    }
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

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Rate {
    pub rate: f64,
}
