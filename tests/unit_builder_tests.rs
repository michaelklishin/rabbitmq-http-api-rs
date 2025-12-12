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

use rabbitmq_http_client::commons::{PolicyTarget, QueueType};
use rabbitmq_http_client::requests::{
    PolicyDefinitionBuilder, PolicyParams, UserParams, VirtualHostParams, XArgumentsBuilder,
};
use serde_json::json;

#[test]
fn test_x_arguments_builder_empty() {
    let args = XArgumentsBuilder::new().build();
    assert!(args.is_none());
}

#[test]
fn test_x_arguments_builder_message_ttl() {
    let args = XArgumentsBuilder::new().message_ttl(60000).build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-message-ttl"), Some(&json!(60000)));
}

#[test]
fn test_x_arguments_builder_queue_ttl() {
    let args = XArgumentsBuilder::new().queue_ttl(300000).build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-expires"), Some(&json!(300000)));
}

#[test]
fn test_x_arguments_builder_max_length() {
    let args = XArgumentsBuilder::new().max_length(10000).build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-max-length"), Some(&json!(10000)));
}

#[test]
fn test_x_arguments_builder_max_length_bytes() {
    let args = XArgumentsBuilder::new().max_length_bytes(1048576).build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-max-length-bytes"), Some(&json!(1048576)));
}

#[test]
fn test_x_arguments_builder_dead_letter_exchange() {
    let args = XArgumentsBuilder::new().dead_letter_exchange("dlx").build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-dead-letter-exchange"), Some(&json!("dlx")));
}

#[test]
fn test_x_arguments_builder_dead_letter_routing_key() {
    let args = XArgumentsBuilder::new()
        .dead_letter_routing_key("dlx.routing.key")
        .build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(
        map.get("x-dead-letter-routing-key"),
        Some(&json!("dlx.routing.key"))
    );
}

#[test]
fn test_x_arguments_builder_overflow_drop_head() {
    let args = XArgumentsBuilder::new().overflow_drop_head().build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-overflow"), Some(&json!("drop-head")));
}

#[test]
fn test_x_arguments_builder_overflow_reject_publish() {
    let args = XArgumentsBuilder::new().overflow_reject_publish().build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-overflow"), Some(&json!("reject-publish")));
}

#[test]
fn test_x_arguments_builder_overflow_reject_publish_dlx() {
    let args = XArgumentsBuilder::new()
        .overflow_reject_publish_dlx()
        .build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-overflow"), Some(&json!("reject-publish-dlx")));
}

#[test]
fn test_x_arguments_builder_max_priority() {
    let args = XArgumentsBuilder::new().max_priority(10).build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-max-priority"), Some(&json!(10)));
}

#[test]
fn test_x_arguments_builder_quorum_initial_group_size() {
    let args = XArgumentsBuilder::new()
        .quorum_initial_group_size(3)
        .build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-quorum-initial-group-size"), Some(&json!(3)));
}

#[test]
fn test_x_arguments_builder_delivery_limit() {
    let args = XArgumentsBuilder::new().delivery_limit(5).build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-delivery-limit"), Some(&json!(5)));
}

#[test]
fn test_x_arguments_builder_single_active_consumer() {
    let args = XArgumentsBuilder::new()
        .single_active_consumer(true)
        .build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-single-active-consumer"), Some(&json!(true)));
}

#[test]
fn test_x_arguments_builder_custom() {
    let args = XArgumentsBuilder::new()
        .custom("x-custom-key", json!("custom-value"))
        .build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-custom-key"), Some(&json!("custom-value")));
}

#[test]
fn test_x_arguments_builder_chained() {
    let args = XArgumentsBuilder::new()
        .message_ttl(60000)
        .max_length(10000)
        .dead_letter_exchange("dlx")
        .delivery_limit(5)
        .build();
    assert!(args.is_some());
    let map = args.unwrap();
    assert_eq!(map.get("x-message-ttl"), Some(&json!(60000)));
    assert_eq!(map.get("x-max-length"), Some(&json!(10000)));
    assert_eq!(map.get("x-dead-letter-exchange"), Some(&json!("dlx")));
    assert_eq!(map.get("x-delivery-limit"), Some(&json!(5)));
}

#[test]
fn test_policy_definition_builder_empty() {
    let def = PolicyDefinitionBuilder::new().build();
    assert!(def.is_empty());
}

#[test]
fn test_policy_definition_builder_message_ttl() {
    let def = PolicyDefinitionBuilder::new().message_ttl(60000).build();
    assert_eq!(def.get("message-ttl"), Some(&json!(60000)));
}

#[test]
fn test_policy_definition_builder_expires() {
    let def = PolicyDefinitionBuilder::new().expires(300000).build();
    assert_eq!(def.get("expires"), Some(&json!(300000)));
}

#[test]
fn test_policy_definition_builder_max_length() {
    let def = PolicyDefinitionBuilder::new().max_length(10000).build();
    assert_eq!(def.get("max-length"), Some(&json!(10000)));
}

