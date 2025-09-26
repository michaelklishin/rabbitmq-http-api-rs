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

use rabbitmq_http_client::commons::TlsPeerVerificationMode;
use rabbitmq_http_client::uris::{TlsClientSettings, UriBuilder};
use std::collections::HashMap;
use url::Url;

fn get_query_params(uri_str: &str) -> HashMap<String, String> {
    let url = Url::parse(uri_str).expect("Valid URL");
    url.query_pairs().into_owned().collect()
}

fn has_query_param(uri_str: &str, key: &str, expected_value: &str) -> bool {
    let params = get_query_params(uri_str);
    params
        .get(key)
        .map_or(false, |value| value == expected_value)
}

fn has_query_param_key(uri_str: &str, key: &str) -> bool {
    let params = get_query_params(uri_str);
    params.contains_key(key)
}

#[test]
fn test_uri_builder_basic_construction() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder.build().unwrap();
    assert_eq!(result, uri);
}

#[test]
fn test_uri_builder_invalid_uri() {
    let invalid_uri = "not-a-valid-uri";
    let result = UriBuilder::new(invalid_uri);
    assert!(result.is_err());
}

#[test]
fn test_uri_builder_with_tls_peer_verification() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .build()
        .unwrap();

    assert!(has_query_param(&result, "verify", "verify_peer"));
    assert!(has_query_param(
        &result,
        "verify",
        TlsPeerVerificationMode::Enabled.as_ref()
    ));
}

#[test]
fn test_uri_builder_with_tls_peer_verification_enabled_via_uri_query_parameters() {
    let uri = "amqps://user:pass@localhost:5671/vhost?verify=verify_peer&cacertfile=/path/to/ca_bundle.pem";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder.build().unwrap();

    assert!(has_query_param(&result, "verify", "verify_peer"));
    assert!(has_query_param(
        &result,
        "verify",
        TlsPeerVerificationMode::Enabled.as_ref()
    ));
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/path/to/ca_bundle.pem"
    ));
}

#[test]
fn test_uri_builder_with_tls_peer_verification_disabled_via_uri_query_parameters() {
    let uri = "amqps://user:pass@localhost:5671/vhost?verify=verify_none";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder.build().unwrap();

    assert!(has_query_param(
        &result,
        "verify",
        TlsPeerVerificationMode::Disabled.as_ref()
    ));
}

#[test]
fn test_uri_builder_with_tls_peer_verification_disabled() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Disabled)
        .build()
        .unwrap();

    assert!(has_query_param(
        &result,
        "verify",
        TlsPeerVerificationMode::Disabled.as_ref()
    ));
}

#[test]
fn test_uri_builder_with_tls_peer_verification_flip() {
    let uri = "amqps://user:pass@localhost:5671/vhost?verify=verify_peer&cacertfile=/path/to/ca_bundle.pem";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Disabled)
        .build()
        .unwrap();

    assert!(has_query_param(
        &result,
        "verify",
        TlsPeerVerificationMode::Disabled.as_ref()
    ));
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/path/to/ca_bundle.pem"
    ));

    // Compare query parameters as a map to avoid flakiness due to iteration order
    let params = get_query_params(&result);
    let expected: HashMap<String, String> = vec![
        (
            "cacertfile".to_string(),
            "/path/to/ca_bundle.pem".to_string(),
        ),
        (
            "verify".to_string(),
            TlsPeerVerificationMode::Disabled.as_ref().to_string(),
        ),
    ]
    .into_iter()
    .collect();
    assert_eq!(params, expected);
}

#[test]
fn test_uri_builder_with_ca_cert_file() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_ca_cert_file("/path/to/ca_bundle.pem")
        .build()
        .unwrap();

    assert!(has_query_param(
        &result,
        "cacertfile",
        "/path/to/ca_bundle.pem"
    ));
}

#[test]
fn test_uri_builder_with_client_cert_file() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_client_cert_file("/path/to/client.pem")
        .build()
        .unwrap();

    assert!(has_query_param(&result, "certfile", "/path/to/client.pem"));
}

#[test]
fn test_uri_builder_with_client_key_file() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_client_key_file("/path/to/key.pem")
        .build()
        .unwrap();

    assert!(has_query_param(&result, "keyfile", "/path/to/key.pem"));
}

#[test]
fn test_uri_builder_with_server_name_indication() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_server_name_indication("myhost.example.com")
        .build()
        .unwrap();

    assert!(has_query_param(
        &result,
        "server_name_indication",
        "myhost.example.com"
    ));
}

#[test]
fn test_uri_builder_with_custom_query_param() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_query_param("custom_param", "custom_value")
        .build()
        .unwrap();

    assert!(has_query_param(&result, "custom_param", "custom_value"));
}

#[test]
fn test_uri_builder_without_query_param() {
    let uri = "amqps://user:pass@localhost:5671/vhost?existing_param=value";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .without_query_param("existing_param")
        .build()
        .unwrap();

    assert!(!has_query_param_key(&result, "existing_param"));
}

