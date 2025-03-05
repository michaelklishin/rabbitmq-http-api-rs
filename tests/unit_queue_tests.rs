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

use rabbitmq_http_client::commons::{PolicyTarget, QueueType};
use rabbitmq_http_client::responses::{
    NamedPolicyTargetObject, Policy, PolicyDefinition, QueueInfo, QueueOps,
};
use serde_json::{json, Map};

#[test]
fn test_unit_queue_type_from_str() {
    assert_eq!(QueueType::Classic, QueueType::from("classic"));
    assert_eq!(QueueType::Classic, QueueType::from("Classic"));

    assert_eq!(QueueType::Quorum, QueueType::from("quorum"));
    assert_eq!(QueueType::Quorum, QueueType::from("Quorum"));

    assert_eq!(QueueType::Stream, QueueType::from("stream"));
    assert_eq!(QueueType::Stream, QueueType::from("Stream"));

    assert_eq!(QueueType::Delayed, QueueType::from("delayed"));
    assert_eq!(QueueType::Delayed, QueueType::from("Delayed"));
}

#[test]
fn test_unit_policy_target_type_case1() {
    let input = r#"{
        "arguments": {
          "x-queue-type": "classic"
        },
        "auto_delete": false,
        "consumer_capacity": 0,
        "consumer_utilisation": 0,
        "consumers": 0,
        "durable": true,
        "effective_policy_definition": {},
        "exclusive": false,
        "memory": 34600,
        "message_bytes": 0,
        "message_bytes_paged_out": 0,
        "message_bytes_persistent": 0,
        "message_bytes_ram": 0,
        "message_bytes_ready": 0,
        "message_bytes_unacknowledged": 0,
        "messages": 0,
        "messages_details": {
          "rate": 0.0
        },
        "messages_paged_out": 0,
        "messages_persistent": 0,
        "messages_ram": 0,
        "messages_ready": 0,
        "messages_ready_details": {
          "rate": 0.0
        },
        "messages_ready_ram": 0,
        "messages_unacknowledged": 0,
        "messages_unacknowledged_details": {
          "rate": 0.0
        },
        "messages_unacknowledged_ram": 0,
        "name": "cq.1",
        "node": "rabbit@sunnyside",
        "reductions": 9799,
        "reductions_details": {
          "rate": 0.0
        },
        "state": "running",
        "storage_version": 2,
        "type": "classic",
        "vhost": "/"
    }"#;
    let cq = serde_json::from_str::<QueueInfo>(input).unwrap();

    assert_eq!(cq.queue_type(), QueueType::Classic);
    assert_eq!(cq.policy_target_type(), PolicyTarget::ClassicQueues);
}

#[test]
fn test_unit_policy_target_type_case2() {
    let input = r#"{
        "arguments": {
          "x-message-ttl": 100000,
          "x-queue-type": "quorum",
          "x-quorum-target-group-size": 1
        },
        "auto_delete": false,
        "consumer_capacity": 0,
        "consumer_utilisation": 0,
        "consumers": 0,
        "delivery_limit": 20,
        "durable": true,
        "effective_policy_definition": {},
        "exclusive": false,
        "leader": "rabbit@sunnyside",
        "members": [
          "rabbit@sunnyside"
        ],
        "memory": 47316,
        "message_bytes": 0,
        "message_bytes_dlx": 0,
        "message_bytes_persistent": 0,
        "message_bytes_ram": 0,
        "message_bytes_ready": 0,
        "message_bytes_unacknowledged": 0,
        "messages": 0,
        "messages_details": {
          "rate": 0.0
        },
        "messages_dlx": 0,
        "messages_persistent": 0,
        "messages_ram": 0,
        "messages_ready": 0,
        "messages_ready_details": {
          "rate": 0.0
        },
        "messages_ready_high": 0,
        "messages_ready_normal": 0,
        "messages_ready_returned": 0,
        "messages_unacknowledged": 0,
        "messages_unacknowledged_details": {
          "rate": 0.0
        },
        "name": "qq.1",
        "node": "rabbit@sunnyside",
        "online": [
          "rabbit@sunnyside"
        ],
        "open_files": {
          "rabbit@sunnyside": 0
        },
        "publishers": 0,
        "reductions": 41261,
        "reductions_details": {
          "rate": 69.8
        },
        "state": "running",
        "type": "quorum",
        "vhost": "/"
    }"#;
    let qq = serde_json::from_str::<QueueInfo>(input).unwrap();

    assert_eq!(qq.queue_type(), QueueType::Quorum);
    assert_eq!(qq.policy_target_type(), PolicyTarget::QuorumQueues);
}

