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

use serde::Serialize;

/// Represents a user's [permission in a particular virtual host](https://rabbitmq.com/docs/access-control/).
///
/// Permissions are defined using regular expression patterns that match resource names.
///
/// Use ".*" to grant full access, or "" to deny access for a group of operations
#[derive(Serialize, Debug)]
pub struct Permissions<'a> {
    pub user: &'a str,
    pub vhost: &'a str,
    /// Regex pattern for resources user can configure (create/delete)
    pub configure: &'a str,
    /// Regex pattern for resources user can read from
    pub read: &'a str,
    /// Regex pattern for resources user can write to
    pub write: &'a str,
}

/// Represents a user's [topic permission in a particular virtual host](https://www.rabbitmq.com/docs/access-control#topic-authorisation).
///
/// Topic permissions are defined using regular expression patterns that match exchange names.
#[derive(Serialize, Debug)]
pub struct TopicPermissions<'a> {
    pub user: &'a str,
    pub vhost: &'a str,
    /// Regex pattern for the topics the user can publish to
    pub write: &'a str,
    /// Regex pattern for the topics the user can consume from (subscribe to)
    pub read: &'a str,
    /// The topic exchange these permissions apply to
    pub exchange: &'a str,
}
