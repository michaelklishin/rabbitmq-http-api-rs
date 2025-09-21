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

use crate::error::Error::{ClientErrorResponse, NotFound, ServerErrorResponse};
use backtrace::Backtrace;
use reqwest::{StatusCode, blocking::Client as HttpClient, header::HeaderMap};
use serde::Serialize;
use std::fmt;

pub type HttpClientResponse = reqwest::blocking::Response;
pub type HttpClientError = crate::error::HttpClientError;

pub type Result<T> = std::result::Result<T, HttpClientError>;

/// A `ClientBuilder` can be used to create a `Client` with custom configuration.
///
/// Example
/// ```rust
/// use rabbitmq_http_client::blocking_api::ClientBuilder;
///
/// let endpoint = "http://localhost:15672/api";
/// let username = "username";
/// let password = "password";
/// let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();
/// // list cluster nodes
/// let _ = rc.list_nodes();
/// // list user connections
/// let _ = rc.list_connections();
/// // fetch information and metrics of a specific queue
/// let _ = rc.get_queue_info("/", "qq.1");
/// ```
pub struct ClientBuilder<E = &'static str, U = &'static str, P = &'static str> {
    endpoint: E,
    username: U,
    password: P,
    client: HttpClient,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Constructs a new `ClientBuilder`.
    ///
    /// The default configuration uses `http://localhost:15672/api` as the endpoint
    /// and `guest/guest` as the credentials. This is the same as calling `Client::builder()`.
    ///
    /// Note that the default credentials are [limited to local connections](https://www.rabbitmq.com/docs/access-control)
    /// for security reasons.
    pub fn new() -> Self {
        let client = HttpClient::new();
        Self {
            endpoint: "http://localhost:15672/api",
            username: "guest",
            password: "guest",
            client,
        }
    }
}

