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

use std::borrow::Cow;
use crate::commons::TlsPeerVerificationMode;
use std::collections::HashMap;
use url::{EncodingOverride, Url};

/// A builder for RabbitMQ-specific connection URIs with
/// [TLS settings in query parameters](https://www.rabbitmq.com/docs/federation#tls-connections).
///
/// This builder provides convenient methods for (re)configuring TLS parameters
/// for RabbitMQ federation links and shovels.
///
/// # Example
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
}

impl UriBuilder {
    /// Creates a new URI builder from a base URI string.
    pub fn new(base_uri: &str) -> Result<Self, url::ParseError> {
        let url = Url::parse(base_uri)?;
        Ok(Self { url })
    }

    pub fn new_with_encoding_override(base_uri: &str, encoder: EncodingOverride) -> Result<Self, url::ParseError> {
        let url = Url::options()
            .encoding_override(encoder)
            .parse(base_uri)?;

        Ok(Self { url })
    }

    pub fn new_with_passthrough_encoder(base_uri: &str) -> Result<Self, url::ParseError> {
        let f: &dyn Fn(&str) -> Cow<'_, [u8]> = &passthrough_encoder_fn;
        let encoder: EncodingOverride = Some(f);

        Self::new_with_encoding_override(base_uri, encoder)
    }

    /// Sets the [TLS peer verification mode](https://www.rabbitmq.com/docs/ssl#peer-verification).
    pub fn with_tls_peer_verification(mut self, mode: TlsPeerVerificationMode) -> Self {
        self.set_query_param("verify", mode.as_ref());
        self
    }

    /// Sets the CA certificate file path for TLS verification.
    pub fn with_ca_cert_file<S: AsRef<str>>(mut self, path: S) -> Self {
        self.set_query_param("cacertfile", path.as_ref());
        self
    }

    /// Sets the client certificate (public key) file path (the `certfile` query parameter)
    pub fn with_client_cert_file<S: AsRef<str>>(mut self, path: S) -> Self {
        self.set_query_param("certfile", path.as_ref());
        self
    }

    /// Sets the client private key file path (the `keyfile` query parameter).
    pub fn with_client_key_file<S: AsRef<str>>(mut self, path: S) -> Self {
        self.set_query_param("keyfile", path.as_ref());
        self
    }

    /// Sets the [server name indication (SNI)](https://www.rabbitmq.com/docs/ssl#erlang-ssl) value using the `server_name_indication` key.
    pub fn with_server_name_indication<S: AsRef<str>>(mut self, hostname: S) -> Self {
        self.set_query_param("server_name_indication", hostname.as_ref());
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
        const TLS_PARAMS: &[&str] = &[
            "verify",
            "cacertfile",
            "certfile",
            "keyfile",
            "server_name_indication",
        ];
        let non_tls_params: Vec<(String, String)> = self
            .url
            .query_pairs()
            .filter(|(k, _)| !TLS_PARAMS.contains(&k.as_ref()))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();

        self.url.set_query(None);

        for (k, v) in non_tls_params {
            self.url.query_pairs_mut().append_pair(&k, &v);
        }

        if let Some(verification) = config.peer_verification {
            self.url
                .query_pairs_mut()
                .append_pair("verify", verification.as_ref());
        }
        if let Some(ca_cert) = config.ca_cert_file {
            self.url
                .query_pairs_mut()
                .append_pair("cacertfile", &ca_cert);
        }
        if let Some(client_cert) = config.client_cert_file {
            self.url
                .query_pairs_mut()
                .append_pair("certfile", &client_cert);
        }
        if let Some(client_key) = config.client_key_file {
            self.url
                .query_pairs_mut()
                .append_pair("keyfile", &client_key);
        }
        if let Some(sni) = config.server_name_indication {
            self.url
                .query_pairs_mut()
                .append_pair("server_name_indication", &sni);
        }

        self
    }

    /// Merges TLS settings into an existing URI, updating only the specified parameters.
    ///
    /// Unlike [`replace`], this method preserves existing TLS-related query parameters that are not
    /// specified (set to [`None`]) in the provided `TlsClientSettings`.
    pub fn merge(mut self, settings: TlsClientSettings) -> Self {
        let mut params: HashMap<String, String> = self.url.query_pairs().into_owned().collect();

        if let Some(verification) = settings.peer_verification {
            params.insert("verify".to_string(), verification.as_ref().to_string());
        }
        if let Some(ca_cert) = settings.ca_cert_file {
            params.insert("cacertfile".to_string(), ca_cert);
        }
        if let Some(client_cert) = settings.client_cert_file {
            params.insert("certfile".to_string(), client_cert);
        }
        if let Some(client_key) = settings.client_key_file {
            params.insert("keyfile".to_string(), client_key);
        }
        if let Some(sni) = settings.server_name_indication {
            params.insert("server_name_indication".to_string(), sni);
        }

        self.url.set_query(None);
        self.url.query_pairs_mut().extend_pairs(params);

        self
    }

    /// Returns the final URI string.
    pub fn build(self) -> Result<String, url::ParseError> {
        Ok(self.url.to_string())
    }

    /// Returns the current URL for inspection.
    pub fn as_url(&self) -> &Url {
        &self.url
    }

    /// Gets all current query parameters as a map.
    pub fn query_params(&self) -> HashMap<String, String> {
        self.url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect()
    }

    //
    // Implementation
    //

    fn set_query_param(&mut self, key: &str, value: &str) {
        // Collect existing parameters first, filtering out the key we're replacing
        let mut params: Vec<(String, String)> = self
            .url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .filter(|(k, _)| k != key)
            .collect();

        // Add the new parameter
        params.push((key.to_string(), value.to_string()));

        for (k, v) in params {
            self.url.query_pairs_mut().append_pair(&k, &v);
        }
    }

    fn remove_query_param(&mut self, key: &str) {
        let params: Vec<(String, String)> = self
            .url
            .query_pairs()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .filter(|(k, _)| k != key)
            .collect();

        self.url.set_query(None);
        for (k, v) in params {
            self.url.query_pairs_mut().append_pair(&k, &v);
        }
    }
}

pub fn passthrough_encoder_fn(s: &str) -> Cow<'_, [u8]> {
    Cow::Borrowed(s.as_bytes())
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
