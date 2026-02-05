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

use crate::{path, responses};
use std::collections::HashSet;
use std::fmt::Display;

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists cluster nodes.
    /// See [RabbitMQ Clustering Guide](https://www.rabbitmq.com/docs/clustering) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn list_nodes(&self) -> Result<Vec<responses::ClusterNode>> {
        self.get_api_request("nodes")
    }

    /// Returns information about a cluster node.
    /// See [Clustering Guide](https://www.rabbitmq.com/docs/clustering) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn get_node_info(&self, name: &str) -> Result<responses::ClusterNode> {
        self.get_api_request(path!("nodes", name))
    }

    /// Returns memory usage information for a cluster node.
    /// See [Reasoning About Memory Footprint](https://www.rabbitmq.com/docs/memory-use) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn get_node_memory_footprint(&self, name: &str) -> Result<responses::NodeMemoryFootprint> {
        let response = self.http_get(path!("nodes", name, "memory"), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns a unique set of plugins enabled on all cluster nodes.
    /// See [RabbitMQ Plugins Guide](https://www.rabbitmq.com/docs/plugins) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn list_all_cluster_plugins(&self) -> Result<responses::PluginList> {
        let nodes = self.list_nodes()?;

        let mut aggregated_set: Vec<String> = nodes
            .into_iter()
            .flat_map(|node| node.enabled_plugins.into_iter())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        aggregated_set.sort();
        Ok(responses::PluginList(aggregated_set))
    }

    /// Returns the list of plugins enabled on a specific cluster node.
    /// This is a convenience method equivalent to `get_node_info(name).enabled_plugins`.
    /// See [RabbitMQ Plugins Guide](https://www.rabbitmq.com/docs/plugins) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn list_node_plugins(&self, name: &str) -> Result<responses::PluginList> {
        let node = self.get_node_info(name)?;
        Ok(node.enabled_plugins)
    }
}
