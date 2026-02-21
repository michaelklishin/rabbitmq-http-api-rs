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
    commons::{MessageTransferAcknowledgementMode, QueueType},
    requests::{
        FederationResourceCleanupMode, FederationUpstreamParams, OwnedFederationUpstreamParams,
        RuntimeParameterDefinition,
    },
    responses::{FederationLink, FederationType, FederationUpstream, RuntimeParameter},
};

#[test]
fn test_unit_deserialize_federation_upstream_case1() {
    let json = r#"
        {
          "value": {
            "ack-mode": "on-publish",
            "consumer-tag": "fed.tags.1",
            "queue": "fed.cq.1",
            "trust-user-id": false,
            "uri": "amqp://localhost:5673/%2f"
          },
          "vhost": "/",
          "component": "federation-upstream",
          "name": "up-1"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let upstream = FederationUpstream::try_from(param.clone()).unwrap();

    assert_eq!(param.name, upstream.name);
    assert_eq!(param.vhost, upstream.vhost);

    assert_eq!("amqp://localhost:5673/%2f", upstream.uri);
    assert_eq!("fed.cq.1", upstream.queue.unwrap());
    assert_eq!("fed.tags.1", upstream.consumer_tag.unwrap());
    assert_eq!(
        MessageTransferAcknowledgementMode::WhenPublished,
        upstream.ack_mode
    );
}

#[test]
fn test_unit_deserialize_federation_upstream_case2() {
    let json = r#"
        {
          "value": {
            "ack-mode": "on-confirm",
            "exchange": "fed.ex.up",
            "expires": 10000000000000000,
            "max-hops": 1,
            "message-ttl": 10000000000000000,
            "prefetch-count": 100,
            "queue-type": "quorum",
            "reconnect-delay": 5,
            "trust-user-id": true,
            "uri": "amqp://localhost:5673/%2f"
          },
          "vhost": "/",
          "component": "federation-upstream",
          "name": "up-2"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let upstream = FederationUpstream::try_from(param.clone()).unwrap();

    assert_eq!(param.name, upstream.name);
    assert_eq!(param.vhost, upstream.vhost);

    assert_eq!("amqp://localhost:5673/%2f", upstream.uri);
    assert_eq!("fed.ex.up", upstream.exchange.unwrap());
    assert_eq!(1, upstream.max_hops.unwrap());
    assert_eq!(
        MessageTransferAcknowledgementMode::WhenConfirmed,
        upstream.ack_mode
    );
}

#[test]
fn test_unit_deserialize_federation_link_case1() {
    let json = r#"
        {
          "node": "rabbit@sunnyside",
          "queue": "fed.cq.1",
          "upstream_queue": "fed.cq.overridden",
          "consumer_tag": "hgksdh98s7f98au9u",
          "type": "queue",
          "vhost": "/",
          "upstream": "up-1",
          "id": "e178dfad",
          "status": "running",
          "local_connection": "<rabbit@sunnyside.1741991552.108078.0>",
          "uri": "amqp://localhost:5672/fed",
          "timestamp": "2025-03-16 0:41:29",
          "local_channel": {
            "acks_uncommitted": 0,
            "confirm": true,
            "connection_details": {
              "name": "<rabbit@sunnyside.1741991552.108078.0>",
              "peer_host": "undefined",
              "peer_port": "undefined"
            },
            "consumer_count": 0,
            "garbage_collection": {
              "fullsweep_after": 65535,
              "max_heap_size": 0,
              "min_bin_vheap_size": 1727361,
              "min_heap_size": 233,
              "minor_gcs": 6
            },
            "idle_since": "2025-03-16T00:41:30.097-04:00",
            "messages_unacknowledged": 0,
            "messages_uncommitted": 0,
            "messages_unconfirmed": 0,
            "name": "<rabbit@sunnyside.1741991552.108078.0> (1)",
            "node": "rabbit@sunnyside",
            "number": 1,
            "pending_raft_commands": 0,
            "prefetch_count": 0,
            "reductions": 1127,
            "reductions_details": {
              "rate": 0.0
            },
            "state": "running",
            "transactional": false,
            "user": "none",
            "user_who_performed_action": "none",
            "vhost": "/"
          }
        }
    "#;

    let link: FederationLink = serde_json::from_str(json).unwrap();
    assert_eq!(link.uri, "amqp://localhost:5672/fed");
    assert_eq!(link.id, "e178dfad");
    assert_eq!(link.typ, FederationType::Queue);
    assert_eq!(link.upstream, "up-1");
    assert_eq!(link.consumer_tag.unwrap(), "hgksdh98s7f98au9u");
}

#[test]
fn test_federation_upstream_to_params_conversion_queue_federation() {
    // Test converting a queue federation upstream from responses to requests
    let json = r#"
        {
          "value": {
            "ack-mode": "on-publish",
            "consumer-tag": "fed.tags.1",
            "queue": "fed.cq.1",
            "trust-user-id": false,
            "reconnect-delay": 10,
            "prefetch-count": 500,
            "uri": "amqp://user:pass@localhost:5673/%2f"
          },
          "vhost": "/",
          "component": "federation-upstream",
          "name": "test-upstream"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let upstream = FederationUpstream::try_from(param).unwrap();

    // Convert to OwnedFederationUpstreamParams
    let owned_params = OwnedFederationUpstreamParams::from(upstream.clone());

    // Verify the conversion
    assert_eq!(owned_params.name, "test-upstream");
    assert_eq!(owned_params.vhost, "/");
    assert_eq!(owned_params.uri, "amqp://user:pass@localhost:5673/%2f");
    assert_eq!(
        owned_params.ack_mode,
        MessageTransferAcknowledgementMode::WhenPublished
    );
    assert_eq!(owned_params.trust_user_id, false);
    assert_eq!(owned_params.reconnect_delay, 10);
    assert_eq!(owned_params.prefetch_count, 500);

    // Check queue federation params
    assert!(owned_params.queue_federation.is_some());
    let queue_fed = owned_params.queue_federation.as_ref().unwrap();
    assert_eq!(queue_fed.queue.as_ref().unwrap(), "fed.cq.1");
    assert_eq!(queue_fed.consumer_tag.as_ref().unwrap(), "fed.tags.1");

    // Exchange federation should be None for queue federation
    assert!(owned_params.exchange_federation.is_none());

    // Convert to FederationUpstreamParams
    let params = FederationUpstreamParams::from(&owned_params);
    assert_eq!(params.name, "test-upstream");
    assert_eq!(params.vhost, "/");
    assert_eq!(params.uri, "amqp://user:pass@localhost:5673/%2f");
    assert!(params.queue_federation.is_some());
    assert!(params.exchange_federation.is_none());
}

#[test]
fn test_federation_upstream_to_params_conversion_exchange_federation() {
    // Test converting an exchange federation upstream from responses to requests
    let json = r#"
        {
          "value": {
            "ack-mode": "on-confirm",
            "exchange": "fed.ex.upstream",
            "expires": 3600000,
            "max-hops": 2,
            "message-ttl": 7200000,
            "queue-type": "quorum",
            "reconnect-delay": 15,
            "trust-user-id": true,
            "resource-cleanup-mode": "never",
            "uri": "amqps://user:pass@remote-host:5671/vhost"
          },
          "vhost": "test-vhost",
          "component": "federation-upstream",
          "name": "exchange-upstream"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let upstream = FederationUpstream::try_from(param).unwrap();

    // Convert to OwnedFederationUpstreamParams
    let owned_params = OwnedFederationUpstreamParams::from(upstream.clone());

    // Verify the conversion
    assert_eq!(owned_params.name, "exchange-upstream");
    assert_eq!(owned_params.vhost, "test-vhost");
    assert_eq!(owned_params.uri, "amqps://user:pass@remote-host:5671/vhost");
    assert_eq!(
        owned_params.ack_mode,
        MessageTransferAcknowledgementMode::WhenConfirmed
    );
    assert_eq!(owned_params.trust_user_id, true);
    assert_eq!(owned_params.reconnect_delay, 15);

    // Check exchange federation params
    assert!(owned_params.exchange_federation.is_some());
    let exchange_fed = owned_params.exchange_federation.as_ref().unwrap();
    assert_eq!(exchange_fed.exchange.as_ref().unwrap(), "fed.ex.upstream");
    assert_eq!(exchange_fed.max_hops.unwrap(), 2);
    assert_eq!(exchange_fed.queue_type, Some(QueueType::Quorum));
    assert_eq!(exchange_fed.ttl.unwrap(), 3600000);
    assert_eq!(exchange_fed.message_ttl.unwrap(), 7200000);
    assert_eq!(
        exchange_fed.resource_cleanup_mode,
        FederationResourceCleanupMode::Never
    );

    // Queue federation should be None for exchange federation
    assert!(owned_params.queue_federation.is_none());

    // Convert to FederationUpstreamParams
    let params = FederationUpstreamParams::from(&owned_params);
    assert_eq!(params.name, "exchange-upstream");
    assert_eq!(params.vhost, "test-vhost");
    assert_eq!(params.uri, "amqps://user:pass@remote-host:5671/vhost");
    assert!(params.exchange_federation.is_some());
    assert!(params.queue_federation.is_none());
}

#[test]
fn test_federation_upstream_roundtrip_conversion() {
    let original_json = r#"
        {
          "value": {
            "ack-mode": "on-confirm",
            "consumer-tag": "my-consumer",
            "queue": "source-queue",
            "trust-user-id": true,
            "reconnect-delay": 5,
            "prefetch-count": 1000,
            "uri": "amqp://guest:guest@localhost:5672/%2f"
          },
          "vhost": "/",
          "component": "federation-upstream",
          "name": "test-roundtrip"
        }
    "#;

    let original_param: RuntimeParameter = serde_json::from_str(original_json).unwrap();
    let upstream = FederationUpstream::try_from(original_param.clone()).unwrap();
    let owned_params = OwnedFederationUpstreamParams::from(upstream);
    let params = FederationUpstreamParams::from(&owned_params);
    let runtime_def = RuntimeParameterDefinition::from(params);

    assert_eq!(runtime_def.name, "test-roundtrip");
    assert_eq!(runtime_def.vhost, "/");
    assert_eq!(runtime_def.component, "federation-upstream");

    let uri_value = runtime_def.value.get("uri").unwrap();
    assert_eq!(
        uri_value.as_str().unwrap(),
        "amqp://guest:guest@localhost:5672/%2f"
    );

    let ack_mode_value = runtime_def.value.get("ack-mode").unwrap();
    assert_eq!(ack_mode_value.as_str().unwrap(), "on-confirm");

    let queue_value = runtime_def.value.get("queue").unwrap();
    assert_eq!(queue_value.as_str().unwrap(), "source-queue");

    let consumer_tag_value = runtime_def.value.get("consumer-tag").unwrap();
    assert_eq!(consumer_tag_value.as_str().unwrap(), "my-consumer");

    let trust_user_id_value = runtime_def.value.get("trust-user-id").unwrap();
    assert_eq!(trust_user_id_value.as_bool().unwrap(), true);

    let reconnect_delay_value = runtime_def.value.get("reconnect-delay").unwrap();
    assert_eq!(reconnect_delay_value.as_u64().unwrap(), 5);

    let prefetch_count_value = runtime_def.value.get("prefetch-count").unwrap();
    assert_eq!(prefetch_count_value.as_u64().unwrap(), 1000);
}

#[test]
fn test_federation_upstream_update_scenario() {
    // Simulate fetching an upstream, modifying it, and updating
    let original_json = r#"
        {
          "value": {
            "ack-mode": "on-publish",
            "queue": "old-queue",
            "trust-user-id": false,
            "reconnect-delay": 5,
            "uri": "amqp://localhost:5672/%2f"
          },
          "vhost": "/",
          "component": "federation-upstream",
          "name": "updatable-upstream"
        }
    "#;

    // 1. Fetch and parse the upstream
    let param: RuntimeParameter = serde_json::from_str(original_json).unwrap();
    let upstream = FederationUpstream::try_from(param).unwrap();

    // 2. Convert to mutable params
    let mut owned_params = OwnedFederationUpstreamParams::from(upstream);

    // 3. Update some fields
    owned_params.uri = "amqps://user:newpass@remote:5671/new-vhost".to_string();
    owned_params.ack_mode = MessageTransferAcknowledgementMode::WhenConfirmed;
    owned_params.trust_user_id = true;
    owned_params.reconnect_delay = 10;

    // Update queue federation params
    if let Some(ref mut queue_fed) = owned_params.queue_federation {
        queue_fed.queue = Some("new-queue".to_string());
    }

    // 4. Convert back to params for API usage
    let updated_params = FederationUpstreamParams::from(&owned_params);

    // 5. Verify the updates
    assert_eq!(updated_params.name, "updatable-upstream");
    assert_eq!(updated_params.vhost, "/");
    assert_eq!(
        updated_params.uri,
        "amqps://user:newpass@remote:5671/new-vhost"
    );
    assert_eq!(
        updated_params.ack_mode,
        MessageTransferAcknowledgementMode::WhenConfirmed
    );
    assert_eq!(updated_params.trust_user_id, true);
    assert_eq!(updated_params.reconnect_delay, 10);

    // Check updated queue federation
    assert!(updated_params.queue_federation.is_some());
    let queue_fed = updated_params.queue_federation.as_ref().unwrap();
    assert_eq!(queue_fed.queue.unwrap().to_owned(), "new-queue".to_owned());

    // 6. Convert to runtime parameter definition for API call
    let runtime_def = RuntimeParameterDefinition::from(updated_params);

    // Verify the final runtime parameter
    assert_eq!(runtime_def.name, "updatable-upstream");
    assert_eq!(runtime_def.vhost, "/");

    let uri_value = runtime_def.value.get("uri").unwrap();
    assert_eq!(
        uri_value.as_str().unwrap(),
        "amqps://user:newpass@remote:5671/new-vhost"
    );

    let queue_value = runtime_def.value.get("queue").unwrap();
    assert_eq!(queue_value.as_str().unwrap(), "new-queue");
}
