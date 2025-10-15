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
use std::ops::Deref;

use crate::commons::VirtualHostName;
use crate::formatting::*;
use serde::{Deserialize, Serialize};
use serde_json::Map;

use super::TagList;

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq)]
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct VirtualHost {
    /// Virtual host name
    pub name: VirtualHostName,
    /// Optional tags
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub tags: Option<TagList>,
    /// Optional description
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub description: Option<String>,
    /// Default queue type used in this virtual host when clients
    /// do not explicitly specify one
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub default_queue_type: Option<String>,
    /// All virtual host metadata combined
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub metadata: Option<VirtualHostMetadata>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct EnforcedLimits(pub Map<String, serde_json::Value>);

impl Deref for EnforcedLimits {
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
    pub vhost: VirtualHostName,
    #[serde(rename(deserialize = "value"))]
    pub limits: EnforcedLimits,
}