#[test]
fn test_uri_builder_chained_operations() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .with_ca_cert_file("/path/to/ca_bundle.pem")
        .with_client_cert_file("/path/to/client.pem")
        .with_client_key_file("/path/to/key.pem")
        .with_server_name_indication("myhost.example.com")
        .build()
        .unwrap();

    assert!(has_query_param(
        &result,
        "verify",
        TlsPeerVerificationMode::Enabled.as_ref()
    ));
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/path/to/ca_bundle.pem"
    ));
    assert!(has_query_param(&result, "certfile", "/path/to/client.pem"));
    assert!(has_query_param(&result, "keyfile", "/path/to/key.pem"));
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "myhost.example.com"
    ));
}

#[test]
fn test_uri_builder_query_params_method() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let mut builder = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .with_ca_cert_file("/path/to/ca_bundle.pem");

    let params = builder.query_params();
    assert_eq!(
        params.get("verify"),
        Some(&TlsPeerVerificationMode::Enabled.as_ref().to_string())
    );
    assert_eq!(
        params.get("cacertfile"),
        Some(&"/path/to/ca_bundle.pem".to_string())
    );
}

#[test]
fn test_uri_builder_as_url_method() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let mut builder = UriBuilder::new(uri).unwrap();
    let url = builder.as_url();
    assert_eq!(url.scheme(), "amqps");
    assert_eq!(url.host_str(), Some("localhost"));
    assert_eq!(url.port(), Some(5671));
    assert_eq!(url.path(), "/vhost");
    assert_eq!(url.username(), "user");
}

#[test]
fn test_tls_client_settings_new() {
    let settings = TlsClientSettings::new();
    assert!(settings.peer_verification.is_none());
    assert!(settings.ca_cert_file.is_none());
    assert!(settings.client_cert_file.is_none());
    assert!(settings.client_key_file.is_none());
    assert!(settings.server_name_indication.is_none());
}

#[test]
fn test_tls_client_settings_with_verification() {
    let settings = TlsClientSettings::with_verification();
    assert_eq!(
        settings.peer_verification,
        Some(TlsPeerVerificationMode::Enabled)
    );
}

#[test]
fn test_tls_client_settings_without_verification() {
    let settings = TlsClientSettings::without_verification();
    assert_eq!(
        settings.peer_verification,
        Some(TlsPeerVerificationMode::Disabled)
    );
}

#[test]
fn test_tls_client_settings_builder_pattern() {
    let settings = TlsClientSettings::new()
        .peer_verification(TlsPeerVerificationMode::Enabled)
        .ca_cert_file("/path/to/ca_bundle.pem")
        .client_cert_file("/path/to/client.pem")
        .client_key_file("/path/to/key.pem")
        .server_name_indication("example.com");

    assert_eq!(
        settings.peer_verification,
        Some(TlsPeerVerificationMode::Enabled)
    );
    assert_eq!(
        settings.ca_cert_file,
        Some("/path/to/ca_bundle.pem".to_string())
    );
    assert_eq!(
        settings.client_cert_file,
        Some("/path/to/client.pem".to_string())
    );
    assert_eq!(
        settings.client_key_file,
        Some("/path/to/key.pem".to_string())
    );
    assert_eq!(
        settings.server_name_indication,
        Some("example.com".to_string())
    );
}

#[test]
fn test_uri_builder_replace_tls_settings() {
    let uri = "amqps://user:pass@localhost:5671/vhost?verify=verify_none&cacertfile=/path/to/ca_bundle.pem";
    let builder = UriBuilder::new(uri).unwrap();

    let settings = TlsClientSettings::new()
        .peer_verification(TlsPeerVerificationMode::Enabled)
        .ca_cert_file("/new/ca_bundle.pem")
        .client_cert_file("/path/to/client.pem");

    let result = builder.replace(settings).build().unwrap();

    assert!(has_query_param(&result, "verify", "verify_peer"));
    assert!(has_query_param(&result, "cacertfile", "/new/ca_bundle.pem"));
    assert!(has_query_param(&result, "certfile", "/path/to/client.pem"));
    assert!(!has_query_param(&result, "verify", "verify_none"));
    assert!(!has_query_param(
        &result,
        "cacertfile",
        "/path/to/ca_bundle.pem"
    ));
}

#[test]
fn test_uri_builder_merge_tls_settings() {
    let uri = "amqps://user:pass@localhost:5671/vhost?verify=verify_none&cacertfile=/path/to/ca_bundle.pem";
    let builder = UriBuilder::new(uri).unwrap();

    let settings = TlsClientSettings::new()
        .peer_verification(TlsPeerVerificationMode::Enabled)
        .client_cert_file("/path/to/client.pem");

    let result = builder.merge(settings).build().unwrap();

    // merge should update peer verification and add client cert, but keep the existing CA
    // certificate bundle parameter
    assert!(has_query_param(&result, "verify", "verify_peer"));
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/path/to/ca_bundle.pem"
    ));
    assert!(has_query_param(&result, "certfile", "/path/to/client.pem"));
}

#[test]
fn test_uri_builder_with_non_tls_parameters() {
    let uri = "amqp://user:pass@localhost:5672/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_query_param("heartbeat", "60")
        .with_query_param("channel_max", "2047")
        .build()
        .unwrap();

    assert!(has_query_param(&result, "heartbeat", "60"));
    assert!(has_query_param(&result, "channel_max", "2047"));
}

