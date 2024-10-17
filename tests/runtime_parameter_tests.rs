use rabbitmq_http_client::requests::{RuntimeParameterDefinition, RuntimeParameterValue};
use rabbitmq_http_client::responses::RuntimeParameter;
use rabbitmq_http_client::{blocking::Client, requests::VirtualHostParams};
use serde_json::{json, Map, Value};

mod common;
use crate::common::{await_metric_emission, endpoint, PASSWORD, USERNAME};

#[test]
fn test_upsert_runtime_parameter() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_upsert_runtime_parameter");
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let mut val = max_connections_limit(9988);
    let rpf = example_runtime_parameter_definition(vh_params.name, &mut val);
    let result2 = rc.upsert_runtime_parameter(&rpf);
    assert!(result2.is_ok());

    let result3 = rc.get_runtime_parameter(&rpf.component, &rpf.vhost, &rpf.name);
    assert!(result3.is_ok());
    assert_eq!(
        9988,
        result3
            .unwrap()
            .value
            .get("max-connections")
            .unwrap()
            .as_u64()
            .unwrap()
    );

    let _ = rc.clear_runtime_parameter(&rpf.component, &rpf.vhost, &rpf.name);
    let _ = rc.delete_vhost(vh_params.name);
}

#[test]
fn test_clear_runtime_parameter() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_clear_runtime_parameter");
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let mut val = max_queue_limit(4444);
    let rp = example_runtime_parameter_definition(vh_params.name, &mut val);
    let result2 = rc.upsert_runtime_parameter(&rp);
    assert!(result2.is_ok());
    await_metric_emission(1000);

    let result3 = rc.clear_runtime_parameter("vhost-limits", vh_params.name, "limits");
    assert!(result3.is_ok());

    let result4 = rc.list_runtime_parameters();
    assert!(
        result4.is_ok(),
        "list_runtime_parameters returned {:?}",
        result4
    );
    let vec = result4.unwrap();
    assert!(!vec
        .iter()
        .any(|p| p.component == "vhost-limits" && p.vhost == *vh_params.name));

    let _ = rc.delete_vhost(vh_params.name);
}

#[test]
fn test_deserialize_sequence_value() {
    let json = r#"
      {
        "name": "my_param",
        "vhost": "test",
        "component": "limits",
        "value": []
      }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();

    assert_eq!(param.name, "my_param");
    assert_eq!(param.vhost, "test");
    assert_eq!(param.component, "limits");

    let expected_value: RuntimeParameterValue = serde_json::Map::new();

    assert_eq!(param.value.0, expected_value);
}

//
// Implementation
//

fn max_connections_limit(n: usize) -> Map<String, Value> {
    let mut val = Map::<String, Value>::new();
    val.insert(String::from("max-connections"), json!(n));
    val
}

fn max_queue_limit(n: usize) -> Map<String, Value> {
    let mut val = Map::<String, Value>::new();
    val.insert(String::from("max-queues"), json!(n));
    val
}

fn example_runtime_parameter_definition(
    vhost: &str,
    val: &mut Map<String, Value>,
) -> RuntimeParameterDefinition {
    RuntimeParameterDefinition {
        vhost: vhost.to_owned(),
        name: "limits".to_owned(),
        component: "vhost-limits".to_owned(),
        value: val.clone(),
    }
}