#[test]
fn test_unit_policy_target_type_case3() {
    let input = r#"{
        "arguments": {
          "x-queue-leader-locator": "client-local",
          "x-queue-type": "stream",
          "x-stream-filter-size-bytes": 32
        },
        "auto_delete": false,
        "consumers": 0,
        "durable": true,
        "effective_policy_definition": {},
        "exclusive": false,
        "leader": "rabbit@sunnyside",
        "members": [
          "rabbit@sunnyside"
        ],
        "memory": 143056,
        "messages": 0,
        "messages_details": {
          "rate": 0.0
        },
        "messages_ready": 0,
        "messages_ready_details": {
          "rate": 0.0
        },
        "messages_unacknowledged": 0,
        "messages_unacknowledged_details": {
          "rate": 0.0
        },
        "name": "sq.1",
        "node": "rabbit@sunnyside",
        "online": [
          "rabbit@sunnyside"
        ],
        "readers": {
          "rabbit@sunnyside": 0
        },
        "reductions": 0,
        "reductions_details": {
          "rate": 0.0
        },
        "segments": 1,
        "state": "running",
        "type": "stream",
        "vhost": "/"
    }"#;
    let sq = serde_json::from_str::<QueueInfo>(input).unwrap();

    assert_eq!(sq.queue_type(), QueueType::Stream);
    assert_eq!(sq.policy_target_type(), PolicyTarget::Streams);
}

#[test]
fn test_unit_queue_info_policy_matching_case1() {
    let mut m = Map::new();
    m.insert("max-length".to_owned(), json!(100000));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.1".to_owned(),
        vhost: "/".to_owned(),
        pattern: r"^events\.".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    let input = r#"{
        "arguments": {
          "x-queue-type": "classic"
        },
        "auto_delete": false,
        "consumer_capacity": 0,
        "consumer_utilisation": 0,
        "consumers": 0,
        "durable": true,
        "effective_policy_definition": {},
        "exclusive": false,
        "memory": 21968,
        "message_bytes": 21,
        "message_bytes_paged_out": 0,
        "message_bytes_persistent": 21,
        "message_bytes_ram": 3,
        "message_bytes_ready": 21,
        "message_bytes_unacknowledged": 0,
        "messages": 7,
        "messages_details": {
          "rate": 0.0
        },
        "messages_paged_out": 0,
        "messages_persistent": 7,
        "messages_ram": 1,
        "messages_ready": 7,
        "messages_ready_details": {
          "rate": 0.0
        },
        "messages_ready_ram": 1,
        "messages_unacknowledged": 0,
        "messages_unacknowledged_details": {
          "rate": 0.0
        },
        "messages_unacknowledged_ram": 0,
        "name": "events.matching.cq.1",
        "node": "rabbit@hostname",
        "reductions": 6939,
        "reductions_details": {
          "rate": 0.0
        },
        "state": "running",
        "storage_version": 2,
        "type": "classic",
        "vhost": "/"
    }"#;
    let cq = serde_json::from_str::<QueueInfo>(input).unwrap();

    assert!(cq.does_match(&p));
    assert!(p.does_match_object(&cq));
}

#[test]
fn test_unit_queue_info_policy_matching_case2() {
    let mut m = Map::new();
    m.insert("max-length".to_owned(), json!(100000));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.2".to_owned(),
        vhost: "/".to_owned(),
        pattern: r"^events\.".to_owned(),
        apply_to: PolicyTarget::Queues,
        priority: 11,
        definition: defs.clone(),
    };

    let input = r#"{
        "arguments": {
          "x-queue-type": "classic"
        },
        "auto_delete": false,
        "consumer_capacity": 0,
        "consumer_utilisation": 0,
        "consumers": 0,
        "durable": true,
        "effective_policy_definition": {},
        "exclusive": false,
        "memory": 21968,
        "message_bytes": 21,
        "message_bytes_paged_out": 0,
        "message_bytes_persistent": 21,
        "message_bytes_ram": 3,
        "message_bytes_ready": 21,
        "message_bytes_unacknowledged": 0,
        "messages": 7,
        "messages_details": {
          "rate": 0.0
        },
        "messages_paged_out": 0,
        "messages_persistent": 7,
        "messages_ram": 1,
        "messages_ready": 7,
        "messages_ready_details": {
          "rate": 0.0
        },
        "messages_ready_ram": 1,
        "messages_unacknowledged": 0,
        "messages_unacknowledged_details": {
          "rate": 0.0
        },
        "messages_unacknowledged_ram": 0,
        "name": "orders.regions.na.east",
        "node": "rabbit@hostname",
        "reductions": 6939,
        "reductions_details": {
          "rate": 0.0
        },
        "state": "running",
        "storage_version": 2,
        "type": "classic",
        "vhost": "/"
    }"#;
    let cq = serde_json::from_str::<QueueInfo>(input).unwrap();

    // names do not match
    assert_eq!(false, cq.does_match(&p));
    assert_eq!(false, p.does_match_object(&cq));
}

