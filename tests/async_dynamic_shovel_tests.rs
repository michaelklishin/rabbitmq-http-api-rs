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
use rabbitmq_http_client::commons::ShovelAcknowledgementMode;
use rabbitmq_http_client::requests::{
    Amqp091ShovelDestinationParams, Amqp091ShovelParams, Amqp091ShovelSourceParams,
};
use rabbitmq_http_client::{api::Client, requests::VirtualHostParams};

mod test_helpers;
use crate::test_helpers::{
    amqp_endpoint_with_vhost, await_metric_emission, endpoint, PASSWORD, USERNAME,
};

#[tokio::test]
async fn test_declare_a_dynamic_amqp091_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust.http.api.async.test_declare_a_dynamic_amqp091_shovel";
    let sh = "test_declare_a_dynamic_amqp091_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let src_q = format!("{0}.src.q", sh);
    let dest_q = format!("{0}.dest.q", sh);

    let amqp_endpoint = amqp_endpoint_with_vhost(&vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: &vh,
        name: sh,
        acknowledgement_mode: ShovelAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result2 = rc.declare_amqp091_shovel(shovel_params).await;
    assert!(result2.is_ok());

    await_metric_emission(300);
    let result3 = rc.get_queue_info(&vh, &src_q).await;
    assert!(result3.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false).await;
}

#[tokio::test]
async fn test_delete_a_dynamic_amqp091_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust.http.api.async.test_delete_a_dynamic_amqp091_shovel";
    let sh = "test_delete_a_dynamic_amqp091_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let src_q = format!("{0}.src.q", sh);
    let dest_q = format!("{0}.dest.q", sh);

    let amqp_endpoint = amqp_endpoint_with_vhost(&vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: &vh,
        name: sh,
        acknowledgement_mode: ShovelAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result2 = rc.declare_amqp091_shovel(shovel_params).await;
    assert!(result2.is_ok());

    await_metric_emission(200);
    let result3 = rc.delete_shovel(&vh, &sh, false).await;
    assert!(result3.is_ok());

    let result4 = rc.delete_shovel(&vh, &sh, true).await;
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false);
}
