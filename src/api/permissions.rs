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

use crate::{error::Error, path, requests, responses};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists all permissions in the cluster.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn list_permissions(&self) -> Result<Vec<responses::Permissions>> {
        self.get_api_request("permissions").await
    }

    /// Lists permissions in a virtual host.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn list_permissions_in(&self, vhost: &str) -> Result<Vec<responses::Permissions>> {
        let response = self
            .http_get(path!("vhosts", vhost, "permissions"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists permissions for a specific user.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn list_permissions_of(&self, user: &str) -> Result<Vec<responses::Permissions>> {
        let response = self
            .http_get(path!("users", user, "permissions"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Gets permissions for a user in a virtual host.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn get_permissions(&self, vhost: &str, user: &str) -> Result<responses::Permissions> {
        let response = self
            .http_get(path!("permissions", vhost, user), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Declares permissions for a user in a virtual host.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn declare_permissions(&self, params: &requests::Permissions<'_>) -> Result<()> {
        self.put_api_request(path!("permissions", params.vhost, params.user), params)
            .await
    }

    /// An easier to remember alias for [`declare_permissions`].
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn grant_permissions(&self, params: &requests::Permissions<'_>) -> Result<()> {
        self.declare_permissions(params).await
    }

    /// Revokes user permissions in a specific virtual host.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn clear_permissions(
        &self,
        vhost: &str,
        username: &str,
        idempotently: bool,
    ) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("permissions", vhost, username),
            idempotently,
        )
        .await
    }

    /// Sets [topic permissions](https://www.rabbitmq.com/docs/access-control#topic-authorisation) in a specific virtual host.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn declare_topic_permissions(
        &self,
        params: &requests::TopicPermissions<'_>,
    ) -> Result<()> {
        self.put_api_request(
            path!("topic-permissions", params.vhost, params.user),
            params,
        )
        .await
    }

    /// Lists all topic permissions in the cluster.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn list_topic_permissions(&self) -> Result<Vec<responses::TopicPermission>> {
        self.get_api_request("topic-permissions").await
    }

    /// Lists all topic permissions in a virtual host.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn list_topic_permissions_in(
        &self,
        vhost: &str,
    ) -> Result<Vec<responses::TopicPermission>> {
        self.get_api_request(path!("vhosts", vhost, "topic-permissions"))
            .await
    }

    /// Lists all topic permissions of a user.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn list_topic_permissions_of(
        &self,
        user: &str,
    ) -> Result<Vec<responses::TopicPermission>> {
        let response = self
            .http_get(path!("users", user, "topic-permissions"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Gets topic permissions for a user in a specific virtual host.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn get_topic_permissions_of(
        &self,
        vhost: &str,
        user: &str,
    ) -> Result<responses::TopicPermission> {
        // For some reason this endpoint returns a list instead of a single object
        let response: Vec<responses::TopicPermission> = self
            .get_api_request(path!("topic-permissions", vhost, user))
            .await?;
        match response.first() {
            Some(p) => Ok(p.clone()),
            None => Err(Error::NotFound),
        }
    }

    /// Clears [topic permissions](https://www.rabbitmq.com/docs/access-control#topic-authorisation) for a user in a specific virtual host.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn clear_topic_permissions(
        &self,
        vhost: &str,
        user: &str,
        idempotently: bool,
    ) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("topic-permissions", vhost, user),
            idempotently,
        )
        .await
    }

    /// Convenience method: grants full permissions (configure, write, read) to a user in a virtual host.
    pub async fn grant_full_permissions(&self, user: &str, vhost: &str) -> Result<()> {
        let params = requests::Permissions {
            user,
            vhost,
            configure: ".*",
            write: ".*",
            read: ".*",
        };
        self.declare_permissions(&params).await
    }
}
