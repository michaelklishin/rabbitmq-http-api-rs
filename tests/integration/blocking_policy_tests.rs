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
    blocking_api::Client,
    commons::PolicyTarget,
    requests::{PolicyParams, VirtualHostParams},
};

use crate::test_helpers::{PASSWORD, USERNAME, endpoint};
use serde_json::{Map, Value, json};

#[test]
fn test_blocking_message_ttl_policy() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_message_ttl_policy");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
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
    test_a_policy(&rc, &message_ttl_policy);

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_dlx_policy() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map = Map::<String, Value>::new();
    map.insert("dead-letter-exchange".to_owned(), json!("my-dlx"));
    let policy_definition = map.clone();

    let vh_params = VirtualHostParams::named("test_dlx_policy");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let dlx_policy = PolicyParams {
        vhost: vh_params.name,
        name: "dlx_policy",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 0,
        definition: policy_definition,
    };
    test_a_policy(&rc, &dlx_policy);

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_multiple_policies_case1() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map1 = Map::<String, Value>::new();
    map1.insert("dead-letter-exchange".to_owned(), json!("my-dlx"));
    let policy_definition1 = map1.clone();

    let mut map2 = Map::<String, Value>::new();
    map2.insert("message-ttl".to_owned(), json!(10_000));
    let policy_definition2 = map2.clone();

    let vh_params = VirtualHostParams::named("test_blocking_multiple_policies_case1");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result0 = rc.create_vhost(&vh_params);
    assert!(result0.is_ok());

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

    let result1 = rc.declare_policies(vec![&dlx_policy, &message_ttl_policy]);
    assert!(result1.is_ok());

    let result2 = rc.list_policies_in(vh_params.name);
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().len(), 2);

    let result3 = rc.delete_policies_in(
        vh_params.name,
        vec![dlx_policy.name, message_ttl_policy.name],
    );
    assert!(result3.is_ok());

    let result4 = rc.list_policies_in(vh_params.name);
    assert!(result4.is_ok());
    assert_eq!(result4.unwrap().len(), 0);

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_operator_policy() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map = Map::<String, Value>::new();
    map.insert("delivery-limit".to_owned(), json!(13));
    let policy_definition = map.clone();

    let vh_params = VirtualHostParams::named("test_operator_policy");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let operator_policy = PolicyParams {
        vhost: vh_params.name,
        name: "operator_policy",
        pattern: ".*",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 0,
        definition: policy_definition,
    };
    test_an_operator_policy(&rc, &operator_policy);

    let _ = rc.delete_vhost(vh_params.name, true);
}

#[test]
fn test_blocking_multiple_operator_policies_case1() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let mut map1 = Map::<String, Value>::new();
    map1.insert("delivery-limit".to_owned(), json!(13));
    let policy_definition1 = map1.clone();

    let mut map2 = Map::<String, Value>::new();
    map2.insert("delivery-limit".to_owned(), json!(67));
    let policy_definition2 = map2.clone();

    let vh_params = VirtualHostParams::named("test_blocking_multiple_operator_policies_case1");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result0 = rc.create_vhost(&vh_params);
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

    let result1 = rc.declare_operator_policies(vec![&dlx_policy, &message_ttl_policy]);
    assert!(result1.is_ok());

    let result2 = rc.list_operator_policies_in(vh_params.name);
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap().len(), 2);

    let result3 = rc.delete_operator_policies_in(
        vh_params.name,
        vec![dlx_policy.name, message_ttl_policy.name],
    );
    assert!(result3.is_ok());

    let result4 = rc.list_operator_policies_in(vh_params.name);
    assert!(result4.is_ok());
    assert_eq!(result4.unwrap().len(), 0);

    let _ = rc.delete_vhost(vh_params.name, false);
}

//
// Implementation
//

