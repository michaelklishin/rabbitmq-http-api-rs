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

use rabbitmq_http_client::api::{ClientBuilder, EndpointValidationError, RedirectPolicy};
use std::time::Duration;

//
// Scheme validation
//

#[test]
fn test_async_build_accepts_http() {
    let result = ClientBuilder::new()
        .with_endpoint("http://localhost:15672/api")
        .build();
    assert!(result.is_ok());
}

#[test]
fn test_async_build_accepts_https() {
    let result = ClientBuilder::new()
        .with_endpoint("https://rabbitmq.example.com:15672/api")
        .build();
    assert!(result.is_ok());
}

#[test]
fn test_async_build_rejects_ftp_scheme() {
    let result = ClientBuilder::new()
        .with_endpoint("ftp://example.com/api")
        .build();
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        EndpointValidationError::UnsupportedScheme { .. }
    ));
}

#[test]
fn test_async_build_rejects_file_scheme() {
    let result = ClientBuilder::new()
        .with_endpoint("file:///etc/passwd")
        .build();
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        EndpointValidationError::UnsupportedScheme { .. }
    ));
}

#[test]
fn test_async_build_rejects_empty_endpoint() {
    let result = ClientBuilder::new().with_endpoint("").build();
    assert!(result.is_err());
}

#[test]
fn test_async_build_rejects_schemeless_endpoint() {
    let result = ClientBuilder::new()
        .with_endpoint("localhost:15672/api")
        .build();
    assert!(result.is_err());
}

#[test]
fn test_async_build_rejects_data_scheme() {
    let result = ClientBuilder::new()
        .with_endpoint("data:text/html,<h1>hi</h1>")
        .build();
    assert!(result.is_err());
}

#[test]
fn test_async_build_rejects_javascript_scheme() {
    let result = ClientBuilder::new()
        .with_endpoint("javascript:alert(1)")
        .build();
    assert!(result.is_err());
}

#[test]
fn test_async_error_message_includes_endpoint() {
    let result = ClientBuilder::new()
        .with_endpoint("ftp://evil.example.com")
        .build();
    let err = result.err().unwrap();
    let msg = err.to_string();
    assert!(msg.contains("ftp://evil.example.com"));
    assert!(msg.contains("http://"));
    assert!(msg.contains("https://"));
}

//
// Redirect policy
//

#[test]
fn test_async_with_redirect_policy() {
    let result = ClientBuilder::new()
        .with_redirect_policy(RedirectPolicy::none())
        .build();
    assert!(result.is_ok());
}

#[test]
fn test_async_with_redirect_policy_limited() {
    let result = ClientBuilder::new()
        .with_redirect_policy(RedirectPolicy::limited(5))
        .build();
    assert!(result.is_ok());
}

#[test]
fn test_async_redirect_policy_overrides_recommended_defaults() {
    let result = ClientBuilder::new()
        .with_recommended_defaults()
        .with_redirect_policy(RedirectPolicy::limited(3))
        .build();
    assert!(result.is_ok());
}

//
// Recommended defaults
//

#[test]
fn test_async_recommended_defaults_builds_successfully() {
    let result = ClientBuilder::new().with_recommended_defaults().build();
    assert!(result.is_ok());
}

//
// Custom client bypasses builder-level redirect policy
//

#[test]
fn test_async_custom_client_ignores_redirect_policy() {
    let custom_client = reqwest::Client::new();
    let result = ClientBuilder::new()
        .with_redirect_policy(RedirectPolicy::none())
        .with_client(custom_client)
        .build();
    assert!(result.is_ok());
}

#[test]
fn test_async_custom_client_still_validates_endpoint_scheme() {
    let custom_client = reqwest::Client::new();
    let result = ClientBuilder::new()
        .with_endpoint("ftp://example.com/api")
        .with_client(custom_client)
        .build();
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        EndpointValidationError::UnsupportedScheme { .. }
    ));
}

//
// Builder method chaining preserves redirect_policy
//

#[test]
fn test_async_redirect_policy_preserved_across_with_endpoint() {
    let result = ClientBuilder::new()
        .with_redirect_policy(RedirectPolicy::none())
        .with_endpoint("http://localhost:15672/api")
        .build();
    assert!(result.is_ok());
}

#[test]
fn test_async_redirect_policy_preserved_across_with_credentials() {
    let result = ClientBuilder::new()
        .with_redirect_policy(RedirectPolicy::none())
        .with_basic_auth_credentials("user", "pass")
        .build();
    assert!(result.is_ok());
}

#[test]
fn test_async_all_builder_settings_combined() {
    let result = ClientBuilder::new()
        .with_endpoint("https://rabbitmq.example.com:15672/api")
        .with_basic_auth_credentials("admin", "secret")
        .with_connect_timeout(Duration::from_secs(5))
        .with_request_timeout(Duration::from_secs(30))
        .with_redirect_policy(RedirectPolicy::none())
        .build();
    assert!(result.is_ok());
}

//
// Owned String endpoints
//

#[test]
fn test_async_build_accepts_owned_string_endpoint() {
    let endpoint = String::from("http://localhost:15672/api");
    let result = ClientBuilder::new().with_endpoint(endpoint).build();
    assert!(result.is_ok());
}

#[test]
fn test_async_build_rejects_owned_string_ftp_endpoint() {
    let endpoint = String::from("ftp://example.com/api");
    let result = ClientBuilder::new().with_endpoint(endpoint).build();
    assert!(result.is_err());
    assert!(matches!(
        result.err().unwrap(),
        EndpointValidationError::UnsupportedScheme { .. }
    ));
}

//
// Scheme validation is case-sensitive (http:// and https:// only)
//

#[test]
fn test_async_build_rejects_uppercase_http() {
    let result = ClientBuilder::new()
        .with_endpoint("HTTP://localhost:15672/api")
        .build();
    assert!(result.is_err());
}

#[test]
fn test_async_build_rejects_mixed_case_https() {
    let result = ClientBuilder::new()
        .with_endpoint("Https://localhost:15672/api")
        .build();
    assert!(result.is_err());
}
