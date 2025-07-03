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
    assert!(!p.has_cmq_keys());
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

    assert!(!defs.has_cmq_keys());
    assert!(!p.has_cmq_keys());
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

    assert!(!defs.has_cmq_keys());
    assert!(!p.has_cmq_keys());
}

#[test]
fn test_unit_policy_definition_merge_case1() {
    let mut m1 = Map::new();
    m1.insert("max-age".to_owned(), json!("1D"));
    let mut defs_a = PolicyDefinition(Some(m1));

    let mut m2 = Map::new();
    m2.insert("max-age".to_owned(), json!("3D"));
    m2.insert("max-length-bytes".to_owned(), json!("2000"));
    let defs_b = PolicyDefinition(Some(m2));

    defs_a.merge(&defs_b);

    assert_eq!(2, defs_a.len());
    assert!(defs_a.contains_key("max-age"));
    assert!(defs_a.contains_key("max-length-bytes"));
    assert!(!defs_a.contains_key("abc"));
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
    assert!(!m2.contains_key(&k2));
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
    assert!(!m2.contains_key(&k1));
    assert!(!m2.contains_key(&k2));
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
    assert!(!m2.contains_key(&k1));
    assert!(!m2.contains_key(&k2));
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
    assert!(!m2.contains_key(&k1));
    assert!(!m2.contains_key(&k2));
    assert!(!m2.contains_key(&k3));
    assert!(!m2.contains_key(&k4));
    assert!(!m2.contains_key(&k5));
    assert!(m2.contains_key(&k6));
}

#[test]
fn test_unit_policy_definition_insert_case1() {
    let k1 = "max-age".to_owned();
    let k2 = "max-length-bytes".to_owned();

    let mut m = Map::new();
    m.insert(k1.clone(), json!("1D"));

    let mut defs = PolicyDefinition(Some(m));
    assert_eq!(1, defs.len());

    defs.insert(k1.clone(), json!("2D"));
    assert_eq!(1, defs.len());

    defs.insert(k2.clone(), json!(1000000));
    assert_eq!(2, defs.len());
}

#[test]
fn test_unit_policy_insert_definition_key_case1() {
    let k1 = "max-age".to_owned();
    let k2 = "max-length-bytes".to_owned();

    let mut m = Map::new();
    m.insert(k1.clone(), json!("1D"));

    let defs = PolicyDefinition(Some(m));
    let mut pol = Policy {
        name: "test_unit_policy_insert_case1".to_owned(),
        vhost: "/".to_owned(),
        pattern: ".*".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    assert_eq!(1, pol.definition.len());

    pol.insert_definition_key(k1.clone(), json!("2D"));
    assert_eq!(1, pol.definition.len());

    pol.insert_definition_key(k2.clone(), json!(1000000));
    assert_eq!(2, pol.definition.len());
}

#[test]
fn test_unit_policy_does_match_case1() {
    let mut m = Map::new();
    m.insert("max-length".to_owned(), json!(100000));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.1".to_owned(),
        vhost: "/".to_owned(),
        pattern: "^events".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    assert!(p.does_match_name("/", "events.1", PolicyTarget::Queues))
}

#[test]
fn test_unit_policy_does_match_case2() {
    let mut m = Map::new();
    m.insert("max-length".to_owned(), json!(100000));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.2".to_owned(),
        vhost: "/".to_owned(),
        pattern: r"^ca\.".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    assert!(p.does_match_name("/", "ca.on.to.1", PolicyTarget::Queues));

    assert!(!p.does_match_name("/", "cdi.r.1", PolicyTarget::Queues));
    assert!(!p.does_match_name("/", "ca", PolicyTarget::Queues));
    assert!(!p.does_match_name("/", "abc.r.1", PolicyTarget::Queues));
    assert!(!p.does_match_name("/", "us.ny.nyc.1", PolicyTarget::Queues));
    assert!(!p.does_match_name("a-different-vhost", "ca.on.to.2", PolicyTarget::Queues));
}

#[test]
fn test_unit_policy_does_match_case3() {
    let mut m = Map::new();
    m.insert("alternate-exchange".to_owned(), json!("amq.fanout"));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.3".to_owned(),
        vhost: "/".to_owned(),
        pattern: r"^events\.".to_owned(),
        apply_to: PolicyTarget::Exchanges,
        priority: 11,
        definition: defs.clone(),
    };

    assert!(p.does_match_name("/", "events.regional.na", PolicyTarget::Exchanges));

    assert!(!p.does_match_name("/", "events.regional.na.partitions.1", PolicyTarget::Queues));
    assert!(!p.does_match_name(
        "/",
        "events.regional.na.partitions.1",
        PolicyTarget::ClassicQueues
    ));
    assert!(!p.does_match_name(
        "/",
        "events.regional.na.partitions.1",
        PolicyTarget::QuorumQueues
    ));
    assert!(!p.does_match_name(
        "/",
        "events.regional.na.partitions.1",
        PolicyTarget::Streams
    ));
}

#[test]
fn test_unit_policy_with_overrides_case1() {
    let mut m1 = Map::new();
    m1.insert("max-age".to_owned(), json!("1D"));
    let defs_a = PolicyDefinition(Some(m1));

    let mut m2 = Map::new();
    m2.insert("max-age".to_owned(), json!("3D"));
    m2.insert("max-length-bytes".to_owned(), json!("2000"));
    let defs_b = PolicyDefinition(Some(m2));

    let p1 = Policy {
        name: "test_unit_policy_with_overrides.1".to_owned(),
        vhost: "/".to_owned(),
        pattern: r"^events\.".to_owned(),
        apply_to: PolicyTarget::Exchanges,
        priority: 11,
        definition: defs_a.clone(),
    };

    let new_name = "test_unit_policy_with_overrides.1.override".to_owned();
    let new_priority = 21;
    let p2 = p1.with_overrides(&new_name, new_priority, &defs_b);

    assert_eq!(new_name, p2.name);
    assert_eq!(new_priority, p2.priority);

    assert_eq!(2, p2.definition.len());
    assert!(p2.definition.contains_key("max-age"));
    assert!(p2.definition.contains_key("max-length-bytes"));
    assert!(!p2.definition.contains_key("abc"));
}
