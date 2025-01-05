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
use core::fmt::Display;
use std::{fmt, ops};

use crate::commons::{BindingDestinationType, PolicyTarget};
use crate::utils::{percentage, percentage_as_text};
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Serialize,
};
use serde_aux::prelude::*;
use serde_json::Map;

#[cfg(feature = "tabled")]
use std::borrow::Cow;
#[cfg(feature = "tabled")]
use tabled::Tabled;

#[allow(dead_code)]
fn fmt_list_as_json_array(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "[]")
        }
        _ => {
            write!(f, "[")?;
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                write!(f, "{}, ", elem)?;
            }
            write!(f, "{}", last_element)?;
            write!(f, "]")?;
            Ok(())
        }
    }
}

fn fmt_comma_separated_list(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "")
        }
        _ => {
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                write!(f, "{}, ", elem)?;
            }
            write!(f, "{}", last_element)?;
            Ok(())
        }
    }
}

fn fmt_vertical_list_with_bullets(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "")
        }
        _ => {
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                writeln!(f, "* {}", elem)?;
            }
            write!(f, "* {}", last_element)?;
            Ok(())
        }
    }
}

fn fmt_vertical_list_without_bullets(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "")
        }
        _ => {
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                writeln!(f, "{}", elem)?;
            }
            write!(f, "{}", last_element)?;
            Ok(())
        }
    }
}

fn fmt_map_as_colon_separated_pairs(
    f: &mut fmt::Formatter<'_>,
    xs: &Map<String, serde_json::Value>,
) -> fmt::Result {
    for (k, v) in xs.iter() {
        writeln!(f, "{}: {}", k, v)?;
    }

    Ok(())
}

#[cfg(feature = "tabled")]
fn display_option<T>(opt: &Option<T>) -> String
where
    T: fmt::Display,
{
    match opt {
        None => "".to_owned(),
        Some(val) => format!("{}", val).to_owned(),
    }
}

#[cfg(feature = "tabled")]
fn display_arg_table(xs: &XArguments) -> String {
    let mut s = String::new();
    for (k, v) in xs.0.iter() {
        let line = format!("{}: {}\n", k, v);
        s += line.as_str()
    }

    s.clone()
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TagList(pub Vec<String>);

impl fmt::Display for TagList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_comma_separated_list(f, &self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginList(pub Vec<String>);

impl fmt::Display for PluginList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_vertical_list_with_bullets(f, &self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct XArguments(pub Map<String, serde_json::Value>);

impl fmt::Display for XArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
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
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_arg_table"))]
    pub properties: XArguments,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(transparent)]
pub struct RuntimeParameterValue(pub Map<String, serde_json::Value>);

impl ops::Deref for RuntimeParameterValue {
    type Target = Map<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for RuntimeParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeList(Vec<String>);

impl fmt::Display for NodeList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_vertical_list_without_bullets(f, &self.0)
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
    percentage_fn!(connection_writers_percentage_as_text, connection_writers);
    percentage_as_text_fn!(connection_writers_percentage, connection_writers);
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
            writeln!(f, "{}: {}", k, v)?;
        }

        Ok(())
    }
}

/// Represents a number of key OAuth 2 configuration settings.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct OAuthConfiguration {
    pub oauth_enabled: bool,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub oauth_client_id: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub oauth_provider_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[allow(dead_code)]
pub struct VirtualHostMetadata {
    /// Optional tags
    pub tags: Option<TagList>,
    /// Optional description
    pub description: Option<String>,
    /// Default queue type used in this virtual host when clients
    /// do not explicitly specify one
    pub default_queue_type: Option<String>,
}

/// Represents a [RabbitMQ virtual host](https://rabbitmq.com/docs/vhosts/).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct VirtualHost {
    /// Virtual host name
    pub name: String,
    /// Optional tags
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub tags: Option<TagList>,
    /// Optional description
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub description: Option<String>,
    /// Default queue type used in this virtual host when clients
    /// do not explicitly specify one
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub default_queue_type: Option<String>,
    /// All virtual host metadata combined
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub metadata: VirtualHostMetadata,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnforcedLimits(pub Map<String, serde_json::Value>);

