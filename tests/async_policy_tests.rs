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
use rabbitmq_http_client::{
    api::Client,
    commons::PolicyTarget,
    requests::{PolicyParams, VirtualHostParams},
};

use serde_json::{json, Map, Value};
mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[tokio::test]
async fn test_async_message_ttl_policy() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_message_ttl_policy");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let mut map = Map::<String, Value>::new();
    map.insert("message-ttl".to_owned(), json!(10_000));
    let policy_definition = map.clone();

    let message_ttl_policy = PolicyParams {
        vhost: vh_params.name,
        name: "message_ttl_policy",
        pattern: ".*",
        apply_to: PolicyTarget::ClassicQueues,
        priority: 42,
        definition: policy_definition,
    };
    test_a_policy(&rc, &message_ttl_policy).await;

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

#[tokio::test]
async fn test_async_dlx_policy() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map = Map::<String, Value>::new();
    map.insert("dead-letter-exchange".to_owned(), json!("my-dlx"));
    let policy_definition = map.clone();

    let vh_params = VirtualHostParams::named("test_dlx_policy");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let dlx_policy = PolicyParams {
        vhost: vh_params.name,
        name: "dlx_policy",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 0,
        definition: policy_definition,
    };
    test_a_policy(&rc, &dlx_policy).await;

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

#[tokio::test]
async fn test_async_operator_policy() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map = Map::<String, Value>::new();
    map.insert("delivery-limit".to_owned(), json!(13));
    let policy_definition = map.clone();

    let vh_params = VirtualHostParams::named("test_operator_policy");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let operator_policy = PolicyParams {
        vhost: vh_params.name,
        name: "operator_policy",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 0,
        definition: policy_definition,
    };
    test_an_operator_policy(&rc, &operator_policy).await;

    let _ = rc.delete_vhost(vh_params.name, true).await;
}

async fn test_a_policy(rc: &Client<&str, &str, &str>, policy: &PolicyParams<'_>) {
    // initially, there should be no such policy
    let policies = rc.list_policies_in(policy.vhost).await.unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));

    let result = rc.declare_policy(policy).await;
    assert!(result.is_ok(), "declare_policy returned {:?}", result);

    // validate it was created as expected
    let fetched_policy = rc.get_policy(policy.vhost, policy.name).await.unwrap();
    assert_eq!(fetched_policy.definition.0.unwrap(), policy.definition);

    // delete it
    assert!(rc.delete_policy(policy.vhost, policy.name).await.is_ok());

    // there should be no such policy anymore
    let policies = rc.list_policies().await.unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));
}

async fn test_an_operator_policy(rc: &Client<&str, &str, &str>, policy: &PolicyParams<'_>) {
    // initially, there should be no such policy
    let policies = rc.list_operator_policies_in(policy.vhost).await.unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));

    let result = rc.declare_operator_policy(policy).await;
    assert!(result.is_ok(), "declare_policy returned {:?}", result);

    // validate it was created as expected
    let fetched_policy = rc
        .get_operator_policy(policy.vhost, policy.name)
        .await
        .unwrap();
    assert_eq!(fetched_policy.definition.0.unwrap(), policy.definition);

    // delete it
    assert!(rc
        .delete_operator_policy(policy.vhost, policy.name)
        .await
        .is_ok());

    // there should be no such policy anymore
    let policies = rc.list_operator_policies().await.unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));
}
