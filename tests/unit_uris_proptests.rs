// Copyright (C) 2023-2025 RabbitMQ Core Team (teamrabbitmq@gmail.com)
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied
// See the License for the specific language governing permissions and
// limitations under the License

use proptest::prelude::*;
use rabbitmq_http_client::commons::TlsPeerVerificationMode;
use rabbitmq_http_client::uris::{TlsClientSettings, UriBuilder};
use std::collections::HashMap;
use url::Url;

fn arb_host() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9.-]+").unwrap()
}

fn arb_path() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"/[a-zA-Z0-9/]*").unwrap()
}

fn arb_query_key() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9_]+").unwrap()
}

fn arb_query_value() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9_.-/]+").unwrap()
}

fn arb_abs_path() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"/[a-zA-Z0-9/._-]+\.pem").unwrap()
}

fn arb_hostname() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+local")
        .unwrap()
        .prop_filter("hostname length >= 5", |s| s.len() >= 5)
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

            assert_eq!(
                params.get(UriBuilder::PEER_VERIFICATION_MODE_KEY),
                Some(&TlsPeerVerificationMode::Enabled.as_ref().to_string())
            );
            assert_eq!(params.get("cacertfile"), Some(&"/new/ca.pem".to_string()));
        }
    }

    #[test]
    fn test_merge_tls_settings_proptest(
        host in arb_host(),
        path in arb_path()
    ) {
        let base_uri = format!(
            "amqps://user:pass@{}:5671{}?{}={}&cacertfile=/old/ca.pem",
            host,
            path,
            UriBuilder::PEER_VERIFICATION_MODE_KEY,
            TlsPeerVerificationMode::Disabled.as_ref()
        );
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let settings = TlsClientSettings::new()
                .peer_verification(TlsPeerVerificationMode::Enabled)
                .client_cert_file("/new/client.pem");

            let result = builder.merge(settings).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            assert_eq!(
                params.get(UriBuilder::PEER_VERIFICATION_MODE_KEY),
                Some(&TlsPeerVerificationMode::Enabled.as_ref().to_string())
            );
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
                if !k.is_empty() && k != key {
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

    #[test]
    fn test_with_server_name_indication_proptest(
        host in arb_host(),
        path in arb_path(),
        sni in arb_hostname()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder.with_server_name_indication(&sni).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();
            assert_eq!(params.get("server_name_indication"), Some(&sni));
        }
    }

    #[test]
    fn test_replace_removes_unspecified_tls_and_preserves_non_tls_proptest(
        host in arb_host(),
        path in arb_path(),
        old_ca in arb_abs_path(),
        old_cert in arb_abs_path(),
        old_key in arb_abs_path(),
        old_sni in arb_hostname(),
        new_ca in arb_abs_path(),
        non_key in arb_query_key(),
        non_val in arb_query_value()
    ) {
        // Ensure non_key does not collide with TLS keys
        let tls_keys = ["verify", "cacertfile", "certfile", "keyfile", "server_name_indication"];
        let non_key_final = if tls_keys.contains(&non_key.as_str()) {
            format!("{}{}", non_key, "_extra")
        } else {
            non_key.clone()
        };

        let base_uri = format!(
            "amqps://user:pass@{}:5671{}?{}={}&cacertfile={}&certfile={}&keyfile={}&server_name_indication={}&{}={}",
            host,
            path,
            UriBuilder::PEER_VERIFICATION_MODE_KEY,
            TlsPeerVerificationMode::Enabled.as_ref(),
            old_ca,
            old_cert,
            old_key,
            old_sni,
            non_key_final,
            non_val
        );
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            // Uses only a subset of TLS settings; unspecified TLS keys must be removed
            let settings = TlsClientSettings::new()
                .peer_verification(TlsPeerVerificationMode::Disabled)
                .ca_cert_file(&new_ca);

            let result = builder.replace(settings).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            // Provided TLS params updated
            assert_eq!(
                params.get(UriBuilder::PEER_VERIFICATION_MODE_KEY),
                Some(&TlsPeerVerificationMode::Disabled.as_ref().to_string())
            );
            assert_eq!(params.get("cacertfile"), Some(&new_ca));

            // Unspecified TLS params removed or preserved unchanged
            if let Some(v) = params.get("certfile") {
                assert_eq!(v, &old_cert);
            }
            if let Some(v) = params.get("keyfile") {
                assert_eq!(v, &old_key);
            }
            if let Some(v) = params.get("server_name_indication") {
                assert_eq!(v, &old_sni);
            }

            // Non-TLS params preserved
            assert_eq!(params.get(&non_key_final), Some(&non_val));
        }
    }

    #[test]
    fn test_merge_overrides_and_preserves_tls_and_non_tls_proptest(
        host in arb_host(),
        path in arb_path(),
        old_ca in arb_abs_path(),
        old_cert in arb_abs_path(),
        old_key in arb_abs_path(),
        old_sni in arb_hostname(),
        new_cert in arb_abs_path(),
        new_sni in arb_hostname(),
        non_key in arb_query_key(),
        non_val in arb_query_value()
    ) {
        // Ensure non_key does not collide with TLS keys
        let tls_keys = ["verify", "cacertfile", "certfile", "keyfile", "server_name_indication"];
        let non_key_final = if tls_keys.contains(&non_key.as_str()) {
            format!("{}{}", non_key, "_extra")
        } else {
            non_key.clone()
        };

        let base_uri = format!(
            "amqps://user:pass@{}:5671{}?{}={}&cacertfile={}&certfile={}&keyfile={}&server_name_indication={}&{}={}",
            host,
            path,
            UriBuilder::PEER_VERIFICATION_MODE_KEY,
            TlsPeerVerificationMode::Disabled.as_ref(),
            old_ca,
            old_cert,
            old_key,
            old_sni,
            non_key_final,
            non_val
        );
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            // Merge only the certificate file and the SNI; verify/cacertfile/keyfile should be preserved
            let settings = TlsClientSettings::new()
                .client_cert_file(&new_cert)
                .server_name_indication(&new_sni);

            let result = builder.merge(settings).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            // Unspecified TLS params preserved
            assert_eq!(
                params.get(UriBuilder::PEER_VERIFICATION_MODE_KEY),
                Some(&TlsPeerVerificationMode::Disabled.as_ref().to_string())
            );
            assert_eq!(params.get("cacertfile"), Some(&old_ca));
            assert_eq!(params.get("keyfile"), Some(&old_key));

            // Specified TLS params updated
            assert_eq!(params.get("certfile"), Some(&new_cert));
            assert_eq!(params.get("server_name_indication"), Some(&new_sni));

            // Non-TLS params preserved
            assert_eq!(params.get(&non_key_final), Some(&non_val));
        }
    }

    #[test]
    fn test_with_tls_peer_verification_proptest(
        host in arb_host(),
        path in arb_path(),
        enabled in proptest::bool::ANY
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let mode = if enabled {
                TlsPeerVerificationMode::Enabled
            } else {
                TlsPeerVerificationMode::Disabled
            };
            let expected = if enabled {
                TlsPeerVerificationMode::Enabled
            } else {
                TlsPeerVerificationMode::Disabled
            };

            let result = builder.with_tls_peer_verification(mode).build().unwrap();
            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            assert_eq!(
                params.get(UriBuilder::PEER_VERIFICATION_MODE_KEY),
                Some(&expected.as_ref().to_string())
            );
        }
    }

    #[test]
    fn test_last_write_wins_for_same_param_proptest(
        host in arb_host(),
        path in arb_path(),
        first in arb_abs_path(),
        second in arb_abs_path()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}", host, path);
        if let Ok(builder) = UriBuilder::new(&base_uri) {
            let result = builder
                .with_ca_cert_file(&first)
                .with_ca_cert_file(&second)
                .build()
                .unwrap();

            let url = Url::parse(&result).unwrap();
            let params: HashMap<_, _> = url.query_pairs().into_owned().collect();

            assert_eq!(params.get("cacertfile"), Some(&second));
        }
    }

    #[test]
    fn test_as_url_and_query_params_apply_cached_changes_proptest(
        host in arb_host(),
        path in arb_path(),
        key in arb_query_key(),
        value in arb_query_value()
    ) {
        let base_uri = format!("amqps://user:pass@{}:5671{}", host, path);
        if let Ok(mut builder) = UriBuilder::new(&base_uri) {
            // Stage a pending change
            builder = builder.with_query_param(&key, &value);

            // as_url should apply pending changes to the internal Url
            let url_snapshot = builder.as_url().clone();
            let params_from_url: HashMap<_, _> = url_snapshot.query_pairs().into_owned().collect();
            assert_eq!(params_from_url.get(&key), Some(&value));

            // query_params should reflect the same applied changes
            let params_map = builder.query_params();
            assert_eq!(params_map.get(&key), Some(&value));
        }
    }
}
