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

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Returns the current OAuth 2.0 configuration for authentication.
    /// See [OAuth 2 Guide](https://www.rabbitmq.com/docs/oauth2) to learn more.
    ///
    /// No authentication required. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn oauth_configuration(&self) -> Result<responses::OAuthConfiguration> {
        let response = self.http_get("auth", None, None)?;
        let response = response.json()?;

        Ok(response)
    }

    /// Returns authentication attempt statistics for a given node.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn auth_attempts_statistics(
        &self,
        node: &str,
    ) -> Result<Vec<responses::AuthenticationAttemptStatistics>> {
        let response = self.http_get(path!("auth", "attempts", node), None, None)?;
        let response = response.json()?;
        Ok(response)
    }
}
