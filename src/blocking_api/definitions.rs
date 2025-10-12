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
    path,
    responses::{ClusterDefinitionSet, VirtualHostDefinitionSet},
};
use serde_json::Value;

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Exports cluster-wide definitions as a JSON document.
    /// This includes all virtual hosts, users, permissions, policies, queues, streams, exchanges, bindings, runtime parameters.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn export_cluster_wide_definitions(&self) -> Result<String> {
        self.export_cluster_wide_definitions_as_string()
    }

    /// Exports cluster-wide definitions as a JSON document.
    /// This includes all virtual hosts, users, permissions, policies, queues, streams, exchanges, bindings, runtime parameters.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn export_cluster_wide_definitions_as_string(&self) -> Result<String> {
        let response = self.http_get("definitions", None, None)?;
        let response = response.text()?;
        Ok(response)
    }

    /// Exports cluster-wide definitions as a data structure.
    /// This includes all virtual hosts, users, permissions, policies, queues, streams, exchanges, bindings, runtime parameters.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn export_cluster_wide_definitions_as_data(&self) -> Result<ClusterDefinitionSet> {
        let response = self.http_get("definitions", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Exports definitions of a single virtual host as a JSON document.
    /// This includes the permissions, policies, queues, streams, exchanges, bindings, runtime parameters associated
    /// with the given virtual host.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn export_vhost_definitions(&self, vhost: &str) -> Result<String> {
        self.export_vhost_definitions_as_string(vhost)
    }

    /// Exports definitions of a single virtual host as a JSON document.
    /// This includes the permissions, policies, queues, streams, exchanges, bindings, runtime parameters associated
    /// with the given virtual host.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn export_vhost_definitions_as_string(&self, vhost: &str) -> Result<String> {
        let response = self.http_get(path!("definitions", vhost), None, None)?;
        let response = response.text()?;
        Ok(response)
    }

    /// Exports definitions of a single virtual host as a data structure.
    /// This includes the permissions, policies, queues, streams, exchanges, bindings, runtime parameters associated
    /// with the given virtual host.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn export_vhost_definitions_as_data(
        &self,
        vhost: &str,
    ) -> Result<VirtualHostDefinitionSet> {
        let response = self.http_get(path!("definitions", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Imports cluster-wide definitions from a JSON document value.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn import_definitions(&self, definitions: Value) -> Result<()> {
        self.import_cluster_wide_definitions(definitions)
    }

    /// Imports cluster-wide definitions from a JSON document value.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn import_cluster_wide_definitions(&self, definitions: Value) -> Result<()> {
        self.http_post("definitions", &definitions, None, None)?;
        Ok(())
    }

    /// Imports definitions of a single virtual host from a JSON document value.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub fn import_vhost_definitions(&self, vhost: &str, definitions: Value) -> Result<()> {
        self.http_post(path!("definitions", vhost), &definitions, None, None)?;
        Ok(())
    }
}
