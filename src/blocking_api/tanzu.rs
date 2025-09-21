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
    responses::tanzu::{SchemaDefinitionSyncStatus, WarmStandbyReplicationStatus},
};

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: std::fmt::Display,
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    /// Returns the status of schema definition synchronization.
    /// See [Schema Definition Synchronization](https://docs.vmware.com/en/VMware-Tanzu-RabbitMQ-for-Kubernetes/index.html) to learn more.
    pub fn schema_definition_sync_status(&self, node: Option<&str>) -> Result<SchemaDefinitionSyncStatus> {
        let response = match node {
            Some(val) => {
                self.http_get(path!("tanzu", "osr", "schema", "status", val), None, None)?
            }
            None => self.http_get("tanzu/osr/schema/status", None, None)?,
        };
        let response = response.json()?;
        Ok(response)
    }

    /// Enables schema definition synchronization on a single node or cluster-wide.
    /// See [Schema Definition Synchronization](https://docs.vmware.com/en/VMware-Tanzu-RabbitMQ-for-Kubernetes/index.html) to learn more.
    pub fn enable_schema_definition_sync_on_node(&self, node: Option<&str>) -> Result<()> {
        let path = match node {
            Some(n) => path!("definitions", "sync", "enable", n),
            None => "definitions/sync/enable".to_string(),
        };
        let _response = self.http_post_without_body(path, None, None)?;
        Ok(())
    }

    /// Disables schema definition synchronization on a specific node.
    /// See [Schema Definition Synchronization](https://docs.vmware.com/en/VMware-Tanzu-RabbitMQ-for-Kubernetes/index.html) to learn more.
    pub fn disable_schema_definition_sync_on_node(&self, node: Option<&str>) -> Result<()> {
        let path = match node {
            Some(n) => path!("definitions", "sync", "disable", n),
            None => "definitions/sync/disable".to_string(),
        };
        let _response = self.http_post_without_body(path, None, None)?;
        Ok(())
    }

    /// Enables schema definition synchronization cluster-wide.
    /// See [Schema Definition Synchronization](https://docs.vmware.com/en/VMware-Tanzu-RabbitMQ-for-Kubernetes/index.html) to learn more.
    pub fn enable_schema_definition_sync(&self) -> Result<()> {
        self.enable_schema_definition_sync_on_node(None)
    }

    /// Disables schema definition synchronization cluster-wide.
    /// See [Schema Definition Synchronization](https://docs.vmware.com/en/VMware-Tanzu-RabbitMQ-for-Kubernetes/index.html) to learn more.
    pub fn disable_schema_definition_sync(&self) -> Result<()> {
        self.disable_schema_definition_sync_on_node(None)
    }

    /// Returns the status of warm standby replication.
    /// See [Warm Standby Replication](https://docs.vmware.com/en/VMware-Tanzu-RabbitMQ-for-Kubernetes/index.html) to learn more.
    pub fn warm_standby_replication_status(&self) -> Result<WarmStandbyReplicationStatus> {
        let response = self.http_get("replication/status", None, None)?;
        let response = response.json()?;
        Ok(response)
    }
}
