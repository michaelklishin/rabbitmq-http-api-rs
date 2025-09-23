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

use crate::requests::RuntimeParameterDefinition;
use crate::requests::federation::{FEDERATION_UPSTREAM_COMPONENT, FederationUpstreamParams};
use crate::responses::{FederationLink, FederationUpstream};

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: std::fmt::Display,
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    /// Lists [federation](https://www.rabbitmq.com/docs/federation) upstreams defined in the cluster.
    pub fn list_federation_upstreams(&self) -> Result<Vec<FederationUpstream>> {
        let response = self.list_runtime_parameters_of_component(FEDERATION_UPSTREAM_COMPONENT)?;
        let upstreams = response
            .into_iter()
            .map(FederationUpstream::try_from)
            // TODO: in theory this can be an Err
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();

        Ok(upstreams)
    }

    /// Lists [federation](https://www.rabbitmq.com/docs/federation) links (connections) running in the cluster.
    pub fn list_federation_links(&self) -> Result<Vec<FederationLink>> {
        let response = self.http_get("federation-links", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Creates or updates a [federation](https://www.rabbitmq.com/docs/federation) upstream.
    ///
    /// Federation upstreams define connection endpoints for federation links (connections that federate
    /// queues or exchanges).
    pub fn declare_federation_upstream(&self, params: FederationUpstreamParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.upsert_runtime_parameter(&runtime_param)
    }

    /// Gets a specific [federation](https://www.rabbitmq.com/docs/federation) upstream by name and virtual host.
    pub fn get_federation_upstream(&self, vhost: &str, name: &str) -> Result<FederationUpstream> {
        let param = self.get_runtime_parameter(FEDERATION_UPSTREAM_COMPONENT, vhost, name)?;
        FederationUpstream::try_from(param).map_err(|e| e.into())
    }

    /// Deletes a [federation](https://www.rabbitmq.com/docs/federation) upstream.
    /// Deleting an upstream will stop any links connected to it.
    pub fn delete_federation_upstream(
        &self,
        vhost: &str,
        name: &str,
        idempotently: bool,
    ) -> Result<()> {
        self.clear_runtime_parameter(FEDERATION_UPSTREAM_COMPONENT, vhost, name, idempotently)
    }
}
