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

use rabbitmq_http_client::{
    commons::MessageTransferAcknowledgementMode,
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
