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

use crate::commons::{ChannelId, VirtualHostName};
use serde::{Deserialize, Serialize};

use super::connections::ConnectionDetails;

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChannelState {
    Starting,
    Running,
    Closing,
    #[serde(untagged)]
    Unknown(String),
}

impl From<&str> for ChannelState {
    fn from(value: &str) -> Self {
        match value {
            "starting" => ChannelState::Starting,
            "running" => ChannelState::Running,
            "closing" => ChannelState::Closing,
            other => ChannelState::Unknown(other.to_owned()),
        }
    }
}

impl From<String> for ChannelState {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl AsRef<str> for ChannelState {
    fn as_ref(&self) -> &str {
        match self {
            ChannelState::Starting => "starting",
            ChannelState::Running => "running",
            ChannelState::Closing => "closing",
            ChannelState::Unknown(s) => s.as_str(),
        }
    }
}

impl fmt::Display for ChannelState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChannelState::Starting => write!(f, "starting"),
            ChannelState::Running => write!(f, "running"),
            ChannelState::Closing => write!(f, "closing"),
            ChannelState::Unknown(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Channel {
    #[serde(rename(deserialize = "number"))]
    pub id: ChannelId,
    pub name: String,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub connection_details: ConnectionDetails,
    pub vhost: VirtualHostName,
    pub state: ChannelState,
    pub consumer_count: u32,
    #[serde(rename(deserialize = "confirm"))]
    pub has_publisher_confirms_enabled: bool,
    pub prefetch_count: u32,
    pub messages_unacknowledged: u32,
    pub messages_unconfirmed: u32,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ChannelDetails {
    #[serde(rename(deserialize = "number"))]
    pub id: ChannelId,
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
