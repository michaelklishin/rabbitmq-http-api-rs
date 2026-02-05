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
    requests::{
        RuntimeParameterDefinition,
        shovels::{Amqp10ShovelParams, Amqp091ShovelParams, SHOVEL_COMPONENT},
    },
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
    /// Lists [shovel](https://www.rabbitmq.com/docs/shovel) across all virtual hosts in the cluster.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_shovels(&self) -> Result<Vec<responses::Shovel>> {
        let response = self.http_get("shovels", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists [dynamic shovels](https://www.rabbitmq.com/docs/shovel-dynamic) in a specific virtual host.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_shovels_in(&self, vhost: &str) -> Result<Vec<responses::Shovel>> {
        let response = self.http_get(path!("shovels", vhost), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Declares [shovel](https://www.rabbitmq.com/docs/shovel) that will use the AMQP 0-9-1 protocol
    /// for both source and destination collection.
    ///
    /// Requires the `policymaker` user tag.
    pub async fn declare_amqp091_shovel(&self, params: Amqp091ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameter(&runtime_param).await
    }

    /// Declares [shovel](https://www.rabbitmq.com/docs/shovel) that will use the AMQP 1.0 protocol
    /// for both source and destination collection.
    ///
    /// Requires the `policymaker` user tag.
    pub async fn declare_amqp10_shovel(&self, params: Amqp10ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameter(&runtime_param).await
    }

    /// Deletes a shovel in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent shovel
    /// will fail.
    ///
    /// Requires the `policymaker` user tag.
    pub async fn delete_shovel(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.clear_runtime_parameter(SHOVEL_COMPONENT, vhost, name, idempotently)
            .await
    }

    pub(crate) async fn declare_shovel_parameter(
        &self,
        runtime_param: &RuntimeParameterDefinition<'_>,
    ) -> Result<()> {
        let _response = self
            .http_put(
                path!(
                    "parameters",
                    SHOVEL_COMPONENT,
                    runtime_param.vhost,
                    runtime_param.name
                ),
                &runtime_param,
                None,
                None,
            )
            .await?;
        Ok(())
    }
}
