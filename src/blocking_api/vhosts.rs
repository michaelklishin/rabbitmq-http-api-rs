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

use crate::{path, requests::VirtualHostParams, responses};

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
    pub fn list_vhosts(&self) -> Result<Vec<responses::VirtualHost>> {
        self.get_api_request("vhosts")
    }

    /// Returns information about a virtual host.
    /// See [Virtual Hosts Guide](https://www.rabbitmq.com/docs/vhosts) to learn more.
    pub fn get_vhost(&self, name: &str) -> Result<responses::VirtualHost> {
        self.get_api_request(path!("vhosts", name))
    }

    /// Creates a virtual host.
    ///
    /// See [`VirtualHostParams`]
    pub fn create_vhost(&self, params: &VirtualHostParams<'_>) -> Result<()> {
        self.update_vhost(params)
    }

    /// Creates a virtual host or updates metadata of an existing one.
    ///
    /// See [`VirtualHostParams`]
    pub fn update_vhost(&self, params: &VirtualHostParams<'_>) -> Result<()> {
        self.put_api_request(path!("vhosts", params.name), params)
    }

    /// Deletes a virtual host and all its contents.
    ///
    /// This is a destructive operation that will permanently remove the virtual host
    /// along with all queues, exchanges, bindings, and messages it contains. All
    /// connections to this virtual host will be closed. If `idempotently` is true,
    /// the operation will succeed even if the virtual host doesn't exist.
    pub fn delete_vhost(&self, vhost: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("vhosts", vhost), idempotently)
    }

    /// Enables deletion protection for a virtual host.
    ///
    /// This prevents the virtual host from being deleted unless the protection
    /// has been explicitly lifted (disabled) using [`disable_vhost_deletion_protection`].
    ///
    /// See [Virtual Host Deletion Protection](https://www.rabbitmq.com/vhosts.html#deletion-protection) to learn more.
    pub fn enable_vhost_deletion_protection(&self, vhost: &str) -> Result<()> {
        self.http_post_without_body(path!("vhosts", vhost, "deletion", "protection"), None, None)?;
        Ok(())
    }

    /// Disables deletion protection for a virtual host.
    ///
    /// This allows the virtual host to be deleted again.
    ///
    /// See [Virtual Host Deletion Protection](https://www.rabbitmq.com/vhosts.html#deletion-protection) to learn more.
    pub fn disable_vhost_deletion_protection(&self, vhost: &str) -> Result<()> {
        self.http_delete(path!("vhosts", vhost, "deletion", "protection"), None, None)?;
        Ok(())
    }
}
