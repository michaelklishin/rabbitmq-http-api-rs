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
#![allow(clippy::result_large_err)]

use crate::commons::RetrySettings;
use crate::error::{
    Error::{ClientErrorResponse, NotFound, ServerErrorResponse},
    ErrorDetails,
};
use backtrace::Backtrace;
use log::trace;
use reqwest::{Client as HttpClient, StatusCode, header::HeaderMap};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Display;
use std::future::Future;
use std::result;
use std::time::Duration;
use tokio::time::sleep;

pub type HttpClientResponse = reqwest::Response;
pub type HttpClientError = crate::error::HttpClientError;

pub type Result<T> = result::Result<T, HttpClientError>;

/// A `ClientBuilder` can be used to create a `Client` with custom configuration.
///
/// Example
/// ```rust
/// use rabbitmq_http_client::api::ClientBuilder;
///
/// let endpoint = "http://localhost:15672/api";
/// let username = "username";
/// let password = "password";
/// let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();
/// // list cluster nodes
/// rc.list_nodes().await;
/// // list user connections
/// rc.list_connections().await;
/// // fetch information and metrics of a specific queue
/// rc.get_queue_info("/", "qq.1").await;
/// ```
pub struct ClientBuilder<E = &'static str, U = &'static str, P = &'static str> {
    endpoint: E,
    username: U,
    password: P,
    client: Option<HttpClient>,
    retry_settings: RetrySettings,
    request_timeout: Option<Duration>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Constructs a new `ClientBuilder` with default settings.
    ///
    /// The default configuration uses `http://localhost:15672/api` as the endpoint
    /// and `guest/guest` as the credentials. This is the same as calling `Client::builder()`.
    ///
    /// Note that the default credentials are [limited to local connections](https://www.rabbitmq.com/docs/access-control)
    /// for security reasons.
    pub fn new() -> Self {
        Self {
            endpoint: "http://localhost:15672/api",
            username: "guest",
            password: "guest",
            client: None,
            retry_settings: RetrySettings::default(),
            request_timeout: None,
        }
    }
}

impl<E, U, P> ClientBuilder<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Sets the API credentials.
    pub fn with_basic_auth_credentials<NewU, NewP>(
        self,
        username: NewU,
        password: NewP,
    ) -> ClientBuilder<E, NewU, NewP>
    where
        NewU: Display,
        NewP: Display,
    {
        ClientBuilder {
            endpoint: self.endpoint,
            username,
            password,
            client: self.client,
            retry_settings: self.retry_settings,
            request_timeout: self.request_timeout,
        }
    }

    /// Sets the HTTP API endpoint URL.
    ///
    /// The endpoint should include the scheme, host, port, and API endpoint path.
    /// Some examples: `http://localhost:15672/api` or `https://rabbitmq.example.com:15672/api`.
    pub fn with_endpoint<T>(self, endpoint: T) -> ClientBuilder<T, U, P>
    where
        T: Display,
    {
        ClientBuilder {
            endpoint,
            username: self.username,
            password: self.password,
            client: self.client,
            retry_settings: self.retry_settings,
            request_timeout: self.request_timeout,
        }
    }

    /// Sets a custom HTTP client.
    ///
    /// Use a custom HTTP client to configure custom timeouts, proxy settings, TLS configuration.
    ///
    /// Note: If you provide a custom client, the timeout set via [`with_request_timeout`]
    /// will be ignored. Configure timeouts directly on your custom client instead.
    pub fn with_client(self, client: HttpClient) -> Self {
        ClientBuilder {
            client: Some(client),
            ..self
        }
    }

    /// Sets the request timeout for HTTP operations.
    ///
    /// This timeout applies to the entire request/response cycle, including connection establishment,
    /// request transmission, and response receipt. If a request takes longer than this duration,
    /// it will be aborted and return a timeout error.
    ///
    /// **Important**: this setting is ignored if a custom HTTP client is used via [`with_client`].
    /// In that case, configure the timeout on the custom client instead.
    ///
    /// # Example
    /// ```rust
    /// use std::time::Duration;
    /// use rabbitmq_http_client::api::ClientBuilder;
    ///
    /// let client = ClientBuilder::new()
    ///     .with_endpoint("http://localhost:15672/api")
    ///     .with_basic_auth_credentials("user", "password")
    ///     .with_request_timeout(Duration::from_secs(30))
    ///     .build();
    /// ```
    pub fn with_request_timeout(self, timeout: Duration) -> Self {
        ClientBuilder {
            request_timeout: Some(timeout),
            ..self
        }
    }

    /// Sets retry settings for HTTP requests. See [`RetrySettings`].
    ///
    /// By default, no retries are performed.
    pub fn with_retry_settings(self, retry_settings: RetrySettings) -> Self {
        ClientBuilder {
            retry_settings,
            ..self
        }
    }

    /// Builds and returns a configured `Client`.
    ///
    /// This consumes the `ClientBuilder`.
    pub fn build(self) -> Client<E, U, P> {
        let client = match self.client {
            Some(c) => c,
            None => {
                let mut builder = HttpClient::builder();
                if let Some(timeout) = self.request_timeout {
                    builder = builder.timeout(timeout);
                }
                builder.build().unwrap()
            }
        };

        Client::from_http_client_with_retry(
            client,
            self.endpoint,
            self.username,
            self.password,
            self.retry_settings,
        )
    }
}

