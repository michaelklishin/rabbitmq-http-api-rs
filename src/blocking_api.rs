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
    FeatureFlagState, FederationUpstream, GetMessage, OAuthConfiguration, VirtualHostDefinitionSet,
    WarmStandbyReplicationStatus,
};
use crate::{
    commons::{BindingDestinationType, SupportedProtocol, UserLimitTarget, VirtualHostLimitTarget},
    path,
    requests::{
        self, BulkUserDelete, EnforcedLimitParams, ExchangeParams, Permissions, PolicyParams,
        QueueParams, RuntimeParameterDefinition, UserParams, VirtualHostParams, XArguments,
    },
    responses::{self, BindingInfo, ClusterDefinitionSet, SchemaDefinitionSyncStatus},
};
use backtrace::Backtrace;
use reqwest::{
    StatusCode,
    blocking::Client as HttpClient,
    header::{HeaderMap, HeaderValue},
};
use serde::Serialize;
use serde_json::{Map, Value, json};
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

    /// Lists cluster nodes.
    pub fn list_nodes(&self) -> Result<Vec<responses::ClusterNode>> {
        self.get_api_request("nodes")
    }

    /// Lists virtual hosts in the cluster.
    pub fn list_vhosts(&self) -> Result<Vec<responses::VirtualHost>> {
        self.get_api_request("vhosts")
    }

    /// Lists users in the internal database.
    pub fn list_users(&self) -> Result<Vec<responses::User>> {
        self.get_api_request("users")
    }

    /// Lists users in the internal database that do not have access
    /// to any virtual hosts.
    pub fn list_users_without_permissions(&self) -> Result<Vec<responses::User>> {
        self.get_api_request("users/without-permissions")
    }

    /// Lists all AMQP 1.0 and 0-9-1 client connections across the cluster.
    pub fn list_connections(&self) -> Result<Vec<responses::Connection>> {
        self.get_api_request("connections")
    }

    /// Returns information about a connection.
    ///
    /// Connection name is usually obtained from `crate::responses::Connection` or `crate::responses::UserConnection`,
    /// e.g. via `Client#list_connections`, `Client#list_connections_in`, `Client#list_user_connections`.
    pub fn get_connection_info(&self, name: &str) -> Result<responses::Connection> {
        self.get_api_request(path!("connections", name))
    }

    /// Returns information about a stream connection.
    ///
    /// Connection name is usually obtained from `crate::responses::Connection` or `crate::responses::UserConnection`,
    /// e.g. via `Client#list_stream_connections`, `Client#list_stream_connections_in`, `Client#list_user_connections`.
    pub fn get_stream_connection_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::Connection> {
        self.get_api_request(path!("stream", "connections", virtual_host, name))
    }

    /// Closes a connection with an optional reason.
    ///
    /// The reason will be passed on in the connection error to the client and will be logged on the RabbitMQ end.
    pub fn close_connection(&self, name: &str, reason: Option<&str>) -> Result<()> {
        match reason {
            None => self.http_delete(
                path!("connections", name),
                Some(StatusCode::NOT_FOUND),
                None,
            )?,
            Some(value) => {
                let mut headers = HeaderMap::new();
                let hdr = HeaderValue::from_str(value)?;
                headers.insert("X-Reason", hdr);
                self.http_delete_with_headers(path!("connections", name), headers, None, None)?
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
    pub fn close_user_connections(&self, username: &str, reason: Option<&str>) -> Result<()> {
        match reason {
            None => self.http_delete(
                path!("connections", "username", username),
                Some(StatusCode::NOT_FOUND),
                None,
            )?,
            Some(value) => {
                let mut headers = HeaderMap::new();
                let hdr = HeaderValue::from_str(value)?;
                headers.insert("X-Reason", hdr);
                self.http_delete_with_headers(
                    path!("connections", "username", username),
                    headers,
                    None,
                    None,
                )?
            }
        };
        Ok(())
    }

    /// Lists all connections in the given virtual host.
    pub fn list_connections_in(&self, virtual_host: &str) -> Result<Vec<responses::Connection>> {
        self.get_api_request(path!("vhosts", virtual_host, "connections"))
    }

    /// Lists all connections of a specific user.
    pub fn list_user_connections(&self, username: &str) -> Result<Vec<responses::UserConnection>> {
        self.get_api_request(path!("connections", "username", username))
    }

    /// Lists all RabbitMQ Stream Protocol client connections across the cluster.
    pub fn list_stream_connections(&self) -> Result<Vec<responses::Connection>> {
        self.get_api_request("stream/connections")
    }

    /// Lists RabbitMQ Stream Protocol client connections in the given virtual host.
    pub fn list_stream_connections_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::Connection>> {
        self.get_api_request(path!("stream", "connections", virtual_host))
    }

    /// Lists all channels across the cluster.
    pub fn list_channels(&self) -> Result<Vec<responses::Channel>> {
        self.get_api_request("channels")
    }

    /// Lists all channels in the given virtual host.
    pub fn list_channels_in(&self, virtual_host: &str) -> Result<Vec<responses::Channel>> {
        self.get_api_request(path!("vhosts", virtual_host, "channels"))
    }

    /// Lists all stream publishers across the cluster.
    pub fn list_stream_publishers(&self) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!("stream", "publishers"))
    }

    /// Lists stream publishers publishing to the given stream.
    pub fn list_stream_publishers_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!("stream", "publishers", virtual_host))
    }

    /// Lists stream publishers of the given stream.
    pub fn list_stream_publishers_of(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!("stream", "publishers", virtual_host, name))
    }

    /// Lists stream publishers on the given stream connection.
    pub fn list_stream_publishers_on_connection(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!(
            "stream",
            "connections",
            virtual_host,
            name,
            "publishers"
        ))
    }

    /// Lists all stream consumers across the cluster.
    pub fn list_stream_consumers(&self) -> Result<Vec<responses::StreamConsumer>> {
        self.get_api_request(path!("stream", "consumers"))
    }

    /// Lists stream consumers on connections in the given virtual host.
    pub fn list_stream_consumers_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::StreamConsumer>> {
        self.get_api_request(path!("stream", "consumers", virtual_host))
    }

    /// Lists stream consumers on the given stream connection.
    pub fn list_stream_consumers_on_connection(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamConsumer>> {
        self.get_api_request(path!(
            "stream",
            "connections",
            virtual_host,
            name,
            "consumers"
        ))
    }

    /// Lists all queues and streams across the cluster.
    pub fn list_queues(&self) -> Result<Vec<responses::QueueInfo>> {
        self.get_api_request("queues")
    }

    /// Lists all queues and streams in the given virtual host.
    pub fn list_queues_in(&self, virtual_host: &str) -> Result<Vec<responses::QueueInfo>> {
        self.get_api_request(path!("queues", virtual_host))
    }

    /// Lists all exchanges across the cluster.
    pub fn list_exchanges(&self) -> Result<Vec<responses::ExchangeInfo>> {
        self.get_api_request("exchanges")
    }

    /// Lists all exchanges in the given virtual host.
    pub fn list_exchanges_in(&self, virtual_host: &str) -> Result<Vec<responses::ExchangeInfo>> {
        self.get_api_request(path!("exchanges", virtual_host))
    }

    /// Lists all bindings (both queue-to-exchange and exchange-to-exchange ones) across the cluster.
    pub fn list_bindings(&self) -> Result<Vec<responses::BindingInfo>> {
        self.get_api_request("bindings")
    }

    /// Lists all bindings (both queue-to-exchange and exchange-to-exchange ones)  in the given virtual host.
    pub fn list_bindings_in(&self, virtual_host: &str) -> Result<Vec<responses::BindingInfo>> {
        self.get_api_request(path!("bindings", virtual_host))
    }

    /// Lists all bindings of a specific queue.
    pub fn list_queue_bindings(
        &self,
        virtual_host: &str,
        queue: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        self.get_api_request(path!("queues", virtual_host, queue, "bindings"))
    }

    /// Lists all bindings of a specific exchange where it is the source.
    pub fn list_exchange_bindings_with_source(
        &self,
        virtual_host: &str,
        exchange: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        self.list_exchange_bindings_with_source_or_destination(
            virtual_host,
            exchange,
            BindindVertex::Source,
        )
    }

    /// Lists all bindings of a specific exchange where it is the destination.
    pub fn list_exchange_bindings_with_destination(
        &self,
        virtual_host: &str,
        exchange: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        self.list_exchange_bindings_with_source_or_destination(
            virtual_host,
            exchange,
            BindindVertex::Destination,
        )
    }

    /// Lists all consumers across the cluster.
    pub fn list_consumers(&self) -> Result<Vec<responses::Consumer>> {
        self.get_api_request("consumers")
    }

    /// Lists all consumers in the given virtual host.
    pub fn list_consumers_in(&self, virtual_host: &str) -> Result<Vec<responses::Consumer>> {
        self.get_api_request(path!("consumers", virtual_host))
    }

    /// Returns information about a cluster node.
    pub fn get_node_info(&self, name: &str) -> Result<responses::ClusterNode> {
        self.get_api_request(path!("nodes", name))
    }

    /// Returns information about a cluster node.
    pub fn get_node_memory_footprint(&self, name: &str) -> Result<responses::NodeMemoryFootprint> {
        self.get_api_request(path!("nodes", name, "memory"))
    }

    /// Returns information about a virtual host.
    pub fn get_vhost(&self, name: &str) -> Result<responses::VirtualHost> {
        self.get_api_request(path!("vhosts", name))
    }

    /// Returns information about a user in the internal database.
    pub fn get_user(&self, name: &str) -> Result<responses::User> {
        self.get_api_request(path!("users", name))
    }

    /// Returns information about a queue or stream.
    pub fn get_queue_info(&self, virtual_host: &str, name: &str) -> Result<responses::QueueInfo> {
        self.get_api_request(path!("queues", virtual_host, name))
    }

    /// Returns information about a stream.
    pub fn get_stream_info(&self, virtual_host: &str, name: &str) -> Result<responses::QueueInfo> {
        self.get_queue_info(virtual_host, name)
    }

    /// Returns information about an exchange.
    pub fn get_exchange_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::ExchangeInfo> {
        self.get_api_request(path!("exchanges", virtual_host, name))
    }

    /// Creates a virtual host.
    ///
    /// See [`VirtualHostParams`]
    pub fn create_vhost(&self, params: &VirtualHostParams) -> Result<()> {
        self.update_vhost(params)
    }

    /// Creates a virtual host or updates metadata of an existing one.
    ///
    /// See [`VirtualHostParams`]
    pub fn update_vhost(&self, params: &VirtualHostParams) -> Result<()> {
        self.put_api_request(path!("vhosts", params.name), params)
    }

    /// Adds a user to the internal database.
    ///
    /// See [`UserParams`] and [`crate::password_hashing`].
    pub fn create_user(&self, params: &UserParams) -> Result<()> {
        self.put_api_request(path!("users", params.name), params)
    }

    /// Sets permissions for a user on a specific virtual host.
    ///
    /// Permissions in RabbitMQ consist of configure, write, and read privileges
    /// that are defined using regular expressions. This function will create or
    /// update the permissions for the specified user and virtual host combination.
    pub fn declare_permissions(&self, params: &Permissions) -> Result<()> {
        self.put_api_request(path!("permissions", params.vhost, params.user), params)
    }

    /// Grants full permissions for a user on a virtual host.
    ///
    /// "Full permissions" here means the permissions that match all objects, that is,
    /// ".*" for every permission category.
    pub fn grant_permissions(&self, vhost: &str, user: &str) -> Result<()> {
        self.http_delete(path!("permissions", vhost, user), None, None)?;
        Ok(())
    }

    /// Declares a [queue](https://www.rabbitmq.com/docs/queues).
    ///
    /// If the queue already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub fn declare_queue(&self, vhost: &str, params: &QueueParams) -> Result<()> {
        self.put_api_request(path!("queues", vhost, params.name), params)
    }

    // Helper methods for common patterns
    fn get_api_request<T, S>(&self, path: S) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        S: AsRef<str>,
    {
        let response = self.http_get(path, None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    fn delete_api_request_with_optional_not_found<S>(&self, path: S, idempotent: bool) -> Result<()>
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

    fn put_api_request<S, T>(&self, path: S, payload: &T) -> Result<()>
    where
        S: AsRef<str>,
        T: Serialize,
    {
        self.http_put(path, payload, None, None)?;
        Ok(())
    }

    /// Declares a [RabbitMQ stream](https://www.rabbitmq.com/docs/streams).
    ///
    /// Streams are a durable, replicated, long-lived data structure in RabbitMQ designed for
    /// high-throughput scenarios. Unlike traditional queues, streams are append-only
    /// logs that support multiple consumers reading from different offsets.
    ///
    /// If the stream already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub fn declare_stream(&self, vhost: &str, params: &StreamParams<'_>) -> Result<()> {
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
        let _response =
            self.http_put(path!("queues", vhost, params.name), &q_params, None, None)?;
        Ok(())
    }

    /// Declares an [exchange](https://www.rabbitmq.com/docs/exchanges).
    ///
    /// If the exchange already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub fn declare_exchange(&self, vhost: &str, params: &ExchangeParams) -> Result<()> {
        self.put_api_request(path!("exchanges", vhost, params.name), params)
    }

    /// Binds a queue or a stream to an exchange.
    ///
    /// Bindings determine how messages published to an exchange are routed to queues.
    /// The exchange type, routing key and arguments define the routing behavior.
    ///
    /// Both the source (exchange) and destination (queue or stream) must exist.
    pub fn bind_queue(
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

        let _response = self.http_post(
            path!("bindings", vhost, "e", exchange, "q", queue),
            &body,
            None,
            None,
        )?;
        Ok(())
    }

    /// Bindgs one exchange to another (creates and [exchange-to-exchange binding](https://www.rabbitmq.com/docs/e2e)).
    ///
    /// This allows messages published to the source exchange to be forwarded to
    ///
    /// Exchange-to-exchange bindings enable complex routing topologies and
    /// message flow patterns.
    ///
    /// Both source and destination exchanges must exist.
    pub fn bind_exchange(
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

        let _response = self.http_post(
            path!("bindings", vhost, "e", source, "e", destination),
            &body,
            None,
            None,
        )?;
        Ok(())
    }

    /// Deletes a virtual host and all its contents.
    ///
    /// This is a destructive operation that will permanently remove the virtual host
    /// along with all queues, exchanges, bindings, and messages it contains. All
    /// connections to this virtual host will be closed. If `idempotently` is true,
    /// the operation will succeed even if the virtual host doesn't exist.
    pub fn delete_vhost(&self, vhost: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("vhosts", vhost), idempotently)
    }

    /// Deletes a user from the internal RabbitMQ user database.
    ///
    /// This removes the user account entirely, including all associated permissions
    /// across all virtual hosts. Active connections belonging to this user will be
    /// closed. If `idempotently` is true, the operation will succeed even if the
    /// user doesn't exist.
    pub fn delete_user(&self, username: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("users", username), idempotently)
    }

    /// Deletes multiple users from the internal database in a single operation.
    ///
    /// This is more efficient than calling `delete_user` multiple times when you
    /// need to remove several user accounts. All specified users will be deleted
    /// along with their permissions, and any active connections will be closed.
    /// Non-existent users in the list are silently ignored.
    pub fn delete_users(&self, usernames: Vec<&str>) -> Result<()> {
        let delete = BulkUserDelete { usernames };
        let _response = self.http_post(path!("users", "bulk-delete"), &delete, None, None)?;
        Ok(())
    }

    /// Removes all permissions for a user on a specific virtual host.
    ///
    /// After this operation, the user will no longer have configure, write, or read
    /// permissions on the specified virtual host, but their permissions on other
    /// virtual hosts remain unchanged. If `idempotently` is true, the operation
    /// succeeds even if no permissions existed.
    pub fn clear_permissions(&self, vhost: &str, username: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("permissions", vhost, username),
            idempotently,
        )
    }

    /// Deletes a queue and all its contents.
    ///
    /// This is a destructive operation that permanently removes the queue and all
    /// messages it contains. Any consumers connected to this queue will be disconnected.
    /// Bindings to this queue are also removed. If `idempotently` is true, the
    /// operation succeeds even if the queue doesn't exist.
    pub fn delete_queue(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("queues", vhost, name), idempotently)
    }

    /// Deletes a RabbitMQ stream and all its data.
    ///
    /// This permanently removes the stream and all stored messages. Unlike traditional
    /// queues, streams may have data replicated across multiple cluster nodes, so
    /// this operation will clean up all replicas. Any stream clients (publishers
    /// and consumers) will be disconnected. If `idempotently` is true, the operation
    /// succeeds even if the stream doesn't exist.
    pub fn delete_stream(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_queue(vhost, name, idempotently)
    }

    /// Deletes an exchange and all its bindings.
    ///
    /// This removes the exchange and all bindings where it serves as either source
    /// or destination. Messages currently being routed through this exchange may
    /// be lost. Built-in exchanges (like the default exchange) cannot be deleted.
    /// If `idempotently` is true, the operation succeeds even if the exchange doesn't exist.
    pub fn delete_exchange(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("exchanges", vhost, name),
            idempotently,
        )
    }

    /// Removes a specific binding between an exchange and queue/exchange.
    ///
    /// This operation requires exact matching of the source, destination, routing key,
    /// and arguments to identify the specific binding to delete. If multiple bindings
    /// match the criteria, an error is returned. The function first queries existing
    /// bindings to find the one to delete, then removes it.
    pub fn delete_binding(
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
            BindingDestinationType::Queue => self.list_queue_bindings(virtual_host, destination)?,
            BindingDestinationType::Exchange => {
                self.list_exchange_bindings_with_destination(virtual_host, destination)?
            }
        };

        let bs: Vec<&BindingInfo> = bindings
            .iter()
            .filter(|b| b.source == source && b.routing_key == routing_key && b.arguments.0 == args)
            .collect();
        match bs.len() {
            0 => Err(NotFound),
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
                let response = self.http_delete(&path, None, None)?;
                Ok(response)
            }
            _ => Err(Error::MultipleMatchingBindings),
        }
    }

    /// Removes all messages from a queue without deleting the queue itself.
    ///
    /// This operation immediately deletes all messages currently in the queue,
    /// but leaves the queue structure, bindings, and consumers intact. This is
    /// useful for clearing out accumulated messages during development or
    /// troubleshooting. The purged messages are permanently lost.
    pub fn purge_queue(&self, virtual_host: &str, name: &str) -> Result<()> {
        let _response =
            self.http_delete(path!("queues", virtual_host, name, "contents"), None, None)?;
        Ok(())
    }

    /// Lists all runtime parameters configured across the cluster.
    ///
    /// Runtime parameters are configuration values that can be set dynamically
    /// without restarting RabbitMQ. They are used by plugins like Federation
    /// and Shovel to store their configuration. This returns parameters from
    /// all components and virtual hosts.
    pub fn list_runtime_parameters(&self) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get("parameters", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all runtime parameters for a specific RabbitMQ component.
    ///
    /// Components like "federation-upstream", "shovel", and others use runtime
    /// parameters to store their configuration. This function returns only the
    /// parameters belonging to the specified component across all virtual hosts.
    pub fn list_runtime_parameters_of_component(
        &self,
        component: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get(path!("parameters", component), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists runtime parameters for a component within a specific virtual host.
    ///
    /// This narrows down the results to only parameters belonging to both the
    /// specified component (like "federation-upstream" or "shovel") and the
    /// specific virtual host. This is useful when managing component configurations
    /// that are scoped to individual virtual hosts.
    pub fn list_runtime_parameters_of_component_in(
        &self,
        component: &str,
        vhost: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get(path!("parameters", component, vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Retrieves a specific runtime parameter by component, virtual host, and name.
    ///
    /// Runtime parameters are uniquely identified by the combination of component
    /// name, virtual host, and parameter name. This function returns the detailed
    /// configuration for the specified parameter, including its value and metadata.
    pub fn get_runtime_parameter(
        &self,
        component: &str,
        vhost: &str,
        name: &str,
    ) -> Result<responses::RuntimeParameter> {
        let response = self.http_get(path!("parameters", component, vhost, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Creates a new runtime parameter or updates an existing one.
    ///
    /// Runtime parameters store configuration for RabbitMQ components like Federation
    /// and Shovel. If a parameter with the same component, virtual host, and name
    /// already exists, it will be updated with the new value. Otherwise, a new
    /// parameter is created.
    pub fn upsert_runtime_parameter<'a>(
        &self,
        param: &'a RuntimeParameterDefinition<'a>,
    ) -> Result<()> {
        let _response = self.http_put(
            path!("parameters", param.component, param.vhost, param.name),
            &param,
            None,
            None,
        )?;
        Ok(())
    }

    /// Removes a specific runtime parameter from the cluster.
    ///
    /// This permanently deletes the runtime parameter identified by the component,
    /// virtual host, and name. Any RabbitMQ component that was using this parameter
    /// will lose access to its configuration and may stop functioning until the
    /// parameter is recreated.
    pub fn clear_runtime_parameter(&self, component: &str, vhost: &str, name: &str) -> Result<()> {
        let _response =
            self.http_delete(path!("parameters", component, vhost, name), None, None)?;
        Ok(())
    }

    /// Removes all runtime parameters from the entire cluster.
    ///
    /// This is a destructive operation that deletes every runtime parameter across
    /// all components and virtual hosts. This will effectively disable all plugins
    /// that rely on runtime parameters (like Federation and Shovel) until their
    /// parameters are reconfigured. Use with extreme caution.
    pub fn clear_all_runtime_parameters(&self) -> Result<()> {
        let params = self.list_runtime_parameters()?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name)?
        }
        Ok(())
    }

    /// Deletes all runtime parameters for a component.
    pub fn clear_all_runtime_parameters_of_component(&self, component: &str) -> Result<()> {
        let params = self.list_runtime_parameters_of_component(component)?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name)?
        }
        Ok(())
    }

    /// Lists global runtime parameters.
    pub fn list_global_runtime_parameters(&self) -> Result<Vec<responses::GlobalRuntimeParameter>> {
        let response = self.http_get("global-parameters", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Gets a global runtime parameter.
    pub fn get_global_runtime_parameter(
        &self,
        name: &str,
    ) -> Result<responses::GlobalRuntimeParameter> {
        let response = self.http_get(path!("global-parameters", name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Creates or updates a global runtime parameter.
    pub fn upsert_global_runtime_parameter<'a>(
        &self,
        param: &'a GlobalRuntimeParameterDefinition<'a>,
    ) -> Result<()> {
        let _response =
            self.http_put(path!("global-parameters", param.name), &param, None, None)?;
        Ok(())
    }

    /// Deletes a global runtime parameter.
    pub fn clear_global_runtime_parameter(&self, name: &str) -> Result<()> {
        let _response = self.http_delete(path!("global-parameters", name), None, None)?;
        Ok(())
    }

    /// Sets a limit for a user.
    pub fn set_user_limit(
        &self,
        username: &str,
        limit: EnforcedLimitParams<UserLimitTarget>,
    ) -> Result<()> {
        let body = json!({"value": limit.value});
        let _response = self.http_put(
            path!("user-limits", username, limit.kind),
            &body,
            None,
            None,
        )?;
        Ok(())
    }

    /// Clears a user limit.
    pub fn clear_user_limit(&self, username: &str, kind: UserLimitTarget) -> Result<()> {
        let _response = self.http_delete(path!("user-limits", username, kind), None, None)?;
        Ok(())
    }

    /// Lists all user limits.
    pub fn list_all_user_limits(&self) -> Result<Vec<responses::UserLimits>> {
        let response = self.http_get("user-limits", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists limits for a specific user.
    pub fn list_user_limits(&self, username: &str) -> Result<Vec<responses::UserLimits>> {
        let response = self.http_get(path!("user-limits", username), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Sets a limit for a virtual host.
    pub fn set_vhost_limit(
        &self,
        vhost: &str,
        limit: EnforcedLimitParams<VirtualHostLimitTarget>,
    ) -> Result<()> {
        let body = json!({"value": limit.value});
        let _response =
            self.http_put(path!("vhost-limits", vhost, limit.kind), &body, None, None)?;
        Ok(())
    }

    /// Clears a virtual host limit.
    pub fn clear_vhost_limit(&self, vhost: &str, kind: VirtualHostLimitTarget) -> Result<()> {
        let _response = self.http_delete(
            path!("vhost-limits", vhost, kind),
            Some(StatusCode::NOT_FOUND),
            None,
        )?;
        Ok(())
    }

    /// Lists all virtual host limits.
    pub fn list_all_vhost_limits(&self) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self.http_get("vhost-limits", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists limits for a specific virtual host.
    pub fn list_vhost_limits(&self, vhost: &str) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self.http_get(path!("vhost-limits", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Gets the cluster name.
    pub fn get_cluster_name(&self) -> Result<responses::ClusterIdentity> {
        let response = self.http_get("cluster-name", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Sets the cluster name.
    pub fn set_cluster_name(&self, new_name: &str) -> Result<()> {
        let body = json!({"name": new_name});
        let _response = self.http_put("cluster-name", &body, None, None)?;
        Ok(())
    }

    /// Gets cluster tags.
    pub fn get_cluster_tags(&self) -> Result<responses::ClusterTags> {
        let response = self.get_global_runtime_parameter("cluster_tags")?;
        Ok(ClusterTags::from(response.value))
    }

    /// Sets cluster tags.
    pub fn set_cluster_tags(&self, tags: Map<String, Value>) -> Result<()> {
        let grp = GlobalRuntimeParameterDefinition {
            name: "cluster_tags",
            value: tags,
        };
        self.upsert_global_runtime_parameter(&grp)?;
        Ok(())
    }

    /// Clears all cluster tags.
    pub fn clear_cluster_tags(&self) -> Result<()> {
        self.clear_global_runtime_parameter("cluster_tags")?;
        Ok(())
    }

    /// Gets a policy.
    pub fn get_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self.http_get(path!("policies", vhost, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all policies in the cluster (across all virtual hosts), taking the user's
    /// permissions into account.
    pub fn list_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("policies", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists policies in a virtual host.
    pub fn list_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("policies", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Declares a policy.
    pub fn declare_policy(&self, params: &PolicyParams) -> Result<()> {
        let _response = self.http_put(
            path!("policies", params.vhost, params.name),
            params,
            None,
            None,
        )?;
        Ok(())
    }

    /// Declares multiple policies. Note that this function will still issue
    /// as many HTTP API requests as there are policies to declare.
    pub fn declare_policies(&self, params: Vec<&PolicyParams>) -> Result<()> {
        for p in params {
            self.declare_policy(p)?;
        }
        Ok(())
    }

    /// Deletes a policy.
    pub fn delete_policy(&self, vhost: &str, name: &str) -> Result<()> {
        let _response = self.http_delete(
            path!("policies", vhost, name),
            Some(StatusCode::NOT_FOUND),
            None,
        )?;
        Ok(())
    }

    /// Deletes multiple policies. Note that this function will still issue
    /// as many HTTP API requests as there are policies to delete.
    pub fn delete_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_policy(vhost, name)?;
        }
        Ok(())
    }

    /// Gets an operator policy.
    pub fn get_operator_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self.http_get(path!("operator-policies", vhost, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all operator policies.
    pub fn list_operator_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("operator-policies", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists operator policies in a virtual host.
    pub fn list_operator_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("operator-policies", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Declares an operator policy.
    pub fn declare_operator_policy(&self, params: &PolicyParams) -> Result<()> {
        let _response = self.http_put(
            path!("operator-policies", params.vhost, params.name),
            params,
            None,
            None,
        )?;
        Ok(())
    }

    /// Declares multiple operator policies. Note that this function will still issue
    /// as many HTTP API requests as there are operator policies to declare.
    pub fn declare_operator_policies(&self, params: Vec<&PolicyParams>) -> Result<()> {
        for p in params {
            self.declare_operator_policy(p)?;
        }
        Ok(())
    }

    /// Deletes an operator policy.
    pub fn delete_operator_policy(&self, vhost: &str, name: &str) -> Result<()> {
        let _response = self.http_delete(
            path!("operator-policies", vhost, name),
            Some(StatusCode::NOT_FOUND),
            None,
        )?;
        Ok(())
    }

    /// Deletes multiple operator policies. Note that this function will still issue
    /// as many HTTP API requests as there are operator policies to delete.
    pub fn delete_operator_policies_in(&self, vhost: &str, names: Vec<&str>) -> Result<()> {
        for name in names {
            self.delete_operator_policy(vhost, name)?;
        }
        Ok(())
    }

    /// Lists all permissions.
    pub fn list_permissions(&self) -> Result<Vec<responses::Permissions>> {
        let response = self.http_get("permissions", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists permissions in a virtual host.
    pub fn list_permissions_in(&self, vhost: &str) -> Result<Vec<responses::Permissions>> {
        let response = self.http_get(path!("vhosts", vhost, "permissions"), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists permissions for a specific user.
    pub fn list_permissions_of(&self, user: &str) -> Result<Vec<responses::Permissions>> {
        let response = self.http_get(path!("users", user, "permissions"), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Gets permissions for a user in a virtual host.
    pub fn get_permissions(&self, vhost: &str, user: &str) -> Result<responses::Permissions> {
        let response = self.http_get(path!("permissions", vhost, user), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    //
    // Rebalancing
    //

    /// Rebalances queue leaders across cluster nodes.
    pub fn rebalance_queue_leaders(&self) -> Result<()> {
        self.http_post("rebalance/queues", &json!({}), None, None)?;
        Ok(())
    }

    //
    // Definitions

    /// Exports cluster-wide definitions as JSON string.
    pub fn export_cluster_wide_definitions(&self) -> Result<String> {
        self.export_cluster_wide_definitions_as_string()
    }

    /// Exports cluster-wide definitions as JSON string.
    pub fn export_cluster_wide_definitions_as_string(&self) -> Result<String> {
        let response = self.http_get("definitions", None, None)?;
        let response = response.text()?;
        Ok(response)
    }

    /// Exports cluster-wide definitions as structured data.
    pub fn export_cluster_wide_definitions_as_data(&self) -> Result<ClusterDefinitionSet> {
        let response = self.http_get("definitions", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Exports virtual host definitions as JSON string.
    pub fn export_vhost_definitions(&self, vhost: &str) -> Result<String> {
        self.export_vhost_definitions_as_string(vhost)
    }

    /// Exports virtual host definitions as JSON string.
    pub fn export_vhost_definitions_as_string(&self, vhost: &str) -> Result<String> {
        let response = self.http_get(path!("definitions", vhost), None, None)?;
        let response = response.text()?;
        Ok(response)
    }

    /// Exports virtual host definitions as structured data.
    pub fn export_vhost_definitions_as_data(
        &self,
        vhost: &str,
    ) -> Result<VirtualHostDefinitionSet> {
        let response = self.http_get(path!("definitions", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Imports cluster-wide definitions.
    pub fn import_definitions(&self, definitions: Value) -> Result<()> {
        self.import_cluster_wide_definitions(definitions)
    }

    /// Imports cluster-wide definitions.
    pub fn import_cluster_wide_definitions(&self, definitions: Value) -> Result<()> {
        self.http_post("definitions", &definitions, None, None)?;
        Ok(())
    }

    /// Imports virtual host definitions.
    pub fn import_vhost_definitions(&self, vhost: &str, definitions: Value) -> Result<()> {
        self.http_post(path!("definitions", vhost), &definitions, None, None)?;
        Ok(())
    }

    //
    // Health Checks
    //

    /// Performs a cluster-wide alarms health check.
    pub fn health_check_cluster_wide_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/alarms")
    }

    /// Performs a local alarms health check.
    pub fn health_check_local_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/local-alarms")
    }

    /// Checks if the node is quorum critical.
    pub fn health_check_if_node_is_quorum_critical(&self) -> Result<()> {
        let path = "health/checks/node-is-quorum-critical";
        self.boolean_health_check(path)
    }

    /// Checks if a port listener is active.
    pub fn health_check_port_listener(&self, port: u16) -> Result<()> {
        let port_s = port.to_string();
        let path = path!("health", "checks", "port-listener", port_s);
        self.boolean_health_check(&path)
    }

    /// Checks if a protocol listener is active.
    pub fn health_check_protocol_listener(&self, protocol: SupportedProtocol) -> Result<()> {
        let proto: String = String::from(protocol);
        let path = path!("health", "checks", "protocol-listener", proto);
        self.boolean_health_check(&path)
    }

    fn boolean_health_check(&self, path: &str) -> std::result::Result<(), HttpClientError> {
        // we expect that StatusCode::SERVICE_UNAVAILABLE may be return and ignore
        // it here to provide a custom error type later
        let response = self.http_get(path, None, Some(StatusCode::SERVICE_UNAVAILABLE))?;

        let status_code = response.status();
        if status_code.is_success() {
            return Ok(());
        }

        let failure_details = response.json()?;
        Err(Error::HealthCheckFailed {
            path: path.to_owned(),
            status_code,
            details: failure_details,
        })
    }

    //
    // Federation
    //

    /// Lists federation upstreams.
    pub fn list_federation_upstreams(&self) -> Result<Vec<responses::FederationUpstream>> {
        let response = self.list_runtime_parameters_of_component(FEDERATION_UPSTREAM_COMPONENT)?;
        let upstreams = response
            .into_iter()
            .map(FederationUpstream::try_from)
            // TODO: in theory this can be an Err
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();

        Ok(upstreams)
    }

    /// Lists [federation](https://www.rabbitmq.com/docs/federation) links (connections) running in the cluster.
    pub fn list_federation_links(&self) -> Result<Vec<responses::FederationLink>> {
        let response = self.http_get("federation-links", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Creates or updates a [federation](https://www.rabbitmq.com/docs/federation) upstream.
    ///
    /// Federation upstreams define connection endpoints for federation links (connections that federate
    /// queues or exchanges).
    pub fn declare_federation_upstream(&self, params: FederationUpstreamParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_federation_upstream_with_parameters(&runtime_param)
    }

    pub fn delete_federation_upstream(&self, vhost: &str, name: &str) -> Result<()> {
        self.clear_runtime_parameter(FEDERATION_UPSTREAM_COMPONENT, vhost, name)
    }

    //
    // Shovels
    //

    /// Lists [shovel](https://www.rabbitmq.com/docs/shovel) across all virtual hosts in the cluster.
    pub fn list_shovels(&self) -> Result<Vec<responses::Shovel>> {
        let response = self.http_get("shovels", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists [dynamic shovels](https://www.rabbitmq.com/docs/shovel-dynamic) in a specific virtual host.
    pub fn list_shovels_in(&self, vhost: &str) -> Result<Vec<responses::Shovel>> {
        let response = self.http_get(path!("shovels", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Declares [shovel](https://www.rabbitmq.com/docs/shovel) that will use the AMQP 0-9-1 protocol
    /// for both source and destination collection.
    pub fn declare_amqp091_shovel(&self, params: Amqp091ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameter(&runtime_param)
    }

    /// Declares [shovel](https://www.rabbitmq.com/docs/shovel) that will use the AMQP 1.0 protocol
    /// for both source and destination collection.
    pub fn declare_amqp10_shovel(&self, params: Amqp10ShovelParams<'_>) -> Result<()> {
        let runtime_param = RuntimeParameterDefinition::from(params);

        self.declare_shovel_parameter(&runtime_param)
    }

    /// Deletes a shovel in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent shovel
    /// will fail.
    pub fn delete_shovel(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self.http_delete(path!("shovels", "vhost", vhost, name), excludes, None)?;
        Ok(())
    }

    //
    // Publish and consume messages
    //

    /// Publishes a message to an exchange.
    pub fn publish_message(
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

        let response = self.http_post(
            path!("exchanges", vhost, exchange, "publish"),
            &body,
            None,
            None,
        )?;
        let response = response.json()?;
        Ok(response)
    }

    /// Gets messages from a queue.
    pub fn get_messages(
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

        let response = self.http_post(path!("queues", vhost, queue, "get"), &body, None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Gets cluster overview information.
    pub fn overview(&self) -> Result<responses::Overview> {
        let response = self.http_get("overview", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Gets the RabbitMQ server version.
    pub fn server_version(&self) -> Result<String> {
        let response = self.http_get("overview", None, None)?;
        let response: responses::Overview = response.json()?;
        Ok(response.rabbitmq_version)
    }

    //
    // Feature flags
    //

    /// Lists all feature flags.
    pub fn list_feature_flags(&self) -> Result<FeatureFlagList> {
        let response = self.http_get("feature-flags", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Enables all stable feature flags.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    pub fn enable_feature_flag(&self, name: &str) -> Result<()> {
        let body = serde_json::json!({
            "name": name
        });
        let _response = self.http_put(path!("feature-flags", name, "enable"), &body, None, None)?;
        Ok(())
    }

    /// Enables all stable feature flags.
    /// This function is idempotent: enabling an already enabled feature flag
    /// will succeed.
    pub fn enable_all_stable_feature_flags(&self) -> Result<()> {
        // PUT /api/feature-flags/{name}/enable does not support the special 'all' value like 'rabbitmqctl enable_feature_flag' does.
        // Thus we do what management UI does: discover the stable disabled flags and enable
        // them one by one.
        let discovered_flags = self.list_feature_flags()?;
        let flags_to_enable: Vec<&FeatureFlag> = discovered_flags
            .0
            .iter()
            .filter(|&ff| {
                ff.state == FeatureFlagState::Disabled
                    && ff.stability == FeatureFlagStability::Stable
            })
            .collect();

        for ff in flags_to_enable {
            self.enable_feature_flag(&ff.name)?;
        }

        Ok(())
    }

    //
    // Deprecated Features
    //

    /// Lists all deprecated features.
    pub fn list_all_deprecated_features(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists deprecated features currently in use.
    pub fn list_deprecated_features_in_use(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features/used", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    //
    // OAuth 2 Configuration
    //

    /// Gets OAuth 2 configuration.
    pub fn oauth_configuration(&self) -> Result<OAuthConfiguration> {
        let response = self.http_get("auth", None, None)?;
        let response = response.json()?;

        Ok(response)
    }

    //
    // Schema Definition Sync (Tanzu RabbitMQ)
    //

    /// Gets schema definition sync status.
    pub fn schema_definition_sync_status(
        &self,
        node: Option<&str>,
    ) -> Result<SchemaDefinitionSyncStatus> {
        let response = match node {
            Some(val) => {
                self.http_get(path!("tanzu", "osr", "schema", "status", val), None, None)?
            }
            None => self.http_get("tanzu/osr/schema/status", None, None)?,
        };
        let response = response.json()?;

        Ok(response)
    }

    /// Enables schema definition sync on a node.
    pub fn enable_schema_definition_sync_on_node(&self, node: &str) -> Result<()> {
        let payload = EmptyPayload::new();
        self.http_put(
            path!("tanzu", "osr", "schema", "enable", node),
            &payload,
            None,
            None,
        )?;

        Ok(())
    }

    /// Disables schema definition sync on a node.
    pub fn disable_schema_definition_sync_on_node(&self, node: &str) -> Result<()> {
        self.http_delete(path!("tanzu", "osr", "schema", "disable", node), None, None)?;

        Ok(())
    }

    /// Enables schema definition sync cluster-wide.
    pub fn enable_schema_definition_sync(&self) -> Result<()> {
        let payload = EmptyPayload::new();
        self.http_put(
            path!("tanzu", "osr", "schema", "enable-cluster-wide"),
            &payload,
            None,
            None,
        )?;

        Ok(())
    }

    /// Disables schema definition sync cluster-wide.
    pub fn disable_schema_definition_sync(&self) -> Result<()> {
        self.http_delete(
            path!("tanzu", "osr", "schema", "disable-cluster-wide"),
            None,
            None,
        )?;

        Ok(())
    }

    //
    // Warm Standby Replication (Tanzu RabbitMQ)
    //

    /// Gets warm standby replication status.
    pub fn warm_standby_replication_status(&self) -> Result<WarmStandbyReplicationStatus> {
        let response = self.http_get("tanzu/osr/standby/status", None, None)?;
        let response = response.json()?;

        Ok(response)
    }

    //
    // Implementation
    //

    fn declare_shovel_parameter(
        &self,
        runtime_param: &RuntimeParameterDefinition<'_>,
    ) -> Result<()> {
        let _response = self.http_put(
            path!(
                "parameters",
                SHOVEL_COMPONENT,
                runtime_param.vhost,
                runtime_param.name
            ),
            &runtime_param,
            None,
            None,
        )?;
        Ok(())
    }

    fn declare_federation_upstream_with_parameters(
        &self,
        runtime_param: &RuntimeParameterDefinition<'_>,
    ) -> Result<()> {
        let _response = self.http_put(
            path!(
                "parameters",
                FEDERATION_UPSTREAM_COMPONENT,
                runtime_param.vhost,
                runtime_param.name
            ),
            &runtime_param,
            None,
            None,
        )?;
        Ok(())
    }

    fn health_check_alarms(&self, path: &str) -> Result<()> {
        // we expect that StatusCode::SERVICE_UNAVAILABLE may be return and ignore
        // it here to provide a custom error type later
        let response = self.http_get(path, None, Some(StatusCode::SERVICE_UNAVAILABLE))?;
        let status_code = response.status();
        if status_code.is_success() {
            return Ok(());
        }

        let body = response.json()?;
        let failure_details = responses::HealthCheckFailureDetails::AlarmCheck(body);
        Err(Error::HealthCheckFailed {
            path: path.to_owned(),
            details: failure_details,
            status_code,
        })
    }

    fn list_exchange_bindings_with_source_or_destination(
        &self,
        vhost: &str,
        exchange: &str,
        vertex: BindindVertex,
    ) -> Result<Vec<BindingInfo>> {
        let response = self.http_get(
            path!("exchanges", vhost, exchange, "bindings", vertex),
            None,
            None,
        )?;
        let response = response.json()?;
        Ok(response)
    }

    fn http_get<S>(
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

    fn http_put<S, T>(
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

    fn http_post<S, T>(
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

    fn http_delete<S>(
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

    fn http_delete_with_headers<S>(
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

    fn ok_or_status_code_error(
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

    fn rooted_path<S>(&self, path: S) -> String
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
