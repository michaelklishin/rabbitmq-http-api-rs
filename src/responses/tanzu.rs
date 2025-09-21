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

use crate::commons::Username;
#[cfg(feature = "tabled")]
use crate::formatting::display_option;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum OperatingMode {
    Upstream,
    Downstream,
}

impl From<OperatingMode> for String {
    fn from(value: OperatingMode) -> Self {
        match value {
            OperatingMode::Upstream => "upstream".to_string(),
            OperatingMode::Downstream => "downstream".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SchemaDefinitionSyncState {
    Recover,
    Connected,
    PublisherInitialized,
    Syncing,
    Disconnected,
}

impl From<String> for SchemaDefinitionSyncState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "recover" => SchemaDefinitionSyncState::Recover,
            "connected" => SchemaDefinitionSyncState::Connected,
            "publisher_initialized" => SchemaDefinitionSyncState::PublisherInitialized,
            "syncing" => SchemaDefinitionSyncState::Syncing,
            "disconnected" => SchemaDefinitionSyncState::Disconnected,
            _ => SchemaDefinitionSyncState::Recover,
        }
    }
}

impl From<SchemaDefinitionSyncState> for String {
    fn from(value: SchemaDefinitionSyncState) -> Self {
        match value {
            SchemaDefinitionSyncState::Recover => "recover".to_string(),
            SchemaDefinitionSyncState::Connected => "connected".to_string(),
            SchemaDefinitionSyncState::PublisherInitialized => "publisher initialized".to_string(),
            SchemaDefinitionSyncState::Syncing => "syncing".to_string(),
            SchemaDefinitionSyncState::Disconnected => "disconnected".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct HostnamePortPairs(pub Vec<String>);

impl From<HostnamePortPairs> for String {
    fn from(value: HostnamePortPairs) -> Self {
        value.0.join(", ")
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum MessagingProtocol {
    #[serde(rename = "amqp091")]
    Amqp091,
    #[serde(rename = "amqp10")]
    Amqp10,
    #[serde(rename = "local")]
    Local,
}

impl fmt::Display for MessagingProtocol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessagingProtocol::Amqp091 => write!(f, "AMQP 0-9-1"),
            MessagingProtocol::Amqp10 => write!(f, "AMQP 1.0"),
            MessagingProtocol::Local => write!(f, "Local"),
        }
    }
}

impl From<String> for MessagingProtocol {
    fn from(value: String) -> Self {
        match value.as_str() {
            "amqp091" => Self::Amqp091,
            "amqp10" => Self::Amqp10,
            "local" => Self::Local,
            _ => Self::Amqp10,
        }
    }
}

impl From<MessagingProtocol> for String {
    fn from(value: MessagingProtocol) -> Self {
        match value {
            MessagingProtocol::Amqp091 => "amqp091".to_owned(),
            MessagingProtocol::Amqp10 => "amqp10".to_owned(),
            MessagingProtocol::Local => "local".to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct SchemaDefinitionSyncStatus {
    pub node: String,
    pub operating_mode: OperatingMode,
    pub state: SchemaDefinitionSyncState,
    pub upstream_username: Username,
    pub upstream_endpoints: HostnamePortPairs,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    #[serde(default)]
    pub last_sync_duration: Option<u32>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    #[serde(
        default,
        rename = "last_connection_completion_stamp",
        with = "time::serde::timestamp::option"
    )]
    pub last_connection_completion_timestamp: Option<OffsetDateTime>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    #[serde(
        default,
        rename = "last_sync_request_stamp",
        with = "time::serde::timestamp::option"
    )]
    pub last_sync_request_timestamp: Option<OffsetDateTime>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WarmStandbyReplicationStateOnUpstream {
    Running,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum WarmStandbyReplicationLinkStateOnDownstream {
    Recover,
    Connecting,
    Connected,
    Disconnected,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum WarmStandbyReplicationState {
    Upstream(WarmStandbyReplicationStateOnUpstream),
    Downstream(WarmStandbyReplicationLinkStateOnDownstream),
    Unknown,
}

impl fmt::Display for WarmStandbyReplicationStateOnUpstream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WarmStandbyReplicationStateOnUpstream::Running => write!(f, "Running"),
            WarmStandbyReplicationStateOnUpstream::Unknown => write!(f, "(unknown)"),
        }
    }
}

impl fmt::Display for WarmStandbyReplicationLinkStateOnDownstream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WarmStandbyReplicationLinkStateOnDownstream::Recover => write!(f, "Recover"),
            WarmStandbyReplicationLinkStateOnDownstream::Connecting => write!(f, "Connecting"),
            WarmStandbyReplicationLinkStateOnDownstream::Connected => write!(f, "Connected"),
            WarmStandbyReplicationLinkStateOnDownstream::Disconnected => write!(f, "Disconnected"),
            WarmStandbyReplicationLinkStateOnDownstream::Unknown => write!(f, "(unknown)"),
        }
    }
}

