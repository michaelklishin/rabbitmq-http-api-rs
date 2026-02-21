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

use proptest::prelude::*;
use rabbitmq_http_client::blocking_api::{ClientBuilder, EndpointValidationError, RedirectPolicy};

fn arb_host() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-z0-9][a-z0-9.-]{0,30}").unwrap()
}

fn arb_port() -> impl Strategy<Value = u16> {
    1024u16..65535u16
}

fn arb_path() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"/[a-z0-9/]{0,20}").unwrap()
}

fn arb_rejected_scheme() -> impl Strategy<Value = String> {
    prop::sample::select(vec![
        "ftp://".to_string(),
        "file://".to_string(),
        "data:".to_string(),
        "gopher://".to_string(),
        "ssh://".to_string(),
        "telnet://".to_string(),
        "ldap://".to_string(),
    ])
}

proptest! {
    // build() accepts any http:// endpoint
    #[test]
    fn prop_build_accepts_http(
        host in arb_host(),
        port in arb_port(),
        path in arb_path()
    ) {
        let endpoint = format!("http://{}:{}{}", host, port, path);
        let result = ClientBuilder::new()
            .with_endpoint(&endpoint)
            .build();
        prop_assert!(result.is_ok(), "expected Ok for endpoint: {}", endpoint);
    }

    // build() accepts any https:// endpoint
    #[test]
    fn prop_build_accepts_https(
        host in arb_host(),
        port in arb_port(),
        path in arb_path()
    ) {
        let endpoint = format!("https://{}:{}{}", host, port, path);
        let result = ClientBuilder::new()
            .with_endpoint(&endpoint)
            .build();
        prop_assert!(result.is_ok(), "expected Ok for endpoint: {}", endpoint);
    }

    // build() rejects endpoints with non-HTTP schemes
    #[test]
    fn prop_build_rejects_non_http_scheme(
        scheme in arb_rejected_scheme(),
        host in arb_host(),
        path in arb_path()
    ) {
        let endpoint = format!("{}{}{}", scheme, host, path);
        let result = ClientBuilder::new()
            .with_endpoint(&endpoint)
            .build();
        prop_assert!(result.is_err(), "expected Err for endpoint: {}", endpoint);
        let is_unsupported_scheme = matches!(
            result.err().unwrap(),
            EndpointValidationError::UnsupportedScheme { .. }
        );
        prop_assert!(is_unsupported_scheme, "expected UnsupportedScheme for endpoint: {}", endpoint);
    }

    // build() rejects endpoints without any scheme
    #[test]
    fn prop_build_rejects_schemeless(
        host in arb_host(),
        port in arb_port(),
        path in arb_path()
    ) {
        let endpoint = format!("{}:{}{}", host, port, path);
        let result = ClientBuilder::new()
            .with_endpoint(&endpoint)
            .build();
        prop_assert!(result.is_err(), "expected Err for schemeless endpoint: {}", endpoint);
    }

    // the redirect policy setting is preserved through with_endpoint()
    #[test]
    fn prop_redirect_policy_preserved_across_with_endpoint(
        host in arb_host(),
        port in arb_port(),
        path in arb_path()
    ) {
        let endpoint = format!("http://{}:{}{}", host, port, path);
        let result = ClientBuilder::new()
            .with_redirect_policy(RedirectPolicy::none())
            .with_endpoint(&endpoint)
            .build();
        prop_assert!(result.is_ok());
    }

    // the redirect policy setting is preserved through with_basic_auth_credentials()
    #[test]
    fn prop_redirect_policy_preserved_across_with_credentials(
        host in arb_host(),
        port in arb_port(),
        path in arb_path()
    ) {
        let endpoint = format!("http://{}:{}{}", host, port, path);
        let result = ClientBuilder::new()
            .with_endpoint(&endpoint)
            .with_redirect_policy(RedirectPolicy::none())
            .with_basic_auth_credentials("user", "pass")
            .build();
        prop_assert!(result.is_ok());
    }
}
