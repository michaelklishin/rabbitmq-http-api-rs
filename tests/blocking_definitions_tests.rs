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
use rabbitmq_http_client::blocking_api::Client;

mod test_helpers;
use crate::test_helpers::{
    PASSWORD, USERNAME, await_metric_emission, await_queue_metric_emission, endpoint,
};
use rabbitmq_http_client::commons::PolicyTarget;
use rabbitmq_http_client::requests::{
    ExchangeParams, PolicyParams, QueueParams, VirtualHostParams,
};
use serde_json::{Map, Value, json};

#[test]
fn test_blocking_export_definitions_as_string() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result = rc.export_cluster_wide_definitions_as_string();
    assert!(
        result.is_ok(),
        "export_definitions_as_string returned {result:?}"
    );
}

#[test]
fn test_blocking_export_cluster_wide_definitions_as_data() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust/http/api/blocking/definitions";
    rc.delete_vhost(vh, true).unwrap();

    let vh_params = VirtualHostParams::named(vh);
    rc.create_vhost(&vh_params).unwrap();

    let x_name = "definitions_test.x.fanout";
    let mut x_args_m = Map::<String, Value>::new();
    x_args_m.insert("x-alternate-exchange".to_owned(), json!("amq.fanout"));
    let x_args = Some(x_args_m);
    let xp = ExchangeParams::durable_fanout(x_name, x_args);
    rc.declare_exchange(vh_params.name, &xp).unwrap();

    let qq_pol_name = "definitions_test.policies.qq.length";
    let mut qq_pol_def_m = Map::<String, Value>::new();
    qq_pol_def_m.insert("max-length".to_string(), json!(99));
    let pol_result = rc.declare_policy(&PolicyParams {
        vhost: vh_params.name,
        name: qq_pol_name,
        pattern: "definitions.qq.limited",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 1,
        definition: qq_pol_def_m,
    });
    assert!(pol_result.is_ok());

    let q_name = "definitions_test.qq.test_export_definitions_as_data";
    let q_result = rc.declare_queue(
        vh_params.name,
        &QueueParams::new_durable_classic_queue(q_name, None),
    );
    assert!(q_result.is_ok(), "failed to declare queue {q_name}");

    let _ = rc.bind_queue(vh_params.name, q_name, x_name, None, None);
    await_metric_emission(1000);

    let result = rc.export_cluster_wide_definitions_as_data();
    assert!(
        result.is_ok(),
        "export_definitions_as_data returned {result:?}"
    );

    let defs = result.unwrap();

    assert!(
        !defs.virtual_hosts.is_empty(),
        "expected more than zero virtual hosts in definitions"
    );
    assert!(
        !defs.users.is_empty(),
        "expected more than zero users in definitions"
    );
    assert!(
        !defs.exchanges.is_empty(),
        "expected more than zero exchanges in definitions"
    );

    let x_found = defs.exchanges.iter().any(|x| x.name == x_name);
    assert!(x_found, "expected to find exchange {x_name} in definitions");

    let qq_pol_found = defs.policies.iter().any(|p| p.name == qq_pol_name);
    assert!(
        qq_pol_found,
        "expected to find policy {qq_pol_name} in definitions"
    );

    let b_found = defs
        .bindings
        .iter()
        .any(|b| b.destination_type == "queue".into() && b.destination == q_name);
    assert!(
        b_found,
        "expected to find a binding for queue {q_name} in definitions"
    );

    rc.delete_exchange(vh, x_name, false).unwrap();
    rc.delete_policy(vh, qq_pol_name).unwrap();
    rc.delete_vhost(vh, true).unwrap();
}

