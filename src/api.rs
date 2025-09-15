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

use crate::error::Error;
use crate::error::Error::{ClientErrorResponse, NotFound, ServerErrorResponse};
use crate::requests::{
    Amqp10ShovelParams, Amqp091ShovelParams, EmptyPayload, FEDERATION_UPSTREAM_COMPONENT,
    FederationUpstreamParams, GlobalRuntimeParameterDefinition, SHOVEL_COMPONENT, StreamParams,
};
use crate::responses::{
    AuthenticationAttemptStatistics, ClusterTags, DeprecatedFeatureList, FeatureFlag,
    FeatureFlagList, FeatureFlagStability, FeatureFlagState, FederationUpstream, GetMessage,
    OAuthConfiguration, Overview, SchemaDefinitionSyncStatus, VirtualHostDefinitionSet,
    WarmStandbyReplicationStatus,
};
use crate::{
    commons::{BindingDestinationType, SupportedProtocol, UserLimitTarget, VirtualHostLimitTarget},
    path,
    requests::{
        self, BulkUserDelete, EnforcedLimitParams, ExchangeParams, Permissions, PolicyParams,
        QueueParams, RuntimeParameterDefinition, UserParams, VirtualHostParams, XArguments,
    },
    responses::{self, BindingInfo, ClusterDefinitionSet},
};
use backtrace::Backtrace;
use reqwest::{
    Client as HttpClient, StatusCode,
    header::{HeaderMap, HeaderValue},
};
use serde::Serialize;
use serde_json::{Map, Value, json};
use std::fmt;

pub type HttpClientResponse = reqwest::Response;
pub type HttpClientError = crate::error::HttpClientError;

