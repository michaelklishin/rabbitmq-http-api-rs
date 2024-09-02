use rabbitmq_http_client::blocking::Client;

mod common;
use crate::common::{await_metric_emission, endpoint, PASSWORD, USERNAME};
use rabbitmq_http_client::commons::PolicyTarget;
use rabbitmq_http_client::requests::{ExchangeParams, PolicyParams, QueueParams};
use serde_json::{json, Map, Value};

#[test]
fn test_export_definitions_as_string() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint).with_basic_auth_credentials(USERNAME, PASSWORD);
    let result = rc.export_definitions_as_string();

    assert!(
        result.is_ok(),
        "export_definitions_as_string returned {:?}",
        result
    );
}

#[test]
fn test_export_definitions_as_data() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint).with_basic_auth_credentials(USERNAME, PASSWORD);

    let x_name = "definitions_test.x.fanout";
    let mut x_args_m = Map::<String, Value>::new();
    x_args_m.insert("x-alternate-exchange".to_owned(), json!("amq.fanout"));
    let x_args = Some(x_args_m);
    let xp = ExchangeParams::durable_fanout(x_name, x_args);
    let _ = rc.declare_exchange("/", &xp);

    let qq_pol_name = "definitions_test.policies.qq.length";
    let mut qq_pol_def_m = Map::<String, Value>::new();
    qq_pol_def_m.insert("max-length".to_string(), json!(99));
    let pol_result = rc.declare_policy(&PolicyParams {
        vhost: "/",
        name: qq_pol_name,
        pattern: "definitions.qq.limited",
        apply_to: PolicyTarget::QuorumQueues,
        priority: 1,
        definition: Some(qq_pol_def_m),
    });
    assert!(pol_result.is_ok());

    let q_name = "definitions_test.qq.test_export_definitions_as_data";
    let q_result = rc.declare_queue("/", &QueueParams::new_durable_classic_queue(q_name, None));
    assert!(q_result.is_ok(), "failed to declare queue {}", q_name);

    let _ = rc.bind_queue("/", q_name, x_name, None, None);
    await_metric_emission(1000);

    let result = rc.export_definitions_as_data();
    println!("defs: {:?}", result);

    assert!(
        result.is_ok(),
        "export_definitions_as_data returned {:?}",
        result
    );

    let defs = result.unwrap();

    assert!(
        defs.virtual_hosts.len() > 0,
        "expected more than zero virtual hosts in definitions"
    );
    assert!(
        defs.users.len() > 0,
        "expected more than zero users in definitions"
    );
    assert!(
        defs.exchanges.len() > 0,
        "expected more than zero exchanges in definitions"
    );

    let u_found = defs.users.iter().any(|x| x.name == "rust3");
    assert!(u_found, "expected to find user {} in definitions", "rust3");

    let x_found = defs.exchanges.iter().any(|x| x.name == x_name);
    assert!(
        x_found,
        "expected to find exchange {} in definitions",
        x_name
    );

    let qq_pol_found = defs.policies.iter().any(|p| p.name == qq_pol_name);
    assert!(
        qq_pol_found,
        "expected to find policy {} in definitions",
        qq_pol_name
    );

    let b_found = defs
        .bindings
        .iter()
        .any(|b| b.destination_type == "queue".into() && b.destination == q_name);
    assert!(
        b_found,
        "expected to find a binding for queue {} in definitions",
        q_name
    );

    let _ = rc.delete_exchange("/", &x_name);
    let _ = rc.delete_policy("/", &qq_pol_name);
}

#[test]
fn test_import_definitions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint).with_basic_auth_credentials(USERNAME, PASSWORD);
    let _ = rc.delete_queue("/", "imported_queue");
    let defs = json!({  "queues": [
      {
        "auto_delete": false,
        "durable": true,
        "name": "imported_queue",
        "vhost": "/"
      }
    ]});

    let result = rc.import_definitions(defs);
    assert!(result.is_ok(), "import_definitions returned {:?}", result);

    let result1 = rc.get_queue_info("/", "imported_queue");
    assert!(
        result1.is_ok(),
        "can't get the imported queue: {:?}",
        result1
    );
}
