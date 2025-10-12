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
    commons::{UserLimitTarget, VirtualHostLimitTarget},
    path,
    requests::EnforcedLimitParams,
    responses,
};
use reqwest::StatusCode;
use serde_json::json;

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    pub async fn set_user_limit(
        &self,
        username: &str,
        limit: EnforcedLimitParams<UserLimitTarget>,
    ) -> Result<()> {
        let body = json!({"value": limit.value});
        let _response = self
            .http_put(
                path!("user-limits", username, limit.kind),
                &body,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn clear_user_limit(&self, username: &str, kind: UserLimitTarget) -> Result<()> {
        let _response = self
            .http_delete(path!("user-limits", username, kind), None, None)
            .await?;
        Ok(())
    }

    pub async fn list_all_user_limits(&self) -> Result<Vec<responses::UserLimits>> {
        let response = self.http_get("user-limits", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_user_limits(&self, username: &str) -> Result<Vec<responses::UserLimits>> {
        let response = self
            .http_get(path!("user-limits", username), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Sets a [virtual host limit](https://www.rabbitmq.com/docs/vhosts#limits).
    pub async fn set_vhost_limit(
        &self,
        vhost: &str,
        limit: EnforcedLimitParams<VirtualHostLimitTarget>,
    ) -> Result<()> {
        let body = json!({"value": limit.value});
        let _response = self
            .http_put(path!("vhost-limits", vhost, limit.kind), &body, None, None)
            .await?;
        Ok(())
    }

    /// Clears (removes) a [virtual host limit](https://www.rabbitmq.com/docs/vhosts#limits).
    pub async fn clear_vhost_limit(&self, vhost: &str, kind: VirtualHostLimitTarget) -> Result<()> {
        let _response = self
            .http_delete(
                path!("vhost-limits", vhost, kind),
                Some(StatusCode::NOT_FOUND),
                None,
            )
            .await?;
        Ok(())
    }

    /// Lists all [virtual host limits](https://www.rabbitmq.com/docs/vhosts#limits) set in the cluster.
    pub async fn list_all_vhost_limits(&self) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self.http_get("vhost-limits", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists the [limits of a given virtual host](https://www.rabbitmq.com/docs/vhosts#limits).
    pub async fn list_vhost_limits(
        &self,
        vhost: &str,
    ) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self
            .http_get(path!("vhost-limits", vhost), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }
}