fn test_a_policy(rc: &Client<&str, &str, &str>, policy: &PolicyParams) {
    // initially, there should be no such policy
    let policies = rc.list_policies_in(policy.vhost).unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));

    let result = rc.declare_policy(policy);
    assert!(result.is_ok(), "declare_policy returned {result:?}");

    // validate it was created as expected
    let fetched_policy = rc.get_policy(policy.vhost, policy.name).unwrap();
    assert_eq!(fetched_policy.definition.0.unwrap(), policy.definition);

    // delete it
    assert!(rc.delete_policy(policy.vhost, policy.name, true).is_ok());

    // idempotent delete should succeed
    assert!(rc.delete_policy(policy.vhost, policy.name, true).is_ok());

    // non-idempotent delete should fail
    assert!(rc.delete_policy(policy.vhost, policy.name, false).is_err());

    // there should be no such policy anymore
    let policies = rc.list_policies().unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));
}

fn test_an_operator_policy(rc: &Client<&str, &str, &str>, policy: &PolicyParams) {
    // initially, there should be no such policy
    let policies = rc.list_operator_policies_in(policy.vhost).unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));

    let result = rc.declare_operator_policy(policy);
    assert!(result.is_ok(), "declare_policy returned {result:?}");

    // validate it was created as expected
    let fetched_policy = rc.get_operator_policy(policy.vhost, policy.name).unwrap();
    assert_eq!(fetched_policy.definition.0.unwrap(), policy.definition);

    // delete it
    assert!(
        rc.delete_operator_policy(policy.vhost, policy.name, true)
            .is_ok()
    );

    // idempotent delete should succeed
    assert!(
        rc.delete_operator_policy(policy.vhost, policy.name, true)
            .is_ok()
    );

    // non-idempotent delete should fail
    assert!(
        rc.delete_operator_policy(policy.vhost, policy.name, false)
            .is_err()
    );

    // there should be no such policy anymore
    let policies = rc.list_operator_policies().unwrap();
    assert!(!policies.iter().any(|p| p.name == policy.name));
}

#[test]
fn test_blocking_policy_validation_error() {
    let endpoint = endpoint();
    let rc = Client::new(endpoint.as_str(), USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_policy_validation_error");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    // Create a policy with an invalid/unknown property in the definition
    let mut map = Map::<String, Value>::new();
    map.insert("foo".to_owned(), json!("bar"));
    map.insert("invalid-setting".to_owned(), json!(12345));
    let invalid_definition = map.clone();

    let invalid_policy = PolicyParams {
        vhost: vh_params.name,
        name: "invalid_policy",
        pattern: "^qq$",
        apply_to: PolicyTarget::Queues,
        priority: 1,
        definition: invalid_definition,
    };

    // Attempting to declare this policy should fail with a validation error
    let result = rc.declare_policy(&invalid_policy);
    assert!(
        result.is_err(),
        "Expected policy declaration to fail with validation error"
    );

    // Extract the error and verify it contains structured error details
    if let Err(err) = result {
        match err {
            rabbitmq_http_client::error::Error::ClientErrorResponse {
                status_code,
                error_details,
                ..
            } => {
                // Should be a 400 Bad Request
                assert_eq!(status_code, reqwest::StatusCode::BAD_REQUEST);

                // Should have parsed error details
                assert!(
                    error_details.is_some(),
                    "Expected error_details to be parsed"
                );

                let details = error_details.unwrap();

                // Should have an error type
                assert!(
                    details.error.is_some(),
                    "Expected error field to be present"
                );
                assert_eq!(details.error.as_deref(), Some("bad_request"));

                // Should have a detailed reason
                assert!(
                    details.reason.is_some(),
                    "Expected reason field to be present"
                );
                let reason = details.reason.as_ref().unwrap();

                // The reason should mention validation failure
                assert!(
                    reason.contains("Validation failed"),
                    "Expected reason to contain 'Validation failed', got: {}",
                    reason
                );

                // The reason should mention unrecognized properties
                assert!(
                    reason.contains("not recognised") || reason.contains("not recognized"),
                    "Expected reason to mention unrecognized properties, got: {}",
                    reason
                );

                // Verify reason() returns the reason (more detailed message)
                let detailed_msg = details.reason();
                assert!(detailed_msg.is_some());
                assert_eq!(detailed_msg, details.reason.as_deref());
            }
            _ => panic!("Expected ClientErrorResponse, got: {:?}", err),
        }
    }

    let _ = rc.delete_vhost(vh_params.name, false);
}