#[test]
fn test_uri_builder_parameter_overwrite() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_ca_cert_file("/first/ca_bundle.pem")
        .with_ca_cert_file("/second/ca_bundle.pem")
        .build()
        .unwrap();

    // Should contain the last value set
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/second/ca_bundle.pem"
    ));
    assert!(!has_query_param(
        &result,
        "cacertfile",
        "/first/ca_bundle.pem"
    ));
}

#[test]
fn test_uri_builder_empty_vhost() {
    let uri = "amqps://user:pass@localhost:5671/";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .build()
        .unwrap();

    assert!(has_query_param(&result, "verify", "verify_peer"));
    assert!(result.ends_with("/?verify=verify_peer") || result.contains("/?verify=verify_peer"));
}

#[test]
fn test_uri_builder_special_characters_in_values() {
    let uri = "amqps://user:pass@localhost:5671/vhost";
    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_ca_cert_file("/path with spaces/ca_bundle.pem")
        .with_server_name_indication("host-with-dashes.example.com")
        .build()
        .unwrap();

    // URL encoding should handle special characters
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/path with spaces/ca_bundle.pem"
    ));
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "host-with-dashes.example.com"
    ));
}

// Optimization-specific tests
#[test]
fn test_builder_api_smoke_case1() {
    // Test the intentional percent-decoding behavior with batched operations
    let mut builder = UriBuilder::new("amqps://user:pass@localhost:5671/vhost?verify=verify_none&cacertfile=%2Fpath%2Fto%2Fca.pem").unwrap();

    // Multiple parameter updates should be batched
    builder = builder
        .with_query_param("certfile", "/path/to/cert.pem")
        .with_query_param("keyfile", "/path/to/key.pem")
        .with_query_param("server_name_indication", "example.com");

    let result = builder.build().unwrap();

    // Verify that percent-encoded paths are decoded
    assert!(
        result.contains("cacertfile=/path/to/ca.pem"),
        "Percent-decoding not preserved. Result: {}",
        result
    );
    assert!(result.contains("certfile=/path/to/cert.pem"));
    assert!(result.contains("keyfile=/path/to/key.pem"));
    assert!(result.contains("server_name_indication=example.com"));
}

#[test]
fn test_builder_api_smoke_case2() {
    // Test that the fluent API still works correctly
    let uri = UriBuilder::new("amqps://user:pass@localhost:5671/vhost")
        .unwrap()
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .with_ca_cert_file("/path/to/ca_bundle.pem")
        .with_client_cert_file("/path/to/client.pem")
        .with_client_key_file("/path/to/key.pem")
        .with_server_name_indication("myhost.example.com")
        .build()
        .unwrap();

    assert!(uri.contains("verify=verify_peer"));
    assert!(uri.contains("cacertfile=/path/to/ca_bundle.pem"));
    assert!(uri.contains("certfile=/path/to/client.pem"));
    assert!(uri.contains("keyfile=/path/to/key.pem"));
    assert!(uri.contains("server_name_indication=myhost.example.com"));
}

#[test]
fn test_builder_api_smoke_case3() {
    let mut builder = UriBuilder::new(
        "amqps://user:pass@localhost:5671/vhost?verify=verify_none&cacertfile=/path/to/ca.pem",
    )
    .unwrap();

    // Test query_params method with cached parameters
    builder = builder.with_query_param("keyfile", "/path/to/key.pem");
    let params = builder.query_params();

    assert_eq!(params.get("verify"), Some(&"verify_none".to_string()));
    assert_eq!(
        params.get("cacertfile"),
        Some(&"/path/to/ca.pem".to_string())
    );
    assert_eq!(params.get("keyfile"), Some(&"/path/to/key.pem".to_string()));
}

#[test]
fn test_builder_api_smoke_case4() {
    let uri = UriBuilder::new(
        "amqps://user:pass@localhost:5671/vhost?verify=verify_none&cacertfile=/path/to/ca.pem",
    )
    .unwrap()
    .with_query_param("keyfile", "/path/to/key.pem")
    .without_query_param("verify")
    .build()
    .unwrap();

    assert!(!uri.contains("verify="));
    assert!(uri.contains("cacertfile=/path/to/ca.pem"));
    assert!(uri.contains("keyfile=/path/to/key.pem"));
}

#[test]
fn test_builder_api_smoke_case5() {
    let mut builder = UriBuilder::new("amqps://user:pass@localhost:5671/vhost")
        .unwrap()
        .with_query_param("verify", "verify_peer");

    let url = builder.as_url();
    assert!(url.to_string().contains("verify=verify_peer"));
}

