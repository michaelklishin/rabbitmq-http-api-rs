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

use crate::commons::{Username, VirtualHostName};
use serde::Deserialize;

#[cfg(feature = "tabled")]
use tabled::Tabled;

/// Represents a client connection.
#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Connection {
    /// Connection name. Use it to close this connection.
    pub name: String,
    /// To what node the client is connected
    pub node: String,
    /// Connection state.
    /// Regular client connections (a.k.a. network connections) will usually
    /// have this state, while direct AMQP 0-9-1 Erlang client connections won't.
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub state: Option<String>,
    /// What protocol the connection uses
    pub protocol: String,
    /// The name of the authenticated user
    #[serde(rename(deserialize = "user"))]
    pub username: Username,
    /// When was this connection opened (a timestamp).
    pub connected_at: u64,
    /// The hostname used to connect.
    #[serde(rename(deserialize = "host"))]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub server_hostname: Option<String>,
    /// The port used to connect.
    #[serde(rename(deserialize = "port"))]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub server_port: Option<u32>,
    /// Client hostname.
    #[serde(rename(deserialize = "peer_host"))]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub client_hostname: Option<String>,
    /// Ephemeral client port.
    #[serde(rename(deserialize = "peer_port"))]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub client_port: Option<u32>,
    /// Maximum number of channels that can be opened on this connection.
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub channel_max: Option<u16>,
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
    #[serde(default)]
    pub authentication_failure_close: bool,
    #[serde(rename(deserialize = "basic.nack"), default)]
    pub basic_nack: bool,
    #[serde(rename(deserialize = "connection.blocked"), default)]
    pub connection_blocked: bool,
    #[serde(rename(deserialize = "consumer_cancel_notify"), default)]
    pub consumer_cancel_notify: bool,
    #[serde(rename(deserialize = "exchange_exchange_bindings"), default)]
    pub exchange_to_exchange_bindings: bool,
    #[serde(default)]
    pub publisher_confirms: bool,
}

#[derive(Debug, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct UserConnection {
    pub name: Username,
    pub node: String,
    #[serde(rename(deserialize = "user"))]
    pub username: Username,
    pub vhost: VirtualHostName,
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
