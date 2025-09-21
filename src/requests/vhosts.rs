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

use crate::commons::QueueType;
use crate::responses::VirtualHost;
use serde::Serialize;

/// Properties of a [virtual host](https://rabbitmq.com/docs/vhosts/) to be created or updated.
///
/// Virtual hosts provide logical separation within a RabbitMQ instance, similar to
/// namespaces. Each virtual host has its own set of exchanges, queues, bindings, and permissions.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::{VirtualHostParams, QueueType};
///
/// // Basic virtual host
/// let basic = VirtualHostParams::named("production");
///
/// // Virtual host with description and settings
/// let configured = VirtualHostParams {
///     name: "app-staging",
///     description: Some("Staging environment for application testing"),
///     tags: Some(vec!["staging", "testing"]),
///     default_queue_type: Some(QueueType::Quorum),
///     tracing: true,
/// };
/// ```
#[derive(Serialize)]
pub struct VirtualHostParams<'a> {
    /// Virtual host name (must be unique within the RabbitMQ instance)
    pub name: &'a str,
    /// Optional description explaining the virtual host's purpose
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    /// List of tags for organizing and categorizing virtual hosts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<&'a str>>,
    /// Default queue type for new queues in this virtual host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_queue_type: Option<QueueType>,
    /// Enable message tracing for debugging and monitoring
    pub tracing: bool,
}

impl<'a> VirtualHostParams<'a> {
    pub fn named(name: &'a str) -> Self {
        VirtualHostParams {
            name,
            description: None,
            tags: None,
            default_queue_type: None,
            tracing: false,
        }
    }
}

impl<'a> From<&'a VirtualHost> for VirtualHostParams<'a> {
    fn from(vhost: &'a VirtualHost) -> Self {
        Self {
            name: &vhost.name,
            description: vhost.description.as_deref(),
            tags: vhost
                .tags
                .as_ref()
                .map(|tags| tags.iter().map(|s| s.as_str()).collect()),
            default_queue_type: vhost
                .default_queue_type
                .as_ref()
                .map(|s| QueueType::from(s.as_str())),
            // this is an inherently transient setting
            tracing: false,
        }
    }
}
