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
use crate::commons::PolicyTarget;
use crate::responses::{Policy, PolicyDefinition as PolDef};
use serde::Serialize;
use serde_json::{Map, Value};

pub type PolicyDefinition = Map<String, Value>;

impl From<PolDef> for PolicyDefinition {
    fn from(policy: PolDef) -> Self {
        match policy.0 {
            None => {
                let empty: Map<String, Value> = Map::new();
                empty
            }
            Some(value) => value,
        }
    }
}

/// Parameters for declaring a [policy](https://rabbitmq.com/docs/policies/).
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::{PolicyParams, PolicyDefinitionBuilder};
/// use rabbitmq_http_client::commons::PolicyTarget;
///
/// let definition = PolicyDefinitionBuilder::new()
///     .max_length(10_000)
///     .overflow_reject_publish()
///     .build();
///
/// let policy = PolicyParams::new("/", "limit-queues", "^limit\\.", definition)
///     .apply_to(PolicyTarget::Queues)
///     .priority(10);
/// ```
#[derive(Serialize, Debug)]
pub struct PolicyParams<'a> {
    pub vhost: &'a str,
    pub name: &'a str,
    pub pattern: &'a str,
    #[serde(rename(serialize = "apply-to"))]
    pub apply_to: PolicyTarget,
    pub priority: i32,
    pub definition: PolicyDefinition,
}

impl<'a> PolicyParams<'a> {
    /// Defaults to `PolicyTarget::All` and priority 0.
    #[must_use]
    pub fn new(
        vhost: &'a str,
        name: &'a str,
        pattern: &'a str,
        definition: PolicyDefinition,
    ) -> Self {
        Self {
            vhost,
            name,
            pattern,
            apply_to: PolicyTarget::All,
            priority: 0,
            definition,
        }
    }

    #[must_use]
    pub fn apply_to(mut self, target: PolicyTarget) -> Self {
        self.apply_to = target;
        self
    }

    #[must_use]
    pub fn priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl<'a> From<&'a Policy> for PolicyParams<'a> {
    fn from(policy: &'a Policy) -> Self {
        PolicyParams {
            vhost: &policy.vhost,
            name: &policy.name,
            pattern: &policy.pattern,
            apply_to: policy.apply_to,
            priority: policy.priority as i32,
            definition: policy.definition.clone().into(),
        }
    }
}

/// A builder for constructing [`PolicyDefinition`] with a fluent API.
///
/// This builder provides convenient methods for common policy keys,
/// eliminating the need for manual JSON construction.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::PolicyDefinitionBuilder;
///
/// let definition = PolicyDefinitionBuilder::new()
///     .max_length(10_000)
///     .dead_letter_exchange("dlx")
///     .delivery_limit(5)
///     .build();
/// ```
#[derive(Debug, Clone, Default)]
#[must_use]
pub struct PolicyDefinitionBuilder {
    inner: Map<String, Value>,
}

impl PolicyDefinitionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn message_ttl(mut self, millis: u64) -> Self {
        self.inner
            .insert("message-ttl".to_owned(), Value::from(millis));
        self
    }

    pub fn expires(mut self, millis: u64) -> Self {
        self.inner.insert("expires".to_owned(), Value::from(millis));
        self
    }

    pub fn max_length(mut self, max: u64) -> Self {
        self.inner.insert("max-length".to_owned(), Value::from(max));
        self
    }

    pub fn max_length_bytes(mut self, max_bytes: u64) -> Self {
        self.inner
            .insert("max-length-bytes".to_owned(), Value::from(max_bytes));
        self
    }

    pub fn overflow_drop_head(mut self) -> Self {
        self.inner
            .insert("overflow".to_owned(), Value::from("drop-head"));
        self
    }

    pub fn overflow_reject_publish(mut self) -> Self {
        self.inner
            .insert("overflow".to_owned(), Value::from("reject-publish"));
        self
    }

    pub fn overflow_reject_publish_dlx(mut self) -> Self {
        self.inner
            .insert("overflow".to_owned(), Value::from("reject-publish-dlx"));
        self
    }

    pub fn dead_letter_exchange(mut self, exchange: &str) -> Self {
        self.inner
            .insert("dead-letter-exchange".to_owned(), Value::from(exchange));
        self
    }

    pub fn dead_letter_routing_key(mut self, routing_key: &str) -> Self {
        self.inner.insert(
            "dead-letter-routing-key".to_owned(),
            Value::from(routing_key),
        );
        self
    }

    pub fn delivery_limit(mut self, limit: u64) -> Self {
        self.inner
            .insert("delivery-limit".to_owned(), Value::from(limit));
        self
    }

    pub fn quorum_group_size(mut self, size: u32) -> Self {
        self.inner
            .insert("target-group-size".to_owned(), Value::from(size));
        self
    }

    pub fn quorum_initial_group_size(mut self, size: u32) -> Self {
        self.inner
            .insert("initial-cluster-size".to_owned(), Value::from(size));
        self
    }

    pub fn max_age(mut self, age: &str) -> Self {
        self.inner.insert("max-age".to_owned(), Value::from(age));
        self
    }

    pub fn stream_max_segment_size_bytes(mut self, bytes: u64) -> Self {
        self.inner.insert(
            "stream-max-segment-size-bytes".to_owned(),
            Value::from(bytes),
        );
        self
    }

    pub fn federation_upstream(mut self, upstream: &str) -> Self {
        self.inner
            .insert("federation-upstream".to_owned(), Value::from(upstream));
        self
    }

    pub fn federation_upstream_set(mut self, upstream_set: &str) -> Self {
        self.inner.insert(
            "federation-upstream-set".to_owned(),
            Value::from(upstream_set),
        );
        self
    }

    pub fn custom(mut self, key: &str, value: Value) -> Self {
        self.inner.insert(key.to_owned(), value);
        self
    }

    pub fn build(self) -> PolicyDefinition {
        self.inner
    }
}
