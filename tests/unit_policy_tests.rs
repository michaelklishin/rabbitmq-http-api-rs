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
mod test_helpers;

use rabbitmq_http_client::commons::PolicyTarget;
use rabbitmq_http_client::responses::{Policy, PolicyDefinition, PolicyPredicates};
use serde_json::{json, Map};

#[test]
fn test_unit_policy_definition_has_cmq_keys_case1() {
    let mut m = Map::new();
    m.insert("max-length".to_owned(), json!(1000));
    let p = PolicyDefinition(Some(m));
    assert_eq!(false, p.has_cmq_keys());
}

#[test]
fn test_unit_policy_definition_has_cmq_keys_case2() {
    let mut m = Map::new();
    m.insert("ha-mode".to_owned(), json!("exactly"));
    m.insert("ha-params".to_owned(), json!(2));
    m.insert("ha-sync-mode".to_owned(), json!("automatic"));
    let p = PolicyDefinition(Some(m));
    assert!(p.has_cmq_keys());
}

#[test]
fn test_unit_policy_definition_has_cmq_keys_case3() {
    let mut m = Map::new();
    m.insert("ha-mode".to_owned(), json!("all"));
    m.insert("ha-promote-on-shutdown".to_owned(), json!("always"));
    let p = PolicyDefinition(Some(m));
    assert!(p.has_cmq_keys());
}

#[test]
fn test_unit_policy_has_cmq_keys_case1() {
    let mut m = Map::new();
    m.insert("max-age".to_owned(), json!("1D"));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.1".to_owned(),
        vhost: "/".to_owned(),
        pattern: "^events".to_owned(),
        apply_to: PolicyTarget::Streams,
        priority: 11,
        definition: defs.clone(),
    };

    assert_eq!(false, defs.has_cmq_keys());
    assert_eq!(false, p.has_cmq_keys());
}

#[test]
fn test_unit_policy_has_cmq_keys_case2() {
    let mut m = Map::new();
    m.insert("ha-mode".to_owned(), json!("exactly"));
    m.insert("ha-params".to_owned(), json!(2));
    m.insert("ha-sync-mode".to_owned(), json!("automatic"));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.1".to_owned(),
        vhost: "/".to_owned(),
        pattern: "^events".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    assert!(defs.has_cmq_keys());
    assert!(p.has_cmq_keys());
}

#[test]
fn test_unit_policy_has_cmq_keys_case3() {
    let mut m = Map::new();
    m.insert("ha-mode".to_owned(), json!("all"));
    m.insert("ha-promote-on-shutdown".to_owned(), json!("always"));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.1".to_owned(),
        vhost: "/".to_owned(),
        pattern: "^events".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    assert!(defs.has_cmq_keys());
    assert!(p.has_cmq_keys());
}

#[test]
fn test_unit_policy_has_cmq_keys_case4() {
    let mut m = Map::new();
    m.insert("ha-mode".to_owned(), json!("all"));
    m.insert("ha-promote-on-shutdown".to_owned(), json!("always"));
    let defs = PolicyDefinition(None);
    let p = Policy {
        name: "policy.1".to_owned(),
        vhost: "/".to_owned(),
        pattern: "^events".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    assert_eq!(false, defs.has_cmq_keys());
    assert_eq!(false, p.has_cmq_keys());
}
