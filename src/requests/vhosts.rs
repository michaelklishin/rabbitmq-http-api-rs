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
    /// Creates minimal virtual host parameters with just a name.
    pub fn named(name: &'a str) -> Self {
        VirtualHostParams {
            name,
            description: None,
            tags: None,
            default_queue_type: None,
            tracing: false,
        }
    }

    /// Returns a builder for creating virtual host parameters with a fluent API.
    pub fn builder(name: &'a str) -> VirtualHostParamsBuilder<'a> {
        VirtualHostParamsBuilder::new(name)
    }

    /// Sets the description.
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets the tags.
    pub fn with_tags(mut self, tags: Vec<&'a str>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the default queue type.
    pub fn with_default_queue_type(mut self, queue_type: QueueType) -> Self {
        self.default_queue_type = Some(queue_type);
        self
    }

    /// Enables message tracing.
    pub fn with_tracing(mut self) -> Self {
        self.tracing = true;
        self
    }
}

/// Builder for [`VirtualHostParams`] with a fluent API.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::VirtualHostParams;
/// use rabbitmq_http_client::commons::QueueType;
///
/// let params = VirtualHostParams::builder("production")
///     .description("Production environment")
///     .tags(vec!["production", "critical"])
///     .default_queue_type(QueueType::Quorum)
///     .tracing(true)
///     .build();
/// ```
#[derive(Debug, Clone)]
#[must_use]
pub struct VirtualHostParamsBuilder<'a> {
    name: &'a str,
    description: Option<&'a str>,
    tags: Option<Vec<&'a str>>,
    default_queue_type: Option<QueueType>,
    tracing: bool,
}

impl<'a> VirtualHostParamsBuilder<'a> {
    /// Creates a new builder with the given virtual host name.
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            description: None,
            tags: None,
            default_queue_type: None,
            tracing: false,
        }
    }

    /// Sets the description for the virtual host.
    pub fn description(mut self, description: &'a str) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets tags for organizing and categorizing the virtual host.
    pub fn tags(mut self, tags: Vec<&'a str>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Sets the default queue type for new queues in this virtual host.
    pub fn default_queue_type(mut self, queue_type: QueueType) -> Self {
        self.default_queue_type = Some(queue_type);
        self
    }

    /// Enables message tracing for debugging and monitoring.
    pub fn tracing(mut self, enabled: bool) -> Self {
        self.tracing = enabled;
        self
    }

    /// Builds the [`VirtualHostParams`].
    pub fn build(self) -> VirtualHostParams<'a> {
        VirtualHostParams {
            name: self.name,
            description: self.description,
            tags: self.tags,
            default_queue_type: self.default_queue_type,
            tracing: self.tracing,
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
