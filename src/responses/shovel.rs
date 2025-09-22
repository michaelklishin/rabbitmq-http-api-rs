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

use serde::{Deserialize, Serialize};
use std::fmt;
#[cfg(feature = "tabled")]
use tabled::Tabled;

#[cfg(feature = "tabled")]
use crate::formatting::display_option;
use crate::responses::MessagingProtocol;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ShovelType {
    Dynamic,
    Static,
}

impl fmt::Display for ShovelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShovelType::Dynamic => write!(f, "dynamic"),
            ShovelType::Static => write!(f, "static"),
        }
    }
}

impl From<String> for ShovelType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "dynamic" => ShovelType::Dynamic,
            "static" => ShovelType::Static,
            _ => ShovelType::Dynamic,
        }
    }
}

impl From<ShovelType> for String {
    fn from(value: ShovelType) -> Self {
        match value {
            ShovelType::Dynamic => "dynamic".to_owned(),
            ShovelType::Static => "static".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ShovelState {
    Starting,
    Running,
    Terminated,
    Unknown,
}

impl fmt::Display for ShovelState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShovelState::Starting => write!(f, "starting"),
            ShovelState::Running => write!(f, "running"),
            ShovelState::Terminated => write!(f, "terminated"),
            ShovelState::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<String> for ShovelState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "starting" => ShovelState::Starting,
            "running" => ShovelState::Running,
            "terminated" => ShovelState::Terminated,
            _ => ShovelState::Unknown,
        }
    }
}

impl From<ShovelState> for String {
    fn from(value: ShovelState) -> Self {
        match value {
            ShovelState::Starting => "starting".to_owned(),
            ShovelState::Running => "running".to_owned(),
            ShovelState::Terminated => "terminated".to_owned(),
            ShovelState::Unknown => "unknown".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ShovelPublishingState {
    Running,
    Blocked,
    Unknown,
}

impl fmt::Display for ShovelPublishingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShovelPublishingState::Running => write!(f, "running"),
            ShovelPublishingState::Blocked => write!(f, "blocked"),
            ShovelPublishingState::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<String> for ShovelPublishingState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "running" => ShovelPublishingState::Running,
            "blocked" => ShovelPublishingState::Blocked,
            _ => ShovelPublishingState::Unknown,
        }
    }
}

impl From<ShovelPublishingState> for String {
    fn from(value: ShovelPublishingState) -> Self {
        match value {
            ShovelPublishingState::Running => "running".to_owned(),
            ShovelPublishingState::Blocked => "blocked".to_owned(),
            ShovelPublishingState::Unknown => "unknown".to_owned(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Shovel {
    pub node: String,
    pub name: String,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub vhost: Option<String>,
    #[serde(rename = "type")]
    #[cfg_attr(feature = "tabled", tabled(rename = "type"))]
    pub typ: ShovelType,
    pub state: ShovelState,

    #[serde(rename = "src_uri")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub source_uri: Option<String>,
    #[serde(rename = "dest_uri")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub destination_uri: Option<String>,
    #[serde(rename = "src_queue")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub source: Option<String>,
    #[serde(rename = "dest_queue")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub destination: Option<String>,

    #[serde(rename = "src_address")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub source_address: Option<String>,
    #[serde(rename = "dest_address")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub destination_address: Option<String>,

    #[serde(rename = "src_protocol")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub source_protocol: Option<MessagingProtocol>,

    #[serde(rename = "dest_protocol")]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub destination_protocol: Option<MessagingProtocol>,
}