#[test]
fn test_policy_definition_builder_max_length_bytes() {
    let def = PolicyDefinitionBuilder::new()
        .max_length_bytes(1048576)
        .build();
    assert_eq!(def.get("max-length-bytes"), Some(&json!(1048576)));
}

#[test]
fn test_policy_definition_builder_overflow_drop_head() {
    let def = PolicyDefinitionBuilder::new().overflow_drop_head().build();
    assert_eq!(def.get("overflow"), Some(&json!("drop-head")));
}

#[test]
fn test_policy_definition_builder_overflow_reject_publish() {
    let def = PolicyDefinitionBuilder::new()
        .overflow_reject_publish()
        .build();
    assert_eq!(def.get("overflow"), Some(&json!("reject-publish")));
}

#[test]
fn test_policy_definition_builder_overflow_reject_publish_dlx() {
    let def = PolicyDefinitionBuilder::new()
        .overflow_reject_publish_dlx()
        .build();
    assert_eq!(def.get("overflow"), Some(&json!("reject-publish-dlx")));
}

#[test]
fn test_policy_definition_builder_dead_letter_exchange() {
    let def = PolicyDefinitionBuilder::new()
        .dead_letter_exchange("dlx")
        .build();
    assert_eq!(def.get("dead-letter-exchange"), Some(&json!("dlx")));
}

#[test]
fn test_policy_definition_builder_dead_letter_routing_key() {
    let def = PolicyDefinitionBuilder::new()
        .dead_letter_routing_key("dlx.routing.key")
        .build();
    assert_eq!(
        def.get("dead-letter-routing-key"),
        Some(&json!("dlx.routing.key"))
    );
}

#[test]
fn test_policy_definition_builder_delivery_limit() {
    let def = PolicyDefinitionBuilder::new().delivery_limit(5).build();
    assert_eq!(def.get("delivery-limit"), Some(&json!(5)));
}

#[test]
fn test_policy_definition_builder_quorum_group_size() {
    let def = PolicyDefinitionBuilder::new().quorum_group_size(5).build();
    assert_eq!(def.get("target-group-size"), Some(&json!(5)));
}

#[test]
fn test_policy_definition_builder_quorum_initial_group_size() {
    let def = PolicyDefinitionBuilder::new()
        .quorum_initial_group_size(3)
        .build();
    assert_eq!(def.get("initial-cluster-size"), Some(&json!(3)));
}

#[test]
fn test_policy_definition_builder_max_age() {
    let def = PolicyDefinitionBuilder::new().max_age("1D").build();
    assert_eq!(def.get("max-age"), Some(&json!("1D")));
}

#[test]
fn test_policy_definition_builder_stream_max_segment_size_bytes() {
    let def = PolicyDefinitionBuilder::new()
        .stream_max_segment_size_bytes(536870912)
        .build();
    assert_eq!(
        def.get("stream-max-segment-size-bytes"),
        Some(&json!(536870912))
    );
}

#[test]
fn test_policy_definition_builder_federation_upstream() {
    let def = PolicyDefinitionBuilder::new()
        .federation_upstream("my-upstream")
        .build();
    assert_eq!(def.get("federation-upstream"), Some(&json!("my-upstream")));
}

#[test]
fn test_policy_definition_builder_federation_upstream_set() {
    let def = PolicyDefinitionBuilder::new()
        .federation_upstream_set("all")
        .build();
    assert_eq!(def.get("federation-upstream-set"), Some(&json!("all")));
}

#[test]
fn test_policy_definition_builder_custom() {
    let def = PolicyDefinitionBuilder::new()
        .custom("custom-key", json!("custom-value"))
        .build();
    assert_eq!(def.get("custom-key"), Some(&json!("custom-value")));
}

#[test]
fn test_policy_definition_builder_chained() {
    let def = PolicyDefinitionBuilder::new()
        .max_length(10000)
        .overflow_reject_publish()
        .dead_letter_exchange("dlx")
        .delivery_limit(5)
        .build();
    assert_eq!(def.get("max-length"), Some(&json!(10000)));
    assert_eq!(def.get("overflow"), Some(&json!("reject-publish")));
    assert_eq!(def.get("dead-letter-exchange"), Some(&json!("dlx")));
    assert_eq!(def.get("delivery-limit"), Some(&json!(5)));
}

#[test]
fn test_policy_params_new() {
    let def = PolicyDefinitionBuilder::new().max_length(10000).build();
    let params = PolicyParams::new("/", "test-policy", "^test\\.", def);
    assert_eq!(params.vhost, "/");
    assert_eq!(params.name, "test-policy");
    assert_eq!(params.pattern, "^test\\.");
    assert_eq!(params.apply_to, PolicyTarget::All);
    assert_eq!(params.priority, 0);
}

#[test]
fn test_policy_params_apply_to() {
    let def = PolicyDefinitionBuilder::new().max_length(10000).build();
    let params =
        PolicyParams::new("/", "test-policy", "^test\\.", def).apply_to(PolicyTarget::Queues);
    assert_eq!(params.apply_to, PolicyTarget::Queues);
}

