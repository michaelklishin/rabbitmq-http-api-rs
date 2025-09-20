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

//! URI builders for RabbitMQ connections, particularly federation TLS connections.
//!
//! This module provides convenient builders for creating and modifying RabbitMQ connection URIs
//! with proper TLS configuration parameters as documented in the
//! [RabbitMQ URI Query Parameters Guide](https://www.rabbitmq.com/docs/uri-query-parameters).

use crate::commons::TlsPeerVerificationMode;
use std::collections::HashMap;
use url::Url;

/// A builder for RabbitMQ-specific connection URIs with
/// [TLS settings in query parameters](https://www.rabbitmq.com/docs/federation#tls-connections).
///
/// This builder provides convenient methods for (re)configuring TLS parameters
/// for RabbitMQ federation links and shovels.
///
/// ## On Query String Parameter Encoding and Decoding
///
/// Most URI libraries always percent-encode URI query parameters. However, in the specific case
/// of RabbitMQ URIs, we know that some parameters contain forward slashes,
/// but encoding and decoding them would not be productive:
/// there is no risk of encoded values being misinterpreted.
///
/// Therefore, for compatibility with a broad range of RabbitMQ release series, this implementation
/// intentionally decodes percent-encoded query parameters.
///
/// ## Example
///
/// ```rust
/// use rabbitmq_http_client::uris::UriBuilder;
/// use rabbitmq_http_client::commons::TlsPeerVerificationMode;
///
/// let uri = UriBuilder::new("amqps://user:pass@localhost:5671/vhost")
///     .unwrap()
///     .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
///     .with_ca_cert_file("/path/to/ca_bundle.pem")
///     .with_client_cert_file("/path/to/client.pem")
///     .with_client_key_file("/path/to/key.pem")
///     .with_server_name_indication("myhost.example.com")
///     .build()
///     .unwrap();
///
/// assert!(uri.contains("verify=verify_peer"));
/// assert!(uri.contains("cacertfile=/path/to/ca_bundle.pem"));
/// ```
#[derive(Debug, Clone)]
pub struct UriBuilder {
    url: Url,
    /// Cached decoded query parameters for efficient batch operations
    /// When None, parameters are recalculated from the URL
    cached_params: Option<HashMap<String, String>>,
    /// Have the cached params been modified?
    has_pending_changes: bool,
}

impl UriBuilder {
    const PEER_VERIFICATION_MODE_KEY: &'static str = "verify";
    const CA_CERTIFICATE_BUNDLE_PATH_KEY: &'static str = "cacertfile";
    const CLIENT_CERTIFICATE_PATH_KEY: &'static str = "certfile";
    const CLIENT_PRIVATE_KEY_PATH_KEY: &'static str = "keyfile";
    const SERVER_NAME_INDICATION_KEY: &'static str = "server_name_indication";

