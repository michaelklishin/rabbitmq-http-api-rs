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
use rabbitmq_http_client::responses::{Policy, PolicyDefinition, PolicyDefinitionOps};
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

#[test]
fn test_unit_policy_definition_without_keys_case1() {
    let k1 = "max-age".to_owned();
    let k2 = "max-length-bytes".to_owned();

    let mut m = Map::new();
    m.insert(k1.clone(), json!("1D"));
    m.insert(k2.clone(), json!(20_000_000));
    let defs1 = PolicyDefinition(Some(m));
    let defs2 = defs1.without_keys(vec!["max-length-bytes"]);

    assert_eq!(2, defs1.len());
    assert_eq!(1, defs2.len());

    let m2 = defs2.0.unwrap();
    assert!(m2.contains_key(&k1));
    assert_eq!(false, m2.contains_key(&k2));
}

#[test]
fn test_unit_policy_definition_without_keys_case2() {
    let k1 = "max-age".to_owned();
    let k2 = "max-length-bytes".to_owned();

    let m = Map::new();
    let defs1 = PolicyDefinition(Some(m));
    let defs2 = defs1.without_keys(vec!["max-length-bytes"]);

    assert_eq!(0, defs1.len());
    assert_eq!(0, defs2.len());

    let m2 = defs2.0.unwrap();
    assert_eq!(false, m2.contains_key(&k1));
    assert_eq!(false, m2.contains_key(&k2));
}

#[test]
fn test_unit_policy_definition_without_keys_case3() {
    let k1 = "max-age".to_owned();
    let k2 = "max-length-bytes".to_owned();

    let mut m = Map::new();
    m.insert(k1.clone(), json!("1D"));
    m.insert(k2.clone(), json!(20_000_000));
    let defs1 = PolicyDefinition(Some(m));
    let defs2 = defs1.without_keys(vec![
        k1.clone().as_str(),
        k2.clone().as_str(),
        "non-existent",
    ]);

    assert_eq!(2, defs1.len());
    assert_eq!(0, defs2.len());

    let m2 = defs2.0.unwrap();
    assert_eq!(false, m2.contains_key(&k1));
    assert_eq!(false, m2.contains_key(&k2));
}

#[test]
fn test_unit_policy_definition_without_keys_case4() {
    let k1 = "max-age".to_owned();
    let k2 = "max-length-bytes".to_owned();

    let defs1 = PolicyDefinition(None);
    let defs2 = defs1.without_keys(vec![
        k1.clone().as_str(),
        k2.clone().as_str(),
        "non-existent",
    ]);

    assert_eq!(0, defs1.len());
    assert_eq!(0, defs2.len());

    assert!(defs1.is_empty());
    assert!(defs2.is_empty());

    assert!(defs2.0.is_none());
}

#[test]
fn test_unit_policy_definition_without_cmq_keys_case1() {
    let k1 = "ha-mode".to_owned();
    let k2 = "ha-params".to_owned();
    let k3 = "ha-promote-on-shutdown".to_owned();
    let k4 = "ha-sync-mode".to_owned();
    let k5 = "ha-sync-batch-size".to_owned();
    let k6 = "max-length".to_owned();

    let mut m = Map::new();
    m.insert(k1.clone(), json!("exactly"));
    m.insert(k2.clone(), json!(2));
    m.insert(k3.clone(), json!("always"));
    m.insert(k4.clone(), json!("automatic"));
    m.insert(k5.clone(), json!(100));
    m.insert(k6.clone(), json!(9000));

    let defs1 = PolicyDefinition(Some(m));
    let defs2 = defs1.without_cmq_keys();

    assert_eq!(6, defs1.len());
    assert_eq!(1, defs2.len());

    let m2 = defs2.0.unwrap();
    assert_eq!(false, m2.contains_key(&k1));
    assert_eq!(false, m2.contains_key(&k2));
    assert_eq!(false, m2.contains_key(&k3));
    assert_eq!(false, m2.contains_key(&k4));
    assert_eq!(false, m2.contains_key(&k5));
    assert!(m2.contains_key(&k6));
}
