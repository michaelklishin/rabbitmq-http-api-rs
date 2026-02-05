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

use crate::{commons::PaginationParams, path, requests::VirtualHostParams, responses};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists virtual hosts in the cluster.
    /// See [Virtual Hosts Guide](https://www.rabbitmq.com/docs/vhosts) to learn more.
    ///
    /// Requires the `monitoring` user tag for all vhosts, or `management` for accessible vhosts only. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_vhosts(&self) -> Result<Vec<responses::VirtualHost>> {
        self.get_api_request("vhosts").await
    }

    /// Lists virtual hosts with pagination.
    ///
    /// Requires the `monitoring` user tag for all vhosts, or `management` for accessible vhosts only. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_vhosts_paged(
        &self,
        params: &PaginationParams,
    ) -> Result<Vec<responses::VirtualHost>> {
        match params.to_query_string() {
            Some(query) => self.get_paginated_api_request("vhosts", &query).await,
            None => self.list_vhosts().await,
        }
    }

    /// Returns information about a virtual host.
    /// See [Virtual Hosts Guide](https://www.rabbitmq.com/docs/vhosts) to learn more.
    ///
    /// Requires the `management` user tag. Does not modify state.
    pub async fn get_vhost(&self, name: &str) -> Result<responses::VirtualHost> {
        self.get_api_request(path!("vhosts", name)).await
    }

    /// Creates a virtual host.
    ///
    /// See [`VirtualHostParams`]
    ///
    /// Requires the `administrator` user tag.
    pub async fn create_vhost(&self, params: &VirtualHostParams<'_>) -> Result<()> {
        self.update_vhost(params).await
    }

    /// Creates a virtual host or updates metadata of an existing one.
    ///
    /// See [`VirtualHostParams`]
    ///
    /// Requires the `administrator` user tag.
    pub async fn update_vhost(&self, params: &VirtualHostParams<'_>) -> Result<()> {
        self.put_api_request(path!("vhosts", params.name), params)
            .await
    }

    /// Deletes a virtual host and all its contents.
    ///
    /// This is a destructive operation that will permanently remove the virtual host
    /// along with all queues, exchanges, bindings, and messages it contains. All
    /// connections to this virtual host will be closed. If `idempotently` is true,
    /// the operation will succeed even if the virtual host doesn't exist.
    ///
    /// Requires the `administrator` user tag.
    pub async fn delete_vhost(&self, vhost: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("vhosts", vhost), idempotently)
            .await
    }

    /// Enables deletion protection for a virtual host.
    ///
    /// This prevents the virtual host from being deleted unless the protection
    /// has been explicitly lifted (disabled) using [`disable_vhost_deletion_protection`].
    ///
    /// See [Virtual Host Deletion Protection](https://www.rabbitmq.com/vhosts.html#deletion-protection) to learn more.
    ///
    /// Requires RabbitMQ 4.1.0 or a later version.
    ///
    /// Requires the `administrator` user tag.
    pub async fn enable_vhost_deletion_protection(&self, vhost: &str) -> Result<()> {
        self.http_post_without_body(path!("vhosts", vhost, "deletion", "protection"), None, None)
            .await?;
        Ok(())
    }

    /// Disables deletion protection for a virtual host.
    ///
    /// This allows the virtual host to be deleted again.
    ///
    /// See [Virtual Host Deletion Protection](https://www.rabbitmq.com/vhosts.html#deletion-protection) to learn more.
    ///
    /// Requires RabbitMQ 4.1.0 or a later version.
    ///
    /// Requires the `administrator` user tag.
    pub async fn disable_vhost_deletion_protection(&self, vhost: &str) -> Result<()> {
        self.http_delete(path!("vhosts", vhost, "deletion", "protection"), None, None)
            .await?;
        Ok(())
    }
}