/// A client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/http-api-reference).
///
/// Most functions provided by this type represent various HTTP API operations.
/// For example,
///
///  * the [`Client::get_queue_info`] function corresponds to the `GET /api/queues/{vhost}/{name}` endpoint
///  * the [`Client::list_user_connections`] function corresponds to the `GET /api/connections/username/{username}` endpoint
///
/// and so on.
///
/// Example
/// ```rust
/// use rabbitmq_http_client::api::Client;
///
/// let endpoint = "http://localhost:15672/api";
/// let username = "username";
/// let password = "password";
/// let rc = Client::new(&endpoint, &username, &password);
/// // list cluster nodes
/// rc.list_nodes().await;
/// // list user connections
/// rc.list_connections().await;
/// // fetch information and metrics of a specific queue
/// rc.get_queue_info("/", "qq.1").await;
/// ```
pub struct Client<E, U, P> {
    endpoint: E,
    username: U,
    password: P,
    client: HttpClient,
    retry_settings: RetrySettings,
}

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Instantiates a client for the specified endpoint with username and password.
    ///
    /// Example
    /// ```rust
    /// use rabbitmq_http_client::api::Client;
    ///
    /// let endpoint = "http://localhost:15672/api";
    /// let username = "username";
    /// let password = "password";
    /// let rc = Client::new(endpoint, username, password);
    /// ```
    /// Creates a new async RabbitMQ HTTP API client.
    ///
    /// This creates a client configured to connect to the specified endpoint
    /// using the provided credentials. The client uses async/await for all
    /// HTTP operations and is suitable for use in async runtime environments
    /// like Tokio.
    pub fn new(endpoint: E, username: U, password: P) -> Self {
        let client = HttpClient::builder().build().unwrap();

        trace!(
            "Created new async RabbitMQ HTTP API client for endpoint: {}",
            endpoint
        );

        Self {
            endpoint,
            username,
            password,
            client,
            retry_settings: RetrySettings::default(),
        }
    }

    /// Instantiates a client for the specified endpoint with username and password and pre-build HttpClient.
    /// Credentials default to guest/guest.
    ///
    /// Example
    /// ```rust
    /// use reqwest::Client as HttpClient;
    /// use rabbitmq_http_client::api::Client;
    ///
    /// let client = HttpClient::new();
    /// let endpoint = "http://localhost:15672/api";
    /// let username = "username";
    /// let password = "password";
    /// let rc = Client::from_http_client(client, endpoint, username, password);
    /// ```
    pub fn from_http_client(client: HttpClient, endpoint: E, username: U, password: P) -> Self {
        Self {
            endpoint,
            username,
            password,
            client,
            retry_settings: RetrySettings::default(),
        }
    }

    /// Instantiates a client for the specified endpoint with user credentials,
    /// a pre-built HttpClient, and retry settings.
    pub fn from_http_client_with_retry(
        client: HttpClient,
        endpoint: E,
        username: U,
        password: P,
        retry_settings: RetrySettings,
    ) -> Self {
        Self {
            endpoint,
            username,
            password,
            client,
            retry_settings,
        }
    }

    /// Creates a `ClientBuilder` to configure a `Client`.
    ///
    /// This is the same as `ClientBuilder::new()`.
    pub fn builder() -> ClientBuilder<&'static str, &'static str, &'static str> {
        ClientBuilder::new()
    }

    //
    // Implementation
    //

    async fn with_retry<F, Fut>(&self, operation: F) -> Result<HttpClientResponse>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<HttpClientResponse>>,
    {
        let mut last_error = None;
        let n = self.retry_settings.max_attempts;
        for attempt in 0..=n {
            match operation().await {
                Ok(response) => {
                    if attempt > 0 {
                        trace!("Request succeeded after {} retry attempt(s)", attempt);
                    }
                    return Ok(response);
                }
                Err(e) => {
                    if attempt < n {
                        trace!(
                            "Request failed on attempt {}/{}, retrying in {}ms: {}",
                            attempt + 1,
                            n + 1,
                            self.retry_settings.delay_ms,
                            e
                        );
                    } else {
                        trace!("Request failed after {} attempt(s): {}", n + 1, e);
                    }
                    last_error = Some(e);

                    // Don't sleep after the last attempt
                    if attempt < n {
                        sleep(Duration::from_millis(self.retry_settings.delay_ms)).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    pub(crate) async fn get_api_request<T, S>(&self, path: S) -> Result<T>
    where
        T: DeserializeOwned,
        S: AsRef<str>,
    {
        let response = self.http_get(path, None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub(crate) async fn delete_api_request_with_optional_not_found<S>(
        &self,
        path: S,
        idempotent: bool,
    ) -> Result<()>
    where
        S: AsRef<str>,
    {
        let excludes = if idempotent {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        self.http_delete(path, excludes, None).await?;
        Ok(())
    }

    pub(crate) async fn put_api_request<S, T>(&self, path: S, payload: &T) -> Result<()>
    where
        S: AsRef<str>,
        T: Serialize,
    {
        self.http_put(path, payload, None, None).await?;
        Ok(())
    }

    pub(crate) async fn http_get<S>(
        &self,
        path: S,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let rooted_path = self.rooted_path(path);
        let username = self.username.to_string();
        let password = self.password.to_string();

        trace!("HTTP GET: {}", rooted_path);

        self.with_retry(|| async {
            let response = self
                .client
                .get(&rooted_path)
                .basic_auth(&username, Some(&password))
                .send()
                .await?;
            self.ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await
        })
        .await
    }

    pub(crate) async fn http_put<S, T>(
        &self,
        path: S,
        payload: &T,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
        T: Serialize,
    {
        let rooted_path = self.rooted_path(path);
        let username = self.username.to_string();
        let password = self.password.to_string();

        if let Ok(body) = serde_json::to_string_pretty(payload) {
            trace!("HTTP PUT: {}\nRequest body:\n{}", rooted_path, body);
        } else {
            trace!("HTTP PUT: {}", rooted_path);
        }

        self.with_retry(|| async {
            let response = self
                .client
                .put(&rooted_path)
                .json(payload)
                .basic_auth(&username, Some(&password))
                .send()
                .await?;
            self.ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await
        })
        .await
    }

    pub(crate) async fn http_post<S, T>(
        &self,
        path: S,
        payload: &T,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
        T: Serialize,
    {
        let rooted_path = self.rooted_path(path);
        let username = self.username.to_string();
        let password = self.password.to_string();

        if let Ok(body) = serde_json::to_string_pretty(payload) {
            trace!("HTTP POST: {}\nRequest body:\n{}", rooted_path, body);
        } else {
            trace!("HTTP POST: {}", rooted_path);
        }

        self.with_retry(|| async {
            let response = self
                .client
                .post(&rooted_path)
                .json(payload)
                .basic_auth(&username, Some(&password))
                .send()
                .await?;
            self.ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await
        })
        .await
    }

    pub(crate) async fn http_post_without_body<S>(
        &self,
        path: S,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let rooted_path = self.rooted_path(path);
        let username = self.username.to_string();
        let password = self.password.to_string();

        self.with_retry(|| async {
            let response = self
                .client
                .post(&rooted_path)
                .basic_auth(&username, Some(&password))
                .send()
                .await?;
            self.ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await
        })
        .await
    }

    pub(crate) async fn http_delete<S>(
        &self,
        path: S,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let rooted_path = self.rooted_path(path);
        let username = self.username.to_string();
        let password = self.password.to_string();

        trace!("HTTP DELETE: {}", rooted_path);

        self.with_retry(|| async {
            let response = self
                .client
                .delete(&rooted_path)
                .basic_auth(&username, Some(&password))
                .send()
                .await?;
            self.ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await
        })
        .await
    }

    pub(crate) async fn http_delete_with_headers<S>(
        &self,
        path: S,
        headers: HeaderMap,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let rooted_path = self.rooted_path(path);
        let username = self.username.to_string();
        let password = self.password.to_string();

        self.with_retry(|| async {
            let response = self
                .client
                .delete(&rooted_path)
                .basic_auth(&username, Some(&password))
                .headers(headers.clone())
                .send()
                .await?;
            self.ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await
        })
        .await
    }

    pub(crate) async fn ok_or_status_code_error(
        &self,
        response: HttpClientResponse,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse> {
        let status = response.status();

        match client_code_to_accept_or_ignore {
            Some(status_code) if status_code == StatusCode::NOT_FOUND => {}
            _ => {
                if status == StatusCode::NOT_FOUND {
                    trace!("Resource not found (404) at {}", response.url());
                    return Err(NotFound);
                }
            }
        }

        if status.is_client_error() {
            match client_code_to_accept_or_ignore {
                Some(expect) if status == expect => {}
                _ => {
                    let url = response.url().clone();
                    let headers = response.headers().clone();
                    // this consumes `self` and makes the response largely useless to the caller,
                    // so we copy the key parts into the error first
                    let body = response.text().await?;
                    trace!("HTTP API response: {} from {}: {}", status, url, body);
                    let error_details = ErrorDetails::from_json(&body);
                    return Err(ClientErrorResponse {
                        url: Some(url),
                        body: Some(body),
                        error_details,
                        headers: Some(headers),
                        status_code: status,
                        backtrace: Backtrace::new(),
                    });
                }
            }
        }

        if status.is_server_error() {
            match server_code_to_accept_or_ignore {
                Some(expect) if status == expect => {}
                _ => {
                    let url = response.url().clone();
                    let headers = response.headers().clone();
                    // this consumes `self` and makes the response largely useless to the caller,
                    // so we copy the key parts into the error first
                    let body = response.text().await?;
                    trace!("HTTP API response: {} from {}: {}", status, url, body);
                    let error_details = ErrorDetails::from_json(&body);
                    return Err(ServerErrorResponse {
                        url: Some(url),
                        body: Some(body),
                        error_details,
                        headers: Some(headers),
                        status_code: status,
                        backtrace: Backtrace::new(),
                    });
                }
            }
        }

        Ok(response)
    }

    pub(crate) fn rooted_path<S>(&self, path: S) -> String
    where
        S: AsRef<str>,
    {
        format!("{}/{}", self.endpoint, path.as_ref())
    }
}

impl Default for Client<&'static str, &'static str, &'static str> {
    fn default() -> Self {
        Self::new("http://localhost:15672/api", "guest", "guest")
    }
}