impl fmt::Display for WarmStandbyReplicationState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WarmStandbyReplicationState::Upstream(val) => write!(f, "{val}"),
            WarmStandbyReplicationState::Downstream(val) => write!(f, "{val}"),
            WarmStandbyReplicationState::Unknown => write!(f, "(unknown)"),
        }
    }
}

impl From<String> for WarmStandbyReplicationLinkStateOnDownstream {
    fn from(value: String) -> Self {
        match value.as_str() {
            "recover" => WarmStandbyReplicationLinkStateOnDownstream::Recover,
            "connecting" => WarmStandbyReplicationLinkStateOnDownstream::Connecting,
            "connected" => WarmStandbyReplicationLinkStateOnDownstream::Connected,
            "disconnected" => WarmStandbyReplicationLinkStateOnDownstream::Disconnected,
            _ => WarmStandbyReplicationLinkStateOnDownstream::Unknown,
        }
    }
}

impl From<String> for WarmStandbyReplicationState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "running" => WarmStandbyReplicationState::Upstream(
                WarmStandbyReplicationStateOnUpstream::Running,
            ),
            "recover" => WarmStandbyReplicationState::Downstream(
                WarmStandbyReplicationLinkStateOnDownstream::Recover,
            ),
            "connecting" => WarmStandbyReplicationState::Downstream(
                WarmStandbyReplicationLinkStateOnDownstream::Connecting,
            ),
            "connected" => WarmStandbyReplicationState::Downstream(
                WarmStandbyReplicationLinkStateOnDownstream::Connected,
            ),
            "disconnected" => WarmStandbyReplicationState::Downstream(
                WarmStandbyReplicationLinkStateOnDownstream::Disconnected,
            ),
            _ => WarmStandbyReplicationState::Unknown,
        }
    }
}

impl From<WarmStandbyReplicationStateOnUpstream> for String {
    fn from(value: WarmStandbyReplicationStateOnUpstream) -> Self {
        match value {
            WarmStandbyReplicationStateOnUpstream::Running => "running".to_owned(),
            WarmStandbyReplicationStateOnUpstream::Unknown => "(unknown)".to_owned(),
        }
    }
}

impl From<WarmStandbyReplicationLinkStateOnDownstream> for String {
    fn from(value: WarmStandbyReplicationLinkStateOnDownstream) -> Self {
        match value {
            WarmStandbyReplicationLinkStateOnDownstream::Recover => "recover".to_owned(),
            WarmStandbyReplicationLinkStateOnDownstream::Connecting => "connecting".to_owned(),
            WarmStandbyReplicationLinkStateOnDownstream::Connected => "connected".to_owned(),
            WarmStandbyReplicationLinkStateOnDownstream::Disconnected => "disconnected".to_owned(),
            WarmStandbyReplicationLinkStateOnDownstream::Unknown => "(unknown)".to_owned(),
        }
    }
}

impl From<WarmStandbyReplicationState> for String {
    fn from(value: WarmStandbyReplicationState) -> Self {
        match value {
            WarmStandbyReplicationState::Upstream(val) => String::from(val),
            WarmStandbyReplicationState::Downstream(val) => String::from(val),
            WarmStandbyReplicationState::Unknown => "(unknown)".to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct WarmStandbyReplicationInVirtualHost {
    #[cfg_attr(feature = "tabled", tabled(rename = "Virtual host"))]
    pub virtual_host: String,
    #[cfg_attr(feature = "tabled", tabled(rename = "Operating mode"))]
    pub operating_mode: OperatingMode,
    #[cfg_attr(feature = "tabled", tabled(rename = "Upstream connection state"))]
    pub state: WarmStandbyReplicationState,
    #[cfg_attr(
        feature = "tabled",
        tabled(display = "display_option", rename = "Upstream endpoints")
    )]
    pub upstream_endpoints: Option<HostnamePortPairs>,
    #[cfg_attr(
        feature = "tabled",
        tabled(rename = "Upstream connection username", display = "display_option")
    )]
    pub upstream_username: Option<String>,
}

impl fmt::Display for WarmStandbyReplicationInVirtualHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Virtual host: {}", self.virtual_host)?;
        writeln!(f, "State: {}", self.state)?;
        writeln!(f, "Operating mode: {}", self.operating_mode)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[serde(transparent)]
pub struct WarmStandbyReplicationStatus {
    pub virtual_hosts: Vec<WarmStandbyReplicationInVirtualHost>,
}
