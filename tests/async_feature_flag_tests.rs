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
use rabbitmq_http_client::{
    api::Client,
    responses::{FeatureFlagStability, FeatureFlagState},
};

mod test_helpers;
use crate::test_helpers::{
    PASSWORD, USERNAME, async_expected_stable_version_feature_flag, async_testing_against_3_13_x,
    endpoint,
};

#[tokio::test]
async fn test_async_list_feature_flags() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let Some(expected_ff) = async_expected_stable_version_feature_flag().await else {
        return;
    };

    let result = rc.list_feature_flags().await;
    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(
        vec.0
            .into_iter()
            .any(|ff| ff.name == expected_ff && ff.stability == FeatureFlagStability::Stable)
    );
}

#[tokio::test]
async fn test_async_enable_a_feature_flag() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    if async_testing_against_3_13_x().await {
        return;
    }

    let ff_name = "detailed_queues_endpoint";
    let result1 = rc.enable_feature_flag(ff_name).await;
    assert!(result1.is_ok());

    let result2 = rc.list_feature_flags().await;

    assert!(result2.is_ok());
    let vec = result2.unwrap();
    assert!(
        vec.0
            .into_iter()
            .any(|ff| ff.name == ff_name && ff.state == FeatureFlagState::Enabled)
    );
}

#[tokio::test]
async fn test_async_enable_all_stable_feature_flags() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let Some(expected_ff) = async_expected_stable_version_feature_flag().await else {
        return;
    };

    let result1 = rc.enable_all_stable_feature_flags().await;
    assert!(result1.is_ok());

    let result2 = rc.list_feature_flags().await;

    assert!(result2.is_ok());
    let vec = result2.unwrap();
    assert!(
        vec.0
            .into_iter()
            .any(|ff| ff.name == expected_ff && ff.state == FeatureFlagState::Enabled)
    );
}
