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
    requests::GlobalRuntimeParameterDefinition,
    responses::{self, cluster::ClusterTags},
};
use serde_json::{Map, Value, json};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Gets cluster name (identifier).
    pub async fn get_cluster_name(&self) -> Result<responses::ClusterIdentity> {
        let response = self.http_get("cluster-name", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Sets cluster name (identifier).
    pub async fn set_cluster_name(&self, new_name: &str) -> Result<()> {
        let body = json!({"name": new_name});
        let _response = self.http_put("cluster-name", &body, None, None).await?;
        Ok(())
    }

    /// Gets cluster tags.
    pub async fn get_cluster_tags(&self) -> Result<responses::ClusterTags> {
        let response = self.get_global_runtime_parameter("cluster_tags").await?;
        Ok(ClusterTags::try_from(response.value)?)
    }

    /// Sets cluster tags.
    pub async fn set_cluster_tags(&self, tags: Map<String, Value>) -> Result<()> {
        let grp = GlobalRuntimeParameterDefinition {
            name: "cluster_tags",
            value: tags,
        };
        self.upsert_global_runtime_parameter(&grp).await?;
        Ok(())
    }

    /// Clears all cluster tags.
    pub async fn clear_cluster_tags(&self) -> Result<()> {
        self.clear_global_runtime_parameter("cluster_tags").await?;
        Ok(())
    }
}