#[test]
fn test_policy_params_priority() {
    let def = PolicyDefinitionBuilder::new().max_length(10000).build();
    let params = PolicyParams::new("/", "test-policy", "^test\\.", def).priority(10);
    assert_eq!(params.priority, 10);
}

#[test]
fn test_policy_params_chained() {
    let def = PolicyDefinitionBuilder::new()
        .max_length(10000)
        .overflow_reject_publish()
        .build();
    let params = PolicyParams::new("/", "test-policy", "^test\\.", def)
        .apply_to(PolicyTarget::QuorumQueues)
        .priority(5);
    assert_eq!(params.vhost, "/");
    assert_eq!(params.name, "test-policy");
    assert_eq!(params.pattern, "^test\\.");
    assert_eq!(params.apply_to, PolicyTarget::QuorumQueues);
    assert_eq!(params.priority, 5);
}

#[test]
fn test_user_params_new() {
    let params = UserParams::new("testuser", "hash123", "administrator");
    assert_eq!(params.name, "testuser");
    assert_eq!(params.password_hash, "hash123");
    assert_eq!(params.tags, "administrator");
}

#[test]
fn test_user_params_administrator() {
    let params = UserParams::administrator("admin", "hash123");
    assert_eq!(params.name, "admin");
    assert_eq!(params.password_hash, "hash123");
    assert_eq!(params.tags, "administrator");
}

#[test]
fn test_user_params_monitoring() {
    let params = UserParams::monitoring("monitor", "hash123");
    assert_eq!(params.name, "monitor");
    assert_eq!(params.password_hash, "hash123");
    assert_eq!(params.tags, "monitoring");
}

#[test]
fn test_user_params_management() {
    let params = UserParams::management("mgmt", "hash123");
    assert_eq!(params.name, "mgmt");
    assert_eq!(params.password_hash, "hash123");
    assert_eq!(params.tags, "management");
}

#[test]
fn test_user_params_policymaker() {
    let params = UserParams::policymaker("policy", "hash123");
    assert_eq!(params.name, "policy");
    assert_eq!(params.password_hash, "hash123");
    assert_eq!(params.tags, "policymaker");
}

#[test]
fn test_user_params_without_tags() {
    let params = UserParams::without_tags("basic", "hash123");
    assert_eq!(params.name, "basic");
    assert_eq!(params.password_hash, "hash123");
    assert_eq!(params.tags, "");
}

#[test]
fn test_vhost_params_named() {
    let params = VirtualHostParams::named("test-vhost");
    assert_eq!(params.name, "test-vhost");
    assert!(params.description.is_none());
    assert!(params.tags.is_none());
    assert!(params.default_queue_type.is_none());
    assert!(!params.tracing);
}

#[test]
fn test_vhost_params_with_description() {
    let params = VirtualHostParams::named("test-vhost").with_description("Test virtual host");
    assert_eq!(params.name, "test-vhost");
    assert_eq!(params.description, Some("Test virtual host"));
}

#[test]
fn test_vhost_params_with_tags() {
    let params = VirtualHostParams::named("test-vhost").with_tags(vec!["prod", "critical"]);
    assert_eq!(params.tags, Some(vec!["prod", "critical"]));
}

#[test]
fn test_vhost_params_with_default_queue_type() {
    let params = VirtualHostParams::named("test-vhost").with_default_queue_type(QueueType::Quorum);
    assert_eq!(params.default_queue_type, Some(QueueType::Quorum));
}

#[test]
fn test_vhost_params_with_tracing() {
    let params = VirtualHostParams::named("test-vhost").with_tracing();
    assert!(params.tracing);
}

#[test]
fn test_vhost_params_builder_basic() {
    let params = VirtualHostParams::builder("test-vhost").build();
    assert_eq!(params.name, "test-vhost");
    assert!(params.description.is_none());
    assert!(params.tags.is_none());
    assert!(params.default_queue_type.is_none());
    assert!(!params.tracing);
}

#[test]
fn test_vhost_params_builder_full() {
    let params = VirtualHostParams::builder("production")
        .description("Production environment")
        .tags(vec!["production", "critical"])
        .default_queue_type(QueueType::Quorum)
        .tracing(true)
        .build();
    assert_eq!(params.name, "production");
    assert_eq!(params.description, Some("Production environment"));
    assert_eq!(params.tags, Some(vec!["production", "critical"]));
    assert_eq!(params.default_queue_type, Some(QueueType::Quorum));
    assert!(params.tracing);
}

#[test]
fn test_vhost_params_builder_partial() {
    let params = VirtualHostParams::builder("staging")
        .description("Staging environment")
        .tracing(false)
        .build();
    assert_eq!(params.name, "staging");
    assert_eq!(params.description, Some("Staging environment"));
    assert!(params.tags.is_none());
    assert!(params.default_queue_type.is_none());
    assert!(!params.tracing);
}