#[test]
fn test_integration_case1() {
    let test_uris = vec![
        "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-one-dev?heartbeat=10&verify=verify_none",
        "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-pdv.dev.megacorp.local/app-five-dev?heartbeat=10&verify=verify_none",
        "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-four-qa-rg?heartbeat=10",
        "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-pdv.dev.megacorp.local/app-seventeen?heartbeat=10&verify=verify_none",
    ];

    for uri in test_uris {
        let builder = UriBuilder::new(uri).unwrap();
        let result = builder
            .with_ca_cert_file("/etc/ssl/certs/ca-certificates.crt")
            .with_client_cert_file("/etc/ssl/certs/megacorp-client.pem")
            .with_client_key_file("/etc/ssl/private/megacorp-client-key.pem")
            .with_server_name_indication("megacorp.local")
            .build()
            .unwrap();

        // The rebuilt URI should contain all the original components
        assert!(result.contains("usern4me:r3d4Ct3D"));
        assert!(result.contains("heartbeat=10"));

        // Should contain TLS configuration
        assert!(has_query_param(
            &result,
            "cacertfile",
            "/etc/ssl/certs/ca-certificates.crt"
        ));
        assert!(has_query_param(
            &result,
            "certfile",
            "/etc/ssl/certs/megacorp-client.pem"
        ));
        assert!(has_query_param(
            &result,
            "keyfile",
            "/etc/ssl/private/megacorp-client-key.pem"
        ));
        assert!(has_query_param(
            &result,
            "server_name_indication",
            "megacorp.local"
        ));

        if uri.contains("verify=verify_none") {
            assert!(result.contains("verify=verify_none"));
        }
    }
}

#[test]
fn test_integration_case2() {
    let bdv_uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-one-dev?heartbeat=10&verify=verify_none";
    let pdv_uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-pdv.dev.megacorp.local/app-five-dev?heartbeat=10&verify=verify_none";

    let bdv_builder = UriBuilder::new(bdv_uri).unwrap();
    let pdv_builder = UriBuilder::new(pdv_uri).unwrap();

    let bdv_result = bdv_builder
        .with_ca_cert_file("/etc/ssl/certs/bdv-ca-bundle.crt")
        .with_client_cert_file("/etc/ssl/certs/bdv-client.pem")
        .with_client_key_file("/etc/ssl/private/bdv-client.key")
        .with_server_name_indication("cluster-abc432-test-bdv.dev.megacorp.local")
        .build()
        .unwrap();

    let pdv_result = pdv_builder
        .with_ca_cert_file("/etc/ssl/certs/pdv-ca-bundle.crt")
        .with_client_cert_file("/etc/ssl/certs/pdv-client.pem")
        .with_client_key_file("/etc/ssl/private/pdv-client.key")
        .with_server_name_indication("cluster-abc432-test-pdv.dev.megacorp.local")
        .build()
        .unwrap();

    assert!(bdv_result.contains("cluster-abc432-test-bdv.dev.megacorp.local"));
    assert!(pdv_result.contains("cluster-abc432-test-pdv.dev.megacorp.local"));
    assert!(bdv_result.contains("app-one-dev"));
    assert!(pdv_result.contains("app-five-dev"));

    // Verify BDV TLS settings
    assert!(has_query_param(
        &bdv_result,
        "cacertfile",
        "/etc/ssl/certs/bdv-ca-bundle.crt"
    ));
    assert!(has_query_param(
        &bdv_result,
        "certfile",
        "/etc/ssl/certs/bdv-client.pem"
    ));
    assert!(has_query_param(
        &bdv_result,
        "keyfile",
        "/etc/ssl/private/bdv-client.key"
    ));
    assert!(has_query_param(
        &bdv_result,
        "server_name_indication",
        "cluster-abc432-test-bdv.dev.megacorp.local"
    ));

    // Verify PDV TLS settings
    assert!(has_query_param(
        &pdv_result,
        "cacertfile",
        "/etc/ssl/certs/pdv-ca-bundle.crt"
    ));
    assert!(has_query_param(
        &pdv_result,
        "certfile",
        "/etc/ssl/certs/pdv-client.pem"
    ));
    assert!(has_query_param(
        &pdv_result,
        "keyfile",
        "/etc/ssl/private/pdv-client.key"
    ));
    assert!(has_query_param(
        &pdv_result,
        "server_name_indication",
        "cluster-abc432-test-pdv.dev.megacorp.local"
    ));
}

#[test]
fn test_integration_case3() {
    let test_cases = vec![
        (
            "app-one-dev",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-one-dev?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/app-one-ca.crt",
            "/etc/ssl/certs/app-one.pem",
            "/etc/ssl/private/app-one.key",
        ),
        (
            "app-eleven-qa-hena",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-eleven-qa-hena?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/app-eleven-qa-ca.crt",
            "/etc/ssl/certs/app-eleven-qa.pem",
            "/etc/ssl/private/app-eleven-qa.key",
        ),
        (
            "app-sixteen-dev1",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-sixteen-dev1?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/app-sixteen-ca.crt",
            "/etc/ssl/certs/app-sixteen.pem",
            "/etc/ssl/private/app-sixteen.key",
        ),
        (
            "app-four-dev-tup",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-four-dev-tup?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/app-four-ca.crt",
            "/etc/ssl/certs/app-four.pem",
            "/etc/ssl/private/app-four.key",
        ),
    ];

    for (expected_vhost, uri, ca_cert, client_cert, client_key) in test_cases {
        let builder = UriBuilder::new(uri).unwrap();
        let result = builder
            .with_ca_cert_file(ca_cert)
            .with_client_cert_file(client_cert)
            .with_client_key_file(client_key)
            .with_server_name_indication("cluster-abc432-test-bdv.dev.megacorp.local")
            .build()
            .unwrap();

        assert!(
            result.contains(expected_vhost),
            "Expected vhost '{}' not found in '{}'",
            expected_vhost,
            result
        );

        // Verify TLS configuration was applied
        assert!(has_query_param(&result, "cacertfile", ca_cert));
        assert!(has_query_param(&result, "certfile", client_cert));
        assert!(has_query_param(&result, "keyfile", client_key));
        assert!(has_query_param(
            &result,
            "server_name_indication",
            "cluster-abc432-test-bdv.dev.megacorp.local"
        ));
    }
}

