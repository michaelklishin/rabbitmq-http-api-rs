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
    responses::{FeatureFlag, FeatureFlagList, FeatureFlagStability, FeatureFlagState},
};

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: std::fmt::Display,
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    /// Lists all feature flags and their current states.
    /// See [Feature Flags Guide](https://www.rabbitmq.com/docs/feature-flags) to learn more.
    pub async fn list_feature_flags(&self) -> Result<FeatureFlagList> {
        let response = self.http_get("feature-flags", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Enables a specific feature flag by name.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    /// See [Feature Flags Guide](https://www.rabbitmq.com/docs/feature-flags) to learn more.
    pub async fn enable_feature_flag(&self, name: &str) -> Result<()> {
        let body = serde_json::json!({
            "name": name
        });
        let _response = self
            .http_put(path!("feature-flags", name, "enable"), &body, None, None)
            .await?;
        Ok(())
    }

    /// Enables all stable feature flags in the cluster.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    /// See [Feature Flags Guide](https://www.rabbitmq.com/docs/feature-flags) to learn more.
    pub async fn enable_all_stable_feature_flags(&self) -> Result<()> {
        // PUT /api/feature-flags/{name}/enable does not support the special 'all' value like 'rabbitmqctl enable_feature_flag' does.
        // Thus we do what management UI does: discover the stable disabled flags and enable
        // them one by one.
        let discovered_flags = self.list_feature_flags().await?;
        let flags_to_enable: Vec<&FeatureFlag> = discovered_flags
            .0
            .iter()
            .filter(|&ff| {
                ff.state == FeatureFlagState::Disabled
                    && ff.stability == FeatureFlagStability::Stable
            })
            .collect();

        for ff in flags_to_enable {
            self.enable_feature_flag(&ff.name).await?;
        }

        Ok(())
    }
}
