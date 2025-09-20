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
use rabbitmq_http_client::commons::TlsPeerVerificationMode;
use rabbitmq_http_client::uris::{TlsClientSettings, UriBuilder};
use std::collections::HashMap;
use url::Url;

fn arb_host() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9.-]+"
}

fn arb_path() -> impl Strategy<Value = String> {
    "/[a-zA-Z0-9/]*"
}

fn arb_query_key() -> impl Strategy<Value = String> {
    "[a-zA-Z0-9_]+"
}

fn arb_query_value() -> impl Strategy<Value = String> {
    r"[a-zA-Z0-9_.-/]+"
}

fn arb_abs_path() -> impl Strategy<Value = String> {
    r"/[a-zA-Z0-9/._-]+\.pem"
}

fn arb_query_params() -> impl Strategy<Value = HashMap<String, String>> {
    prop::collection::hash_map(arb_query_key(), arb_query_value(), 0..5)
}

proptest! {
    #[test]
    fn test_uri_builder_proptest(
        host in arb_host(),
        path in arb_path(),
        key in arb_query_key(),
        value in arb_query_value()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder.with_query_param(&key, &value).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
            assert_eq!(params.get(&key), Some(&value));
        }
    }

    #[test]
    fn test_without_query_param_proptest(
        host in arb_host(),
        path in arb_path(),
        key in arb_query_key(),
        value in arb_query_value()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}?{}={}", host, path, key, value);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder.without_query_param(&key).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
            assert!(!params.contains_key(&key));
        }
    }

    #[test]
    fn test_replace_tls_settings_proptest(
        host in arb_host(),
        path in arb_path()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let settings = TlsClientSettings::new()
                .peer_verification(TlsPeerVerificationMode::Enabled)
                .ca_cert_file("/new/ca.pem");

            let result = builder.replace(settings).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            assert_eq!(params.get("verify"), Some(&"verify_peer".to_string()));
            assert_eq!(params.get("cacertfile"), Some(&"/new/ca.pem".to_string()));
        }
    }

    #[test]
    fn test_merge_tls_settings_proptest(
        host in arb_host(),
        path in arb_path()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}?verify=verify_none&cacertfile=/old/ca.pem", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let settings = TlsClientSettings::new()
                .peer_verification(TlsPeerVerificationMode::Enabled)
                .client_cert_file("/new/client.pem");

            let result = builder.merge(settings).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            assert_eq!(params.get("verify"), Some(&"verify_peer".to_string()));
            assert_eq!(params.get("cacertfile"), Some(&"/old/ca.pem".to_string()));
            assert_eq!(params.get("certfile"), Some(&"/new/client.pem".to_string()));
        }
    }

    #[test]
    fn test_with_existing_params_proptest(
        host in arb_host(),
        path in arb_path(),
        initial_params in arb_query_params(),
        key in arb_query_key(),
        value in arb_query_value()
    ) {
        let query_string: String = initial_params.iter().map(|(k, v)| format!("{}={}", k, v)).collect::<Vec<_>>().join("&");
        let base_uri = format!("amqps://user:pass@{}:5671{}?{}", host, path, query_string);

        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder.with_query_param(&key, &value).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            for (k, v) in initial_params {
                if !k.is_empty() {
                    assert_eq!(params.get(&k), Some(&v));
                }
            }
            assert_eq!(params.get(&key), Some(&value));
        }
    }

    #[test]
    fn test_intentional_percent_decoding_proptest(host in arb_host(), path in arb_path()) {
        let base_uri = format!("amqps://user:pass@{}:5671{}?cacertfile=%2Fpath%2Fto%2Fca.pem", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder.with_query_param("dummy", "dummy").without_query_param("dummy").build().unwrap();
            assert!(result.contains("cacertfile=/path/to/ca.pem"));
        }
    }

    #[test]
    fn test_with_random_paths(
        host in arb_host(),
        path in arb_path(),
        ca_path in arb_abs_path(),
        cert_path in arb_abs_path(),
        key_path in arb_abs_path()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder
                .with_ca_cert_file(&ca_path)
                .with_client_cert_file(&cert_path)
                .with_client_key_file(&key_path)
                .build()
                .unwrap();

            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            assert_eq!(params.get("cacertfile"), Some(&ca_path));
            assert_eq!(params.get("certfile"), Some(&cert_path));
            assert_eq!(params.get("keyfile"), Some(&key_path));
        }
    }

    #[test]
    fn test_path_overwrite(
        host in arb_host(),
        path in arb_path(),
        initial_ca_path in arb_abs_path(),
        new_ca_path in arb_abs_path(),
        initial_cert_path in arb_abs_path(),
        new_cert_path in arb_abs_path(),
        initial_key_path in arb_abs_path(),
        new_key_path in arb_abs_path()
    ) {
        let query_string = format!("cacertfile={}&certfile={}&keyfile={}", initial_ca_path, initial_cert_path, initial_key_path);
        let base_uri = format!("amqps://user:pass@{}:5671{}?{}", host, path, query_string);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder
                .with_ca_cert_file(&new_ca_path)
                .with_client_cert_file(&new_cert_path)
                .with_client_key_file(&new_key_path)
                .build()
                .unwrap();

            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            assert_eq!(params.get("cacertfile"), Some(&new_ca_path));
            assert_eq!(params.get("certfile"), Some(&new_cert_path));
            assert_eq!(params.get("keyfile"), Some(&new_key_path));
        }
    }
}