#[test]
fn test_integration_case4() {
    let with_verify = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-one-dev?heartbeat=10&verify=verify_none";
    let without_verify = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-four-qa-rg?heartbeat=10";

    // Test URI with verify parameter - add comprehensive TLS settings
    let with_verify_builder = UriBuilder::new(with_verify).unwrap();
    let with_verify_result = with_verify_builder
        .with_ca_cert_file("/etc/ssl/certs/production-ca-bundle.crt")
        .with_client_cert_file("/etc/ssl/certs/production-client.pem")
        .with_client_key_file("/etc/ssl/private/production-client.key")
        .with_server_name_indication("production.megacorp.local")
        .build()
        .unwrap();

    // Test URI without verify parameter - add different TLS settings
    let without_verify_builder = UriBuilder::new(without_verify).unwrap();
    let without_verify_result = without_verify_builder
        .with_ca_cert_file("/etc/ssl/certs/qa-ca-bundle.crt")
        .with_client_cert_file("/etc/ssl/certs/qa-client.pem")
        .with_client_key_file("/etc/ssl/private/qa-client.key")
        .with_server_name_indication("qa.megacorp.local")
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .build()
        .unwrap();

    // Verify original parameters preserved
    assert!(has_query_param(&with_verify_result, "heartbeat", "10"));
    assert!(has_query_param(
        &with_verify_result,
        "verify",
        "verify_none"
    ));
    assert!(has_query_param(&without_verify_result, "heartbeat", "10"));
    assert!(has_query_param(
        &without_verify_result,
        "verify",
        "verify_peer"
    ));

    // Verify TLS settings applied to first URI
    assert!(has_query_param(
        &with_verify_result,
        "cacertfile",
        "/etc/ssl/certs/production-ca-bundle.crt"
    ));
    assert!(has_query_param(
        &with_verify_result,
        "certfile",
        "/etc/ssl/certs/production-client.pem"
    ));
    assert!(has_query_param(
        &with_verify_result,
        "keyfile",
        "/etc/ssl/private/production-client.key"
    ));
    assert!(has_query_param(
        &with_verify_result,
        "server_name_indication",
        "production.megacorp.local"
    ));

    // Verify TLS settings applied to second URI
    assert!(has_query_param(
        &without_verify_result,
        "cacertfile",
        "/etc/ssl/certs/qa-ca-bundle.crt"
    ));
    assert!(has_query_param(
        &without_verify_result,
        "certfile",
        "/etc/ssl/certs/qa-client.pem"
    ));
    assert!(has_query_param(
        &without_verify_result,
        "keyfile",
        "/etc/ssl/private/qa-client.key"
    ));
    assert!(has_query_param(
        &without_verify_result,
        "server_name_indication",
        "qa.megacorp.local"
    ));
}

#[test]
fn test_integration_case5() {
    let uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-one-dev?heartbeat=10&verify=verify_none";

    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
        .with_ca_cert_file("/etc/ssl/certs/enterprise-ca-bundle.crt")
        .with_client_cert_file("/etc/ssl/certs/enterprise-client.pem")
        .with_client_key_file("/etc/ssl/private/enterprise-client.key")
        .with_server_name_indication("enterprise.megacorp.local")
        .build()
        .unwrap();

    // Should change verify from verify_none to verify_peer
    assert!(has_query_param(&result, "verify", "verify_peer"));
    assert!(!has_query_param(&result, "verify", "verify_none"));

    // Should preserve heartbeat
    assert!(has_query_param(&result, "heartbeat", "10"));

    // Should add all TLS parameters
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/etc/ssl/certs/enterprise-ca-bundle.crt"
    ));
    assert!(has_query_param(
        &result,
        "certfile",
        "/etc/ssl/certs/enterprise-client.pem"
    ));
    assert!(has_query_param(
        &result,
        "keyfile",
        "/etc/ssl/private/enterprise-client.key"
    ));
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "enterprise.megacorp.local"
    ));
}

#[test]
fn test_integration_case6() {
    let uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-four-qa-rg?heartbeat=10";

    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Disabled)
        .with_ca_cert_file("/etc/ssl/certs/staging-ca-bundle.crt")
        .with_client_cert_file("/etc/ssl/certs/staging-client.pem")
        .with_client_key_file("/etc/ssl/private/staging-client.key")
        .with_server_name_indication("staging-cluster.megacorp.local")
        .build()
        .unwrap();

    assert!(has_query_param(&result, "heartbeat", "10"));
    assert!(has_query_param(&result, "verify", "verify_none"));
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/etc/ssl/certs/staging-ca-bundle.crt"
    ));
    assert!(has_query_param(
        &result,
        "certfile",
        "/etc/ssl/certs/staging-client.pem"
    ));
    assert!(has_query_param(
        &result,
        "keyfile",
        "/etc/ssl/private/staging-client.key"
    ));
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "staging-cluster.megacorp.local"
    ));
}

