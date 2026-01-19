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
use rabbitmq_http_client::requests::QueueParams;
use rabbitmq_http_client::requests::shovels::{
    Amqp10ShovelDestinationParams, Amqp10ShovelParams, Amqp10ShovelSourceParams,
    Amqp091ShovelDestinationParams, Amqp091ShovelParams, Amqp091ShovelSourceParams,
};
use rabbitmq_http_client::{blocking_api::Client, requests::VirtualHostParams};

mod test_helpers;
use crate::test_helpers::{
    PASSWORD, USERNAME, amqp_endpoint_with_vhost, amqp10_endpoint_with_vhost,
    await_metric_emission, endpoint, rabbitmq_version_is_at_least,
};

#[test]
fn test_blocking_declare_a_dynamic_amqp091_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Dynamic shovel management API requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let vh = "rust.http.api.blocking.test_declare_a_dynamic_amqp091_shovel";
    let sh = "test_declare_a_dynamic_amqp091_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let src_q = format!("{sh}.src.q");
    let dest_q = format!("{sh}.dest.q");

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: vh,
        name: sh,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result2 = rc.declare_amqp091_shovel(shovel_params);
    assert!(result2.is_ok());

    await_metric_emission(300);
    let result3 = rc.get_queue_info(vh, &src_q);
    assert!(result3.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_list_all_shovels() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Dynamic shovel management API requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let vh1 = "rust.http.api.blocking.test_blocking_list_all_shovels.1";
    let vh2 = "rust.http.api.blocking.test_blocking_list_all_shovels.2";
    let sh1 = "test_blocking_list_all_shovels.sh-1";
    let sh2 = "test_blocking_list_all_shovels.sh-2";

    let vh_params1 = VirtualHostParams::named(vh1);
    let result1 = rc.create_vhost(&vh_params1);
    assert!(result1.is_ok());

    let vh_params2 = VirtualHostParams::named(vh2);
    let result2 = rc.create_vhost(&vh_params2);
    assert!(result2.is_ok());

    let src_q1 = format!("{sh1}.src.q");
    let dest_q1 = format!("{sh1}.dest.q");

    let src_q2 = format!("{sh2}.src.q");
    let dest_q2 = format!("{sh2}.dest.q");

    let amqp_endpoint1 = amqp_endpoint_with_vhost(vh1);
    let shovel_params1 = Amqp091ShovelParams {
        vhost: vh1,
        name: sh1,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint1, &src_q1),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint1, &dest_q1),
    };
    let result3 = rc.declare_amqp091_shovel(shovel_params1);
    assert!(result3.is_ok());

    let amqp_endpoint2 = amqp_endpoint_with_vhost(vh2);
    let shovel_params2 = Amqp091ShovelParams {
        vhost: vh2,
        name: sh2,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint2, &src_q2),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint2, &dest_q2),
    };
    let result4 = rc.declare_amqp091_shovel(shovel_params2);
    assert!(result4.is_ok());

    await_metric_emission(400);
    let result5 = rc.list_shovels();
    assert!(result5.is_ok());

    let shovels = result5.unwrap();
    assert!(shovels.iter().find(|s| s.name == sh1).is_some());
    assert!(shovels.iter().find(|s| s.name == sh2).is_some());
    assert!(
        shovels
            .iter()
            .find(|s| s.name == "a-non-existent-shovel")
            .is_none()
    );

    let _ = rc.delete_vhost(vh_params1.name, false);
    let _ = rc.delete_vhost(vh_params2.name, false);
}

#[test]
fn test_blocking_list_all_shovels_in_a_virtual_host() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Dynamic shovel management API requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let vh1 = "rust.http.api.blocking.test_blocking_list_all_shovels_in_a_virtual_host.1";
    let vh2 = "rust.http.api.blocking.test_blocking_list_all_shovels_in_a_virtual_host.2";
    let sh1 = "test_blocking_list_all_shovels_in_a_virtual_host";

    let vh_params1 = VirtualHostParams::named(vh1);
    let result1 = rc.create_vhost(&vh_params1);
    assert!(result1.is_ok());

    let vh_params2 = VirtualHostParams::named(vh2);
    let result2 = rc.create_vhost(&vh_params2);
    assert!(result2.is_ok());

    let src_q = format!("{sh1}.src.q");
    let dest_q = format!("{sh1}.dest.q");

    let amqp_endpoint = amqp_endpoint_with_vhost(vh1);
    let shovel_params = Amqp091ShovelParams {
        vhost: vh1,
        name: sh1,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result3 = rc.declare_amqp091_shovel(shovel_params);
    assert!(result3.is_ok());

    await_metric_emission(300);
    let result4 = rc.list_shovels_in(vh1);
    assert!(result4.is_ok());

    let shovels = result4.unwrap();
    assert!(shovels.iter().find(|s| s.name == sh1).is_some());

    let result5 = rc.list_shovels_in(vh2);
    assert!(result5.is_ok());
    assert!(result5.unwrap().is_empty());

    let _ = rc.delete_vhost(vh_params1.name, false);
    let _ = rc.delete_vhost(vh_params2.name, false);
}

