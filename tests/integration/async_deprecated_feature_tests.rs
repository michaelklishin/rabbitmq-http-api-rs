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

use crate::test_helpers::{
    PASSWORD, USERNAME, async_rabbitmq_version_is_at_least, async_testing_against_version, endpoint,
};

#[tokio::test]
async fn test_async_list_all_deprecated_features() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Deprecated features API was introduced in RabbitMQ 3.13
    if !async_rabbitmq_version_is_at_least(3, 13, 0).await {
        return;
    }

    let result = rc.list_all_deprecated_features().await;

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(
        vec.0
            .into_iter()
            .any(|df| df.deprecation_phase == DeprecationPhase::PermittedByDefault)
    );
}

#[tokio::test]
async fn test_async_list_deprecated_features_in_use() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // list_deprecated_features_in_use requires RabbitMQ 4.0+
    if !async_rabbitmq_version_is_at_least(4, 0, 0).await {
        return;
    }

    // because of https://github.com/rabbitmq/rabbitmq-server#14340
    if async_testing_against_version("4.1.3").await {
        return;
    }

    let vh = "/";
    let queue_name = "test_async_list_deprecated_features_in_use";

    rc.delete_queue(vh, queue_name, true).await.unwrap();

    let params = QueueParams::new(queue_name, QueueType::Classic, false, false, None);
    rc.declare_queue(vh, &params).await.unwrap();

    let result2 = rc.list_deprecated_features_in_use().await;
    match result2 {
        Ok(vec) => {
            assert!(
                vec.0
                    .into_iter()
                    .any(|df| df.deprecation_phase == DeprecationPhase::PermittedByDefault)
            );
        }
        Err(_err) => {
            // occasionally happens in this specific test
            rc.delete_queue(vh, queue_name, true).await.unwrap();
        }
    }
}
