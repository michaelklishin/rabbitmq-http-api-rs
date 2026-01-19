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
use rabbitmq_http_client::blocking_api::Client;

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint, rabbitmq_version_is_at_least};

#[test]
fn test_blocking_list_nodes() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_nodes();

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().any(|n| n.name.starts_with("rabbit@")))
}

#[test]
fn test_blocking_get_node_info() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().unwrap();
    let name = nodes.first().unwrap().name.clone();
    let node = &rc.get_node_info(&name).unwrap();

    assert!(node.processors >= 1);
    assert!(node.uptime >= 1);
    assert!(node.total_erlang_processes >= 1);
}

#[test]
fn test_blocking_get_node_memory_footprint() {
    // Node memory breakdown response format requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().unwrap();
    let name = nodes.first().unwrap().name.clone();
    let result = rc.get_node_memory_footprint(&name);

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

#[test]
fn test_blocking_list_all_cluster_plugins() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_all_cluster_plugins();

    assert!(result.is_ok());
    let plugins = result.unwrap();
    assert!(!plugins.is_empty());
    // The management plugin should be enabled since we're using the HTTP API
    assert!(plugins.contains("rabbitmq_management"));
}

#[test]
fn test_blocking_list_node_plugins() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().unwrap();
    let name = nodes.first().unwrap().name.clone();
    let result = rc.list_node_plugins(&name);

    assert!(result.is_ok());
    let plugins = result.unwrap();
    assert!(!plugins.is_empty());
    // The management plugin should be enabled since we're using the HTTP API
    assert!(plugins.contains("rabbitmq_management"));
}
