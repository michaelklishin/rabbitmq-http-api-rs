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
    requests::{GlobalRuntimeParameterDefinition, RuntimeParameterDefinition},
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
    /// Lists all [runtime parameters](https://www.rabbitmq.com/docs/parameters) defined in the cluster.
    pub async fn list_runtime_parameters(&self) -> Result<Vec<responses::RuntimeParameter>> {
        self.get_api_request("parameters").await
    }

    /// Lists all [runtime parameters](https://www.rabbitmq.com/docs/parameters) with a given
    /// component type (like "federation-upstream" or "shovel") defined in the cluster.
    pub async fn list_runtime_parameters_of_component(
        &self,
        component: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        self.get_api_request(path!("parameters", component)).await
    }

    /// Lists all [runtime parameters](https://www.rabbitmq.com/docs/parameters) defined in
    /// a specific virtual host.
    pub async fn list_runtime_parameters_of_component_in(
        &self,
        component: &str,
        vhost: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        self.get_api_request(path!("parameters", component, vhost))
            .await
    }

    pub async fn get_runtime_parameter(
        &self,
        component: &str,
        vhost: &str,
        name: &str,
    ) -> Result<responses::RuntimeParameter> {
        let response = self
            .http_get(path!("parameters", component, vhost, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn upsert_runtime_parameter<'a>(
        &self,
        param: &'a RuntimeParameterDefinition<'a>,
    ) -> Result<()> {
        let _response = self
            .http_put(
                path!("parameters", param.component, param.vhost, param.name),
                &param,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn clear_runtime_parameter(
        &self,
        component: &str,
        vhost: &str,
        name: &str,
        idempotently: bool,
    ) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("parameters", component, vhost, name),
            idempotently,
        )
        .await
    }

    pub async fn clear_all_runtime_parameters(&self) -> Result<()> {
        let params = self.list_runtime_parameters().await?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name, false)
                .await?
        }
        Ok(())
    }

    pub async fn clear_all_runtime_parameters_of_component(&self, component: &str) -> Result<()> {
        let params = self.list_runtime_parameters_of_component(component).await?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name, false)
                .await?
        }
        Ok(())
    }

    pub async fn list_global_runtime_parameters(
        &self,
    ) -> Result<Vec<responses::GlobalRuntimeParameter>> {
        let response = self.http_get("global-parameters", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_global_runtime_parameter(
        &self,
        name: &str,
    ) -> Result<responses::GlobalRuntimeParameter> {
        let response = self
            .http_get(path!("global-parameters", name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn upsert_global_runtime_parameter<'a>(
        &self,
        param: &'a GlobalRuntimeParameterDefinition<'a>,
    ) -> Result<()> {
        let _response = self
            .http_put(path!("global-parameters", param.name), &param, None, None)
            .await?;
        Ok(())
    }

    pub async fn clear_global_runtime_parameter(&self, name: &str) -> Result<()> {
        let _response = self
            .http_delete(path!("global-parameters", name), None, None)
            .await?;
        Ok(())
    }
}
