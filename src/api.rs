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
    ClusterTags, DeprecatedFeatureList, FeatureFlag, FeatureFlagList, FeatureFlagStability,
    FeatureFlagState, FederationUpstream, GetMessage, OAuthConfiguration, Overview,
    SchemaDefinitionSyncStatus, VirtualHostDefinitionSet, WarmStandbyReplicationStatus,
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
    /// Constructs a new `ClientBuilder`.
    ///
    /// This is the same as `Client::builder()`.
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

    pub fn with_client(self, client: HttpClient) -> Self {
        ClientBuilder { client, ..self }
    }

    /// Returns a `Client` that uses this `ClientBuilder` configuration.
    pub fn build(self) -> Client<E, U, P> {
        Client::from_http_client(self.client, self.endpoint, self.username, self.password)
    }
}

/// A client for the [RabbitMQ HTTP API](https://rabbitmq.com/docs/management/#http-api).
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
    pub async fn list_nodes(&self) -> Result<Vec<responses::ClusterNode>> {
        let response = self.http_get("nodes", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists virtual hosts in the cluster.
    pub async fn list_vhosts(&self) -> Result<Vec<responses::VirtualHost>> {
        let response = self.http_get("vhosts", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists users in the internal database.
    pub async fn list_users(&self) -> Result<Vec<responses::User>> {
        let response = self.http_get("users", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists users in the internal database that do not have access
    /// to any virtual hosts.
    pub async fn list_users_without_permissions(&self) -> Result<Vec<responses::User>> {
        let response = self
            .http_get("users/without-permissions", None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all AMQP 1.0 and 0-9-1 client connections across the cluster.
    pub async fn list_connections(&self) -> Result<Vec<responses::Connection>> {
        let response = self.http_get("connections", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_connection_info(&self, name: &str) -> Result<responses::Connection> {
        let response = self
            .http_get(path!("connections", name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn get_stream_connection_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::Connection> {
        let response = self
            .http_get(
                path!("stream", "connections", virtual_host, name),
                None,
                None,
            )
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

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
    pub async fn list_connections_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::Connection>> {
        let response = self
            .http_get(path!("vhosts", virtual_host, "connections"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all connections of a specific user.
    pub async fn list_user_connections(
        &self,
        username: &str,
    ) -> Result<Vec<responses::UserConnection>> {
        let response = self
            .http_get(path!("connections", "username", username), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all RabbitMQ Stream Protocol client connections across the cluster.
    pub async fn list_stream_connections(&self) -> Result<Vec<responses::Connection>> {
        let response = self.http_get("stream/connections", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists RabbitMQ Stream Protocol client connections in the given virtual host.
    pub async fn list_stream_connections_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::Connection>> {
        let response = self
            .http_get(path!("stream", "connections", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all channels across the cluster.
    pub async fn list_channels(&self) -> Result<Vec<responses::Channel>> {
        let response = self.http_get("channels", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all channels in the given virtual host.
    pub async fn list_channels_in(&self, virtual_host: &str) -> Result<Vec<responses::Channel>> {
        let response = self
            .http_get(path!("vhosts", virtual_host, "channels"), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
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
    pub async fn list_queues(&self) -> Result<Vec<responses::QueueInfo>> {
        let response = self.http_get("queues", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all queues and streams in the given virtual host.
    pub async fn list_queues_in(&self, virtual_host: &str) -> Result<Vec<responses::QueueInfo>> {
        let response = self
            .http_get(path!("queues", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all exchanges across the cluster.
    pub async fn list_exchanges(&self) -> Result<Vec<responses::ExchangeInfo>> {
        let response = self.http_get("exchanges", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all exchanges in the given virtual host.
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
    pub async fn list_consumers(&self) -> Result<Vec<responses::Consumer>> {
        let response = self.http_get("consumers", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all consumers in the given virtual host.
    pub async fn list_consumers_in(&self, virtual_host: &str) -> Result<Vec<responses::Consumer>> {
        let response = self
            .http_get(path!("consumers", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a cluster node.
    pub async fn get_node_info(&self, name: &str) -> Result<responses::ClusterNode> {
        let response = self.http_get(path!("nodes", name), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a cluster node.
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
    pub async fn get_vhost(&self, name: &str) -> Result<responses::VirtualHost> {
        let response = self.http_get(path!("vhosts", name), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a user in the internal database.
    pub async fn get_user(&self, name: &str) -> Result<responses::User> {
        let response = self.http_get(path!("users", name), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about a queue or stream.
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
    pub async fn get_stream_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::QueueInfo> {
        self.get_queue_info(virtual_host, name).await
    }

    /// Returns information about an exchange.
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
        let _response = self
            .http_put(path!("vhosts", params.name), params, None, None)
            .await?;
        Ok(())
    }

    /// Adds a user to the internal database.
    ///
    /// See [`UserParams`] and [`crate::password_hashing`].
    pub async fn create_user(&self, params: &UserParams<'_>) -> Result<()> {
        let _response = self
            .http_put(path!("users", params.name), params, None, None)
            .await?;
        Ok(())
    }

    pub async fn declare_permissions(&self, params: &Permissions<'_>) -> Result<()> {
        let _response = self
            .http_put(
                // /api/permissions/vhost/user
                path!("permissions", params.vhost, params.user),
                params,
                None,
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn grant_permissions(&self, vhost: &str, user: &str) -> Result<()> {
        let _response = self
            .http_delete(path!("permissions", vhost, user), None, None)
            .await?;
        Ok(())
    }

    pub async fn declare_queue(&self, vhost: &str, params: &QueueParams<'_>) -> Result<()> {
        let _response = self
            .http_put(path!("queues", vhost, params.name), params, None, None)
            .await?;
        Ok(())
    }

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

    pub async fn declare_exchange(&self, vhost: &str, params: &ExchangeParams<'_>) -> Result<()> {
        let _response = self
            .http_put(path!("exchanges", vhost, params.name), params, None, None)
            .await?;
        Ok(())
    }

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

    pub async fn delete_vhost(&self, vhost: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self
            .http_delete(path!("vhosts", vhost), excludes, None)
            .await?;
        Ok(())
    }

    pub async fn delete_user(&self, username: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self
            .http_delete(path!("users", username), excludes, None)
            .await?;
        Ok(())
    }

    pub async fn delete_users(&self, usernames: Vec<&str>) -> Result<()> {
        let delete = BulkUserDelete { usernames };
        let _response = self
            .http_post(path!("users", "bulk-delete"), &delete, None, None)
            .await?;
        Ok(())
    }

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

    pub async fn delete_queue(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self
            .http_delete(path!("queues", vhost, name), excludes, None)
            .await?;
        Ok(())
    }

    pub async fn delete_stream(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_queue(vhost, name, idempotently).await
    }

    pub async fn delete_exchange(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self
            .http_delete(path!("exchanges", vhost, name), excludes, None)
            .await?;
        Ok(())
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

    pub async fn purge_queue(&self, virtual_host: &str, name: &str) -> Result<()> {
        let _response = self
            .http_delete(path!("queues", virtual_host, name, "contents"), None, None)
            .await?;
        Ok(())
    }

    pub async fn list_runtime_parameters(&self) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get("parameters", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

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

    pub async fn list_all_vhost_limits(&self) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self.http_get("vhost-limits", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

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

    pub async fn get_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self
            .http_get(path!("policies", vhost, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("policies", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn list_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("policies", vhost), None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

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

    pub async fn list_permissions(&self) -> Result<Vec<responses::Permissions>> {
        let response = self.http_get("permissions", None, None).await?;
        let response = response.json().await?;
        Ok(response)
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

    pub async fn get_permissions(&self, vhost: &str, user: &str) -> Result<responses::Permissions> {
        let response = self
            .http_get(path!("permissions", vhost, user), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
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

    pub async fn export_cluster_wide_definitions(&self) -> Result<String> {
        self.export_cluster_wide_definitions_as_string().await
    }

    pub async fn export_cluster_wide_definitions_as_string(&self) -> Result<String> {
        let response = self.http_get("definitions", None, None).await?;
        let response = response.text().await?;
        Ok(response)
    }

    pub async fn export_cluster_wide_definitions_as_data(&self) -> Result<ClusterDefinitionSet> {
        let response = self.http_get("definitions", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn export_vhost_definitions(&self, vhost: &str) -> Result<String> {
        self.export_vhost_definitions_as_string(vhost).await
    }

    pub async fn export_vhost_definitions_as_string(&self, vhost: &str) -> Result<String> {
        let response = self
            .http_get(path!("definitions", vhost), None, None)
            .await?;
        let response = response.text().await?;
        Ok(response)
    }

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

    pub async fn import_definitions(&self, definitions: Value) -> Result<()> {
        self.import_cluster_wide_definitions(definitions).await
    }

    pub async fn import_cluster_wide_definitions(&self, definitions: Value) -> Result<()> {
        self.http_post("definitions", &definitions, None, None)
            .await?;
        Ok(())
    }

    pub async fn import_vhost_definitions(&self, vhost: &str, definitions: Value) -> Result<()> {
        self.http_post(path!("definitions", vhost), &definitions, None, None)
            .await?;
        Ok(())
    }

    //
    // Health Checks
    //

    pub async fn health_check_cluster_wide_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/alarms").await
    }

    pub async fn health_check_local_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/local-alarms").await
    }

    pub async fn health_check_if_node_is_quorum_critical(&self) -> Result<()> {
        let path = "health/checks/node-is-quorum-critical";
        self.boolean_health_check(path).await
    }

    pub async fn health_check_port_listener(&self, port: u16) -> Result<()> {
        let port_s = port.to_string();
        let path = path!("health", "checks", "port-listener", port_s);
        self.boolean_health_check(&path).await
    }

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

    pub async fn list_federation_links(&self) -> Result<Vec<responses::FederationLink>> {
        let response = self.http_get("federation-links", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn declare_federation_upstream(
        &self,
        params: FederationUpstreamParams<'_>,
    ) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_federation_upstream_with_parameters(&runtime_param)
            .await
    }

    pub async fn delete_federation_upstream(&self, vhost: &str, name: &str) -> Result<()> {
        self.clear_runtime_parameter(FEDERATION_UPSTREAM_COMPONENT, vhost, name)
            .await
    }

    //
    // Shovels
    //

    pub async fn list_shovels(&self) -> Result<Vec<responses::Shovel>> {
        let response = self.http_get("shovels", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn declare_amqp091_shovel(&self, params: Amqp091ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameters(&runtime_param).await
    }

    pub async fn declare_amqp10_shovel(&self, params: Amqp10ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameters(&runtime_param).await
    }

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

    pub async fn overview(&self) -> Result<responses::Overview> {
        let response = self.http_get("overview", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    pub async fn server_version(&self) -> Result<String> {
        let response = self.http_get("overview", None, None).await?;
        let response: Overview = response.json().await?;

        Ok(response.rabbitmq_version)
    }

    //
    // Feature flags
    //

    /// Enables a feature flag.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    pub async fn list_feature_flags(&self) -> Result<FeatureFlagList> {
        let response = self.http_get("feature-flags", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Enables all stable feature flags.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    pub async fn enable_feature_flag(&self, name: &str) -> Result<()> {
        let body = serde_json::json!({
            "name": name
        });
        let _response = self
            .http_put(path!("feature-flags", name, "enable"), &body, None, None)
            .await?;
        Ok(())
    }

    /// Enables all stable feature flags.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
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

    pub async fn list_all_deprecated_features(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

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

    pub async fn oauth_configuration(&self) -> Result<OAuthConfiguration> {
        let response = self.http_get("auth", None, None).await?;
        let response = response.json().await?;

        Ok(response)
    }

    //
    // Schema Definition Sync (Tanzu RabbitMQ)
    //

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

    pub async fn enable_schema_definition_sync(&self) -> Result<()> {
        let payload = EmptyPayload::new();
        let _ = self
            .http_put("tanzu/osr/schema/enable-cluster-wide", &payload, None, None)
            .await?;

        Ok(())
    }

    pub async fn disable_schema_definition_sync(&self) -> Result<()> {
        let _ = self
            .http_delete("tanzu/osr/schema/disable-cluster-wide", None, None)
            .await?;

        Ok(())
    }

    //
    // Warm Standby Replication (Tanzu RabbitMQ)
    //

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
