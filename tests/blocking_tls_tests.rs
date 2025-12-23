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

//! TLS integration tests for the blocking client.
//!
//! See `CONTRIBUTING.md` for instructions on how to run these tests.

use rabbitmq_http_client::blocking_api::ClientBuilder;
use reqwest::blocking::Client as HttpClient;
use reqwest::{Certificate, Identity};
use std::fs;

mod test_helpers;
use crate::test_helpers::{
    PASSWORD, TLS_ENDPOINT, USERNAME, ca_cert_path, client_identity_p12_path,
};

/// Build a blocking client with TLS configured using CA certificate validation.
fn build_tls_client()
-> rabbitmq_http_client::blocking_api::Client<&'static str, &'static str, &'static str> {
    let ca_cert = fs::read(ca_cert_path()).expect("Failed to read CA certificate");
    let ca_cert = Certificate::from_pem(&ca_cert).expect("Failed to parse CA certificate");

    let http_client = HttpClient::builder()
        .add_root_certificate(ca_cert)
        .build()
        .expect("Failed to build HTTP client");

    ClientBuilder::new()
        .with_endpoint(TLS_ENDPOINT)
        .with_basic_auth_credentials(USERNAME, PASSWORD)
        .with_client(http_client)
        .build()
}

/// Build a blocking client with TLS configured using client certificate authentication.
/// Uses PKCS#12 format which is compatible with native-tls.
fn build_tls_client_with_cert()
-> rabbitmq_http_client::blocking_api::Client<&'static str, &'static str, &'static str> {
    let ca_cert = fs::read(ca_cert_path()).expect("Failed to read CA certificate");
    let ca_cert = Certificate::from_pem(&ca_cert).expect("Failed to parse CA certificate");

    let p12_data = fs::read(client_identity_p12_path()).expect("Failed to read client PKCS#12");
    // The PKCS#12 file is created with an empty password in CI
    let identity =
        Identity::from_pkcs12_der(&p12_data, "").expect("Failed to parse client identity");

    let http_client = HttpClient::builder()
        .add_root_certificate(ca_cert)
        .identity(identity)
        .build()
        .expect("Failed to build HTTP client");

    ClientBuilder::new()
        .with_endpoint(TLS_ENDPOINT)
        .with_basic_auth_credentials(USERNAME, PASSWORD)
        .with_client(http_client)
        .build()
}

/// Build a blocking client with TLS that skips certificate verification (insecure).
fn build_tls_client_insecure()
-> rabbitmq_http_client::blocking_api::Client<&'static str, &'static str, &'static str> {
    let http_client = HttpClient::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to build HTTP client");

    ClientBuilder::new()
        .with_endpoint(TLS_ENDPOINT)
        .with_basic_auth_credentials(USERNAME, PASSWORD)
        .with_client(http_client)
        .build()
}

#[test]
#[ignore]
fn test_blocking_tls_overview() {
    let rc = build_tls_client();
    let result = rc.overview();
    assert!(result.is_ok(), "overview returned {:?}", result);

    let ov = result.unwrap();
    assert!(ov.object_totals.exchanges > 0);
}

#[test]
#[ignore]
fn test_blocking_tls_overview_insecure() {
    let rc = build_tls_client_insecure();
    let result = rc.overview();
    assert!(result.is_ok(), "overview returned {:?}", result);

    let ov = result.unwrap();
    assert!(ov.object_totals.exchanges > 0);
}

#[test]
#[ignore]
fn test_blocking_tls_list_nodes() {
    let rc = build_tls_client();
    let result = rc.list_nodes();
    assert!(result.is_ok(), "list_nodes returned {:?}", result);

    let vec = result.unwrap();
    assert!(vec.iter().any(|n| n.name.starts_with("rabbit@")));
}

#[test]
#[ignore]
fn test_blocking_tls_list_vhosts() {
    let rc = build_tls_client();
    let result = rc.list_vhosts();
    assert!(result.is_ok(), "list_vhosts returned {:?}", result);

    let vec = result.unwrap();
    assert!(vec.iter().any(|vh| vh.name == "/"));
}

#[test]
#[ignore]
fn test_blocking_tls_get_vhost() {
    let rc = build_tls_client();
    let result = rc.get_vhost("/");
    assert!(result.is_ok(), "get_vhost returned {:?}", result);

    let vh = result.unwrap();
    assert_eq!(vh.name, "/");
}

#[test]
#[ignore]
fn test_blocking_tls_list_users() {
    let rc = build_tls_client();
    let result = rc.list_users();
    assert!(result.is_ok(), "list_users returned {:?}", result);

    let vec = result.unwrap();
    assert!(vec.iter().any(|u| u.name == "guest"));
}

#[test]
#[ignore]
fn test_blocking_tls_get_user() {
    let rc = build_tls_client();
    let result = rc.get_user("guest");
    assert!(result.is_ok(), "get_user returned {:?}", result);

    let u = result.unwrap();
    assert_eq!(u.name, "guest");
}

#[test]
#[ignore]
fn test_blocking_tls_current_user() {
    let rc = build_tls_client();
    let result = rc.current_user();
    assert!(result.is_ok(), "current_user returned {:?}", result);

    let u = result.unwrap();
    assert_eq!(u.name, "guest");
}

#[test]
#[ignore]
fn test_blocking_tls_health_check_cluster_wide_alarms() {
    let rc = build_tls_client();
    let result = rc.health_check_cluster_wide_alarms();
    assert!(result.is_ok(), "health_check returned {:?}", result);
}

#[test]
#[ignore]
fn test_blocking_tls_health_check_local_alarms() {
    let rc = build_tls_client();
    let result = rc.health_check_local_alarms();
    assert!(result.is_ok(), "health_check returned {:?}", result);
}

#[test]
#[ignore]
fn test_blocking_tls_list_queues() {
    let rc = build_tls_client();
    let result = rc.list_queues();
    assert!(result.is_ok(), "list_queues returned {:?}", result);
}

#[test]
#[ignore]
fn test_blocking_tls_list_exchanges() {
    let rc = build_tls_client();
    let result = rc.list_exchanges();
    assert!(result.is_ok(), "list_exchanges returned {:?}", result);

    let vec = result.unwrap();
    // Default exchanges should exist
    assert!(vec.iter().any(|e| e.name == "amq.direct"));
}

#[test]
#[ignore]
fn test_blocking_tls_list_connections() {
    let rc = build_tls_client();
    let result = rc.list_connections();
    assert!(result.is_ok(), "list_connections returned {:?}", result);
}

#[test]
#[ignore]
fn test_blocking_tls_with_client_certificate() {
    let rc = build_tls_client_with_cert();
    let result = rc.overview();
    assert!(
        result.is_ok(),
        "overview with client cert returned {:?}",
        result
    );

    let ov = result.unwrap();
    assert!(ov.object_totals.exchanges > 0);
}
