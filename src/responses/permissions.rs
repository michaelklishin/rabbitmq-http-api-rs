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
#[cfg(feature = "tabled")]
use crate::formatting::display_option;
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[cfg(feature = "tabled")]
use tabled::Tabled;

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

    /// Converts to a request type that can be used to re-declare these permissions.
    pub fn to_request(&self) -> crate::requests::Permissions<'_> {
        crate::requests::Permissions {
            user: &self.user,
            vhost: &self.vhost,
            configure: &self.configure,
            read: &self.read,
            write: &self.write,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct OAuthConfiguration {
    pub oauth_enabled: bool,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub oauth_client_id: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub oauth_provider_url: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct TagMap(pub Map<String, serde_json::Value>);