#[test]
fn test_integration_case7() {
    let complex_vhosts = vec![
        (
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-eleven-qa-hena?heartbeat=10&verify=verify_none",
            "app-eleven-qa-hena",
            "/etc/ssl/certs/app-eleven-ca.crt",
            "/etc/ssl/certs/app-eleven.pem",
            "/etc/ssl/private/app-eleven.key",
        ),
        (
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-fourteen-qae?heartbeat=10&verify=verify_none",
            "app-fourteen-qae",
            "/etc/ssl/certs/app-fourteen-ca.crt",
            "/etc/ssl/certs/app-fourteen.pem",
            "/etc/ssl/private/app-fourteen.key",
        ),
        (
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-sixteen-hen1?heartbeat=10&verify=verify_none",
            "app-sixteen-hen1",
            "/etc/ssl/certs/app-sixteen-ca.crt",
            "/etc/ssl/certs/app-sixteen.pem",
            "/etc/ssl/private/app-sixteen.key",
        ),
    ];

    for (uri, expected_vhost, ca_cert, client_cert, client_key) in complex_vhosts {
        let builder = UriBuilder::new(uri).unwrap();
        let result = builder
            .with_query_param("connection_timeout", "30000")
            .with_ca_cert_file(ca_cert)
            .with_client_cert_file(client_cert)
            .with_client_key_file(client_key)
            .with_server_name_indication("complex-apps.megacorp.local")
            .build()
            .unwrap();

        // Original parameters should be preserved
        assert!(has_query_param(&result, "heartbeat", "10"));
        assert!(has_query_param(&result, "verify", "verify_none"));

        // New parameter should be added
        assert!(has_query_param(&result, "connection_timeout", "30000"));

        // TLS configuration should be applied
        assert!(has_query_param(&result, "cacertfile", ca_cert));
        assert!(has_query_param(&result, "certfile", client_cert));
        assert!(has_query_param(&result, "keyfile", client_key));
        assert!(has_query_param(
            &result,
            "server_name_indication",
            "complex-apps.megacorp.local"
        ));

        // Vhost should be preserved
        assert!(result.contains(expected_vhost));
    }
}

#[test]
fn test_integration_case8() {
    let uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-five-qa?heartbeat=10&verify=verify_none";

    let builder = UriBuilder::new(uri).unwrap();
    let result = builder
        .without_query_param("verify")
        .with_query_param("channel_max", "1024")
        .with_ca_cert_file("/etc/ssl/certs/development-ca.crt")
        .with_client_cert_file("/etc/ssl/certs/development-client.pem")
        .with_client_key_file("/etc/ssl/private/development-client.key")
        .with_server_name_indication("development.megacorp.local")
        .build()
        .unwrap();

    // verify parameter should be removed
    assert!(!has_query_param_key(&result, "verify"));

    // heartbeat should be preserved
    assert!(has_query_param(&result, "heartbeat", "10"));

    // new parameter should be added
    assert!(has_query_param(&result, "channel_max", "1024"));

    // TLS configuration should be applied
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/etc/ssl/certs/development-ca.crt"
    ));
    assert!(has_query_param(
        &result,
        "certfile",
        "/etc/ssl/certs/development-client.pem"
    ));
    assert!(has_query_param(
        &result,
        "keyfile",
        "/etc/ssl/private/development-client.key"
    ));
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "development.megacorp.local"
    ));
}

#[test]
fn test_integration_case9() {
    let uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-three-qa?heartbeat=10&verify=verify_none";

    let settings = TlsClientSettings::new()
        .peer_verification(TlsPeerVerificationMode::Enabled)
        .ca_cert_file("/etc/ssl/certs/production-ca-bundle.crt")
        .client_cert_file("/etc/ssl/certs/production-client-cert.pem")
        .client_key_file("/etc/ssl/private/production-client-key.pem")
        .server_name_indication("production-cluster.megacorp.local");

    let builder = UriBuilder::new(uri).unwrap();
    let result = builder.replace(settings).build().unwrap();

    // All TLS settings should be replaced
    assert!(has_query_param(&result, "verify", "verify_peer"));
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/etc/ssl/certs/production-ca-bundle.crt"
    ));
    assert!(has_query_param(
        &result,
        "certfile",
        "/etc/ssl/certs/production-client-cert.pem"
    ));
    assert!(has_query_param(
        &result,
        "keyfile",
        "/etc/ssl/private/production-client-key.pem"
    ));
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "production-cluster.megacorp.local"
    ));

    // Non-TLS parameters should be preserved
    assert!(has_query_param(&result, "heartbeat", "10"));
}