    const TLS_PARAMS: &'static [&'static str] = &[
        Self::PEER_VERIFICATION_MODE_KEY,
        Self::CA_CERTIFICATE_BUNDLE_PATH_KEY,
        Self::CLIENT_CERTIFICATE_PATH_KEY,
        Self::CLIENT_PRIVATE_KEY_PATH_KEY,
        Self::SERVER_NAME_INDICATION_KEY,
    ];

    pub fn new(base_uri: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(base_uri)?;
        Ok(Self {
            url,
            cached_params: None,
            has_pending_changes: false,
        })
    }

    /// Sets the [TLS peer verification mode](https://www.rabbitmq.com/docs/ssl#peer-verification).
    pub fn with_tls_peer_verification(mut self, mode: TlsPeerVerificationMode) -> Self {
        self.set_query_param(Self::PEER_VERIFICATION_MODE_KEY, mode.as_ref());
        self
    }

    /// Sets the CA certificate file path for TLS verification.
    pub fn with_ca_cert_file<S: AsRef<str>>(mut self, path: S) -> Self {
        self.set_query_param(Self::CA_CERTIFICATE_BUNDLE_PATH_KEY, path.as_ref());
        self
    }

    /// Sets the client certificate (public key) file path (the `certfile` query parameter)
    pub fn with_client_cert_file<S: AsRef<str>>(mut self, path: S) -> Self {
        self.set_query_param(Self::CLIENT_CERTIFICATE_PATH_KEY, path.as_ref());
        self
    }

    /// Sets the client private key file path (the `keyfile` query parameter).
    pub fn with_client_key_file<S: AsRef<str>>(mut self, path: S) -> Self {
        self.set_query_param(Self::CLIENT_PRIVATE_KEY_PATH_KEY, path.as_ref());
        self
    }

    /// Sets the [server name indication (SNI)](https://www.rabbitmq.com/docs/ssl#erlang-ssl) value using the `server_name_indication` key.
    pub fn with_server_name_indication<S: AsRef<str>>(mut self, hostname: S) -> Self {
        self.set_query_param(Self::SERVER_NAME_INDICATION_KEY, hostname.as_ref());
        self
    }

    /// Sets a custom query parameter.
    pub fn with_query_param<K: AsRef<str>, V: AsRef<str>>(mut self, key: K, value: V) -> Self {
        self.set_query_param(key.as_ref(), value.as_ref());
        self
    }

    /// Removes a query parameter.
    pub fn without_query_param<K: AsRef<str>>(mut self, key: K) -> Self {
        self.remove_query_param(key.as_ref());
        self
    }

    /// Replaces a number of TLS-related settings with new values.
    pub fn replace(mut self, config: TlsClientSettings) -> Self {
        self.ensure_params_cached();
        if let Some(ref mut params) = self.cached_params {
            let any_removed = Self::TLS_PARAMS
                .iter()
                .map(|key| params.remove(*key))
                .any(|removed| removed.is_some());
            if any_removed {
                self.has_pending_changes = true;
            }
        }

        self.apply_tls_settings(&config);

        self
    }

    /// Merges TLS settings into an existing URI, updating only the specified parameters.
    ///
    /// Unlike [`replace`], this method preserves existing TLS-related query parameters that are not
    /// specified (set to [`None`]) in the provided `TlsClientSettings`.
    pub fn merge(mut self, settings: TlsClientSettings) -> Self {
        // Ensure we have cached parameters before applying settings
        self.ensure_params_cached();
        self.apply_tls_settings(&settings);
        self
    }

    /// Returns the final URI string.
    pub fn build(mut self) -> Result<String, url::ParseError> {
        self.apply_cached_params_to_url();
        Ok(self.url.to_string())
    }

    /// Returns the current URL for inspection.
    pub fn as_url(&mut self) -> &Url {
        self.apply_cached_params_to_url();
        &self.url
    }

    /// Returns the query parameters as a [`HashMap`]
    pub fn query_params(&mut self) -> HashMap<String, String> {
        self.apply_cached_params_to_url();
        self.url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    //
    // Implementation
    //

    fn set_query_param(&mut self, key: &str, value: &str) {
        self.ensure_params_cached();

        let Some(ref mut params) = self.cached_params else {
            return;
        };
        params.insert(key.to_string(), value.to_string());
        self.has_pending_changes = true;
    }

    fn remove_query_param(&mut self, key: &str) {
        self.ensure_params_cached();

        let Some(ref mut params) = self.cached_params else {
            return;
        };
        if params.remove(key).is_some() {
            self.has_pending_changes = true;
        }
    }

    fn recalculate_query_params(&self) -> HashMap<String, String> {
        self.url
            .query_pairs()
            .map(|(k, v)| {
                let decoded_key = urlencoding::decode(&k).unwrap_or_else(|_| k.clone());
                let decoded_value = urlencoding::decode(&v).unwrap_or_else(|_| v.clone());
                (decoded_key.into_owned(), decoded_value.into_owned())
            })
            .collect()
    }

    fn rebuild_query_string_from_map(&mut self, params: &HashMap<String, String>) {
        if params.is_empty() {
            self.url.set_query(None);
            return;
        }

        // IMPORTANT: we intentionally build the query string manually to avoid percent-encoding
        let query_string = params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");
        self.url.set_query(Some(&query_string));
    }

    fn ensure_params_cached(&mut self) {
        if self.cached_params.is_none() {
            // Recalculate parameters from the URL, intentionally decoding percent-encoded pairs
            let params = self.recalculate_query_params();
            self.cached_params = Some(params);
        }
    }

    fn apply_cached_params_to_url(&mut self) {
        if !self.has_pending_changes {
            return;
        }

        if let Some(params) = self.cached_params.clone() {
            self.rebuild_query_string_from_map(&params);
        }
        self.has_pending_changes = false;
    }

    fn apply_tls_settings(&mut self, settings: &TlsClientSettings) {
        if let Some(verification) = &settings.peer_verification {
            self.set_query_param(Self::PEER_VERIFICATION_MODE_KEY, verification.as_ref());
        }
        if let Some(ca_cert) = &settings.ca_cert_file {
            self.set_query_param(Self::CA_CERTIFICATE_BUNDLE_PATH_KEY, ca_cert);
        }
        if let Some(client_cert) = &settings.client_cert_file {
            self.set_query_param(Self::CLIENT_CERTIFICATE_PATH_KEY, client_cert);
        }
        if let Some(client_key) = &settings.client_key_file {
            self.set_query_param(Self::CLIENT_PRIVATE_KEY_PATH_KEY, client_key);
        }
        if let Some(sni) = &settings.server_name_indication {
            self.set_query_param(Self::SERVER_NAME_INDICATION_KEY, sni);
        }
    }
}

