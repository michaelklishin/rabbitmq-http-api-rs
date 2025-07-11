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
use serde_json::{Map, Value, json};

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint};

#[test]
fn test_blocking_get_cluster_name() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result = rc.get_cluster_name();
    assert!(result.is_ok());
    let meta = result.unwrap();
    // in case of the below's test interference
    let name = meta.name;
    assert!(name.starts_with("rabbit") || name.starts_with("rusty"))
}

#[test]
fn test_blocking_set_cluster_name() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.get_cluster_name();
    assert!(result1.is_ok());
    let meta1 = result1.unwrap();
    assert!(meta1.name.starts_with("rabbit"));

    let result2 = rc.set_cluster_name("rusty");
    assert!(result2.is_ok());

    let result3 = rc.get_cluster_name();
    assert!(result3.is_ok());
    let meta3 = result3.unwrap();
    assert!(meta3.name == *"rusty");

    let _ = rc.set_cluster_name(&meta1.name);
}

#[test]
fn test_blocking_set_cluster_tags() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let mut tags: Map<String, Value> = Map::new();
    tags.insert("region".to_owned(), json!("ca-central-1"));

    let result1 = rc.set_cluster_tags(tags);
    assert!(result1.is_ok());

    let result2 = rc.get_cluster_tags();
    assert!(result2.is_ok());
    assert_eq!(
        &json!(result2.unwrap().0),
        &json!({"region": "ca-central-1"})
    );

    let result3 = rc.clear_cluster_tags();
    assert!(result3.is_ok());
}
