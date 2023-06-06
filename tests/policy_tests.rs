extern crate rabbitmq_http_client;
use rabbitmq_http_client::{
    blocking::Client,
    commons::PolicyTarget,
    requests::{PolicyParams, VirtualHostParams},
};

use serde_json::{json, Map, Value};
extern crate common;
use crate::common::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_message_ttl_policy() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams {
        name: "test_message_ttl_policy",
        tracing: false,
        description: None,
        tags: None,
        default_queue_type: None,
    };
    let _ = rc.delete_vhost(vh_params.name);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let mut map = Map::<String, Value>::new();
    map.insert("message-ttl".to_owned(), json!(10_000));
    let policy_definition = Some(map);

    let message_ttl_policy = PolicyParams {
        vhost: vh_params.name,
        name: "message_ttl_policy",
        pattern: ".*",
        apply_to: PolicyTarget::ClassicQueues,
        priority: 42,
        definition: policy_definition,
    };
    test_a_policy(&rc, &message_ttl_policy);

    let _ = rc.delete_vhost(vh_params.name);
}

#[test]
fn test_dlx_policy() {
    let endpoint = endpoint();
    let rc = Client::new_with_basic_auth_credentials(&endpoint, USERNAME, PASSWORD);

    let mut map = Map::<String, Value>::new();
    map.insert("dead-letter-exchange".to_owned(), json!("my-dlx"));
    let policy_definition = Some(map);

    let vh_params = VirtualHostParams {
        name: "test_dlx_policy",
        tracing: false,
        description: None,
        tags: None,
        default_queue_type: None,
    };
    let _ = rc.delete_vhost(vh_params.name);
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

    let _ = rc.delete_vhost(vh_params.name);
}

fn test_a_policy(rc: &Client, policy: &PolicyParams) {
    // initially, there should be no such policy
    let policies = rc.list_policies_in(policy.vhost).unwrap();
    assert!(policies.iter().find(|p| p.name == policy.name).is_none());

    let result = rc.declare_policy(&policy);
    assert!(result.is_ok());

    // validate it was created as expected
    let result = rc.get_policy(policy.vhost, policy.name);
    assert!(result.unwrap().definition == policy.definition);

    // delete it
    assert!(rc.delete_policy(policy.vhost, policy.name).is_ok());

    // there should be no such policy anymore
    let policies = rc.list_policies().unwrap();
    assert!(policies.iter().find(|p| p.name == policy.name).is_none());
}
