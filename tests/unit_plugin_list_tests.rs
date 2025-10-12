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

use std::collections::HashSet;

use rabbitmq_http_client::responses::PluginList;

#[test]
fn test_unit_plugin_list_deserialization_with_duplicates() {
    let json = r#"["rabbitmq_management", "rabbitmq_prometheus", "rabbitmq_management", "rabbitmq_shovel"]"#;
    let plugins: PluginList = serde_json::from_str(json).unwrap();

    assert_eq!(plugins.len(), 3);
    assert!(plugins.contains("rabbitmq_management"));
    assert!(plugins.contains("rabbitmq_prometheus"));
    assert!(plugins.contains("rabbitmq_shovel"));

    let vec: &Vec<String> = &plugins;
    let unique_count = vec.iter().collect::<HashSet<_>>().len();
    assert_eq!(vec.len(), unique_count);
}

#[test]
fn test_unit_plugin_list_deserialization_already_unique() {
    let json = r#"["rabbitmq_management", "rabbitmq_prometheus", "rabbitmq_shovel"]"#;
    let plugins: PluginList = serde_json::from_str(json).unwrap();

    assert_eq!(plugins.len(), 3);
    assert!(plugins.contains("rabbitmq_management"));
    assert!(plugins.contains("rabbitmq_prometheus"));
    assert!(plugins.contains("rabbitmq_shovel"));
}

#[test]
fn test_unit_plugin_list_is_sorted() {
    let json = r#"["rabbitmq_shovel", "rabbitmq_management", "rabbitmq_prometheus"]"#;
    let plugins: PluginList = serde_json::from_str(json).unwrap();

    assert_eq!(plugins.len(), 3);
    let vec: &Vec<String> = &plugins;
    assert_eq!(vec[0], "rabbitmq_management");
    assert_eq!(vec[1], "rabbitmq_prometheus");
    assert_eq!(vec[2], "rabbitmq_shovel");
}

#[test]
fn test_unit_plugin_list_empty() {
    let json = r#"[]"#;
    let plugins: PluginList = serde_json::from_str(json).unwrap();

    assert_eq!(plugins.len(), 0);
    assert!(plugins.is_empty());
}

#[test]
fn test_unit_plugin_list_serialization() {
    let json = r#"["rabbitmq_management","rabbitmq_prometheus","rabbitmq_shovel"]"#;
    let plugins: PluginList = serde_json::from_str(json).unwrap();

    let serialized = serde_json::to_string(&plugins).unwrap();

    assert_eq!(
        serialized,
        r#"["rabbitmq_management","rabbitmq_prometheus","rabbitmq_shovel"]"#
    );
}