/// TLS-related setting for RabbitMQ federation upstreams and shovels.
#[derive(Debug, Clone, Default)]
pub struct TlsClientSettings {
    /// TLS peer verification mode
    pub peer_verification: Option<TlsPeerVerificationMode>,
    /// Path to CA certificate file
    pub ca_cert_file: Option<String>,
    /// Path to a client certificate file
    pub client_cert_file: Option<String>,
    /// Path to a client private key file
    pub client_key_file: Option<String>,
    /// Server name indication (SNI) hostname
    pub server_name_indication: Option<String>,
}

impl TlsClientSettings {
    pub fn new() -> Self {
        Self::default()
    }

    /// Defaults with [peer verification](https://www.rabbitmq.com/docs/ssl#peer-verification) enabled.
    pub fn with_verification() -> Self {
        Self {
            peer_verification: Some(TlsPeerVerificationMode::Enabled),
            ..Default::default()
        }
    }

    /// Defaults with [peer verification](https://www.rabbitmq.com/docs/ssl#peer-verification) disabled.
    pub fn without_verification() -> Self {
        Self {
            peer_verification: Some(TlsPeerVerificationMode::Disabled),
            ..Default::default()
        }
    }

    /// Sets the peer verification mode.
    pub fn peer_verification(mut self, mode: TlsPeerVerificationMode) -> Self {
        self.peer_verification = Some(mode);
        self
    }

    /// Sets the CA certificate bundle file path.
    pub fn ca_cert_file<S: Into<String>>(mut self, path: S) -> Self {
        self.ca_cert_file = Some(path.into());
        self
    }

    /// Sets the client certificate file path.
    pub fn client_cert_file<S: Into<String>>(mut self, path: S) -> Self {
        self.client_cert_file = Some(path.into());
        self
    }

    /// Sets the client private key file path.
    pub fn client_key_file<S: Into<String>>(mut self, path: S) -> Self {
        self.client_key_file = Some(path.into());
        self
    }

    /// Sets the server name indication hostname.
    pub fn server_name_indication<S: Into<String>>(mut self, hostname: S) -> Self {
        self.server_name_indication = Some(hostname.into());
        self
    }
}