impl<E, U, P> ClientBuilder<E, U, P>
where
    E: fmt::Display,
    U: fmt::Display,
    P: fmt::Display,
{
    /// Sets the basic authentication credentials for HTTP requests.
    ///
    /// These credentials will be used for HTTP Basic Authentication when making
    /// requests to the RabbitMQ Management API. Most RabbitMQ installations
    /// require authentication to access the management interface.
    pub fn with_basic_auth_credentials<NewU, NewP>(
        self,
        username: NewU,
        password: NewP,
    ) -> ClientBuilder<E, NewU, NewP>
    where
        NewU: fmt::Display,
        NewP: fmt::Display,
    {
        ClientBuilder {
            endpoint: self.endpoint,
            username,
            password,
            client: self.client,
        }
    }

    /// Sets the HTTP API endpoint URL for the RabbitMQ Management API.
    ///
    /// The endpoint should include the scheme, host, port, and API endpoint path.
    /// Some examples: `http://localhost:15672/api` or `https://rabbitmq.example.com:15672/api`.
    pub fn with_endpoint<T>(self, endpoint: T) -> ClientBuilder<T, U, P>
    where
        T: fmt::Display,
    {
        ClientBuilder {
            endpoint,
            username: self.username,
            password: self.password,
            client: self.client,
        }
    }

    /// Sets a custom HTTP client.
    ///
    /// Use a custom HTTP client to configure custom timeouts, proxy settings, TLS configuration.
    pub fn with_client(self, client: HttpClient) -> Self {
        ClientBuilder { client, ..self }
    }

    /// Builds and returns a configured `Client`.
    ///
    /// This consumes the `ClientBuilder`.
    pub fn build(self) -> Client<E, U, P> {
        Client::from_http_client(self.client, self.endpoint, self.username, self.password)
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
/// use rabbitmq_http_client::blocking_api::Client;
///
/// let endpoint = "http://localhost:15672/api";
/// let username = "username";
/// let password = "password";
/// let rc = Client::new(&endpoint, &username, &password);
/// // list cluster nodes
/// rc.list_nodes();
/// // list user connections
/// rc.list_connections();
/// // fetch information and metrics of a specific queue
/// rc.get_queue_info("/", "qq.1");
/// ```
pub struct Client<E, U, P> {
    endpoint: E,
    username: U,
    password: P,
    client: HttpClient,
}

impl<E, U, P> Client<E, U, P>
where
    E: fmt::Display,
    U: fmt::Display,
    P: fmt::Display,
{
    /// Instantiates a client for the specified endpoint with username and password.
    ///
    /// Unlike its counterpart from `crate::api`, this client will perform HTTP API requests synchronously.
    ///
    /// Example
    /// ```rust
    /// use rabbitmq_http_client::blocking_api::Client;
    ///
    /// let endpoint = "http://localhost:15672/api";
    /// let username = "username";
    /// let password = "password";
    /// let rc = Client::new(endpoint, username, password);
    /// ```
    pub fn new(endpoint: E, username: U, password: P) -> Self {
        let client = HttpClient::builder().build().unwrap();

        Self {
            endpoint,
            username,
            password,
            client,
        }
    }

    /// Instantiates a client for the specified endpoint with username and password and pre-build HttpClient.
    /// Credentials default to guest/guest.
    ///
    /// Example
    /// ```rust
    /// use reqwest::blocking::Client as HttpClient;
    /// use rabbitmq_http_client::blocking_api::Client;
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

    pub(crate) fn get_api_request<T, S>(&self, path: S) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        S: AsRef<str>,
    {
        let response = self.http_get(path, None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub(crate) fn delete_api_request_with_optional_not_found<S>(
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
        self.http_delete(path, excludes, None)?;
        Ok(())
    }

    pub(crate) fn put_api_request<S, T>(&self, path: S, payload: &T) -> Result<()>
    where
        S: AsRef<str>,
        T: Serialize,
    {
        self.http_put(path, payload, None, None)?;
        Ok(())
    }

    //
    // Implementation
    //

    pub(crate) fn http_get<S>(
        &self,
        path: S,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let response = self
            .client
            .get(self.rooted_path(path))
            .basic_auth(&self.username, Some(&self.password))
            .send()?;
        let response = self.ok_or_status_code_error(
            response,
            client_code_to_accept_or_ignore,
            server_code_to_accept_or_ignore,
        )?;
        Ok(response)
    }

    pub(crate) fn http_put<S, T>(
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
        let response = self
            .client
            .put(self.rooted_path(path))
            .json(&payload)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;
        let response = self.ok_or_status_code_error(
            response,
            client_code_to_accept_or_ignore,
            server_code_to_accept_or_ignore,
        )?;
        Ok(response)
    }

    pub(crate) fn http_post<S, T>(
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
        let response = self
            .client
            .post(self.rooted_path(path))
            .json(&payload)
            .basic_auth(&self.username, Some(&self.password))
            .send()?;
        let response = self.ok_or_status_code_error(
            response,
            client_code_to_accept_or_ignore,
            server_code_to_accept_or_ignore,
        )?;
        Ok(response)
    }

    pub(crate) fn http_post_without_body<S>(
        &self,
        path: S,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let response = self
            .client
            .post(self.rooted_path(path))
            .basic_auth(&self.username, Some(&self.password))
            .send()?;
        let response = self.ok_or_status_code_error(
            response,
            client_code_to_accept_or_ignore,
            server_code_to_accept_or_ignore,
        )?;
        Ok(response)
    }

    pub(crate) fn http_delete<S>(
        &self,
        path: S,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let response = self
            .client
            .delete(self.rooted_path(path))
            .basic_auth(&self.username, Some(&self.password))
            .send()?;
        let response = self.ok_or_status_code_error(
            response,
            client_code_to_accept_or_ignore,
            server_code_to_accept_or_ignore,
        )?;
        Ok(response)
    }

    pub(crate) fn http_delete_with_headers<S>(
        &self,
        path: S,
        headers: HeaderMap,
        client_code_to_accept_or_ignore: Option<StatusCode>,
        server_code_to_accept_or_ignore: Option<StatusCode>,
    ) -> Result<HttpClientResponse>
    where
        S: AsRef<str>,
    {
        let response = self
            .client
            .delete(self.rooted_path(path))
            .basic_auth(&self.username, Some(&self.password))
            .headers(headers)
            .send()?;
        let response = self.ok_or_status_code_error(
            response,
            client_code_to_accept_or_ignore,
            server_code_to_accept_or_ignore,
        )?;
        Ok(response)
    }

    pub(crate) fn ok_or_status_code_error(
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
                    let body = response.text()?;
                    return Err(ClientErrorResponse {
                        url: Some(url),
                        body: Some(body),
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
                    let body = response.text()?;
                    return Err(ServerErrorResponse {
                        url: Some(url),
                        body: Some(body),
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
