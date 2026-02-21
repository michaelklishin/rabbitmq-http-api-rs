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

use crate::test_helpers::{PASSWORD, USERNAME, async_rabbitmq_version_is_at_least, endpoint};

#[tokio::test]
async fn test_async_list_nodes_version_fields() {
    if !async_rabbitmq_version_is_at_least(4, 2, 4).await {
        return;
    }

    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().await.unwrap();

    for node in &nodes {
        assert!(node.rabbitmq_version.is_some());
        assert!(node.erlang_version.is_some());
        assert!(node.erlang_full_version.is_some());
        assert!(node.crypto_lib_version.is_some());
    }
}

#[tokio::test]
async fn test_async_get_node_info_version_fields() {
    if !async_rabbitmq_version_is_at_least(4, 2, 4).await {
        return;
    }

    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().await.unwrap();
    let name = nodes.first().unwrap().name.clone();
    let node = rc.get_node_info(&name).await.unwrap();

    assert!(node.rabbitmq_version.is_some());
    assert!(node.erlang_version.is_some());
    assert!(node.erlang_full_version.is_some());
    assert!(node.crypto_lib_version.is_some());
}

#[tokio::test]
async fn test_async_list_nodes() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_nodes().await;

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().any(|n| n.name.starts_with("rabbit@")));

    for node in &vec {
        assert!(!node.applications.is_empty());
        let v = node.rabbitmq_version();
        assert!(
            v.chars().next().unwrap().is_ascii_digit(),
            "version should start with a digit, got: {}",
            v
        );
    }
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

#[tokio::test]
async fn test_async_get_node_memory_footprint() {
    // Node memory breakdown response format requires RabbitMQ 4.0+
    if !async_rabbitmq_version_is_at_least(4, 0, 0).await {
        return;
    }

    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().await.unwrap();
    let name = nodes.first().unwrap().name.clone();
    let result = rc.get_node_memory_footprint(&name).await;

    assert!(result.is_ok());
    let footprint = result.unwrap();

    // In some cases (e.g. early in node boot), these metrics won't yet be available.
    match footprint.breakdown {
        Some(breakdown) => {
            assert!(breakdown.total.rss > 0);
            assert!(breakdown.total.allocated > 0);
            assert!(breakdown.total.used_by_runtime > 0);
            assert!(!breakdown.calculation_strategy.is_empty());
        }
        None => {
            // OK
        }
    }
}

#[tokio::test]
async fn test_async_list_all_cluster_plugins() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_all_cluster_plugins().await;

    assert!(result.is_ok());
    let plugins = result.unwrap();
    assert!(!plugins.is_empty());
    // The management plugin should be enabled since we're using the HTTP API
    assert!(plugins.contains("rabbitmq_management"));
}

#[tokio::test]
async fn test_async_list_node_plugins() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().await.unwrap();
    let name = nodes.first().unwrap().name.clone();
    let result = rc.list_node_plugins(&name).await;

    assert!(result.is_ok());
    let plugins = result.unwrap();
    assert!(!plugins.is_empty());
    // The management plugin should be enabled since we're using the HTTP API
    assert!(plugins.contains("rabbitmq_management"));
}
