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
use rabbitmq_http_client::commons::MessageTransferAcknowledgementMode;
use rabbitmq_http_client::requests::{
    Amqp091ShovelDestinationParams, Amqp091ShovelParams, Amqp091ShovelSourceParams,
    Amqp10ShovelDestinationParams, Amqp10ShovelParams, Amqp10ShovelSourceParams, QueueParams,
};
use rabbitmq_http_client::{api::Client, requests::VirtualHostParams};

mod test_helpers;
use crate::test_helpers::{
    amqp10_endpoint_with_vhost, amqp_endpoint_with_vhost, async_testing_against_3_13_x,
    await_metric_emission, endpoint, PASSWORD, USERNAME,
};

#[tokio::test]
async fn test_async_declare_a_dynamic_amqp091_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    if async_testing_against_3_13_x().await {
        return;
    }

    let vh = "rust.http.api.async.test_declare_a_dynamic_amqp091_shovel";
    let sh = "test_declare_a_dynamic_amqp091_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let src_q = format!("{0}.src.q", sh);
    let dest_q = format!("{0}.dest.q", sh);

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: vh,
        name: sh,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result2 = rc.declare_amqp091_shovel(shovel_params).await;
    assert!(result2.is_ok());

    await_metric_emission(300);
    let result3 = rc.get_queue_info(vh, &src_q).await;
    assert!(result3.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

#[tokio::test]
async fn test_async_declare_a_dynamic_amqp10_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    if async_testing_against_3_13_x().await {
        return;
    }

    let vh = "rust.http.api.async.test_async_declare_a_dynamic_amqp10_shovel";
    let sh = "test_async_declare_a_dynamic_amqp10_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    // note: 4.1.0 will use a different addressing scheme,
    //       see https://www.rabbitmq.com/docs/next/amqp#address-v2
    let src_queue = format!("{}.src.q", sh);
    let src_address = format!("/queue/{}.src.q", sh);
    let dest_queue = format!("{}.dest.q", sh);
    let dest_address = format!("/queue/{}.dest.q", sh);

    let src_params = QueueParams::new_durable_classic_queue(&src_queue, None);
    let result2 = rc.declare_queue(vh, &src_params).await;
    assert!(result2.is_ok());

    let dest_params = QueueParams::new_durable_classic_queue(&dest_queue, None);
    let result3 = rc.declare_queue(vh, &dest_params).await;
    assert!(result3.is_ok());

    let amqp_endpoint = amqp10_endpoint_with_vhost(vh);
    let shovel_params = Amqp10ShovelParams {
        vhost: vh,
        name: sh,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp10ShovelSourceParams::new(&amqp_endpoint, &src_address),
        destination: Amqp10ShovelDestinationParams::new(&amqp_endpoint, &dest_address),
    };
    let result4 = rc.declare_amqp10_shovel(shovel_params).await;
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

#[tokio::test]
async fn test_async_declare_a_dynamic_amqp091_shovel_with_predeclared_source_topology() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    if async_testing_against_3_13_x().await {
        return;
    }

    let vh = "rust.http.api.blocking.test_declare_a_dynamic_amqp091_shovel_with_predeclared_source_topology";
    let sh = "test_declare_a_dynamic_amqp091_shovel_with_predeclared_source_topology";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let src_q = format!("{0}.src.q", sh);
    let dest_q = format!("{0}.dest.q", sh);

    let q_params = QueueParams::new_durable_classic_queue(&src_q, None);
    let result2 = rc.declare_queue(vh, &q_params).await;
    assert!(result2.is_ok());

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: vh,
        name: sh,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::predeclared_queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result3 = rc.declare_amqp091_shovel(shovel_params).await;
    assert!(result3.is_ok());

    await_metric_emission(300);
    let result4 = rc.get_queue_info(vh, &src_q).await;
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

#[tokio::test]
async fn test_async_declare_a_dynamic_amqp091_shovel_with_predeclared_destination_topology() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    if async_testing_against_3_13_x().await {
        return;
    }

    let vh = "rust.http.api.blocking.test_declare_a_dynamic_amqp091_shovel_with_predeclared_destination_topology";
    let sh = "test_declare_a_dynamic_amqp091_shovel_with_predeclared_destination_topology";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let src_q = format!("{0}.src.q", sh);
    let dest_q = format!("{0}.dest.q", sh);

    let q_params = QueueParams::new_durable_classic_queue(&dest_q, None);
    let result2 = rc.declare_queue(vh, &q_params).await;
    assert!(result2.is_ok());

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: vh,
        name: sh,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::predeclared_queue_destination(
            &amqp_endpoint,
            &dest_q,
        ),
    };
    let result3 = rc.declare_amqp091_shovel(shovel_params).await;
    assert!(result3.is_ok());

    await_metric_emission(300);
    let result4 = rc.get_queue_info(vh, &dest_q).await;
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

#[tokio::test]
async fn test_async_delete_a_dynamic_amqp091_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    if async_testing_against_3_13_x().await {
        return;
    }

    let vh = "rust.http.api.async.test_delete_a_dynamic_amqp091_shovel";
    let sh = "test_delete_a_dynamic_amqp091_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let src_q = format!("{0}.src.q", sh);
    let dest_q = format!("{0}.dest.q", sh);

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: vh,
        name: sh,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result2 = rc.declare_amqp091_shovel(shovel_params).await;
    assert!(result2.is_ok());

    await_metric_emission(200);
    let result3 = rc.delete_shovel(vh, sh, false).await;
    assert!(result3.is_ok());

    let result4 = rc.delete_shovel(vh, sh, true).await;
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false).await;
}
