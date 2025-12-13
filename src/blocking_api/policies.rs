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

use crate::commons::PolicyTarget;
use crate::{path, requests::PolicyParams, responses};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Fetches a [policy](https://www.rabbitmq.com/docs/policies).
    pub fn get_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self.http_get(path!("policies", vhost, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all [policies](https://www.rabbitmq.com/docs/policies) in the cluster (across all virtual hosts), taking the user's
    /// permissions into account.
    pub fn list_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("policies", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists policies in a virtual host.
    pub fn list_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("policies", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Declares a [policy](https://www.rabbitmq.com/docs/policies).
    /// See [`crate::requests::PolicyParams`] and See [`crate::requests::PolicyDefinition`]
    pub fn declare_policy(&self, params: &PolicyParams<'_>) -> Result<()> {
        let _response = self.http_put(
            path!("policies", params.vhost, params.name),
            params,
            None,
            None,
        )?;
        Ok(())
    }

    /// Declares multiple [policies](https://www.rabbitmq.com/docs/policies).
    ///
    /// Note that this function will still issue
    /// as many HTTP API requests as there are policies to declare.
    ///
    /// See [`crate::requests::PolicyParams`] and See [`crate::requests::PolicyDefinition`]
    pub fn declare_policies(&self, params: Vec<&PolicyParams<'_>>) -> Result<()> {
        for p in params {
            self.declare_policy(p)?;
        }
        Ok(())
    }

    /// Deletes a [policy](https://www.rabbitmq.com/docs/policies).
    /// This function is idempotent: deleting a non-existent policy is considered a success.
    pub fn delete_policy(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("policies", vhost, name),
            idempotently,
        )
    }

    /// Deletes multiple [policies](https://www.rabbitmq.com/docs/policies).
    ///
    /// Note that this function will still issue
    /// as many HTTP API requests as there are policies to delete.
    ///
    /// This function is idempotent: deleting a non-existent policy is considered a success.
    pub fn delete_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_policy(vhost, name, true)?;
        }
        Ok(())
    }

    pub fn get_operator_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self.http_get(path!("operator-policies", vhost, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_operator_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("operator-policies", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_operator_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("operator-policies", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn declare_operator_policy(&self, params: &PolicyParams<'_>) -> Result<()> {
        let _response = self.http_put(
            path!("operator-policies", params.vhost, params.name),
            params,
            None,
            None,
        )?;
        Ok(())
    }

    pub fn declare_operator_policies(&self, params: Vec<&PolicyParams<'_>>) -> Result<()> {
        for p in params {
            self.declare_operator_policy(p)?;
        }
        Ok(())
    }

    pub fn delete_operator_policy(
        &self,
        vhost: &str,
        name: &str,
        idempotently: bool,
    ) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("operator-policies", vhost, name),
            idempotently,
        )
    }

    pub fn delete_operator_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_operator_policy(vhost, name, true)?;
        }
        Ok(())
    }

    /// Lists policies in a virtual host that apply to the specified target.
    ///
    /// This is a convenience method that filters policies by their `apply_to` field.
    /// For example, to get only policies that apply to queues:
    /// ```ignore
    /// let queue_policies = client.list_policies_for_target("/", PolicyTarget::Queues)?;
    /// ```
    pub fn list_policies_for_target(
        &self,
        vhost: &str,
        target: PolicyTarget,
    ) -> Result<Vec<responses::Policy>> {
        let policies = self.list_policies_in(vhost)?;
        Ok(policies
            .into_iter()
            .filter(|p| target.does_apply_to(p.apply_to))
            .collect())
    }

    /// Lists policies in a virtual host that would match the given resource name and target.
    ///
    /// This is useful for finding which policies would apply to a specific queue or exchange.
    /// Returns policies sorted by priority (highest first).
    pub fn list_matching_policies(
        &self,
        vhost: &str,
        name: &str,
        target: PolicyTarget,
    ) -> Result<Vec<responses::Policy>> {
        let policies = self.list_policies_in(vhost)?;
        let mut matching: Vec<_> = policies
            .into_iter()
            .filter(|p| p.does_match_name(vhost, name, target))
            .collect();
        matching.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(matching)
    }

    /// Lists operator policies in a virtual host that apply to the specified target.
    pub fn list_operator_policies_for_target(
        &self,
        vhost: &str,
        target: PolicyTarget,
    ) -> Result<Vec<responses::Policy>> {
        let policies = self.list_operator_policies_in(vhost)?;
        Ok(policies
            .into_iter()
            .filter(|p| target.does_apply_to(p.apply_to))
            .collect())
    }

    /// Lists operator policies in a virtual host that would match the given resource name and target.
    ///
    /// Returns policies sorted by priority (highest first).
    pub fn list_matching_operator_policies(
        &self,
        vhost: &str,
        name: &str,
        target: PolicyTarget,
    ) -> Result<Vec<responses::Policy>> {
        let policies = self.list_operator_policies_in(vhost)?;
        let mut matching: Vec<_> = policies
            .into_iter()
            .filter(|p| p.does_match_name(vhost, name, target))
            .collect();
        matching.sort_by(|a, b| b.priority.cmp(&a.priority));
        Ok(matching)
    }
}
