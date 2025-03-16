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
    responses::{FederationUpstream, RuntimeParameter},
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

    let param: RuntimeParameter = serde_json::from_str(&json).unwrap();
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

    let param: RuntimeParameter = serde_json::from_str(&json).unwrap();
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