#[test]
fn test_integration_case10() {
    let uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-eight-staging?heartbeat=10&verify=verify_none";

    let settings = TlsClientSettings::new()
        .ca_cert_file("/etc/ssl/certs/staging-ca-bundle.crt")
        .client_cert_file("/etc/ssl/certs/staging-client.pem")
        .client_key_file("/etc/ssl/private/staging-client.key")
        .server_name_indication("staging-environment.megacorp.local");

    let builder = UriBuilder::new(uri).unwrap();
    let result = builder.merge(settings).build().unwrap();

    // Only specified TLS settings should be updated
    assert!(has_query_param(&result, "verify", "verify_none")); // preserved
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/etc/ssl/certs/staging-ca-bundle.crt"
    )); // added
    assert!(has_query_param(
        &result,
        "certfile",
        "/etc/ssl/certs/staging-client.pem"
    )); // added
    assert!(has_query_param(
        &result,
        "keyfile",
        "/etc/ssl/private/staging-client.key"
    )); // added
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "staging-environment.megacorp.local"
    )); // added

    // Non-TLS parameters should be preserved
    assert!(has_query_param(&result, "heartbeat", "10"));
}

#[test]
fn test_integration_case11() {
    let uris = vec![
        "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-nine-dev?heartbeat=10&verify=verify_none",
        "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-pdv.dev.megacorp.local/app-five-dev?heartbeat=10&verify=verify_none",
        "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-twelve-pert?heartbeat=10&verify=verify_none",
    ];

    for uri in uris {
        let builder = UriBuilder::new(uri).unwrap();
        let result = builder
            .with_query_param("frame_max", "131072")
            .with_query_param("blocked_connection_timeout", "60000")
            .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
            .with_ca_cert_file("/etc/ssl/certs/common-ca-bundle.crt")
            .with_client_cert_file("/etc/ssl/certs/batch-ops-client.pem")
            .with_client_key_file("/etc/ssl/private/batch-ops-client.key")
            .with_server_name_indication("batch-operations.megacorp.local")
            .build()
            .unwrap();

        // All parameters should be present
        assert!(has_query_param(&result, "heartbeat", "10"));
        assert!(has_query_param(&result, "verify", "verify_peer"));
        assert!(has_query_param(&result, "frame_max", "131072"));
        assert!(has_query_param(
            &result,
            "blocked_connection_timeout",
            "60000"
        ));
        assert!(has_query_param(
            &result,
            "cacertfile",
            "/etc/ssl/certs/common-ca-bundle.crt"
        ));
        assert!(has_query_param(
            &result,
            "certfile",
            "/etc/ssl/certs/batch-ops-client.pem"
        ));
        assert!(has_query_param(
            &result,
            "keyfile",
            "/etc/ssl/private/batch-ops-client.key"
        ));
        assert!(has_query_param(
            &result,
            "server_name_indication",
            "batch-operations.megacorp.local"
        ));
    }
}

#[test]
fn test_integration_case12() {
    // Test URI without verify parameter
    let no_verify = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-four-qa-tup?heartbeat=10";
    let builder = UriBuilder::new(no_verify).unwrap();
    let result = builder
        .with_tls_peer_verification(TlsPeerVerificationMode::Disabled)
        .with_ca_cert_file("/etc/ssl/certs/edge-case-ca.crt")
        .with_client_cert_file("/etc/ssl/certs/edge-case-client.pem")
        .with_client_key_file("/etc/ssl/private/edge-case-client.key")
        .with_server_name_indication("edge-cases.megacorp.local")
        .build()
        .unwrap();

    assert!(has_query_param(&result, "heartbeat", "10"));
    assert!(has_query_param(&result, "verify", "verify_none"));
    assert!(has_query_param(
        &result,
        "cacertfile",
        "/etc/ssl/certs/edge-case-ca.crt"
    ));
    assert!(has_query_param(
        &result,
        "certfile",
        "/etc/ssl/certs/edge-case-client.pem"
    ));
    assert!(has_query_param(
        &result,
        "keyfile",
        "/etc/ssl/private/edge-case-client.key"
    ));
    assert!(has_query_param(
        &result,
        "server_name_indication",
        "edge-cases.megacorp.local"
    ));

    // Test URI with single character vhost extensions
    let single_char = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-five-s4?heartbeat=10&verify=verify_none";
    let builder2 = UriBuilder::new(single_char).unwrap();
    let result2 = builder2
        .with_ca_cert_file("/etc/ssl/certs/single-char-ca.crt")
        .with_client_cert_file("/etc/ssl/certs/single-char-client.pem")
        .with_client_key_file("/etc/ssl/private/single-char-client.key")
        .with_server_name_indication("single-char-apps.megacorp.local")
        .build()
        .unwrap();

    assert!(result2.contains("app-five-s4"));
    assert!(has_query_param(&result2, "heartbeat", "10"));
    assert!(has_query_param(&result2, "verify", "verify_none"));
    assert!(has_query_param(
        &result2,
        "cacertfile",
        "/etc/ssl/certs/single-char-ca.crt"
    ));
    assert!(has_query_param(
        &result2,
        "certfile",
        "/etc/ssl/certs/single-char-client.pem"
    ));
    assert!(has_query_param(
        &result2,
        "keyfile",
        "/etc/ssl/private/single-char-client.key"
    ));
    assert!(has_query_param(
        &result2,
        "server_name_indication",
        "single-char-apps.megacorp.local"
    ));
}

