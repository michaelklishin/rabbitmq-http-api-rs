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

use crate::commons::Username;
use crate::responses::{TagList, vhosts::EnforcedLimits};
use serde::{Deserialize, Serialize};

#[cfg(feature = "tabled")]
use tabled::Tabled;

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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct CurrentUser {
    pub name: Username,
    pub tags: TagList,
}
