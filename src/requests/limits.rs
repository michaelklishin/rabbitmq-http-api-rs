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

/// Represents a resource usage limit to be enforced on a [virtual host](https://rabbitmq.com/docs/vhosts/) or a user.
///
/// Can enforce limits on connections, queues, or other resources depending on the limit type.
/// The `kind` parameter specifies what type of resource to limit, while `value` sets the maximum allowed.
///
/// # Examples
///
/// ```rust
/// use rabbitmq_http_client::requests::EnforcedLimitParams;
///
/// // A limit of 100 connections
/// let max_connections = EnforcedLimitParams::new("max-connections", 100);
///
/// // A limit of 50 queues
/// let max_queues = EnforcedLimitParams::new("max-queues", 50);
/// ```
#[derive(Serialize)]
pub struct EnforcedLimitParams<T> {
    pub kind: T,
    pub value: i64,
}

impl<T> EnforcedLimitParams<T> {
    pub fn new(kind: T, value: i64) -> Self {
        EnforcedLimitParams { kind, value }
    }
}
