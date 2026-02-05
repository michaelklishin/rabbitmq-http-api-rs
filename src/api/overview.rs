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

use crate::responses;

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Provides an overview of the most commonly used cluster metrics.
    /// See `crate::responses::Overview`.
    ///
    /// Requires the `management` user tag. Does not modify state.
    pub async fn overview(&self) -> Result<responses::Overview> {
        let response = self.http_get("overview", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns the version of RabbitMQ used by the API endpoint.
    ///
    /// Requires the `management` user tag. Does not modify state.
    pub async fn server_version(&self) -> Result<String> {
        let response = self.http_get("overview", None, None).await?;
        let response: responses::Overview = response.json().await?;

        Ok(response.rabbitmq_version)
    }
}
