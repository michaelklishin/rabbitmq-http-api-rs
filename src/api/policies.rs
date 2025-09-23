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

use crate::{path, requests::PolicyParams, responses};

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: std::fmt::Display,
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    /// Fetches a [policy](https://www.rabbitmq.com/docs/policies).
    pub async fn get_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self
            .http_get(path!("policies", vhost, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all [policies](https://www.rabbitmq.com/docs/policies) in the cluster (across all virtual hosts), taking the user's
    /// permissions into account.
    pub async fn list_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("policies", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists policies in a virtual host.
    pub async fn list_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("policies", vhost), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Declares a [policy](https://www.rabbitmq.com/docs/policies).
    /// See [`crate::requests::PolicyParams`] and See [`crate::requests::PolicyDefinition`]
    pub async fn declare_policy(&self, params: &PolicyParams<'_>) -> Result<()> {
        let _response = self
            .http_put(
                path!("policies", params.vhost, params.name),
                params,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    /// Declares multiple [policies](https://www.rabbitmq.com/docs/policies).
    ///
    /// Note that this function will still issue
    /// as many HTTP API requests as there are policies to declare.
    ///
    /// See [`crate::requests::PolicyParams`] and See [`crate::requests::PolicyDefinition`]
    pub async fn declare_policies(&self, params: Vec<&PolicyParams<'_>>) -> Result<()> {
        for p in params {
            self.declare_policy(p).await?;
        }
        Ok(())
    }

    /// Deletes a [policy](https://www.rabbitmq.com/docs/policies).
    /// This function is idempotent: deleting a non-existent policy is considered a success.
    pub async fn delete_policy(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("policies", vhost, name),
            idempotently,
        )
        .await
    }

    /// Deletes multiple [policies](https://www.rabbitmq.com/docs/policies).
    ///
    /// Note that this function will still issue
    /// as many HTTP API requests as there are policies to delete.
    ///
    /// This function is idempotent: deleting a non-existent policy is considered a success.
    pub async fn delete_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_policy(vhost, name, true).await?;
        }
        Ok(())
    }

    pub async fn get_operator_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self
            .http_get(path!("operator-policies", vhost, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_operator_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("operator-policies", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_operator_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self
            .http_get(path!("operator-policies", vhost), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn declare_operator_policy(&self, params: &PolicyParams<'_>) -> Result<()> {
        let _response = self
            .http_put(
                path!("operator-policies", params.vhost, params.name),
                params,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn declare_operator_policies(&self, params: Vec<&PolicyParams<'_>>) -> Result<()> {
        for p in params {
            self.declare_operator_policy(p).await?;
        }
        Ok(())
    }

    pub async fn delete_operator_policy(
        &self,
        vhost: &str,
        name: &str,
        idempotently: bool,
    ) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("operator-policies", vhost, name),
            idempotently,
        )
        .await
    }

    pub async fn delete_operator_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_operator_policy(vhost, name, true).await?;
        }
        Ok(())
    }
}
