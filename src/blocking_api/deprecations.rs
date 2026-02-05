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

use crate::responses::DeprecatedFeatureList;

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists all deprecated features.
    /// See [Deprecated Features](https://www.rabbitmq.com/docs/deprecated) to learn more.
    ///
    /// Requires RabbitMQ 3.13.0 or a later version.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn list_all_deprecated_features(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists deprecated features that are in use.
    /// See [Deprecated Features](https://www.rabbitmq.com/docs/deprecated) to learn more.
    ///
    /// Requires RabbitMQ 3.13.0 or a later version.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub fn list_deprecated_features_in_use(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features/used", None, None)?;
        let response = response.json()?;
        Ok(response)
    }
}