pub type Result<T> = std::result::Result<T, HttpClientError>;

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
    client: HttpClient,
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
    /// Sets the API credentials.
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

    /// Sets the HTTP API endpoint URL.
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
/// use rabbitmq_http_client::api::Client;
///
/// let endpoint = "http://localhost:15672/api";
/// let username = "username";
/// let password = "password";
/// let rc = Client::new(&endpoint, &username, &password);
/// // list cluster nodes
/// let _ = rc.list_nodes().await?;
/// // list user connections
/// let _ = rc.list_connections().await?;
/// // fetch information and metrics of a specific queue
/// let _ = rc.get_queue_info("/", "qq.1").await;
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
        }
    }

    /// Creates a `ClientBuilder` to configure a `Client`.
    ///
    /// This is the same as `ClientBuilder::new()`.
    pub fn builder() -> ClientBuilder<&'static str, &'static str, &'static str> {
        ClientBuilder::new()
    }

    /// Lists cluster nodes.
    /// See [RabbitMQ Clustering Guide](https://www.rabbitmq.com/docs/clustering) to learn more.
    pub async fn list_nodes(&self) -> Result<Vec<responses::ClusterNode>> {
        self.get_api_request("nodes").await
    }

    /// Lists virtual hosts in the cluster.
    /// See [Virtual Hosts Guide](https://www.rabbitmq.com/docs/vhosts) to learn more.
    pub async fn list_vhosts(&self) -> Result<Vec<responses::VirtualHost>> {
        self.get_api_request("vhosts").await
    }

    /// Lists users in the internal database.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn list_users(&self) -> Result<Vec<responses::User>> {
        self.get_api_request("users").await
    }

    /// Lists users in the internal database that do not have access to any virtual hosts.
    /// This is useful for finding users that may need permissions granted, or are not used
    /// and should be cleaned up.
    pub async fn list_users_without_permissions(&self) -> Result<Vec<responses::User>> {
        self.get_api_request("users/without-permissions").await
    }

    /// Lists all AMQP 1.0 and 0-9-1 client connections across the cluster.
    /// See [Connections Guide](https://www.rabbitmq.com/docs/connections) to learn more.
    pub async fn list_connections(&self) -> Result<Vec<responses::Connection>> {
        self.get_api_request("connections").await
    }

    /// Returns information about a connection.
    ///
    /// Connection name is usually obtained from `crate::responses::Connection` or `crate::responses::UserConnection`,
    /// e.g. via `Client#list_connections`, `Client#list_connections_in`, `Client#list_user_connections`.
    pub async fn get_connection_info(&self, name: &str) -> Result<responses::Connection> {
        self.get_api_request(path!("connections", name)).await
    }

    /// Returns information about a stream connection.
    ///
    /// Connection name is usually obtained from `crate::responses::Connection` or `crate::responses::UserConnection`,
    /// e.g. via `Client#list_stream_connections`, `Client#list_stream_connections_in`, `Client#list_user_connections`.
    pub async fn get_stream_connection_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::Connection> {
        self.get_api_request(path!("stream", "connections", virtual_host, name))
            .await
    }

    /// Closes a connection with an optional reason.
    ///
    /// The reason will be passed on in the connection error to the client and will be logged on the RabbitMQ end.
    pub async fn close_connection(&self, name: &str, reason: Option<&str>) -> Result<()> {
        match reason {
            None => {
                self.http_delete(
                    path!("connections", name),
                    Some(StatusCode::NOT_FOUND),
                    None,
                )
                .await?
            }
            Some(value) => {
                let mut headers = HeaderMap::new();
                let hdr = HeaderValue::from_str(value)?;
                headers.insert("X-Reason", hdr);
                self.http_delete_with_headers(path!("connections", name), headers, None, None)
                    .await?
            }
        };
        Ok(())
    }

    /// Closes all connections for a user with an optional reason.
    ///
    /// The reason will be passed on in the connection error to the client and will be logged on the RabbitMQ end.
    ///
    /// This is en equivalent of listing all connections of a user with `Client#list_user_connections` and then
    /// closing them one by one.
    pub async fn close_user_connections(&self, username: &str, reason: Option<&str>) -> Result<()> {
        match reason {
            None => {
                self.http_delete(
                    path!("connections", "username", username),
                    Some(StatusCode::NOT_FOUND),
                    None,
                )
                .await?
            }
            Some(value) => {
                let mut headers = HeaderMap::new();
                let hdr = HeaderValue::from_str(value)?;
                headers.insert("X-Reason", hdr);
                self.http_delete_with_headers(
                    path!("connections", "username", username),
                    headers,
                    None,
                    None,
                )
                .await?
            }
        };
        Ok(())
    }

    /// Lists all connections in the given virtual host.
    /// See [Connections Guide](https://www.rabbitmq.com/docs/connections) to learn more.
    pub async fn list_connections_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::Connection>> {
        self.get_api_request(path!("vhosts", virtual_host, "connections"))
            .await
    }

    /// Lists all connections of a specific user.
    /// See [Connection Guide](https://www.rabbitmq.com/docs/connections) to learn more.
    pub async fn list_user_connections(
        &self,
        username: &str,
    ) -> Result<Vec<responses::UserConnection>> {
        self.get_api_request(path!("connections", "username", username))
            .await
    }

    /// Lists all RabbitMQ Stream Protocol client connections across the cluster.
    /// See [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_stream_connections(&self) -> Result<Vec<responses::Connection>> {
        self.get_api_request("stream/connections").await
    }

    /// Lists RabbitMQ Stream Protocol client connections in the given virtual host.
    /// See [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_stream_connections_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::Connection>> {
        self.get_api_request(path!("stream", "connections", virtual_host))
            .await
    }

    /// Lists all channels across the cluster.
    /// See [Channels Guide](https://www.rabbitmq.com/docs/channels) to learn more.
    pub async fn list_channels(&self) -> Result<Vec<responses::Channel>> {
        self.get_api_request("channels").await
    }

    /// Lists all channels in the given virtual host.
    /// See [Channels Guide](https://www.rabbitmq.com/docs/channels) to learn more.
    pub async fn list_channels_in(&self, virtual_host: &str) -> Result<Vec<responses::Channel>> {
        self.get_api_request(path!("vhosts", virtual_host, "channels"))
            .await
    }

    /// Lists all channels on a given AMQP 0-9-1 connection.
    /// See [Channels Guide](https://www.rabbitmq.com/docs/channels) to learn more.
    pub async fn list_channels_on(&self, connection_name: &str) -> Result<Vec<responses::Channel>> {
        self.get_api_request(path!("connections", connection_name, "channels"))
            .await
    }

    /// Lists all stream publishers across the cluster.
    pub async fn list_stream_publishers(&self) -> Result<Vec<responses::StreamPublisher>> {
        let response = self
            .http_get(path!("stream", "publishers"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists stream publishers on connections in the given virtual host.
    pub async fn list_stream_publishers_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        let response = self
            .http_get(path!("stream", "publishers", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists stream publishers publishing to the given stream.
    /// Useful for detecting publishers that are publishing to a specific stream.
    pub async fn list_stream_publishers_of(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        let response = self
            .http_get(
                path!("stream", "publishers", virtual_host, name),
                None,
                None,
            )
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists stream publishers on the given stream connection.
    /// Use this function for inspecting stream publishers on a specific connection.
    pub async fn list_stream_publishers_on_connection(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        let response = self
            .http_get(
                path!("stream", "connections", virtual_host, name, "publishers"),
                None,
                None,
            )
            .await?;

        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all stream consumers across the cluster.
    pub async fn list_stream_consumers(&self) -> Result<Vec<responses::StreamConsumer>> {
        let response = self
            .http_get(path!("stream", "consumers"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists stream consumers on connections in the given virtual host.
    pub async fn list_stream_consumers_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::StreamConsumer>> {
        let response = self
            .http_get(path!("stream", "consumers", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists stream consumers on the given stream connection.
    /// Use this function for inspecting stream consumers on a specific connection.
    pub async fn list_stream_consumers_on_connection(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamConsumer>> {
        let response = self
            .http_get(
                path!("stream", "connections", virtual_host, name, "consumers"),
                None,
                None,
            )
            .await?;

        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all queues and streams across the cluster.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_queues(&self) -> Result<Vec<responses::QueueInfo>> {
        let response = self.http_get("queues", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all queues and streams in the given virtual host.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_queues_in(&self, virtual_host: &str) -> Result<Vec<responses::QueueInfo>> {
        let response = self
            .http_get(path!("queues", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all queues and streams across the cluster. Compared to [`list_queues`], provides more queue metrics.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_queues_with_details(&self) -> Result<Vec<responses::DetailedQueueInfo>> {
        let response = self.http_get("queues/detailed", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all exchanges across the cluster.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub async fn list_exchanges(&self) -> Result<Vec<responses::ExchangeInfo>> {
        let response = self.http_get("exchanges", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all exchanges in the given virtual host.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub async fn list_exchanges_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::ExchangeInfo>> {
        let response = self
            .http_get(path!("exchanges", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all bindings (both queue-to-exchange and exchange-to-exchange ones) across the cluster.
    pub async fn list_bindings(&self) -> Result<Vec<responses::BindingInfo>> {
        let response = self.http_get("bindings", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all bindings (both queue-to-exchange and exchange-to-exchange ones)  in the given virtual host.
    pub async fn list_bindings_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        let response = self
            .http_get(path!("bindings", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all bindings of a specific queue.
    /// Use this function for troubleshooting routing of a particular queue.
    pub async fn list_queue_bindings(
        &self,
        virtual_host: &str,
        queue: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        let response = self
            .http_get(path!("queues", virtual_host, queue, "bindings"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all bindings of a specific exchange where it is the source.
    /// Use this function for troubleshooting routing of a particular exchange.
    pub async fn list_exchange_bindings_with_source(
        &self,
        virtual_host: &str,
        exchange: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        self.list_exchange_bindings_with_source_or_destination(
            virtual_host,
            exchange,
            BindindVertex::Source,
        )
        .await
    }

    /// Lists all bindings of a specific exchange where it is the destination.
    /// Use this function for troubleshooting routing of a particular exchange.
    pub async fn list_exchange_bindings_with_destination(
        &self,
        virtual_host: &str,
        exchange: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        self.list_exchange_bindings_with_source_or_destination(
            virtual_host,
            exchange,
            BindindVertex::Destination,
        )
        .await
    }

    /// Lists all consumers across the cluster.
    /// See [Consumers Guide](https://www.rabbitmq.com/docs/consumers) to learn more.
    pub async fn list_consumers(&self) -> Result<Vec<responses::Consumer>> {
        let response = self.http_get("consumers", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all consumers in the given virtual host.
    /// See [Consumers Guide](https://www.rabbitmq.com/docs/consumers) to learn more.
    pub async fn list_consumers_in(&self, virtual_host: &str) -> Result<Vec<responses::Consumer>> {
        let response = self
            .http_get(path!("consumers", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a cluster node.
    /// See [Clustering Guide](https://www.rabbitmq.com/docs/clustering) to learn more.
    pub async fn get_node_info(&self, name: &str) -> Result<responses::ClusterNode> {
        let response = self.http_get(path!("nodes", name), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns memory usage information for a cluster node.
    ///See [Reasoning About Memory Footprint](https://www.rabbitmq.com/docs/memory-use) to learn more
    pub async fn get_node_memory_footprint(
        &self,
        name: &str,
    ) -> Result<responses::NodeMemoryFootprint> {
        let response = self
            .http_get(path!("nodes", name, "memory"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a virtual host.
    /// See [Virtual Hosts Guide](https://www.rabbitmq.com/docs/vhosts) to learn more.
    pub async fn get_vhost(&self, name: &str) -> Result<responses::VirtualHost> {
        let response = self.http_get(path!("vhosts", name), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a user in the internal database.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn get_user(&self, name: &str) -> Result<responses::User> {
        let response = self.http_get(path!("users", name), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about the authenticated user.
    /// See [Access Control Guide](https://www.rabbitmq.com/docs/access-control) to learn more.
    pub async fn current_user(&self) -> Result<responses::CurrentUser> {
        let response = self.http_get("whoami", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a queue or stream.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) to learn more.
    pub async fn get_queue_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::QueueInfo> {
        let response = self
            .http_get(path!("queues", virtual_host, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a stream.
    /// See [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn get_stream_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::QueueInfo> {
        self.get_queue_info(virtual_host, name).await
    }

    /// Returns information about an exchange.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub async fn get_exchange_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::ExchangeInfo> {
        let response = self
            .http_get(path!("exchanges", virtual_host, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Creates a virtual host.
    ///
    /// See [`VirtualHostParams`]
    pub async fn create_vhost(&self, params: &VirtualHostParams<'_>) -> Result<()> {
        self.update_vhost(params).await
    }

    /// Creates a virtual host or updates metadata of an existing one.
    ///
    /// See [`VirtualHostParams`]
    pub async fn update_vhost(&self, params: &VirtualHostParams<'_>) -> Result<()> {
        self.put_api_request(path!("vhosts", params.name), params)
            .await
    }

    /// Adds a user to the internal database.
    ///
    /// See [`UserParams`] and [`crate::password_hashing`].
    pub async fn create_user(&self, params: &UserParams<'_>) -> Result<()> {
        self.put_api_request(path!("users", params.name), params)
            .await
    }

    /// Sets [user permissions](https://www.rabbitmq.com/docs/access-control) in a specific virtual host.
    pub async fn declare_permissions(&self, params: &Permissions<'_>) -> Result<()> {
        self.put_api_request(path!("permissions", params.vhost, params.user), params)
            .await
    }

    /// An easier to remember alias for [`declare_permissions`].
    pub async fn grant_permissions(&self, params: &Permissions<'_>) -> Result<()> {
        self.declare_permissions(params).await
    }

    /// Declares a [queue](https://www.rabbitmq.com/docs/queues).
    ///
    /// If the queue already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub async fn declare_queue(&self, vhost: &str, params: &QueueParams<'_>) -> Result<()> {
        self.put_api_request(path!("queues", vhost, params.name), params)
            .await
    }

    /// Declares a [RabbitMQ stream](https://www.rabbitmq.com/docs/streams).
    ///
    /// Streams are a durable, replicated, long-lived data structure in RabbitMQ designed for
    /// high-throughput scenarios. Unlike traditional queues, consuming from a stream is
    /// a non-destructive operation. Stream data is deleted according to an effective
    /// stream retention policy.
    ///
    /// If the stream already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub async fn declare_stream(&self, vhost: &str, params: &StreamParams<'_>) -> Result<()> {
        let mut m: Map<String, Value> = Map::new();

        if let Some(m2) = params.arguments.clone() {
            m.extend(m2);
        };

        if let Some(val) = params.max_length_bytes {
            m.insert("max_length_bytes".to_owned(), json!(val));
        };
        if let Some(val) = params.max_segment_length_bytes {
            m.insert("max_segment_length_bytes".to_owned(), json!(val));
        };

        let q_params = QueueParams::new_stream(params.name, Some(m));
        let _response = self
            .http_put(path!("queues", vhost, params.name), &q_params, None, None)
            .await?;
        Ok(())
    }

    /// Declares an [exchange](https://www.rabbitmq.com/docs/exchanges).
    ///
    /// If the exchange already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub async fn declare_exchange(&self, vhost: &str, params: &ExchangeParams<'_>) -> Result<()> {
        self.put_api_request(path!("exchanges", vhost, params.name), params)
            .await
    }

    /// Binds a queue or a stream to an exchange.
    ///
    /// Bindings determine how messages published to an exchange are routed to queues.
    /// The exchange type, routing key and arguments define the routing behavior.
    ///
    /// Both the source (exchange) and destination (queue or stream) must exist.
    pub async fn bind_queue(
        &self,
        vhost: &str,
        queue: &str,
        exchange: &str,
        routing_key: Option<&str>,
        arguments: XArguments,
    ) -> Result<()> {
        let mut body = Map::<String, Value>::new();
        if let Some(rk) = routing_key {
            body.insert("routing_key".to_owned(), json!(rk));
        }
        if let Some(args) = arguments {
            body.insert("arguments".to_owned(), json!(args));
        }

        let _response = self
            .http_post(
                path!("bindings", vhost, "e", exchange, "q", queue),
                &body,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    /// Bindings one exchange to another (creates an [exchange-to-exchange binding](https://www.rabbitmq.com/docs/e2e)).
    ///
    /// This allows messages published to the source exchange to be forwarded to
    ///
    /// Exchange-to-exchange bindings enable complex routing topologies and
    /// message flow patterns.
    ///
    /// Both source and destination exchanges must exist.
    pub async fn bind_exchange(
        &self,
        vhost: &str,
        destination: &str,
        source: &str,
        routing_key: Option<&str>,
        arguments: XArguments,
    ) -> Result<()> {
        let mut body = Map::<String, Value>::new();
        if let Some(rk) = routing_key {
            body.insert("routing_key".to_owned(), json!(rk));
        }
        if let Some(args) = arguments {
            body.insert("arguments".to_owned(), json!(args));
        }

        let _response = self
            .http_post(
                path!("bindings", vhost, "e", source, "e", destination),
                &body,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    /// Deletes a virtual host and all its contents.
    ///
    /// This is a destructive operation that will permanently remove the virtual host
    /// along with all queues, exchanges, bindings, and messages it contains. All
    /// connections to this virtual host will be closed. If `idempotently` is true,
    /// the operation will succeed even if the virtual host doesn't exist.
    pub async fn delete_vhost(&self, vhost: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("vhosts", vhost), idempotently)
            .await
    }

    /// Deletes a user from the internal RabbitMQ user database.
    ///
    /// This removes the user account entirely, including all associated permissions
    /// across all virtual hosts. Active connections belonging to this user will be
    /// closed. If `idempotently` is true, the operation will succeed even if the
    /// user doesn't exist.
    pub async fn delete_user(&self, username: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("users", username), idempotently)
            .await
    }

    /// Deletes multiple users from the internal database in a single operation.
    ///
    /// This is more efficient than calling [`Client::delete_user`] multiple times when you
    /// need to remove several user accounts. All specified users will be deleted
    /// along with their permissions, and any active connections will be closed.
    /// Non-existent users in the list are silently ignored.
    pub async fn delete_users(&self, usernames: Vec<&str>) -> Result<()> {
        let delete = BulkUserDelete { usernames };
        let _response = self
            .http_post(path!("users", "bulk-delete"), &delete, None, None)
            .await?;
        Ok(())
    }

    /// Revokes user permissions in a specific virtual host.
    pub async fn clear_permissions(
        &self,
        vhost: &str,
        username: &str,
        idempotently: bool,
    ) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self
            .http_delete(path!("permissions", vhost, username), excludes, None)
            .await?;
        Ok(())
    }

    /// Deletes a queue in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent queue
    /// will fail.
    pub async fn delete_queue(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("queues", vhost, name), idempotently)
            .await
    }

    /// Deletes a stream in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent stream
    /// will fail.
    pub async fn delete_stream(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_queue(vhost, name, idempotently).await
    }

    /// Deletes an exchange in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent exchange
    /// will fail.
    pub async fn delete_exchange(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("exchanges", vhost, name),
            idempotently,
        )
        .await
    }

    pub async fn delete_binding(
        &self,
        virtual_host: &str,
        source: &str,
        destination: &str,
        destination_type: BindingDestinationType,
        routing_key: &str,
        arguments: XArguments,
    ) -> Result<HttpClientResponse> {
        let args = arguments.unwrap();

        // to delete a binding, we need properties, that we can get from the server
        // so we search for the binding before deleting it
        let bindings = match destination_type {
            BindingDestinationType::Queue => {
                self.list_queue_bindings(virtual_host, destination).await?
            }
            BindingDestinationType::Exchange => {
                self.list_exchange_bindings_with_destination(virtual_host, destination)
                    .await?
            }
        };

        let bs: Vec<&BindingInfo> = bindings
            .iter()
            .filter(|b| b.source == source && b.routing_key == routing_key && b.arguments.0 == args)
            .collect();
        match bs.len() {
            0 => Err(Error::NotFound),
            1 => {
                let first_key = bs.first().unwrap().properties_key.clone();
                let path_appreviation = destination_type.path_appreviation();
                let path = match first_key {
                    Some(pk) => {
                        path!(
                            // /api/bindings/vhost/e/exchange/[eq]/destination/props
                            "bindings",
                            virtual_host,
                            "e",
                            source,
                            path_appreviation,
                            destination,
                            pk.as_str()
                        )
                    }
                    None => {
                        path!(
                            // /api/bindings/vhost/e/exchange/[eq]/destination/
                            "bindings",
                            virtual_host,
                            "e",
                            source,
                            path_appreviation,
                            destination
                        )
                    }
                };
                let response = self.http_delete(&path, None, None).await?;
                Ok(response)
            }
            _ => Err(Error::MultipleMatchingBindings),
        }
    }

    /// Removes all messages in "ready for delivery" state from a queue without deleting the queue itself.
    ///
    /// Messages that were delivered but are pending acknowledgement will not be deleted
    /// by purging.
    pub async fn purge_queue(&self, virtual_host: &str, name: &str) -> Result<()> {
        let _response = self
            .http_delete(path!("queues", virtual_host, name, "contents"), None, None)
            .await?;
        Ok(())
    }

    /// Lists all [runtime parameters](https://www.rabbitmq.com/docs/parameters) defined in the cluster.
    pub async fn list_runtime_parameters(&self) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get("parameters", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all [runtime parameters](https://www.rabbitmq.com/docs/parameters) with a given
    /// component type (like "federation-upstream" or "shovel") defined in the cluster.
    pub async fn list_runtime_parameters_of_component(
        &self,
        component: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self
            .http_get(path!("parameters", component), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all [runtime parameters](https://www.rabbitmq.com/docs/parameters) defined in
    /// a specific virtual host.
    pub async fn list_runtime_parameters_of_component_in(
        &self,
        component: &str,
        vhost: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self
            .http_get(path!("parameters", component, vhost), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_runtime_parameter(
        &self,
        component: &str,
        vhost: &str,
        name: &str,
    ) -> Result<responses::RuntimeParameter> {
        let response = self
            .http_get(path!("parameters", component, vhost, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn upsert_runtime_parameter<'a>(
        &self,
        param: &'a RuntimeParameterDefinition<'a>,
    ) -> Result<()> {
        let _response = self
            .http_put(
                path!("parameters", param.component, param.vhost, param.name),
                &param,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn clear_runtime_parameter(
        &self,
        component: &str,
        vhost: &str,
        name: &str,
    ) -> Result<()> {
        let _response = self
            .http_delete(path!("parameters", component, vhost, name), None, None)
            .await?;
        Ok(())
    }

    pub async fn clear_all_runtime_parameters(&self) -> Result<()> {
        let params = self.list_runtime_parameters().await?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name)
                .await?
        }
        Ok(())
    }

    pub async fn clear_all_runtime_parameters_of_component(&self, component: &str) -> Result<()> {
        let params = self.list_runtime_parameters_of_component(component).await?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name)
                .await?
        }
        Ok(())
    }

    pub async fn list_global_runtime_parameters(
        &self,
    ) -> Result<Vec<responses::GlobalRuntimeParameter>> {
        let response = self.http_get("global-parameters", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_global_runtime_parameter(
        &self,
        name: &str,
    ) -> Result<responses::GlobalRuntimeParameter> {
        let response = self
            .http_get(path!("global-parameters", name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn upsert_global_runtime_parameter<'a>(
        &self,
        param: &'a GlobalRuntimeParameterDefinition<'a>,
    ) -> Result<()> {
        let _response = self
            .http_put(path!("global-parameters", param.name), &param, None, None)
            .await?;
        Ok(())
    }

    pub async fn clear_global_runtime_parameter(&self, name: &str) -> Result<()> {
        let _response = self
            .http_delete(path!("global-parameters", name), None, None)
            .await?;
        Ok(())
    }

    pub async fn set_user_limit(
        &self,
        username: &str,
        limit: EnforcedLimitParams<UserLimitTarget>,
    ) -> Result<()> {
        let body = json!({"value": limit.value});
        let _response = self
            .http_put(
                path!("user-limits", username, limit.kind),
                &body,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn clear_user_limit(&self, username: &str, kind: UserLimitTarget) -> Result<()> {
        let _response = self
            .http_delete(path!("user-limits", username, kind), None, None)
            .await?;
        Ok(())
    }

    pub async fn list_all_user_limits(&self) -> Result<Vec<responses::UserLimits>> {
        let response = self.http_get("user-limits", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_user_limits(&self, username: &str) -> Result<Vec<responses::UserLimits>> {
        let response = self
            .http_get(path!("user-limits", username), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Sets a [virtual host limit](https://www.rabbitmq.com/docs/vhosts#limits).
    pub async fn set_vhost_limit(
        &self,
        vhost: &str,
        limit: EnforcedLimitParams<VirtualHostLimitTarget>,
    ) -> Result<()> {
        let body = json!({"value": limit.value});
        let _response = self
            .http_put(path!("vhost-limits", vhost, limit.kind), &body, None, None)
            .await?;
        Ok(())
    }

    /// Clears (removes) a [virtual host limit](https://www.rabbitmq.com/docs/vhosts#limits).
    pub async fn clear_vhost_limit(&self, vhost: &str, kind: VirtualHostLimitTarget) -> Result<()> {
        let _response = self
            .http_delete(
                path!("vhost-limits", vhost, kind),
                Some(StatusCode::NOT_FOUND),
                None,
            )
            .await?;
        Ok(())
    }

    /// Lists all [virtual host limits](https://www.rabbitmq.com/docs/vhosts#limits) set in the cluster.
    pub async fn list_all_vhost_limits(&self) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self.http_get("vhost-limits", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists the [limits of a given virtual host](https://www.rabbitmq.com/docs/vhosts#limits).
    pub async fn list_vhost_limits(
        &self,
        vhost: &str,
    ) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self
            .http_get(path!("vhost-limits", vhost), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_cluster_name(&self) -> Result<responses::ClusterIdentity> {
        let response = self.http_get("cluster-name", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn set_cluster_name(&self, new_name: &str) -> Result<()> {
        let body = json!({"name": new_name});
        let _response = self.http_put("cluster-name", &body, None, None).await?;
        Ok(())
    }

    pub async fn get_cluster_tags(&self) -> Result<responses::ClusterTags> {
        let response = self.get_global_runtime_parameter("cluster_tags").await?;
        Ok(ClusterTags::from(response.value))
    }

    pub async fn set_cluster_tags(&self, tags: Map<String, Value>) -> Result<()> {
        let grp = GlobalRuntimeParameterDefinition {
            name: "cluster_tags",
            value: tags,
        };
        self.upsert_global_runtime_parameter(&grp).await?;
        Ok(())
    }

    pub async fn clear_cluster_tags(&self) -> Result<()> {
        self.clear_global_runtime_parameter("cluster_tags").await?;
        Ok(())
    }

    /// Fetches a [policy](https://www.rabbitmq.com/docs/policies).
    pub async fn get_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self
            .http_get(path!("policies", vhost, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all [policies](https://www.rabbitmq.com/docs/policies) in the cluster (across all virtual hosts), taking the user's
    /// permissions into account.
    pub async fn list_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("policies", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists policies in a virtual host.
    pub async fn list_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("policies", vhost), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Declares a [policy](https://www.rabbitmq.com/docs/policies).
    /// See [`crate::requests::PolicyParams`] and See [`crate::requests::PolicyDefinition`]
    pub async fn declare_policy(&self, params: &PolicyParams<'_>) -> Result<()> {
        let _response = self
            .http_put(
                path!("policies", params.vhost, params.name),
                params,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    /// Declares multiple [policies](https://www.rabbitmq.com/docs/policies).
    ///
    /// Note that this function will still issue
    /// as many HTTP API requests as there are policies to declare.
    ///
    /// See [`crate::requests::PolicyParams`] and See [`crate::requests::PolicyDefinition`]
    pub async fn declare_policies(&self, params: Vec<&PolicyParams<'_>>) -> Result<()> {
        for p in params {
            self.declare_policy(p).await?;
        }
        Ok(())
    }

    /// Deletes a [policy](https://www.rabbitmq.com/docs/policies).
    /// This function is idempotent: deleting a non-existent policy is considered a success.
    pub async fn delete_policy(&self, vhost: &str, name: &str) -> Result<()> {
        let _response = self
            .http_delete(
                path!("policies", vhost, name),
                Some(StatusCode::NOT_FOUND),
                None,
            )
            .await?;
        Ok(())
    }

    /// Deletes multiple [policies](https://www.rabbitmq.com/docs/policies).
    ///
    /// Note that this function will still issue
    /// as many HTTP API requests as there are policies to delete.
    ///
    /// This function is idempotent: deleting a non-existent policy is considered a success.
    pub async fn delete_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_policy(vhost, name).await?;
        }
        Ok(())
    }

    pub async fn get_operator_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self
            .http_get(path!("operator-policies", vhost, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_operator_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("operator-policies", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_operator_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self
            .http_get(path!("operator-policies", vhost), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn declare_operator_policy(&self, params: &PolicyParams<'_>) -> Result<()> {
        let _response = self
            .http_put(
                path!("operator-policies", params.vhost, params.name),
                params,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn declare_operator_policies(&self, params: Vec<&PolicyParams<'_>>) -> Result<()> {
        for p in params {
            self.declare_operator_policy(p).await?;
        }
        Ok(())
    }

    pub async fn delete_operator_policy(&self, vhost: &str, name: &str) -> Result<()> {
        let _response = self
            .http_delete(
                path!("operator-policies", vhost, name),
                Some(StatusCode::NOT_FOUND),
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn delete_operator_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_operator_policy(vhost, name).await?;
        }
        Ok(())
    }

    pub async fn list_permissions(&self) -> Result<Vec<responses::Permissions>> {
        self.get_api_request("permissions").await
    }

    pub async fn list_permissions_in(&self, vhost: &str) -> Result<Vec<responses::Permissions>> {
        let response = self
            .http_get(path!("vhosts", vhost, "permissions"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_permissions_of(&self, user: &str) -> Result<Vec<responses::Permissions>> {
        let response = self
            .http_get(path!("users", user, "permissions"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all topic permissions of a user.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn list_topic_permissions_of(
        &self,
        user: &str,
    ) -> Result<Vec<responses::TopicPermission>> {
        let response = self
            .http_get(path!("users", user, "topic-permissions"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all topic permissions in the cluster.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn list_topic_permissions(&self) -> Result<Vec<responses::TopicPermission>> {
        self.get_api_request("topic-permissions").await
    }

    /// Lists all topic permissions in a virtual host.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn list_topic_permissions_in(
        &self,
        vhost: &str,
    ) -> Result<Vec<responses::TopicPermission>> {
        self.get_api_request(path!("vhosts", vhost, "topic-permissions"))
            .await
    }

    /// Gets topic permissions for a user in a specific virtual host.
    /// See [Topic Authorisation](https://www.rabbitmq.com/docs/access-control#topic-authorisation) to learn more.
    pub async fn get_topic_permissions_of(
        &self,
        vhost: &str,
        user: &str,
    ) -> Result<responses::TopicPermission> {
        // For some reason this endpoint returns a list instead of a single object
        let response: Vec<responses::TopicPermission> = self
            .get_api_request(path!("topic-permissions", vhost, user))
            .await?;
        match response.first() {
            Some(p) => Ok(p.clone()),
            None => Err(Error::NotFound),
        }
    }

    pub async fn get_permissions(&self, vhost: &str, user: &str) -> Result<responses::Permissions> {
        let response = self
            .http_get(path!("permissions", vhost, user), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Sets [topic permissions](https://www.rabbitmq.com/docs/access-control#topic-authorisation) in a specific virtual host.
    pub async fn declare_topic_permissions(
        &self,
        params: &requests::TopicPermissions<'_>,
    ) -> Result<()> {
        self.put_api_request(
            path!("topic-permissions", params.vhost, params.user),
            params,
        )
        .await
    }

    /// Clears [topic permissions](https://www.rabbitmq.com/docs/access-control#topic-authorisation) for a user in a specific virtual host.
    pub async fn clear_topic_permissions(
        &self,
        vhost: &str,
        user: &str,
        idempotently: bool,
    ) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self
            .http_delete(path!("topic-permissions", vhost, user), excludes, None)
            .await?;
        Ok(())
    }

    //
    // Rebalancing
    //

    pub async fn rebalance_queue_leaders(&self) -> Result<()> {
        self.http_post("rebalance/queues", &json!({}), None, None)
            .await?;
        Ok(())
    }

    //
    // Definitions

    /// Exports cluster-wide definitions as a JSON document.
    /// This includes all virtual hosts, users, permissions, policies, queues, streams, exchanges, bindings, runtime parameters.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn export_cluster_wide_definitions(&self) -> Result<String> {
        self.export_cluster_wide_definitions_as_string().await
    }

    /// Exports cluster-wide definitions as a JSON document.
    /// This includes all virtual hosts, users, permissions, policies, queues, streams, exchanges, bindings, runtime parameters.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn export_cluster_wide_definitions_as_string(&self) -> Result<String> {
        let response = self.http_get("definitions", None, None).await?;
        let response = response.text().await?;
        Ok(response)
    }

    /// Exports cluster-wide definitions as a data structure.
    /// This includes all virtual hosts, users, permissions, policies, queues, streams, exchanges, bindings, runtime parameters.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn export_cluster_wide_definitions_as_data(&self) -> Result<ClusterDefinitionSet> {
        let response = self.http_get("definitions", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Exports definitions of a single virtual host as a JSON document.
    /// This includes the permissions, policies, queues, streams, exchanges, bindings, runtime parameters associated
    /// with the given virtual host.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn export_vhost_definitions(&self, vhost: &str) -> Result<String> {
        self.export_vhost_definitions_as_string(vhost).await
    }

    /// Exports definitions of a single virtual host as a JSON document.
    /// This includes the permissions, policies, queues, streams, exchanges, bindings, runtime parameters associated
    /// with the given virtual host.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn export_vhost_definitions_as_string(&self, vhost: &str) -> Result<String> {
        let response = self
            .http_get(path!("definitions", vhost), None, None)
            .await?;
        let response = response.text().await?;
        Ok(response)
    }

    /// Exports definitions of a single virtual host as a data structure.
    /// This includes the permissions, policies, queues, streams, exchanges, bindings, runtime parameters associated
    /// with the given virtual host.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn export_vhost_definitions_as_data(
        &self,
        vhost: &str,
    ) -> Result<VirtualHostDefinitionSet> {
        let response = self
            .http_get(path!("definitions", vhost), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Imports cluster-wide definitions from a JSON document value.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn import_definitions(&self, definitions: Value) -> Result<()> {
        self.import_cluster_wide_definitions(definitions).await
    }

    /// Imports cluster-wide definitions from a JSON document value.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn import_cluster_wide_definitions(&self, definitions: Value) -> Result<()> {
        self.http_post("definitions", &definitions, None, None)
            .await?;
        Ok(())
    }

    /// Imports definitions of a single virtual host from a JSON document value.
    ///
    /// See [Definition Export and Import](https://www.rabbitmq.com/docs/definitions) to learn more.
    pub async fn import_vhost_definitions(&self, vhost: &str, definitions: Value) -> Result<()> {
        self.http_post(path!("definitions", vhost), &definitions, None, None)
            .await?;
        Ok(())
    }

    //
    // Health Checks
    //

    /// Performs a cluster-wide health check for any active resource alarms in the cluster.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks) to learn more.
    pub async fn health_check_cluster_wide_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/alarms").await
    }

    /// Performs a health check for alarms on the target node only.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks) to learn more.
    pub async fn health_check_local_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/local-alarms").await
    }

    /// Will fail if target node is critical to the quorum of some quorum queues, streams or the Khepri metadata store.
    /// See [Upgrades Guide](https://www.rabbitmq.com/docs/upgrade#maintaining-quorum) to learn more.
    pub async fn health_check_if_node_is_quorum_critical(&self) -> Result<()> {
        let path = "health/checks/node-is-quorum-critical";
        self.boolean_health_check(path).await
    }

    /// Checks if a specific port has an active listener.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks)
    /// and [Networking Guide](https://www.rabbitmq.com/docs/networking) to learn more.
    pub async fn health_check_port_listener(&self, port: u16) -> Result<()> {
        let port_s = port.to_string();
        let path = path!("health", "checks", "port-listener", port_s);
        self.boolean_health_check(&path).await
    }

    /// Checks if a specific protocol listener is active.
    /// See [Monitoring and Health Checks Guide](https://www.rabbitmq.com/docs/monitoring#health-checks)
    /// and [Networking Guide](https://www.rabbitmq.com/docs/networking) to learn more.
    pub async fn health_check_protocol_listener(&self, protocol: SupportedProtocol) -> Result<()> {
        let proto: String = String::from(protocol);
        let path = path!("health", "checks", "protocol-listener", proto);
        self.boolean_health_check(&path).await
    }

    async fn boolean_health_check(&self, path: &str) -> std::result::Result<(), HttpClientError> {
        // we expect that StatusCode::SERVICE_UNAVAILABLE may be return and ignore
        // it here to provide a custom error type later
        let response = self
            .http_get(path, None, Some(StatusCode::SERVICE_UNAVAILABLE))
            .await?;

        let status_code = response.status();
        if status_code.is_success() {
            return Ok(());
        }

        let failure_details = response.json().await?;
        Err(Error::HealthCheckFailed {
            path: path.to_owned(),
            status_code,
            details: failure_details,
        })
    }

    //
    // Federation
    //

    /// Lists [federation](https://www.rabbitmq.com/docs/federation) upstreams defined in the cluster.
    pub async fn list_federation_upstreams(&self) -> Result<Vec<responses::FederationUpstream>> {
        let response = self
            .list_runtime_parameters_of_component(FEDERATION_UPSTREAM_COMPONENT)
            .await?;
        let upstreams = response
            .into_iter()
            .map(FederationUpstream::try_from)
            // TODO: in theory this can be an Err
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();

        Ok(upstreams)
    }

    /// Lists [federation](https://www.rabbitmq.com/docs/federation) links (connections) running in the cluster.
    pub async fn list_federation_links(&self) -> Result<Vec<responses::FederationLink>> {
        let response = self.http_get("federation-links", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Creates or updates a [federation](https://www.rabbitmq.com/docs/federation) upstream.
    ///
    /// Federation upstreams define connection endpoints for federation links (connections that federate
    /// queues or exchanges).
    pub async fn declare_federation_upstream(
        &self,
        params: FederationUpstreamParams<'_>,
    ) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_federation_upstream_with_parameters(&runtime_param)
            .await
    }

    /// Deletes a [federation](https://www.rabbitmq.com/docs/federation) upstream.
    /// Deleting an upstream will stop any links connected to it.
    pub async fn delete_federation_upstream(&self, vhost: &str, name: &str) -> Result<()> {
        self.clear_runtime_parameter(FEDERATION_UPSTREAM_COMPONENT, vhost, name)
            .await
    }

    //
    // Shovels
    //

    /// Lists [shovel](https://www.rabbitmq.com/docs/shovel) across all virtual hosts in the cluster.
    pub async fn list_shovels(&self) -> Result<Vec<responses::Shovel>> {
        let response = self.http_get("shovels", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists [dynamic shovels](https://www.rabbitmq.com/docs/shovel-dynamic) in a specific virtual host.
    pub async fn list_shovels_in(&self, vhost: &str) -> Result<Vec<responses::Shovel>> {
        let response = self.http_get(path!("shovels", vhost), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Declares [shovel](https://www.rabbitmq.com/docs/shovel) that will use the AMQP 0-9-1 protocol
    /// for both source and destination collection.
    pub async fn declare_amqp091_shovel(&self, params: Amqp091ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameters(&runtime_param).await
    }

    /// Declares [shovel](https://www.rabbitmq.com/docs/shovel) that will use the AMQP 1.0 protocol
    /// for both source and destination collection.
    pub async fn declare_amqp10_shovel(&self, params: Amqp10ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameters(&runtime_param).await
    }

    /// Deletes a shovel in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent shovel
    /// will fail.
    pub async fn delete_shovel(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self
            .http_delete(path!("shovels", "vhost", vhost, name), excludes, None)
            .await?;
        Ok(())
    }

    //
    // Publish and consume messages
    //

    /// Only use this function in tests and experiments.
    /// Always use a messaging or streaming protocol client for publishing in production.
    pub async fn publish_message(
        &self,
        vhost: &str,
        exchange: &str,
        routing_key: &str,
        payload: &str,
        properties: requests::MessageProperties,
    ) -> Result<responses::MessageRouted> {
        let body = serde_json::json!({
          "routing_key": routing_key,
          "payload": payload,
          "payload_encoding": "string",
          "properties": properties,
        });

        let response = self
            .http_post(
                path!("exchanges", vhost, exchange, "publish"),
                &body,
                None,
                None,
            )
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Only use this function in tests and experiments.
    /// Always use a messaging or streaming protocol client for consuming in production.
    pub async fn get_messages(
        &self,
        vhost: &str,
        queue: &str,
        count: u32,
        ack_mode: &str,
    ) -> Result<Vec<GetMessage>> {
        let body = json!({
          "count": count,
          "ackmode": ack_mode,
          "encoding": "auto"
        });

        let response = self
            .http_post(path!("queues", vhost, queue, "get"), &body, None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Provides an overview of the most commonly used cluster metrics.
    /// See `crate::responses::Overview`.
    pub async fn overview(&self) -> Result<responses::Overview> {
        let response = self.http_get("overview", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns the version of RabbitMQ used by the API endpoint.
    pub async fn server_version(&self) -> Result<String> {
        let response = self.http_get("overview", None, None).await?;
        let response: Overview = response.json().await?;

        Ok(response.rabbitmq_version)
    }

    //
    // Feature flags
    //

    /// Lists all feature flags and their current states.
    /// See [Feature Flags Guide](https://www.rabbitmq.com/docs/feature-flags) to learn more.
    pub async fn list_feature_flags(&self) -> Result<FeatureFlagList> {
        let response = self.http_get("feature-flags", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Enables a specific feature flag by name.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    /// See [Feature Flags Guide](https://www.rabbitmq.com/docs/feature-flags) to learn more.
    pub async fn enable_feature_flag(&self, name: &str) -> Result<()> {
        let body = serde_json::json!({
            "name": name
        });
        let _response = self
            .http_put(path!("feature-flags", name, "enable"), &body, None, None)
            .await?;
        Ok(())
    }

    /// Enables all stable feature flags in the cluster.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    /// See [Feature Flags Guide](https://www.rabbitmq.com/docs/feature-flags) to learn more.
    pub async fn enable_all_stable_feature_flags(&self) -> Result<()> {
        // PUT /api/feature-flags/{name}/enable does not support the special 'all' value like 'rabbitmqctl enable_feature_flag' does.
        // Thus we do what management UI does: discover the stable disabled flags and enable
        // them one by one.
        let discovered_flags = self.list_feature_flags().await?;
        let flags_to_enable: Vec<&FeatureFlag> = discovered_flags
            .0
            .iter()
            .filter(|&ff| {
                ff.state == FeatureFlagState::Disabled
                    && ff.stability == FeatureFlagStability::Stable
            })
            .collect();

        for ff in flags_to_enable {
            self.enable_feature_flag(&ff.name).await?;
        }

        Ok(())
    }

    //
    // Deprecated Features
    //

    /// Lists all deprecated features and their usage status.
    /// Deprecated features may be removed in future RabbitMQ versions.
    /// See [Deprecated Features Guide](https://www.rabbitmq.com/docs/deprecated-features) to learn more.
    pub async fn list_all_deprecated_features(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists deprecated features that are currently being used in the cluster.
    /// These features should be migrated away from as soon as possible.
    /// See [Deprecated Features Guide](https://www.rabbitmq.com/docs/deprecated-features) to learn more.
    pub async fn list_deprecated_features_in_use(&self) -> Result<DeprecatedFeatureList> {
        let response = self
            .http_get("deprecated-features/used", None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    //
    // OAuth 2 Configuration
    //

    /// Returns the current OAuth 2.0 configuration for authentication.
    /// See [OAuth 2 Guide](https://www.rabbitmq.com/docs/oauth2) to learn more.
    pub async fn oauth_configuration(&self) -> Result<OAuthConfiguration> {
        let response = self.http_get("auth", None, None).await?;
        let response = response.json().await?;

        Ok(response)
    }

    //
    // Authentication attempt statistics
    //

    /// Returns authentication attempt statistics for a given node.
    pub async fn auth_attempts_statistics(
        &self,
        node: &str,
    ) -> Result<Vec<AuthenticationAttemptStatistics>> {
        let response = self
            .http_get(path!("auth", "attempts", node), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    //
    // Schema Definition Sync (Tanzu RabbitMQ)
    //

    /// Returns the status of schema definition synchronization.
    /// Schema definition sync is a Tanzu RabbitMQ-specific feature.
    pub async fn schema_definition_sync_status(
        &self,
        node: Option<&str>,
    ) -> Result<SchemaDefinitionSyncStatus> {
        let response = match node {
            Some(val) => {
                self.http_get(path!("tanzu", "osr", "schema", "status", val), None, None)
                    .await?
            }
            None => self.http_get("tanzu/osr/schema/status", None, None).await?,
        };
        let response = response.json().await?;

        Ok(response)
    }

    /// Enables schema definition synchronization on a single node or cluster-wide.
    /// Schema definition sync is a Tanzu RabbitMQ-specific feature.
    pub async fn enable_schema_definition_sync_one_node(&self, node: Option<&str>) -> Result<()> {
        let payload = EmptyPayload::new();
        let _ = match node {
            Some(val) => {
                self.http_put(
                    path!("tanzu", "osr", "schema", "enable", val),
                    &payload,
                    None,
                    None,
                )
                .await?
            }
            None => {
                self.http_put("tanzu/osr/schema/enable", &payload, None, None)
                    .await?
            }
        };

        Ok(())
    }

    /// Disables schema definition synchronization on a specific node.
    /// Schema definition sync is a Tanzu RabbitMQ-specific feature.
    pub async fn disable_schema_definition_sync_on_node(&self, node: Option<&str>) -> Result<()> {
        let _ = match node {
            Some(val) => {
                self.http_delete(path!("tanzu", "osr", "schema", "disable", val), None, None)
                    .await?
            }
            None => {
                self.http_delete("tanzu/osr/schema/disable", None, None)
                    .await?
            }
        };

        Ok(())
    }

    /// Enables schema definition synchronization cluster-wide.
    /// Schema definition sync is a Tanzu RabbitMQ-specific feature.
    pub async fn enable_schema_definition_sync(&self) -> Result<()> {
        let payload = EmptyPayload::new();
        let _ = self
            .http_put("tanzu/osr/schema/enable-cluster-wide", &payload, None, None)
            .await?;

        Ok(())
    }

    /// Disables schema definition synchronization cluster-wide.
    /// Schema definition sync is a Tanzu RabbitMQ-specific feature.
    pub async fn disable_schema_definition_sync(&self) -> Result<()> {
        let _ = self
            .http_delete("tanzu/osr/schema/disable-cluster-wide", None, None)
            .await?;

        Ok(())
    }

    //
    // Warm Standby Replication (Tanzu RabbitMQ)
    //

    /// Returns the status of warm standby replication.
    /// Warm Standby Replication is a Tanzu RabbitMQ-specific feature.
    pub async fn warm_standby_replication_status(&self) -> Result<WarmStandbyReplicationStatus> {
        let response = self
            .http_get("tanzu/osr/standby/status", None, None)
            .await?;
        let response = response.json().await?;

        Ok(response)
    }

    //
    // Implementation
    //

    async fn get_api_request<T, S>(&self, path: S) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        S: AsRef<str>,
    {
        let response = self.http_get(path, None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    async fn delete_api_request_with_optional_not_found<S>(
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

    async fn put_api_request<S, T>(&self, path: S, payload: &T) -> Result<()>
    where
        S: AsRef<str>,
        T: Serialize,
    {
        self.http_put(path, payload, None, None).await?;
        Ok(())
    }

    async fn declare_shovel_parameters(
        &self,
        runtime_param: &RuntimeParameterDefinition<'_>,
    ) -> Result<()> {
        let _response = self
            .http_put(
                path!(
                    "parameters",
                    SHOVEL_COMPONENT,
                    runtime_param.vhost,
                    runtime_param.name
                ),
                &runtime_param,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    async fn declare_federation_upstream_with_parameters(
        &self,
        runtime_param: &RuntimeParameterDefinition<'_>,
    ) -> Result<()> {
        let _response = self
            .http_put(
                path!(
                    "parameters",
                    FEDERATION_UPSTREAM_COMPONENT,
                    runtime_param.vhost,
                    runtime_param.name
                ),
                &runtime_param,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    async fn health_check_alarms(&self, path: &str) -> Result<()> {
        // we expect that StatusCode::SERVICE_UNAVAILABLE may be return and ignore
        // it here to provide a custom error type later
        let response = self
            .http_get(path, None, Some(StatusCode::SERVICE_UNAVAILABLE))
            .await?;
        let status_code = response.status();
        if status_code.is_success() {
            return Ok(());
        }

        let body = response.json().await?;
        let failure_details = responses::HealthCheckFailureDetails::AlarmCheck(body);
        Err(Error::HealthCheckFailed {
            path: path.to_owned(),
            details: failure_details,
            status_code,
        })
    }

    async fn list_exchange_bindings_with_source_or_destination(
        &self,
        vhost: &str,
        exchange: &str,
        vertex: BindindVertex,
    ) -> Result<Vec<responses::BindingInfo>> {
        let response = self
            .http_get(
                path!("exchanges", vhost, exchange, "bindings", vertex),
                None,
                None,
            )
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    async fn http_get<S>(
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
            .send()
            .await?;
        let response = self
            .ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await?;
        Ok(response)
    }

    async fn http_put<S, T>(
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
            .send()
            .await?;
        let response = self
            .ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await?;
        Ok(response)
    }

    async fn http_post<S, T>(
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
            .send()
            .await?;
        let response = self
            .ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await?;
        Ok(response)
    }

    async fn http_delete<S>(
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
            .send()
            .await?;
        let response = self
            .ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await?;
        Ok(response)
    }

    async fn http_delete_with_headers<S>(
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
            .send()
            .await?;
        let response = self
            .ok_or_status_code_error(
                response,
                client_code_to_accept_or_ignore,
                server_code_to_accept_or_ignore,
            )
            .await?;
        Ok(response)
    }

    async fn ok_or_status_code_error(
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
                    let body = response.text().await?;
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
                    let body = response.text().await?;
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

    fn rooted_path<S>(&self, path: S) -> String
    where
        S: AsRef<str>,
    {
        format!("{}/{}", self.endpoint, path.as_ref())
    }
}

impl Default for Client<&'static str, &'static str, &'static str> {
    fn default() -> Self {
        Self::new("http://localhost:15672", "guest", "guest")
    }
}

#[derive(Debug, Clone, Copy)]
enum BindindVertex {
    Source,
    Destination,
}

impl AsRef<str> for BindindVertex {
    fn as_ref(&self) -> &str {
        match self {
            Self::Source => "source",
            Self::Destination => "destination",
        }
    }
}