#[test]
fn test_blocking_declare_a_dynamic_amqp10_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Dynamic shovel management API requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let vh = "rust.http.api.blocking.test_blocking_declare_a_dynamic_amqp10_shovel";
    let sh = "test_async_declare_a_dynamic_amqp10_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    // note: 4.1.0 will use a different addressing scheme,
    //       see https://www.rabbitmq.com/docs/next/amqp#address-v2
    let src_queue = format!("{sh}.src.q");
    let src_address = format!("/queue/{sh}.src.q");
    let dest_queue = format!("{sh}.dest.q");
    let dest_address = format!("/queue/{sh}.dest.q");

    let src_params = QueueParams::new_durable_classic_queue(&src_queue, None);
    let result2 = rc.declare_queue(vh, &src_params);
    assert!(result2.is_ok());

    let dest_params = QueueParams::new_durable_classic_queue(&dest_queue, None);
    let result3 = rc.declare_queue(vh, &dest_params);
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
    let result4 = rc.declare_amqp10_shovel(shovel_params);
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_declare_a_dynamic_amqp091_shovel_with_predeclared_source_topology() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Dynamic shovel management API requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let vh = "rust.http.api.blocking.test_declare_a_dynamic_amqp091_shovel_with_predeclared_source_topology";
    let sh = "test_declare_a_dynamic_amqp091_shovel_with_predeclared_source_topology";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let src_q = format!("{sh}.src.q");
    let dest_q = format!("{sh}.dest.q");

    let q_params = QueueParams::new_durable_classic_queue(&src_q, None);
    let result2 = rc.declare_queue(vh, &q_params);
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
    let result3 = rc.declare_amqp091_shovel(shovel_params);
    assert!(result3.is_ok());

    await_metric_emission(300);
    let result4 = rc.get_queue_info(vh, &src_q);
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_declare_a_dynamic_amqp091_shovel_with_predeclared_destination_topology() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Dynamic shovel management API requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let vh = "rust.http.api.blocking.test_declare_a_dynamic_amqp091_shovel_with_predeclared_destination_topology";
    let sh = "test_declare_a_dynamic_amqp091_shovel_with_predeclared_destination_topology";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let src_q = format!("{sh}.src.q");
    let dest_q = format!("{sh}.dest.q");

    let q_params = QueueParams::new_durable_classic_queue(&dest_q, None);
    let result2 = rc.declare_queue(vh, &q_params);
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
    let result3 = rc.declare_amqp091_shovel(shovel_params);
    assert!(result3.is_ok());

    await_metric_emission(300);
    let result4 = rc.get_queue_info(vh, &dest_q);
    assert!(result4.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false);
}

#[test]
fn test_blocking_delete_a_dynamic_amqp091_shovel() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    // Dynamic shovel management API requires RabbitMQ 4.0+
    if !rabbitmq_version_is_at_least(4, 0, 0) {
        return;
    }

    let vh = "rust.http.api.blocking.test_delete_a_dynamic_amqp091_shovel";
    let sh = "test_delete_a_dynamic_amqp091_shovel";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let src_q = format!("{sh}.src.q");
    let dest_q = format!("{sh}.dest.q");

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let shovel_params = Amqp091ShovelParams {
        vhost: vh,
        name: sh,
        acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
        reconnect_delay: Some(5),
        source: Amqp091ShovelSourceParams::queue_source(&amqp_endpoint, &src_q),
        destination: Amqp091ShovelDestinationParams::queue_destination(&amqp_endpoint, &dest_q),
    };
    let result2 = rc.declare_amqp091_shovel(shovel_params);
    assert!(result2.is_ok());

    await_metric_emission(200);
    let result3 = rc.delete_shovel(vh, sh, false);
    assert!(result3.is_ok());

    let result4 = rc.delete_shovel(vh, sh, true);
    assert!(result4.is_ok());

    let result5 = rc.delete_shovel(vh, sh, false);
    assert!(result5.is_err());

    let _ = rc.delete_vhost(vh_params.name, false);
}
