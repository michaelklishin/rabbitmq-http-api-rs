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
#[cfg(feature = "zeroize")]
use zeroize::{Zeroize, ZeroizeOnDrop};

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

impl<'a> UserParams<'a> {
    /// Creates a new user with the specified parameters.
    pub fn new(name: &'a str, password_hash: &'a str, tags: &'a str) -> Self {
        Self {
            name,
            password_hash,
            tags,
        }
    }

    /// Creates a user with administrator privileges.
    ///
    /// The `administrator` tag grants full access to the management interface.
    pub fn administrator(name: &'a str, password_hash: &'a str) -> Self {
        Self {
            name,
            password_hash,
            tags: "administrator",
        }
    }

    /// Creates a user with monitoring privileges.
    ///
    /// The `monitoring` tag grants read-only access to monitoring endpoints.
    pub fn monitoring(name: &'a str, password_hash: &'a str) -> Self {
        Self {
            name,
            password_hash,
            tags: "monitoring",
        }
    }

    /// Creates a user with management privileges.
    ///
    /// The `management` tag grants access to manage this user's own resources.
    pub fn management(name: &'a str, password_hash: &'a str) -> Self {
        Self {
            name,
            password_hash,
            tags: "management",
        }
    }

    /// Creates a user with policymaker privileges.
    ///
    /// Users tagged as policymakers can manage policies.
    pub fn policymaker(name: &'a str, password_hash: &'a str) -> Self {
        Self {
            name,
            password_hash,
            tags: "policymaker",
        }
    }

    /// Creates a user with no special privileges (no tags).
    pub fn without_tags(name: &'a str, password_hash: &'a str) -> Self {
        Self {
            name,
            password_hash,
            tags: "",
        }
    }
}

/// Owned version of [`UserParams`] for cases where owned strings are needed.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "zeroize", derive(Zeroize, ZeroizeOnDrop))]
pub struct OwnedUserParams {
    pub name: String,
    pub password_hash: String,
    pub tags: String,
}

impl OwnedUserParams {
    pub fn new(
        name: impl Into<String>,
        password_hash: impl Into<String>,
        tags: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            password_hash: password_hash.into(),
            tags: tags.into(),
        }
    }

    pub fn administrator(name: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            password_hash: password_hash.into(),
            tags: "administrator".to_owned(),
        }
    }

    pub fn monitoring(name: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            password_hash: password_hash.into(),
            tags: "monitoring".to_owned(),
        }
    }

    pub fn management(name: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            password_hash: password_hash.into(),
            tags: "management".to_owned(),
        }
    }

    pub fn policymaker(name: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            password_hash: password_hash.into(),
            tags: "policymaker".to_owned(),
        }
    }

    pub fn without_tags(name: impl Into<String>, password_hash: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            password_hash: password_hash.into(),
            tags: String::new(),
        }
    }

    pub fn as_ref(&self) -> UserParams<'_> {
        UserParams {
            name: &self.name,
            password_hash: &self.password_hash,
            tags: &self.tags,
        }
    }
}

impl<'a> From<UserParams<'a>> for OwnedUserParams {
    fn from(params: UserParams<'a>) -> Self {
        Self {
            name: params.name.to_owned(),
            password_hash: params.password_hash.to_owned(),
            tags: params.tags.to_owned(),
        }
    }
}

/// Represents a bulk user delete operation.
/// Used by [`crate::api::Client`] and [`crate::blocking_api::Client`]'s functions
/// that delete multiple users in a single operation.
#[derive(Serialize, Deserialize)]
pub struct BulkUserDelete<'a> {
    #[serde(borrow, rename = "users")]
    pub usernames: Vec<&'a str>,
}
