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

use serde_json::{Map, Value, json};
mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint};

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
async fn test_async_multiple_policies_case1() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map1 = Map::<String, Value>::new();
    map1.insert("dead-letter-exchange".to_owned(), json!("my-dlx"));
    let policy_definition1 = map1.clone();

    let mut map2 = Map::<String, Value>::new();
    map2.insert("message-ttl".to_owned(), json!(10_000));
    let policy_definition2 = map2.clone();

    let vh_params = VirtualHostParams::named("test_blocking_multiple_policies_case1");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let dlx_policy = PolicyParams {
        vhost: vh_params.name,
        name: "dlx_policy",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 3,
        definition: policy_definition1,
    };
    let message_ttl_policy = PolicyParams {
        vhost: vh_params.name,
        name: "message_ttl_policy",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 3,
        definition: policy_definition2,
    };

    let result1 = rc
        .declare_policies(vec![&dlx_policy, &message_ttl_policy])
        .await;
    assert!(result1.is_ok());

    let result2 = rc.list_policies_in(vh_params.name).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().len(), 2);

    let result3 = rc
        .delete_policies_in(
            vh_params.name,
            vec![dlx_policy.name, message_ttl_policy.name],
        )
        .await;
    assert!(result3.is_ok());

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

#[tokio::test]
async fn test_async_multiple_operator_policies_case1() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map1 = Map::<String, Value>::new();
    map1.insert("delivery-limit".to_owned(), json!(13));
    let policy_definition1 = map1.clone();

    let mut map2 = Map::<String, Value>::new();
    map2.insert("delivery-limit".to_owned(), json!(67));
    let policy_definition2 = map2.clone();

    let vh_params = VirtualHostParams::named("test_async_multiple_operator_policies_case1");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result0 = rc.create_vhost(&vh_params).await;
    assert!(result0.is_ok());

    let dlx_policy = PolicyParams {
        vhost: vh_params.name,
        name: "operator_policy.1",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 2,
        definition: policy_definition1,
    };
    let message_ttl_policy = PolicyParams {
        vhost: vh_params.name,
        name: "message_ttl_policy",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 3,
        definition: policy_definition2,
    };

    let result1 = rc
        .declare_operator_policies(vec![&dlx_policy, &message_ttl_policy])
        .await;
    assert!(result1.is_ok());

    let result2 = rc.list_operator_policies_in(vh_params.name).await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().len(), 2);

    let result3 = rc
        .delete_operator_policies_in(
            vh_params.name,
            vec![dlx_policy.name, message_ttl_policy.name],
        )
        .await;
    assert!(result3.is_ok());

    let result4 = rc.list_operator_policies_in(vh_params.name).await;
    assert!(result4.is_ok());
    assert_eq!(result4.unwrap().len(), 0);

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

//
// Implementation
//

async fn test_a_policy(rc: &Client<&str, &str, &str>, policy: &PolicyParams<'_>) {
    // initially, there should be no such policy
    let policies = rc.list_policies_in(policy.vhost).await.unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));

    let result = rc.declare_policy(policy).await;
    assert!(result.is_ok(), "declare_policy returned {result:?}");

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
    assert!(result.is_ok(), "declare_policy returned {result:?}");

    // validate it was created as expected
    let fetched_policy = rc
        .get_operator_policy(policy.vhost, policy.name)
        .await
        .unwrap();
    assert_eq!(fetched_policy.definition.0.unwrap(), policy.definition);

    // delete it
    assert!(
        rc.delete_operator_policy(policy.vhost, policy.name)
            .await
            .is_ok()
    );

    // there should be no such policy anymore
    let policies = rc.list_operator_policies().await.unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));
}
