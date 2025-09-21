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

/// Properties of a [user](https://rabbitmq.com/docs/access-control/#user-management) to be created or updated.
///
/// Use the functions in [`crate::password_hashing`] to generate
/// [salted password hashes](https://rabbitmq.com/docs/passwords/#computing-password-hash).
#[derive(Serialize)]
pub struct UserParams<'a> {
    /// Username (must be unique within the RabbitMQ cluster)
    pub name: &'a str,
    /// Pre-hashed and salted password, see [RabbitMQ doc guide on passwords](https://www.rabbitmq.com/docs/passwords) to learn more
    /// Use [`crate::password_hashing`] functions to generate secure hashes.
    pub password_hash: &'a str,
    /// Comma-separated list of user tags (e.g., "administrator", "monitoring", "management")
    pub tags: &'a str,
}

/// Represents a bulk user delete operation.
/// Used by [`crate::api::Client`] and [`crate::blocking_api::Client`]'s functions
/// that delete multiple users in a single operation.
#[derive(Serialize, Deserialize)]
pub struct BulkUserDelete<'a> {
    #[serde(borrow, rename = "users")]
    pub usernames: Vec<&'a str>,
}