#[test]
fn test_integration_case13() {
    let uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-fifteen-sit?heartbeat=10&verify=verify_none";

    let builder = UriBuilder::new(uri).unwrap();
    let mut builder = builder
        .with_query_param("channel_max", "2047")
        .with_ca_cert_file("/etc/ssl/certs/query-params-test-ca.crt")
        .with_client_cert_file("/etc/ssl/certs/query-params-test-client.pem")
        .with_client_key_file("/etc/ssl/private/query-params-test-client.key")
        .with_server_name_indication("query-params-test.megacorp.local");

    let params = builder.query_params();

    assert_eq!(params.get("heartbeat"), Some(&"10".to_string()));
    assert_eq!(params.get("verify"), Some(&"verify_none".to_string()));
    assert_eq!(params.get("channel_max"), Some(&"2047".to_string()));
    assert_eq!(
        params.get("cacertfile"),
        Some(&"/etc/ssl/certs/query-params-test-ca.crt".to_string())
    );
    assert_eq!(
        params.get("certfile"),
        Some(&"/etc/ssl/certs/query-params-test-client.pem".to_string())
    );
    assert_eq!(
        params.get("keyfile"),
        Some(&"/etc/ssl/private/query-params-test-client.key".to_string())
    );
    assert_eq!(
        params.get("server_name_indication"),
        Some(&"query-params-test.megacorp.local".to_string())
    );
}

#[test]
fn test_integration_case14() {
    let uri = "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-pdv.dev.megacorp.local/app-eighteen?heartbeat=10&verify=verify_none";

    let mut builder = UriBuilder::new(uri)
        .unwrap()
        .with_ca_cert_file("/etc/ssl/certs/url-inspection-ca.crt")
        .with_client_cert_file("/etc/ssl/certs/url-inspection-client.pem")
        .with_client_key_file("/etc/ssl/private/url-inspection-client.key")
        .with_server_name_indication("url-inspection.megacorp.local");

    let url = builder.as_url();

    assert_eq!(url.scheme(), "amqps");
    assert_eq!(
        url.host_str(),
        Some("cluster-abc432-test-pdv.dev.megacorp.local")
    );
    assert_eq!(url.port(), None); // Default AMQPS port
    assert_eq!(url.path(), "/app-eighteen");
    assert_eq!(url.username(), "usern4me");
    assert_eq!(url.password(), Some("r3d4Ct3D"));

    // Verify TLS parameters are in the query string
    let url_string = url.to_string();
    assert!(url_string.contains("cacertfile=/etc/ssl/certs/url-inspection-ca.crt"));
    assert!(url_string.contains("certfile=/etc/ssl/certs/url-inspection-client.pem"));
    assert!(url_string.contains("keyfile=/etc/ssl/private/url-inspection-client.key"));
    assert!(url_string.contains("server_name_indication=url-inspection.megacorp.local"));
}

#[test]
fn test_integration_case15() {
    let environments = vec![
        (
            "dev",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-thirteen-dev?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/dev-ca.crt",
            "/etc/ssl/certs/dev-client.pem",
            "/etc/ssl/private/dev-client.key",
        ),
        (
            "qa",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-six-qa?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/qa-ca.crt",
            "/etc/ssl/certs/qa-client.pem",
            "/etc/ssl/private/qa-client.key",
        ),
        (
            "uat",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-seven-uat?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/uat-ca.crt",
            "/etc/ssl/certs/uat-client.pem",
            "/etc/ssl/private/uat-client.key",
        ),
        (
            "staging",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-eight-staging?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/staging-ca.crt",
            "/etc/ssl/certs/staging-client.pem",
            "/etc/ssl/private/staging-client.key",
        ),
        (
            "intg",
            "amqps://usern4me:r3d4Ct3D@cluster-abc432-test-bdv.dev.megacorp.local/app-three-intg?heartbeat=10&verify=verify_none",
            "/etc/ssl/certs/intg-ca.crt",
            "/etc/ssl/certs/intg-client.pem",
            "/etc/ssl/private/intg-client.key",
        ),
    ];

    for (env_type, uri, ca_cert, client_cert, client_key) in environments {
        let builder = UriBuilder::new(uri).unwrap();
        let result = builder
            .with_query_param("env", env_type)
            .with_ca_cert_file(ca_cert)
            .with_client_cert_file(client_cert)
            .with_client_key_file(client_key)
            .with_server_name_indication(&format!("{}.megacorp.local", env_type))
            .build()
            .unwrap();

        assert!(has_query_param(&result, "heartbeat", "10"));
        assert!(has_query_param(&result, "verify", "verify_none"));
        assert!(has_query_param(&result, "env", env_type));
        assert!(result.contains(&format!("app-")));

        // Verify environment-specific TLS configuration
        assert!(has_query_param(&result, "cacertfile", ca_cert));
        assert!(has_query_param(&result, "certfile", client_cert));
        assert!(has_query_param(&result, "keyfile", client_key));
        assert!(has_query_param(
            &result,
            "server_name_indication",
            &format!("{}.megacorp.local", env_type)
        ));
    }
}
