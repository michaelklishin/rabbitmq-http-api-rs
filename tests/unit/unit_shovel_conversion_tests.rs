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

use std::error::Error;

use rabbitmq_http_client::{
    commons::{MessageTransferAcknowledgementMode, TlsPeerVerificationMode},
    requests::{RuntimeParameterDefinition, shovels::OwnedShovelParams},
    responses::{MessagingProtocol, RuntimeParameter, Shovel, ShovelState, ShovelType},
    uris::UriBuilder,
};

#[test]
fn test_unit_deserialize_shovel_runtime_parameter_amqp091() {
    let json = r#"
        {
          "value": {
            "src-protocol": "amqp091",
            "dest-protocol": "amqp091",
            "src-uri": "amqp://localhost:5672/%2f",
            "dest-uri": "amqp://localhost:5673/%2f",
            "src-queue": "source-queue",
            "dest-queue": "dest-queue",
            "ack-mode": "on-confirm",
            "reconnect-delay": 5,
            "src-predeclared": false,
            "dest-predeclared": false
          },
          "vhost": "/",
          "component": "shovel",
          "name": "test-shovel-091"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let shovel_params = OwnedShovelParams::try_from(param.clone()).unwrap();

    assert_eq!(param.name, shovel_params.name);
    assert_eq!(param.vhost.to_string(), shovel_params.vhost);

    assert_eq!("amqp091", shovel_params.source_protocol);
    assert_eq!("amqp091", shovel_params.destination_protocol);
    assert_eq!("amqp://localhost:5672/%2f", shovel_params.source_uri);
    assert_eq!("amqp://localhost:5673/%2f", shovel_params.destination_uri);
    assert_eq!("source-queue", shovel_params.source_queue.unwrap());
    assert_eq!("dest-queue", shovel_params.destination_queue.unwrap());
    assert_eq!(
        MessageTransferAcknowledgementMode::WhenConfirmed,
        shovel_params.acknowledgement_mode
    );
    assert_eq!(5, shovel_params.reconnect_delay.unwrap());
    assert_eq!(false, shovel_params.source_predeclared.unwrap());
    assert_eq!(false, shovel_params.destination_predeclared.unwrap());
}

#[test]
fn test_unit_deserialize_shovel_runtime_parameter_amqp10() {
    let json = r#"
        {
          "value": {
            "src-protocol": "amqp10",
            "dest-protocol": "amqp10",
            "src-uri": "amqp://user:pass@localhost:5672/vhost",
            "dest-uri": "amqp://user:pass@remote:5673/vhost",
            "src-address": "source.address",
            "dest-address": "dest.address",
            "ack-mode": "on-publish",
            "reconnect-delay": 10
          },
          "vhost": "test-vhost",
          "component": "shovel",
          "name": "test-shovel-10"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let shovel_params = OwnedShovelParams::try_from(param.clone()).unwrap();

    assert_eq!(param.name, shovel_params.name);
    assert_eq!(param.vhost.to_string(), shovel_params.vhost);

    assert_eq!("amqp10", shovel_params.source_protocol);
    assert_eq!("amqp10", shovel_params.destination_protocol);
    assert_eq!(
        "amqp://user:pass@localhost:5672/vhost",
        shovel_params.source_uri
    );
    assert_eq!(
        "amqp://user:pass@remote:5673/vhost",
        shovel_params.destination_uri
    );
    assert_eq!("source.address", shovel_params.source_address.unwrap());
    assert_eq!("dest.address", shovel_params.destination_address.unwrap());
    assert_eq!(
        MessageTransferAcknowledgementMode::WhenPublished,
        shovel_params.acknowledgement_mode
    );
    assert_eq!(10, shovel_params.reconnect_delay.unwrap());
}

#[test]
fn test_unit_deserialize_shovel_runtime_parameter_exchange_to_exchange() {
    let json = r#"
        {
          "value": {
            "src-protocol": "amqp091",
            "dest-protocol": "amqp091",
            "src-uri": "amqp://localhost:5672/%2f",
            "dest-uri": "amqp://localhost:5673/%2f",
            "src-exchange": "source-exchange",
            "src-exchange-key": "routing.key",
            "dest-exchange": "dest-exchange",
            "dest-exchange-key": "dest.key",
            "ack-mode": "no-ack"
          },
          "vhost": "/",
          "component": "shovel",
          "name": "exchange-shovel"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let shovel_params = OwnedShovelParams::try_from(param.clone()).unwrap();

    assert_eq!("source-exchange", shovel_params.source_exchange.unwrap());
    assert_eq!(
        "routing.key",
        shovel_params.source_exchange_routing_key.unwrap()
    );
    assert_eq!("dest-exchange", shovel_params.destination_exchange.unwrap());
    assert_eq!(
        "dest.key",
        shovel_params.destination_exchange_routing_key.unwrap()
    );
    assert_eq!(
        MessageTransferAcknowledgementMode::Immediate,
        shovel_params.acknowledgement_mode
    );
}

#[test]
fn test_shovel_response_to_params_conversion() {
    let shovel = Shovel {
        node: "rabbit@node1".to_string(),
        name: "test-shovel".to_string(),
        vhost: Some("/".to_string()),
        typ: ShovelType::Dynamic,
        state: ShovelState::Running,
        source_uri: Some("amqp://localhost:5672/%2f".to_string()),
        destination_uri: Some("amqp://remote:5673/%2f".to_string()),
        source: Some("source-queue".to_string()),
        destination: Some("dest-queue".to_string()),
        source_address: None,
        destination_address: None,
        source_protocol: Some(MessagingProtocol::Amqp091),
        destination_protocol: Some(MessagingProtocol::Amqp091),
    };

    let owned_params = OwnedShovelParams::from(shovel);

    assert_eq!(owned_params.name, "test-shovel");
    assert_eq!(owned_params.vhost, "/");
    assert_eq!(owned_params.source_protocol, "AMQP 0-9-1");
    assert_eq!(owned_params.destination_protocol, "AMQP 0-9-1");
    assert_eq!(owned_params.source_uri, "amqp://localhost:5672/%2f");
    assert_eq!(owned_params.destination_uri, "amqp://remote:5673/%2f");
    assert_eq!(owned_params.source_queue.unwrap(), "source-queue");
    assert_eq!(owned_params.destination_queue.unwrap(), "dest-queue");
    assert_eq!(
        owned_params.acknowledgement_mode,
        MessageTransferAcknowledgementMode::default()
    );
}

#[test]
fn test_shovel_params_roundtrip_conversion() {
    let original_json = r#"
        {
          "value": {
            "src-protocol": "amqp091",
            "dest-protocol": "amqp091",
            "src-uri": "amqp://guest:guest@localhost:5672/%2f",
            "dest-uri": "amqp://guest:guest@remote:5673/%2f",
            "src-queue": "input-queue",
            "dest-queue": "output-queue",
            "ack-mode": "on-confirm",
            "reconnect-delay": 15,
            "src-predeclared": true,
            "dest-predeclared": false
          },
          "vhost": "/",
          "component": "shovel",
          "name": "roundtrip-shovel"
        }
    "#;

    let original_param: RuntimeParameter = serde_json::from_str(original_json).unwrap();
    let owned_params = OwnedShovelParams::try_from(original_param.clone()).unwrap();
    let runtime_def = RuntimeParameterDefinition::from(&owned_params);

    assert_eq!(runtime_def.name, "roundtrip-shovel");
    assert_eq!(runtime_def.vhost, "/");
    assert_eq!(runtime_def.component, "shovel");

    let src_uri_value = runtime_def.value.get("src-uri").unwrap();
    assert_eq!(
        src_uri_value.as_str().unwrap(),
        "amqp://guest:guest@localhost:5672/%2f"
    );

    let dest_uri_value = runtime_def.value.get("dest-uri").unwrap();
    assert_eq!(
        dest_uri_value.as_str().unwrap(),
        "amqp://guest:guest@remote:5673/%2f"
    );

    let ack_mode_value = runtime_def.value.get("ack-mode").unwrap();
    assert_eq!(ack_mode_value.as_str().unwrap(), "on-confirm");

    let src_queue_value = runtime_def.value.get("src-queue").unwrap();
    assert_eq!(src_queue_value.as_str().unwrap(), "input-queue");

    let dest_queue_value = runtime_def.value.get("dest-queue").unwrap();
    assert_eq!(dest_queue_value.as_str().unwrap(), "output-queue");

    let reconnect_delay_value = runtime_def.value.get("reconnect-delay").unwrap();
    assert_eq!(reconnect_delay_value.as_u64().unwrap(), 15);

    let src_predeclared_value = runtime_def.value.get("src-predeclared").unwrap();
    assert_eq!(src_predeclared_value.as_bool().unwrap(), true);

    let dest_predeclared_value = runtime_def.value.get("dest-predeclared").unwrap();
    assert_eq!(dest_predeclared_value.as_bool().unwrap(), false);
}

#[test]
fn test_shovel_update_scenario() {
    // Simulate fetching a shovel, modifying URIs, and updating
    let original_json = r#"
        {
          "value": {
            "src-protocol": "amqp091",
            "dest-protocol": "amqp091",
            "src-uri": "amqp://localhost:5672/%2f",
            "dest-uri": "amqp://localhost:5673/%2f",
            "src-queue": "old-source",
            "dest-queue": "old-dest",
            "ack-mode": "on-publish",
            "reconnect-delay": 5
          },
          "vhost": "/",
          "component": "shovel",
          "name": "updatable-shovel"
        }
    "#;

    // 1. Fetch and parse the shovel
    let param: RuntimeParameter = serde_json::from_str(original_json).unwrap();
    let mut owned_params = OwnedShovelParams::try_from(param).unwrap();

    // 2. Update URIs and other fields
    owned_params.source_uri = "amqps://user:newpass@new-source:5671/%2f".to_string();
    owned_params.destination_uri = "amqps://user:newpass@new-dest:5671/%2f".to_string();
    owned_params.acknowledgement_mode = MessageTransferAcknowledgementMode::WhenConfirmed;
    owned_params.reconnect_delay = Some(10);
    owned_params.source_queue = Some("new-source-queue".to_string());
    owned_params.destination_queue = Some("new-dest-queue".to_string());

    // 3. Convert back to runtime parameter definition for API call
    let runtime_def = RuntimeParameterDefinition::from(&owned_params);

    // 4. Verify the updates
    assert_eq!(runtime_def.name, "updatable-shovel");
    assert_eq!(runtime_def.vhost, "/");

    let src_uri_value = runtime_def.value.get("src-uri").unwrap();
    assert_eq!(
        src_uri_value.as_str().unwrap(),
        "amqps://user:newpass@new-source:5671/%2f"
    );

    let dest_uri_value = runtime_def.value.get("dest-uri").unwrap();
    assert_eq!(
        dest_uri_value.as_str().unwrap(),
        "amqps://user:newpass@new-dest:5671/%2f"
    );

    let ack_mode_value = runtime_def.value.get("ack-mode").unwrap();
    assert_eq!(ack_mode_value.as_str().unwrap(), "on-confirm");

    let src_queue_value = runtime_def.value.get("src-queue").unwrap();
    assert_eq!(src_queue_value.as_str().unwrap(), "new-source-queue");

    let dest_queue_value = runtime_def.value.get("dest-queue").unwrap();
    assert_eq!(dest_queue_value.as_str().unwrap(), "new-dest-queue");

    let reconnect_delay_value = runtime_def.value.get("reconnect-delay").unwrap();
    assert_eq!(reconnect_delay_value.as_u64().unwrap(), 10);
}

#[test]
fn test_shovel_missing_optional_fields() {
    let json = r#"
        {
          "value": {
            "src-protocol": "amqp10",
            "dest-protocol": "amqp10",
            "src-uri": "amqp://localhost:5672/",
            "dest-uri": "amqp://remote:5673/",
            "src-address": "source.addr",
            "dest-address": "dest.addr",
            "ack-mode": "on-confirm"
          },
          "vhost": "test",
          "component": "shovel",
          "name": "minimal-shovel"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let shovel_params = OwnedShovelParams::try_from(param).unwrap();

    // Optional fields should be None when not present
    assert!(shovel_params.reconnect_delay.is_none());
    assert!(shovel_params.source_queue.is_none());
    assert!(shovel_params.destination_queue.is_none());
    assert!(shovel_params.source_exchange.is_none());
    assert!(shovel_params.destination_exchange.is_none());
    assert!(shovel_params.source_exchange_routing_key.is_none());
    assert!(shovel_params.destination_exchange_routing_key.is_none());
    assert!(shovel_params.source_predeclared.is_none());
    assert!(shovel_params.destination_predeclared.is_none());

    // Required fields should be present
    assert_eq!(shovel_params.source_address.unwrap(), "source.addr");
    assert_eq!(shovel_params.destination_address.unwrap(), "dest.addr");
}

#[test]
fn test_shovel_conversion_error_missing_required_field() {
    let json = r#"
        {
          "value": {
            "dest-protocol": "amqp091",
            "src-uri": "amqp://localhost:5672/",
            "dest-uri": "amqp://remote:5673/",
            "ack-mode": "on-confirm"
          },
          "vhost": "/",
          "component": "shovel",
          "name": "incomplete-shovel"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(json).unwrap();
    let result = OwnedShovelParams::try_from(param);

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("Missing") || e.to_string().contains("src-protocol"));
    }
}

#[test]
fn test_shovel_disable_tls_peer_verification_scenario() {
    // Simulate updating a shovel to disable TLS peer verification for both source and destination
    let original_json = r#"
        {
          "value": {
            "src-protocol": "amqp091",
            "dest-protocol": "amqp091",
            "src-uri": "amqps://user:pass@source-host:5671/vhost?verify=verify_peer&cacertfile=/path/to/ca.pem",
            "dest-uri": "amqps://user:pass@dest-host:5671/vhost?verify=verify_peer&certfile=/path/to/cert.pem&keyfile=/path/to/key.pem",
            "src-queue": "secure-source",
            "dest-queue": "secure-dest",
            "ack-mode": "on-confirm",
            "reconnect-delay": 5
          },
          "vhost": "/",
          "component": "shovel",
          "name": "secure-shovel"
        }
    "#;

    // 1. Fetch and parse the shovel (simulating get_runtime_parameter response)
    let param: RuntimeParameter = serde_json::from_str(original_json).unwrap();
    let mut owned_params = OwnedShovelParams::try_from(param).unwrap();

    // Verify original URIs have TLS peer verification enabled
    assert!(owned_params.source_uri.contains("verify=verify_peer"));
    assert!(owned_params.destination_uri.contains("verify=verify_peer"));
    assert!(
        owned_params
            .source_uri
            .contains("cacertfile=/path/to/ca.pem")
    );
    assert!(
        owned_params
            .destination_uri
            .contains("certfile=/path/to/cert.pem")
    );
    assert!(
        owned_params
            .destination_uri
            .contains("keyfile=/path/to/key.pem")
    );

    // 2. Function to disable TLS peer verification in a URI
    let disable_tls_peer_verification = |uri: &str| -> Result<String, Box<dyn Error>> {
        let updated_uri = UriBuilder::new(uri)?
            .with_tls_peer_verification(TlsPeerVerificationMode::Disabled)
            .build()?;
        Ok(updated_uri)
    };

    // 3. Update both source and destination URIs to disable TLS peer verification
    let updated_source_uri = disable_tls_peer_verification(&owned_params.source_uri).unwrap();
    let updated_dest_uri = disable_tls_peer_verification(&owned_params.destination_uri).unwrap();

    // Verify the URIs were actually changed
    assert_ne!(owned_params.source_uri, updated_source_uri);
    assert_ne!(owned_params.destination_uri, updated_dest_uri);

    // Apply the updates
    owned_params.source_uri = updated_source_uri.clone();
    owned_params.destination_uri = updated_dest_uri.clone();

    // 4. Verify the updated URIs have TLS peer verification disabled
    assert!(updated_source_uri.contains("verify=verify_none"));
    assert!(updated_dest_uri.contains("verify=verify_none"));

    // Verify there is only one 'verify' parameter in each URI (no duplicates)
    let source_verify_count = updated_source_uri.matches("verify=").count();
    let dest_verify_count = updated_dest_uri.matches("verify=").count();
    assert_eq!(
        source_verify_count, 1,
        "Source URI should have exactly one 'verify' parameter, found {}: {}",
        source_verify_count, updated_source_uri
    );
    assert_eq!(
        dest_verify_count, 1,
        "Destination URI should have exactly one 'verify' parameter, found {}: {}",
        dest_verify_count, updated_dest_uri
    );

    // Verify specifically that verify_peer is not present (was replaced, not added)
    assert!(
        !updated_source_uri.contains("verify=verify_peer"),
        "Source URI should not contain verify_peer: {}",
        updated_source_uri
    );
    assert!(
        !updated_dest_uri.contains("verify=verify_peer"),
        "Destination URI should not contain verify_peer: {}",
        updated_dest_uri
    );

    // Verify other TLS settings are preserved
    assert!(updated_source_uri.contains("cacertfile=/path/to/ca.pem"));
    assert!(updated_dest_uri.contains("certfile=/path/to/cert.pem"));
    assert!(updated_dest_uri.contains("keyfile=/path/to/key.pem"));

    // 5. Convert back to runtime parameter definition for API update
    let runtime_def = RuntimeParameterDefinition::from(&owned_params);

    // 6. Verify the final runtime parameter has the updated URIs
    assert_eq!(runtime_def.name, "secure-shovel");
    assert_eq!(runtime_def.vhost, "/");
    assert_eq!(runtime_def.component, "shovel");

    let src_uri_value = runtime_def.value.get("src-uri").unwrap();
    let src_uri_str = src_uri_value.as_str().unwrap();
    assert_eq!(src_uri_str, &updated_source_uri);
    assert!(src_uri_str.contains("verify=verify_none"));

    // Verify only one verify parameter in the runtime parameter
    assert_eq!(
        src_uri_str.matches("verify=").count(),
        1,
        "Runtime parameter source URI should have exactly one 'verify' parameter: {}",
        src_uri_str
    );

    let dest_uri_value = runtime_def.value.get("dest-uri").unwrap();
    let dest_uri_str = dest_uri_value.as_str().unwrap();
    assert_eq!(dest_uri_str, &updated_dest_uri);
    assert!(dest_uri_str.contains("verify=verify_none"));

    // Verify only one verify parameter in the runtime parameter
    assert_eq!(
        dest_uri_str.matches("verify=").count(),
        1,
        "Runtime parameter destination URI should have exactly one 'verify' parameter: {}",
        dest_uri_str
    );

    // Verify other shovel parameters are unchanged
    let src_queue_value = runtime_def.value.get("src-queue").unwrap();
    assert_eq!(src_queue_value.as_str().unwrap(), "secure-source");

    let dest_queue_value = runtime_def.value.get("dest-queue").unwrap();
    assert_eq!(dest_queue_value.as_str().unwrap(), "secure-dest");

    let ack_mode_value = runtime_def.value.get("ack-mode").unwrap();
    assert_eq!(ack_mode_value.as_str().unwrap(), "on-confirm");
}

#[test]
fn test_shovel_add_tls_peer_verification_to_plain_uri() {
    // Test adding TLS peer verification settings to URIs that don't have them
    let original_json = r#"
        {
          "value": {
            "src-protocol": "amqp091",
            "dest-protocol": "amqp091",
            "src-uri": "amqps://user:pass@source-host:5671/vhost",
            "dest-uri": "amqps://user:pass@dest-host:5671/vhost",
            "src-queue": "plain-source",
            "dest-queue": "plain-dest",
            "ack-mode": "on-publish"
          },
          "vhost": "/",
          "component": "shovel",
          "name": "plain-shovel"
        }
    "#;

    let param: RuntimeParameter = serde_json::from_str(original_json).unwrap();
    let mut owned_params = OwnedShovelParams::try_from(param).unwrap();

    // Verify original URIs don't have TLS parameters
    assert!(!owned_params.source_uri.contains("verify="));
    assert!(!owned_params.destination_uri.contains("verify="));

    // Function to add comprehensive TLS settings
    let add_tls_settings = |uri: &str| -> Result<String, Box<dyn Error>> {
        let updated_uri = UriBuilder::new(uri)?
            .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
            .with_ca_cert_file("/path/to/ca_bundle.pem")
            .with_server_name_indication("my-rabbitmq-server.example.com")
            .build()?;
        Ok(updated_uri)
    };

    // Update both URIs
    owned_params.source_uri = add_tls_settings(&owned_params.source_uri).unwrap();
    owned_params.destination_uri = add_tls_settings(&owned_params.destination_uri).unwrap();

    // Convert to runtime parameter definition
    let runtime_def = RuntimeParameterDefinition::from(&owned_params);

    // Verify the TLS settings were added
    let src_uri_value = runtime_def.value.get("src-uri").unwrap().as_str().unwrap();
    assert!(src_uri_value.contains("verify=verify_peer"));
    assert!(src_uri_value.contains("cacertfile=/path/to/ca_bundle.pem"));
    assert!(src_uri_value.contains("server_name_indication=my-rabbitmq-server.example.com"));

    let dest_uri_value = runtime_def.value.get("dest-uri").unwrap().as_str().unwrap();
    assert!(dest_uri_value.contains("verify=verify_peer"));
    assert!(dest_uri_value.contains("cacertfile=/path/to/ca_bundle.pem"));
    assert!(dest_uri_value.contains("server_name_indication=my-rabbitmq-server.example.com"));

    // Verify the original scheme, host, and path are preserved
    assert!(src_uri_value.starts_with("amqps://user:pass@source-host:5671/vhost"));
    assert!(dest_uri_value.starts_with("amqps://user:pass@dest-host:5671/vhost"));
}

#[test]
fn test_realistic_shovel_tls_bulk_update_scenario() {
    // Realistic scenario: bulk update multiple shovels to disable TLS peer verification
    // This simulates what the user's example function would do

    // Helper function to simulate the disable_tls_peer_verification function
    fn disable_tls_peer_verification(uri: &str) -> Result<String, Box<dyn Error>> {
        let updated_uri = UriBuilder::new(uri)?
            .with_tls_peer_verification(TlsPeerVerificationMode::Disabled)
            .build()?;
        Ok(updated_uri)
    }

    // Simulate list_shovels() returning multiple shovels with different TLS configurations
    let shovels = vec![
        // Shovel 1: Basic TLS with peer verification
        Shovel {
            node: "rabbit@node1".to_string(),
            name: "shovel-1".to_string(),
            vhost: Some("/".to_string()),
            typ: ShovelType::Dynamic,
            state: ShovelState::Running,
            source_uri: Some("amqps://user:pass@source1:5671/vhost?verify=verify_peer".to_string()),
            destination_uri: Some("amqps://user:pass@dest1:5671/vhost?verify=verify_peer".to_string()),
            source: Some("queue1".to_string()),
            destination: Some("queue1-copy".to_string()),
            source_address: None,
            destination_address: None,
            source_protocol: Some(MessagingProtocol::Amqp091),
            destination_protocol: Some(MessagingProtocol::Amqp091),
        },
        // Shovel 2: TLS with comprehensive settings
        Shovel {
            node: "rabbit@node1".to_string(),
            name: "shovel-2".to_string(),
            vhost: Some("/".to_string()),
            typ: ShovelType::Dynamic,
            state: ShovelState::Running,
            source_uri: Some("amqps://user:pass@source2:5671/vhost?verify=verify_peer&cacertfile=/ca.pem&server_name_indication=source.example.com".to_string()),
            destination_uri: Some("amqps://user:pass@dest2:5671/vhost?verify=verify_peer&certfile=/cert.pem&keyfile=/key.pem".to_string()),
            source: Some("queue2".to_string()),
            destination: Some("queue2-copy".to_string()),
            source_address: None,
            destination_address: None,
            source_protocol: Some(MessagingProtocol::Amqp091),
            destination_protocol: Some(MessagingProtocol::Amqp091),
        },
        // Shovel 3: Already has TLS disabled (should be skipped)
        Shovel {
            node: "rabbit@node1".to_string(),
            name: "shovel-3".to_string(),
            vhost: Some("/".to_string()),
            typ: ShovelType::Dynamic,
            state: ShovelState::Running,
            source_uri: Some("amqps://user:pass@source3:5671/vhost?verify=verify_none".to_string()),
            destination_uri: Some("amqps://user:pass@dest3:5671/vhost?verify=verify_none".to_string()),
            source: Some("queue3".to_string()),
            destination: Some("queue3-copy".to_string()),
            source_address: None,
            destination_address: None,
            source_protocol: Some(MessagingProtocol::Amqp091),
            destination_protocol: Some(MessagingProtocol::Amqp091),
        },
    ];

    let mut runtime_parameter_updates: Vec<(String, String, String, String)> = Vec::new();

    // Process each shovel (simulating the bulk update function)
    for shovel in shovels {
        let mut shovel_params = OwnedShovelParams::from(shovel);

        let original_source_uri = &shovel_params.source_uri;
        let original_dest_uri = &shovel_params.destination_uri;

        let updated_source_uri = disable_tls_peer_verification(original_source_uri).unwrap();
        let updated_dest_uri = disable_tls_peer_verification(original_dest_uri).unwrap();

        // Only update if URIs actually changed
        if original_source_uri != &updated_source_uri || original_dest_uri != &updated_dest_uri {
            shovel_params.source_uri = updated_source_uri;
            shovel_params.destination_uri = updated_dest_uri;

            // For this test, we'll collect the updated parameters info rather than references
            let name = shovel_params.name.clone();
            let vhost = shovel_params.vhost.clone();
            let updated_src_uri = shovel_params.source_uri.clone();
            let updated_dest_uri = shovel_params.destination_uri.clone();

            runtime_parameter_updates.push((name, vhost, updated_src_uri, updated_dest_uri));
        }
    }

    // Verify that we got updates for shovels 1 and 2, but not 3
    assert_eq!(runtime_parameter_updates.len(), 2);

    // Check shovel-1 update (name, vhost, src_uri, dest_uri)
    let (name1, vhost1, src_uri1, dest_uri1) = &runtime_parameter_updates[0];
    assert_eq!(name1, "shovel-1");
    assert_eq!(vhost1, "/");
    assert!(src_uri1.contains("verify=verify_none"));
    assert!(dest_uri1.contains("verify=verify_none"));

    // Check shovel-2 update
    let (name2, vhost2, src_uri2, dest_uri2) = &runtime_parameter_updates[1];
    assert_eq!(name2, "shovel-2");
    assert_eq!(vhost2, "/");
    assert!(src_uri2.contains("verify=verify_none"));
    assert!(dest_uri2.contains("verify=verify_none"));

    // Verify other TLS settings are preserved in shovel-2
    assert!(src_uri2.contains("cacertfile=/ca.pem"));
    assert!(src_uri2.contains("server_name_indication=source.example.com"));
    assert!(dest_uri2.contains("certfile=/cert.pem"));
    assert!(dest_uri2.contains("keyfile=/key.pem"));

    // Verify that both updated shovels had their TLS peer verification disabled
    for (_name, vhost, src_uri, dest_uri) in &runtime_parameter_updates {
        assert_eq!(vhost, "/");
        assert!(src_uri.contains("verify=verify_none"));
        assert!(dest_uri.contains("verify=verify_none"));

        // Verify no duplicate verify parameters
        assert_eq!(
            src_uri.matches("verify=").count(),
            1,
            "Source URI should have exactly one 'verify' parameter: {}",
            src_uri
        );
        assert_eq!(
            dest_uri.matches("verify=").count(),
            1,
            "Destination URI should have exactly one 'verify' parameter: {}",
            dest_uri
        );

        // Verify old verify_peer is completely replaced
        assert!(
            !src_uri.contains("verify=verify_peer"),
            "Source URI should not contain verify_peer: {}",
            src_uri
        );
        assert!(
            !dest_uri.contains("verify=verify_peer"),
            "Destination URI should not contain verify_peer: {}",
            dest_uri
        );
    }
}