#[test]
fn test_blocking_export_vhost_definitions_as_data() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust/http/api/blocking/vhost.definitions";
    rc.delete_vhost(vh, true).unwrap();

    let vh_params = VirtualHostParams::named(vh);
    rc.create_vhost(&vh_params).unwrap();

    let x_name = "vhost.definitions_test.x.fanout";
    let mut x_args_m = Map::<String, Value>::new();
    x_args_m.insert("x-alternate-exchange".to_owned(), json!("amq.fanout"));
    let x_args = Some(x_args_m);
    let xp = ExchangeParams::durable_fanout(x_name, x_args);
    rc.declare_exchange(vh_params.name, &xp).unwrap();

    let qq_pol_name = "vhost.definitions_test.policies.qq.length";
    let mut qq_pol_def_m = Map::<String, Value>::new();
    qq_pol_def_m.insert("max-length".to_string(), json!(99));
    let pol_result = rc.declare_policy(&PolicyParams {
        vhost: vh_params.name,
        name: qq_pol_name,
        pattern: "vhost.definitions.qq.limited",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 1,
        definition: qq_pol_def_m,
    });
    assert!(pol_result.is_ok());

    let q_name = "vhost.definitions_test.qq.test_export_definitions_as_data";
    let q_result = rc.declare_queue(
        vh_params.name,
        &QueueParams::new_durable_classic_queue(q_name, None),
    );
    assert!(q_result.is_ok(), "failed to declare queue {q_name}");

    let _ = rc.bind_queue(vh_params.name, q_name, x_name, None, None);
    await_metric_emission(1000);

    let result = rc.export_vhost_definitions_as_data(vh);
    assert!(
        result.is_ok(),
        "export_definitions_as_data returned {result:?}"
    );

    let defs = result.unwrap();

    assert!(
        !defs.exchanges.is_empty(),
        "expected more than zero exchanges in definitions"
    );

    let x_found = defs.exchanges.iter().any(|x| x.name == x_name);
    assert!(x_found, "expected to find exchange {x_name} in definitions");

    let qq_pol_found = defs.policies.iter().any(|p| p.name == qq_pol_name);
    assert!(
        qq_pol_found,
        "expected to find policy {qq_pol_name} in definitions"
    );

    let b_found = defs
        .bindings
        .iter()
        .any(|b| b.destination_type == "queue".into() && b.destination == q_name);
    assert!(
        b_found,
        "expected to find a binding for queue {q_name} in definitions"
    );

    rc.delete_exchange(vh, x_name, false).unwrap();
    rc.delete_policy(vh, qq_pol_name).unwrap();
    rc.delete_vhost(vh, true).unwrap();
}

#[test]
fn test_blocking_import_cluster_definitions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let queue_name = "test_blocking_import_cluster_definitions";

    let _ = rc.delete_queue("/", queue_name, false);
    let defs = json!({  "queues": [
      {
        "auto_delete": false,
        "durable": true,
        "name": queue_name,
        "vhost": "/"
      }
    ]});

    let result = rc.import_cluster_wide_definitions(defs);
    assert!(
        result.is_ok(),
        "import_cluster_wide_definitions returned {result:?}"
    );

    let result1 = rc.get_queue_info("/", queue_name);
    assert!(
        result1.is_ok(),
        "can't get the imported import_cluster_wide_definitions: {result1:?}"
    );

    rc.delete_queue("/", queue_name, true).unwrap();
}

#[test]
fn test_blocking_import_vhost_definitions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust/http/api/blocking/vhost.definitions.import";
    rc.delete_vhost(vh, true).unwrap();

    let vh_params = VirtualHostParams::named(vh);
    rc.create_vhost(&vh_params).unwrap();

    let q = "imported_queue";
    let defs = json!({  "queues": [
      {
        "auto_delete": false,
        "durable": true,
        "name": q,
      }
    ]});

    let result = rc.import_vhost_definitions(vh, defs);
    assert!(
        result.is_ok(),
        "import_vhost_definitions returned {result:?}"
    );

    await_queue_metric_emission();

    let result1 = rc.get_queue_info(vh, q);
    assert!(result1.is_ok(), "can't get the imported queue: {result1:?}");

    rc.delete_vhost(vh, true).unwrap();
}
