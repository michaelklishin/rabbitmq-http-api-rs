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

use crate::{
    commons::PaginationParams,
    path,
    requests::{BulkUserDelete, UserParams},
    responses,
};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists users in the internal database.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn list_users(&self) -> Result<Vec<responses::User>> {
        self.get_api_request("users").await
    }

    /// Lists users with pagination.
    pub async fn list_users_paged(
        &self,
        params: &PaginationParams,
    ) -> Result<Vec<responses::User>> {
        match params.to_query_string() {
            Some(query) => self.get_paginated_api_request("users", &query).await,
            None => self.list_users().await,
        }
    }

    /// Lists users in the internal database that do not have access to any virtual hosts.
    /// This is useful for finding users that may need permissions granted, or are not used
    /// and should be cleaned up.
    pub async fn list_users_without_permissions(&self) -> Result<Vec<responses::User>> {
        self.get_api_request("users/without-permissions").await
    }

    /// Returns information about a user in the internal database.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn get_user(&self, name: &str) -> Result<responses::User> {
        let response = self.http_get(path!("users", name), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about the authenticated user.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn current_user(&self) -> Result<responses::CurrentUser> {
        let response = self.http_get("whoami", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Adds a user to the internal database.
    ///
    /// See [`UserParams`] and [`crate::password_hashing`].
    pub async fn create_user(&self, params: &UserParams<'_>) -> Result<()> {
        self.put_api_request(path!("users", params.name), params)
            .await
    }

    /// Deletes a user from the internal RabbitMQ user database.
    ///
    /// This removes the user account entirely, including all associated permissions
    /// across all virtual hosts. Active connections belonging to this user will be
    /// closed. If `idempotently` is true, the operation will succeed even if the
    /// user doesn't exist.
    pub async fn delete_user(&self, username: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("users", username), idempotently)
            .await
    }

    /// Deletes multiple users from the internal database in a single operation.
    ///
    /// This is more efficient than calling [`Client::delete_user`] multiple times when you
    /// need to remove several user accounts. All specified users will be deleted
    /// along with their permissions, and any active connections will be closed.
    /// Non-existent users in the list are silently ignored.
    pub async fn delete_users(&self, usernames: Vec<&str>) -> Result<()> {
        let delete = BulkUserDelete { usernames };
        let _response = self
            .http_post(path!("users", "bulk-delete"), &delete, None, None)
            .await?;
        Ok(())
    }
}
