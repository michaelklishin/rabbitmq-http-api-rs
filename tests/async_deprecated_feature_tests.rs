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
use rabbitmq_http_client::responses::DeprecationPhase;
use rabbitmq_http_client::{api::Client, commons::QueueType, requests::QueueParams};

mod test_helpers;
use crate::test_helpers::{async_testing_against_3_13_x, endpoint, PASSWORD, USERNAME};

#[tokio::test]
async fn test_async_list_all_deprecated_features() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_all_deprecated_features().await;

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec
        .0
        .into_iter()
        .any(|df| df.deprecation_phase == DeprecationPhase::PermittedByDefault));
}

#[tokio::test]
async fn test_async_list_deprecated_features_in_use() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    if async_testing_against_3_13_x().await {
        return;
    }

    let vh = "/";
    let q = "test_list_deprecated_features_in_use";

    rc.delete_queue(vh, q, true).await.unwrap();

    let params = QueueParams::new(q, QueueType::Classic, false, false, None);
    rc.declare_queue(vh, &params).await.unwrap();

    let result2 = rc.list_deprecated_features_in_use().await;
    assert!(result2.is_ok());
    let vec = result2.unwrap();
    assert!(vec
        .0
        .into_iter()
        .any(|df| df.deprecation_phase == DeprecationPhase::PermittedByDefault));

    rc.delete_queue(vh, q, true).await.unwrap();
}
