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

use std::collections::HashMap;
use rabbitmq_http_client::commons::TlsPeerVerificationMode;
use rabbitmq_http_client::uris::{TlsClientSettings, UriBuilder};
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