#[test]
fn test_unit_queue_info_policy_matching_case3() {
    let mut m = Map::new();
    m.insert("max-length".to_owned(), json!(100000));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.2".to_owned(),
        vhost: "/".to_owned(),
        pattern: r"^events\.".to_owned(),
        // won't match a classic queue
        apply_to: PolicyTarget::QuorumQueues,
        priority: 11,
        definition: defs.clone(),
    };

    let input = r#"{
        "arguments": {
          "x-queue-type": "classic"
        },
        "auto_delete": false,
        "consumer_capacity": 0,
        "consumer_utilisation": 0,
        "consumers": 0,
        "durable": true,
        "effective_policy_definition": {},
        "exclusive": false,
        "memory": 21968,
        "message_bytes": 21,
        "message_bytes_paged_out": 0,
        "message_bytes_persistent": 21,
        "message_bytes_ram": 3,
        "message_bytes_ready": 21,
        "message_bytes_unacknowledged": 0,
        "messages": 7,
        "messages_details": {
          "rate": 0.0
        },
        "messages_paged_out": 0,
        "messages_persistent": 7,
        "messages_ram": 1,
        "messages_ready": 7,
        "messages_ready_details": {
          "rate": 0.0
        },
        "messages_ready_ram": 1,
        "messages_unacknowledged": 0,
        "messages_unacknowledged_details": {
          "rate": 0.0
        },
        "messages_unacknowledged_ram": 0,
        "name": "events.signin.attempts",
        "node": "rabbit@hostname",
        "reductions": 6939,
        "reductions_details": {
          "rate": 0.0
        },
        "state": "running",
        "storage_version": 2,
        "type": "classic",
        "vhost": "/"
    }"#;
    let cq = serde_json::from_str::<QueueInfo>(input).unwrap();

    // policy target does not match
    assert_eq!(false, cq.does_match(&p));
    assert_eq!(false, p.does_match_object(&cq));
}

#[test]
fn test_unit_queue_info_policy_matching_case4() {
    let mut m = Map::new();
    m.insert("max-length".to_owned(), json!(100000));
    let defs = PolicyDefinition(Some(m));
    let p = Policy {
        name: "policy.3".to_owned(),
        vhost: "vh-1".to_owned(),
        pattern: r"^events\.".to_owned(),
        // won't match a classic queue
        apply_to: PolicyTarget::QuorumQueues,
        priority: 11,
        definition: defs.clone(),
    };

    let input = r#"{
        "arguments": {
          "x-queue-type": "quorum"
        },
        "auto_delete": false,
        "consumer_capacity": 0,
        "consumer_utilisation": 0,
        "consumers": 0,
        "durable": true,
        "effective_policy_definition": {},
        "exclusive": false,
        "memory": 21968,
        "message_bytes": 21,
        "message_bytes_paged_out": 0,
        "message_bytes_persistent": 21,
        "message_bytes_ram": 3,
        "message_bytes_ready": 21,
        "message_bytes_unacknowledged": 0,
        "messages": 7,
        "messages_details": {
          "rate": 0.0
        },
        "messages_paged_out": 0,
        "messages_persistent": 7,
        "messages_ram": 1,
        "messages_ready": 7,
        "messages_ready_details": {
          "rate": 0.0
        },
        "messages_ready_ram": 1,
        "messages_unacknowledged": 0,
        "messages_unacknowledged_details": {
          "rate": 0.0
        },
        "messages_unacknowledged_ram": 0,
        "name": "events.signin.attempts",
        "node": "rabbit@hostname",
        "reductions": 6939,
        "reductions_details": {
          "rate": 0.0
        },
        "state": "running",
        "storage_version": 2,
        "type": "quorum",
        "vhost": "vh-abc-126387"
    }"#;
    let cq = serde_json::from_str::<QueueInfo>(input).unwrap();

    // virtual hosts do not match
    assert_eq!(false, cq.does_match(&p));
    assert_eq!(false, p.does_match_object(&cq));
}
