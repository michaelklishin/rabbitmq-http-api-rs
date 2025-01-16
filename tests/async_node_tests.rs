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
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[tokio::test]
async fn test_async_list_nodes() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_nodes().await;

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().any(|n| n.name.starts_with("rabbit@")))
}

#[tokio::test]
async fn test_async_get_node_info() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().await.unwrap();
    let name = nodes.first().unwrap().name.clone();
    let node = &rc.get_node_info(&name).await.unwrap();

    assert!(node.processors >= 1);
    assert!(node.uptime >= 1);
    assert!(node.total_erlang_processes >= 1);
}
