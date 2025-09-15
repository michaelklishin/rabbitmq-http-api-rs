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
use rabbitmq_http_client::api::Client;

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint};

//
// Authentication attempt statistics
//

#[tokio::test]
pub async fn test_async_auth_attempts_statistics() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let nodes = rc.list_nodes().await;
    assert!(nodes.is_ok());

    let nodes = nodes.unwrap();
    if !nodes.is_empty() {
        let node_name = &nodes[0].name;
        let result = rc.auth_attempts_statistics(node_name).await;

        // Not all versions of RabbitMQ support this endpoint
        match result {
            Ok(stats) => {
                for proto in stats {
                    assert!(proto.failure_count + proto.success_count <= proto.all_attempt_count);
                }
            },
            Err(_) => {
                // Assume the endpoint wasn't available in this RabbitMQ version
            }
        }
    }
}
