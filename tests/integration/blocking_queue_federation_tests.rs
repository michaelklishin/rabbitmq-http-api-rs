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
use rabbitmq_http_client::commons::{MessageTransferAcknowledgementMode, QueueType};
use rabbitmq_http_client::requests::{
    ExchangeFederationParams, FederationUpstreamParams, OwnedFederationUpstreamParams,
    QueueFederationParams,
};
use rabbitmq_http_client::{blocking_api::Client, requests::VirtualHostParams};

use crate::test_helpers::{
    PASSWORD, USERNAME, amqp_endpoint_with_vhost, endpoint, rabbitmq_version_is_at_least,
};

#[test]
fn test_blocking_declare_a_federation_upstream_with_queue_federation_parameters() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust.http.api.blocking.test_blocking_declare_a_federation_upstream_with_queue_federation_parameters";
    let name = "upstream.1";
    let q = "test_blocking_declare_a_federation_upstream";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let params = QueueFederationParams::new(q);
    let upstream_params =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &amqp_endpoint, params);

    let result2 = rc.declare_federation_upstream(upstream_params);
    assert!(result2.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_federation_upstream_fetch_and_update_workflow() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust.http.api.blocking.test_federation_upstream_fetch_and_update_workflow";
    let upstream_name = "test-fetch-update-upstream";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let queue_params = QueueFederationParams::new("test-queue");
    let original_upstream_params = FederationUpstreamParams::new_queue_federation_upstream(
        vh,
        upstream_name,
        &amqp_endpoint,
        queue_params,
    );

    let result2 = rc.declare_federation_upstream(original_upstream_params);
    assert!(result2.is_ok());

    let upstreams = rc.list_federation_upstreams();
    assert!(upstreams.is_ok());
    let upstreams_list = upstreams.unwrap();
    let fetched_upstream = upstreams_list
        .iter()
        .find(|u| u.name == upstream_name && u.vhost == vh)
        .expect("Recently reated upstream was not found");

    let owned_params = OwnedFederationUpstreamParams::from(fetched_upstream.clone());

    assert_eq!(owned_params.name, upstream_name);
    assert_eq!(owned_params.vhost, vh);
    assert_eq!(owned_params.uri, amqp_endpoint);
    assert_eq!(
        owned_params.ack_mode,
        MessageTransferAcknowledgementMode::WhenConfirmed
    );

    let mut modified_params = owned_params;
    modified_params.ack_mode = MessageTransferAcknowledgementMode::WhenPublished;
    modified_params.trust_user_id = true;
    modified_params.reconnect_delay = 15;
    modified_params.prefetch_count = 2000;

    if let Some(ref mut queue_fed) = modified_params.queue_federation {
        queue_fed.queue = Some("updated-queue".to_string());
        queue_fed.consumer_tag = Some("updated-consumer-tag".to_string());
    }

    let updated_upstream_params = FederationUpstreamParams::from(&modified_params);
    let result3 = rc.declare_federation_upstream(updated_upstream_params);
    assert!(result3.is_ok());

    let updated_upstreams = rc.list_federation_upstreams();
    assert!(updated_upstreams.is_ok());
    let updated_upstreams_list = updated_upstreams.unwrap();
    let updated_upstream = updated_upstreams_list
        .iter()
        .find(|u| u.name == upstream_name && u.vhost == vh)
        .expect("Should find the updated upstream");
    assert_eq!(
        updated_upstream.ack_mode,
        MessageTransferAcknowledgementMode::WhenPublished
    );
    assert_eq!(updated_upstream.trust_user_id, Some(true));
    assert_eq!(updated_upstream.reconnect_delay, Some(15));
    assert_eq!(updated_upstream.queue.as_ref().unwrap(), "updated-queue");
    assert_eq!(
        updated_upstream.consumer_tag.as_ref().unwrap(),
        "updated-consumer-tag"
    );

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_exchange_federation_upstream_fetch_and_update_workflow() {
    // Exchange federation queue-type parameter requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust.http.api.blocking.test_exchange_federation_upstream_fetch_and_update_workflow";
    let upstream_name = "test-exchange-fetch-update-upstream";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let exchange_params = ExchangeFederationParams::new(QueueType::Quorum);
    let original_upstream_params = FederationUpstreamParams::new_exchange_federation_upstream(
        vh,
        upstream_name,
        &amqp_endpoint,
        exchange_params,
    );

    let result2 = rc.declare_federation_upstream(original_upstream_params);
    assert!(result2.is_ok());

    let upstreams = rc.list_federation_upstreams();
    assert!(upstreams.is_ok());
    let upstreams_list = upstreams.unwrap();
    let fetched_upstream = upstreams_list
        .iter()
        .find(|u| u.name == upstream_name && u.vhost == vh)
        .expect("Recently reated upstream was not found");

    let owned_params = OwnedFederationUpstreamParams::from(fetched_upstream.clone());

    assert_eq!(owned_params.name, upstream_name);
    assert_eq!(owned_params.vhost, vh);
    assert_eq!(owned_params.uri, amqp_endpoint);

    let mut modified_params = owned_params;
    modified_params.ack_mode = MessageTransferAcknowledgementMode::Immediate;
    modified_params.trust_user_id = false;
    modified_params.reconnect_delay = 20;

    if let Some(ref mut exchange_fed) = modified_params.exchange_federation {
        exchange_fed.exchange = Some("updated-exchange".to_string());
        exchange_fed.max_hops = Some(3);
        exchange_fed.queue_type = Some(QueueType::Classic);
    }

    let updated_upstream_params = FederationUpstreamParams::from(&modified_params);
    let result3 = rc.declare_federation_upstream(updated_upstream_params);
    assert!(result3.is_ok());

    let updated_upstreams = rc.list_federation_upstreams();
    assert!(updated_upstreams.is_ok());
    let updated_upstreams_list = updated_upstreams.unwrap();
    let updated_upstream = updated_upstreams_list
        .iter()
        .find(|u| u.name == upstream_name && u.vhost == vh)
        .expect("Should find the updated upstream");
    assert_eq!(
        updated_upstream.ack_mode,
        MessageTransferAcknowledgementMode::Immediate
    );
    assert_eq!(updated_upstream.trust_user_id, Some(false));
    assert_eq!(updated_upstream.reconnect_delay, Some(20));
    assert_eq!(
        updated_upstream.exchange.as_ref().unwrap(),
        "updated-exchange"
    );
    assert_eq!(updated_upstream.max_hops.unwrap(), 3);
    assert_eq!(
        updated_upstream.queue_type.clone().unwrap(),
        QueueType::Classic
    );

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_delete_federation_upstream() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust.http.api.blocking.test_delete_federation_upstream";
    let upstream_name = "test-delete-upstream";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let queue_params = QueueFederationParams::new("test-queue");
    let upstream_params = FederationUpstreamParams::new_queue_federation_upstream(
        vh,
        upstream_name,
        &amqp_endpoint,
        queue_params,
    );

    let result2 = rc.declare_federation_upstream(upstream_params);
    assert!(result2.is_ok());

    let result3 = rc.delete_federation_upstream(vh, upstream_name, false);
    assert!(result3.is_ok());

    // idempotent delete should succeed
    let result4 = rc.delete_federation_upstream(vh, upstream_name, true);
    assert!(result4.is_ok());

    // non-idempotent delete should fail
    let result5 = rc.delete_federation_upstream(vh, upstream_name, false);
    assert!(result5.is_err());

    let _ = rc.delete_vhost(vh_params.name, false);
}
