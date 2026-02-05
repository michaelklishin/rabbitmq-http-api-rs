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

use crate::{commons::SupportedProtocol, error::Error, path, responses};
use reqwest::StatusCode;
use std::fmt::Display;
use std::result::Result as StdResult;

use super::client::{Client, HttpClientError, Result};

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Performs a cluster-wide health check for any active resource alarms in the cluster.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks) to learn more.
    ///
    /// No authentication required. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn health_check_cluster_wide_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/alarms").await
    }

    /// Performs a health check for alarms on the target node only.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks) to learn more.
    ///
    /// No authentication required. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn health_check_local_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/local-alarms").await
    }

    /// Will fail if target node is critical to the quorum of some quorum queues, streams or the Khepri metadata store.
    /// See [Upgrades Guide](https://www.rabbitmq.com/docs/upgrade#maintaining-quorum) to learn more.
    ///
    /// No authentication required. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn health_check_if_node_is_quorum_critical(&self) -> Result<()> {
        let path = "health/checks/node-is-quorum-critical";
        self.boolean_health_check(path).await
    }

    /// Checks if a specific port has an active listener.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks)
    /// and [Networking Guide](https://www.rabbitmq.com/docs/networking) to learn more.
    ///
    /// No authentication required. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn health_check_port_listener(&self, port: u16) -> Result<()> {
        let port_s = port.to_string();
        let path = path!("health", "checks", "port-listener", port_s);
        self.boolean_health_check(&path).await
    }

    /// Checks if a specific protocol listener is active.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks)
    /// and [Networking Guide](https://www.rabbitmq.com/docs/networking) to learn more.
    ///
    /// No authentication required. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn health_check_protocol_listener(&self, protocol: SupportedProtocol) -> Result<()> {
        let proto: String = String::from(protocol);
        let path = path!("health", "checks", "protocol-listener", proto);
        self.boolean_health_check(&path).await
    }

    pub(crate) async fn boolean_health_check(&self, path: &str) -> StdResult<(), HttpClientError> {
        // we expect that StatusCode::SERVICE_UNAVAILABLE may be return and ignore
        // it here to provide a custom error type later
        let response = self
            .http_get(path, None, Some(StatusCode::SERVICE_UNAVAILABLE))
            .await?;

        let status_code = response.status();
        if status_code.is_success() {
            return Ok(());
        }

        let failure_details = response.json().await?;
        Err(Error::HealthCheckFailed {
            path: path.to_owned(),
            status_code,
            details: failure_details,
        })
    }

    pub(crate) async fn health_check_alarms(&self, path: &str) -> Result<()> {
        // we expect that StatusCode::SERVICE_UNAVAILABLE may be return and ignore
        // it here to provide a custom error type later
        let response = self
            .http_get(path, None, Some(StatusCode::SERVICE_UNAVAILABLE))
            .await?;
        let status_code = response.status();
        if status_code.is_success() {
            return Ok(());
        }

        let body = response.json().await?;
        let failure_details = responses::HealthCheckFailureDetails::AlarmCheck(body);
        Err(Error::HealthCheckFailed {
            path: path.to_owned(),
            details: failure_details,
            status_code,
        })
    }
}