impl ops::Deref for EnforcedLimits {
    type Target = Map<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for EnforcedLimits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct VirtualHostLimits {
    pub vhost: String,
    #[serde(rename(deserialize = "value"))]
    pub limits: EnforcedLimits,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct UserLimits {
    #[serde(rename(deserialize = "user"))]
    pub username: String,
    #[serde(rename(deserialize = "value"))]
    pub limits: EnforcedLimits,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct User {
    pub name: String,
    pub tags: TagList,
    pub password_hash: String,
}

/// Represents a client connection.
#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Connection {
    /// Connection name. Use it to close this connection.
    pub name: String,
    /// To what node the client is connected
    pub node: String,
    /// Connection state
    #[serde(default = "undefined")]
    pub state: String,
    /// What protocol the connection uses
    pub protocol: String,
    /// The name of the authenticated user
    #[serde(rename(deserialize = "user"))]
    pub username: String,
    /// When was this connection opened (a timestamp).
    pub connected_at: u64,
    /// The hostname used to connect.
    #[serde(rename(deserialize = "host"))]
    pub server_hostname: String,
    /// The port used to connect.
    #[serde(rename(deserialize = "port"))]
    pub server_port: u32,
    /// Client hostname.
    #[serde(rename(deserialize = "peer_host"))]
    pub client_hostname: String,
    /// Ephemeral client port.
    #[serde(rename(deserialize = "peer_port"))]
    pub client_port: u32,
    /// Maximum number of channels that can be opened on this connection.
    pub channel_max: u16,
    /// How many channels are opened on this connection.
    #[serde(rename(deserialize = "channels"))]
    #[serde(default)]
    pub channel_count: u16,
    /// Client-provided properties (metadata and capabilities).
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub client_properties: ClientProperties,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ClientProperties {
    #[serde(default)]
    pub connection_name: String,
    #[serde(default)]
    pub platform: String,
    #[serde(default)]
    pub product: String,
    #[serde(default)]
    pub version: String,
    pub capabilities: Option<ClientCapabilities>,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ClientCapabilities {
    pub authentication_failure_close: bool,
    #[serde(rename(deserialize = "basic.nack"))]
    pub basic_nack: bool,
    #[serde(rename(deserialize = "connection.blocked"))]
    pub connection_blocked: bool,
    #[serde(rename(deserialize = "consumer_cancel_notify"))]
    pub consumer_cancel_notify: bool,
    #[serde(rename(deserialize = "exchange_exchange_bindings"))]
    pub exchange_to_exchange_bindings: bool,
    pub publisher_confirms: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct UserConnection {
    pub name: String,
    pub node: String,
    #[serde(rename(deserialize = "user"))]
    pub username: String,
    pub vhost: String,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Channel {
    #[serde(rename(deserialize = "number"))]
    pub id: u32,
    pub name: String,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub connection_details: ConnectionDetails,
    pub vhost: String,
    pub state: String,
    pub consumer_count: u32,
    #[serde(rename(deserialize = "confirm"))]
    pub has_publisher_confirms_enabled: bool,
    pub prefetch_count: u32,
    pub messages_unacknowledged: u32,
    pub messages_unconfirmed: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct ConnectionDetails {
    pub name: String,
    #[serde(rename(deserialize = "peer_host"))]
    pub client_hostname: String,
    #[serde(rename(deserialize = "peer_port"))]
    pub client_port: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ChannelDetails {
    #[serde(rename(deserialize = "number"))]
    pub id: u32,
    pub name: String,
    pub connection_name: String,
    pub node: String,
    #[serde(rename(deserialize = "peer_host"))]
    pub client_hostname: String,
    #[serde(rename(deserialize = "peer_port"))]
    pub client_port: u32,
    #[serde(rename(deserialize = "user"))]
    pub username: String,
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
    pub channel_details: ChannelDetails,
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
    pub vhost: String,
}

impl fmt::Display for NameAndVirtualHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "'{}' in virtual host '{}'", self.name, self.vhost)
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct QueueInfo {
    pub name: String,
    pub vhost: String,
    #[serde(rename(deserialize = "type"))]
    pub queue_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    pub exclusive: bool,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_arg_table"))]
    pub arguments: XArguments,

    #[serde(default = "undefined")]
    pub node: String,
    #[serde(default)]
    pub state: String,
    // only quorum queues and streams will have this
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub leader: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub members: Option<NodeList>,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
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

    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct QueueDefinition {
    pub name: String,
    pub vhost: String,
    pub durable: bool,
    pub auto_delete: bool,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub arguments: XArguments,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct ExchangeInfo {
    pub name: String,
    pub vhost: String,
    #[serde(rename = "type")]
    pub exchange_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_arg_table"))]
    pub arguments: XArguments,
}
type ExchangeDefinition = ExchangeInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct BindingInfo {
    pub vhost: String,
    pub source: String,
    pub destination: String,
    pub destination_type: BindingDestinationType,
    pub routing_key: String,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_arg_table"))]
    pub arguments: XArguments,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub properties_key: Option<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct RuntimeParameter {
    pub name: String,
    pub vhost: String,
    pub component: String,
    #[serde(deserialize_with = "deserialize_runtime_parameter_value")]
    pub value: RuntimeParameterValue,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ClusterIdentity {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PolicyDefinition(pub Option<Map<String, serde_json::Value>>);

impl fmt::Display for PolicyDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let maybe_val = self.0.clone();
        match maybe_val {
            Some(val) => fmt_map_as_colon_separated_pairs(f, &val),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Policy {
    pub name: String,
    pub vhost: String,
    pub pattern: String,
    #[serde(rename(deserialize = "apply-to"))]
    pub apply_to: PolicyTarget,
    pub priority: i16,
    pub definition: PolicyDefinition,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Permissions {
    pub user: String,
    pub vhost: String,
    pub configure: String,
    pub read: String,
    pub write: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct DefinitionSet {
    #[serde(rename(deserialize = "rabbitmq_version"))]
    pub server_version: String,
    pub users: Vec<User>,
    #[serde(rename(deserialize = "vhosts"))]
    pub virtual_hosts: Vec<VirtualHost>,
    pub permissions: Vec<Permissions>,

    pub parameters: Vec<RuntimeParameter>,
    pub policies: Vec<Policy>,

    pub queues: Vec<QueueDefinition>,
    pub exchanges: Vec<ExchangeDefinition>,
    pub bindings: Vec<BindingInfo>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum HealthCheckFailureDetails {
    AlarmCheck(ClusterAlarmCheckDetails),
    NodeIsQuorumCritical(QuorumCriticalityCheckDetails),
    NoActivePortListener(NoActivePortListenerDetails),
    NoActiveProtocolListener(NoActiveProtocolListenerDetails),
}

impl HealthCheckFailureDetails {
    pub fn reason(&self) -> String {
        match self {
            HealthCheckFailureDetails::AlarmCheck(details) => details.reason.clone(),
            HealthCheckFailureDetails::NodeIsQuorumCritical(details) => details.reason.clone(),
            HealthCheckFailureDetails::NoActivePortListener(details) => details.reason.clone(),
            HealthCheckFailureDetails::NoActiveProtocolListener(details) => details.reason.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct ClusterAlarmCheckDetails {
    pub reason: String,
    pub alarms: Vec<ResourceAlarm>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct ResourceAlarm {
    pub node: String,
    pub resource: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct QuorumCriticalityCheckDetails {
    pub reason: String,
    pub queues: Vec<QuorumEndangeredQueue>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct NoActivePortListenerDetails {
    pub status: String,
    pub reason: String,
    #[serde(rename(deserialize = "missing"))]
    #[serde(default)]
    pub inactive_port: u16,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct NoActiveProtocolListenerDetails {
    pub status: String,
    pub reason: String,
    #[serde(rename(deserialize = "missing"))]
    // Note: switching this to SupportedProtocol will break serde's
    //       detection of various HealthCheckFailureDetails variants since
    //       that enum is untagged
    pub inactive_protocol: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct QuorumEndangeredQueue {
    pub name: String,
    pub readable_name: String,
    #[serde(rename(deserialize = "virtual_host"))]
    pub vhost: String,
    #[serde(rename(deserialize = "type"))]
    pub queue_type: String,
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

impl Display for GetMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "payload: {}", self.payload)?;
        writeln!(f, "exchange: {}", self.exchange)?;
        writeln!(f, "routing key: {}", self.routing_key)?;
        writeln!(f, "redelivered: {}", self.redelivered)?;
        writeln!(f, "properties: {}", self.properties)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct MessageList(pub Vec<GetMessage>);

impl Display for MessageList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for msg in &self.0 {
            writeln!(f, "{}", msg)?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct MessageRouted {
    pub routed: bool,
}

impl Display for MessageRouted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.routed {
            true => write!(f, "Message published and routed successfully"),
            false => write!(f, "Message published but NOT routed"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct MessageProperties(pub Map<String, serde_json::Value>);

impl Display for MessageProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
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
impl fmt::Display for ChurnRates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "connection_created: {}", self.connection_created)?;
        writeln!(f, "connection_closed: {}", self.connection_closed)?;
        writeln!(f, "queue_declared: {}", self.queue_declared)?;
        writeln!(f, "queue_created: {}", self.queue_created)?;
        writeln!(f, "queue_deleted: {}", self.queue_deleted)?;
        writeln!(f, "channel_created: {}", self.channel_created)?;
        writeln!(f, "channel_closed: {}", self.channel_closed)?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct ObjectTotals {
    pub connections: u64,
    pub channels: u64,
    pub queues: u64,
    pub exchanges: u64,
}
impl Display for ObjectTotals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "connections: {}", self.connections)?;
        writeln!(f, "channels: {}", self.channels)?;
        writeln!(f, "queues: {}", self.queues)?;
        writeln!(f, "exchanges: {}", self.exchanges)?;

        Ok(())
    }
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

impl Display for TagMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (k, v) in &self.0 {
            writeln!(f, "{}: {}", k, v)?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
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
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub cluster_tags: Option<TagMap>,
    #[cfg_attr(feature = "tabled", tabled(display_with = "display_option"))]
    pub node_tags: Option<TagMap>,

    pub statistics_db_event_queue: u64,
    pub churn_rates: ChurnRates,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlagState {
    Enabled,
    Disabled,
    StateChanging,
    Unavailable,
}

impl From<&str> for FeatureFlagState {
    fn from(value: &str) -> Self {
        match value {
            "enabled" => FeatureFlagState::Enabled,
            "disabled" => FeatureFlagState::Disabled,
            "state_changing" => FeatureFlagState::StateChanging,
            _ => FeatureFlagState::Unavailable,
        }
    }
}

impl From<String> for FeatureFlagState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "enabled" => FeatureFlagState::Enabled,
            "disabled" => FeatureFlagState::Disabled,
            "state_changing" => FeatureFlagState::StateChanging,
            _ => FeatureFlagState::Unavailable,
        }
    }
}

impl From<FeatureFlagState> for String {
    fn from(value: FeatureFlagState) -> Self {
        match value {
            FeatureFlagState::Enabled => "enabled".to_owned(),
            FeatureFlagState::Disabled => "disbled".to_owned(),
            FeatureFlagState::StateChanging => "state_changing".to_owned(),
            FeatureFlagState::Unavailable => "unavailable".to_owned(),
        }
    }
}

impl fmt::Display for FeatureFlagState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureFlagState::Enabled => writeln!(f, "enabled")?,
            FeatureFlagState::Disabled => writeln!(f, "disabled")?,
            FeatureFlagState::StateChanging => writeln!(f, "state_changing")?,
            FeatureFlagState::Unavailable => writeln!(f, "unavailable")?,
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlagStability {
    Required,
    Stable,
    Experimental,
}

impl From<&str> for FeatureFlagStability {
    fn from(value: &str) -> Self {
        match value {
            "required" => FeatureFlagStability::Required,
            "stable" => FeatureFlagStability::Stable,
            "experimental" => FeatureFlagStability::Experimental,
            _ => FeatureFlagStability::Stable,
        }
    }
}

impl From<String> for FeatureFlagStability {
    fn from(value: String) -> Self {
        match value.as_ref() {
            "required" => FeatureFlagStability::Required,
            "stable" => FeatureFlagStability::Stable,
            "experimental" => FeatureFlagStability::Experimental,
            _ => FeatureFlagStability::Stable,
        }
    }
}

impl From<FeatureFlagStability> for String {
    fn from(value: FeatureFlagStability) -> Self {
        match value {
            FeatureFlagStability::Required => "required".to_owned(),
            FeatureFlagStability::Stable => "stable".to_owned(),
            FeatureFlagStability::Experimental => "experimental".to_owned(),
        }
    }
}

impl fmt::Display for FeatureFlagStability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureFlagStability::Required => writeln!(f, "required")?,
            FeatureFlagStability::Stable => writeln!(f, "stable")?,
            FeatureFlagStability::Experimental => writeln!(f, "experimental")?,
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct FeatureFlag {
    pub name: String,
    pub state: FeatureFlagState,
    #[serde(rename = "desc")]
    pub description: String,
    pub doc_url: String,
    pub stability: FeatureFlagStability,
    pub provided_by: String,
}

impl Display for FeatureFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: {}", self.name)?;
        writeln!(f, "description: {}", self.description)?;
        writeln!(f, "doc URL: {}", self.doc_url)?;
        writeln!(f, "stability: {}", self.stability)?;
        writeln!(f, "provided by: {}", self.provided_by)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct FeatureFlagList(pub Vec<FeatureFlag>);

impl Display for FeatureFlagList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ff in &self.0 {
            writeln!(f, "{}", ff)?;
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DeprecationPhase {
    PermittedByDefault,
    DeniedByDefault,
    Disconnected,
    Removed,
    Undefined,
}

impl From<&str> for DeprecationPhase {
    fn from(value: &str) -> Self {
        match value {
            "permitted_by_default" => DeprecationPhase::PermittedByDefault,
            "denited_by_default" => DeprecationPhase::DeniedByDefault,
            "disconnected" => DeprecationPhase::Disconnected,
            "removed" => DeprecationPhase::Removed,
            _ => DeprecationPhase::Undefined,
        }
    }
}

impl From<String> for DeprecationPhase {
    fn from(value: String) -> Self {
        match value.as_str() {
            "permitted_by_default" => DeprecationPhase::PermittedByDefault,
            "denited_by_default" => DeprecationPhase::DeniedByDefault,
            "disconnected" => DeprecationPhase::Disconnected,
            "removed" => DeprecationPhase::Removed,
            _ => DeprecationPhase::Undefined,
        }
    }
}

impl From<DeprecationPhase> for String {
    fn from(value: DeprecationPhase) -> Self {
        match value {
            DeprecationPhase::PermittedByDefault => "permitted_by_default".to_owned(),
            DeprecationPhase::DeniedByDefault => "denied_by_default".to_owned(),
            DeprecationPhase::Disconnected => "disconnected".to_owned(),
            DeprecationPhase::Removed => "removed".to_owned(),
            DeprecationPhase::Undefined => "undefined".to_owned(),
        }
    }
}

impl fmt::Display for DeprecationPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeprecationPhase::PermittedByDefault => writeln!(f, "permitted_by_default")?,
            DeprecationPhase::DeniedByDefault => writeln!(f, "denied_by_default")?,
            DeprecationPhase::Disconnected => writeln!(f, "disconnected")?,
            DeprecationPhase::Removed => writeln!(f, "removed")?,
            DeprecationPhase::Undefined => writeln!(f, "undefined")?,
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct DeprecatedFeature {
    pub name: String,
    #[serde(rename = "desc")]
    pub description: String,
    pub deprecation_phase: DeprecationPhase,
    pub doc_url: String,
    pub provided_by: String,
}

impl Display for DeprecatedFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: {}", self.name)?;
        writeln!(f, "description: {}", self.description)?;
        writeln!(f, "deprecation_phase: {}", self.deprecation_phase)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct DeprecatedFeatureList(pub Vec<DeprecatedFeature>);

impl Display for DeprecatedFeatureList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for df in &self.0 {
            writeln!(f, "{}", df)?;
        }

        Ok(())
    }
}

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

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let deserializer = serde::de::value::MapAccessDeserializer::new(map);
            let m = Deserialize::deserialize(deserializer)?;
            Ok(m)
        }

        fn visit_seq<A>(self, _seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            // Treat a sequence as the default for the type.
            Ok(self.default)
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

fn deserialize_runtime_parameter_value<'de, D>(
    deserializer: D,
) -> Result<RuntimeParameterValue, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_map_or_seq::<RuntimeParameterValue, D>(deserializer)
}
