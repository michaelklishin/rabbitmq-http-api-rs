// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use crate::responses::{
    DeprecatedFeatureList, FeatureFlag, FeatureFlagList, FeatureFlagStability, FeatureFlagState,
    OAuthConfiguration,
};
use crate::{
    commons::{BindingDestinationType, UserLimitTarget, VirtualHostLimitTarget},
    path,
    requests::{
        self, BulkUserDelete, EnforcedLimitParams, ExchangeParams, Permissions, PolicyParams,
        QueueParams, RuntimeParameterDefinition, UserParams, VirtualHostParams, XArguments,
    },
    responses::{self, BindingInfo, DefinitionSet},
};
use backtrace::Backtrace;
use reqwest::{
    blocking::Client as HttpClient,
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use serde::Serialize;
use serde_json::{json, Map, Value};
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
pub struct ClientBuilder<E, U, P> {
    endpoint: E,
    username: U,
    password: P,
    client: HttpClient,
}

impl Default for ClientBuilder<&'static str, &'static str, &'static str> {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder<&'static str, &'static str, &'static str> {
    /// Constructs a new `ClientBuilder`.
    ///
    /// This is the same as `Client::builder()`.
    pub fn new() -> Self {
        let client = HttpClient::new();
        Self {
            endpoint: "http://localhost:15672",
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
        let response = self.http_get("nodes", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists virtual hosts in the cluster.
    pub fn list_vhosts(&self) -> Result<Vec<responses::VirtualHost>> {
        let response = self.http_get("vhosts", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists users in the internal database.
    pub fn list_users(&self) -> Result<Vec<responses::User>> {
        let response = self.http_get("users", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists users in the internal database that do not have access
    /// to any virtual hosts.
    pub fn list_users_without_permissions(&self) -> Result<Vec<responses::User>> {
        let response = self.http_get("users/without-permissions", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all client connections across the cluster.
    pub fn list_connections(&self) -> Result<Vec<responses::Connection>> {
        let response = self.http_get("connections", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn get_connection_info(&self, name: &str) -> Result<responses::Connection> {
        let response = self.http_get(path!("connections", name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

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

    /// Lists all connections in the given virtual host.
    pub fn list_connections_in(&self, virtual_host: &str) -> Result<Vec<responses::Connection>> {
        let response = self.http_get(path!("vhosts", virtual_host, "connections"), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all connections of a specific user.
    pub fn list_user_connections(&self, username: &str) -> Result<Vec<responses::UserConnection>> {
        let response = self.http_get(path!("connections", "username", username), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all channels across the cluster.
    pub fn list_channels(&self) -> Result<Vec<responses::Channel>> {
        let response = self.http_get("channels", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all channels in the given virtual host.
    pub fn list_channels_in(&self, virtual_host: &str) -> Result<Vec<responses::Channel>> {
        let response = self.http_get(path!("vhosts", virtual_host, "channels"), None, None)?;

        let response = response.json()?;
        Ok(response)
    }

    /// Lists all queues and streams across the cluster.
    pub fn list_queues(&self) -> Result<Vec<responses::QueueInfo>> {
        let response = self.http_get("queues", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all queues and streams in the given virtual host.
    pub fn list_queues_in(&self, virtual_host: &str) -> Result<Vec<responses::QueueInfo>> {
        let response = self.http_get(path!("queues", virtual_host), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all exchanges across the cluster.
    pub fn list_exchanges(&self) -> Result<Vec<responses::ExchangeInfo>> {
        let response = self.http_get("exchanges", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all exchanges in the given virtual host.
    pub fn list_exchanges_in(&self, virtual_host: &str) -> Result<Vec<responses::ExchangeInfo>> {
        let response = self.http_get(path!("exchanges", virtual_host), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all bindings (both queue-to-exchange and exchange-to-exchange ones) across the cluster.
    pub fn list_bindings(&self) -> Result<Vec<responses::BindingInfo>> {
        let response = self.http_get("bindings", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all bindings (both queue-to-exchange and exchange-to-exchange ones)  in the given virtual host.
    pub fn list_bindings_in(&self, virtual_host: &str) -> Result<Vec<responses::BindingInfo>> {
        let response = self.http_get(path!("bindings", virtual_host), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all bindings of a specific queue.
    pub fn list_queue_bindings(
        &self,
        virtual_host: &str,
        queue: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        let response =
            self.http_get(path!("queues", virtual_host, queue, "bindings"), None, None)?;
        let response = response.json()?;
        Ok(response)
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
        let response = self.http_get("consumers", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Lists all consumers in the given virtual host.
    pub fn list_consumers_in(&self, virtual_host: &str) -> Result<Vec<responses::Consumer>> {
        let response = self.http_get(path!("consumers", virtual_host), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns information about a cluster node.
    pub fn get_node_info(&self, name: &str) -> Result<responses::ClusterNode> {
        let response = self.http_get(path!("nodes", name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns information about a cluster node.
    pub fn get_node_memory_footprint(&self, name: &str) -> Result<responses::NodeMemoryFootprint> {
        let response = self.http_get(path!("nodes", name, "memory"), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns information about a virtual host.
    pub fn get_vhost(&self, name: &str) -> Result<responses::VirtualHost> {
        let response = self.http_get(path!("vhosts", name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns information about a user in the internal database.
    pub fn get_user(&self, name: &str) -> Result<responses::User> {
        let response = self.http_get(path!("users", name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns information about a queue or stream.
    pub fn get_queue_info(&self, virtual_host: &str, name: &str) -> Result<responses::QueueInfo> {
        let response = self.http_get(path!("queues", virtual_host, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns information about an exchange.
    pub fn get_exchange_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::ExchangeInfo> {
        let response = self.http_get(path!("exchanges", virtual_host, name), None, None)?;
        let response = response.json()?;
        Ok(response)
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
        let _response = self.http_put(path!("vhosts", params.name), params, None, None)?;
        Ok(())
    }

    /// Adds a user to the internal database.
    ///
    /// See [`UserParams`] and [`crate::password_hashing`].
    pub fn create_user(&self, params: &UserParams) -> Result<()> {
        let _response = self.http_put(path!("users", params.name), params, None, None)?;
        Ok(())
    }

    pub fn declare_permissions(&self, params: &Permissions) -> Result<()> {
        let _response = self.http_put(
            // /api/permissions/vhost/user
            path!("permissions", params.vhost, params.user),
            params,
            None,
            None,
        )?;
        Ok(())
    }

    pub fn grant_permissions(&self, vhost: &str, user: &str) -> Result<()> {
        let _response = self.http_delete(path!("permissions", vhost, user), None, None)?;
        Ok(())
    }

    pub fn declare_queue(&self, vhost: &str, params: &QueueParams) -> Result<()> {
        let _response = self.http_put(path!("queues", vhost, params.name), params, None, None)?;
        Ok(())
    }

    pub fn declare_exchange(&self, vhost: &str, params: &ExchangeParams) -> Result<()> {
        let _response =
            self.http_put(path!("exchanges", vhost, params.name), params, None, None)?;
        Ok(())
    }

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

    pub fn delete_vhost(&self, vhost: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self.http_delete(path!("vhosts", vhost), excludes, None)?;
        Ok(())
    }

    pub fn delete_user(&self, username: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self.http_delete(path!("users", username), excludes, None)?;
        Ok(())
    }

    pub fn delete_users(&self, usernames: Vec<&str>) -> Result<()> {
        let delete = BulkUserDelete { usernames };
        let _response = self.http_post(path!("users", "bulk-delete"), &delete, None, None)?;
        Ok(())
    }

    pub fn clear_permissions(&self, vhost: &str, username: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self.http_delete(path!("permissions", vhost, username), excludes, None)?;
        Ok(())
    }

    pub fn delete_queue(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self.http_delete(path!("queues", vhost, name), excludes, None)?;
        Ok(())
    }

    pub fn delete_exchange(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };
        let _response = self.http_delete(path!("exchanges", vhost, name), excludes, None)?;
        Ok(())
    }

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

    pub fn purge_queue(&self, virtual_host: &str, name: &str) -> Result<()> {
        let _response =
            self.http_delete(path!("queues", virtual_host, name, "contents"), None, None)?;
        Ok(())
    }

    pub fn list_runtime_parameters(&self) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get("parameters", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_runtime_parameters_of_component(
        &self,
        component: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get(path!("parameters", component), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_runtime_parameters_of_component_in(
        &self,
        component: &str,
        vhost: &str,
    ) -> Result<Vec<responses::RuntimeParameter>> {
        let response = self.http_get(path!("parameters", component, vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

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

    pub fn upsert_runtime_parameter(&self, param: &RuntimeParameterDefinition) -> Result<()> {
        let _response = self.http_put(
            path!("parameters", param.component, param.vhost, param.name),
            &param,
            None,
            None,
        )?;
        Ok(())
    }

    pub fn clear_runtime_parameter(&self, component: &str, vhost: &str, name: &str) -> Result<()> {
        let _response =
            self.http_delete(path!("parameters", component, vhost, name), None, None)?;
        Ok(())
    }

    pub fn clear_all_runtime_parameters(&self) -> Result<()> {
        let params = self.list_runtime_parameters()?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name)?
        }
        Ok(())
    }

    pub fn clear_all_runtime_parameters_of_component(&self, component: &str) -> Result<()> {
        let params = self.list_runtime_parameters_of_component(component)?;
        for rp in params {
            self.clear_runtime_parameter(&rp.component, &rp.vhost, &rp.name)?
        }
        Ok(())
    }

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

    pub fn clear_user_limit(&self, username: &str, kind: UserLimitTarget) -> Result<()> {
        let _response = self.http_delete(path!("user-limits", username, kind), None, None)?;
        Ok(())
    }

    pub fn list_all_user_limits(&self) -> Result<Vec<responses::UserLimits>> {
        let response = self.http_get("user-limits", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_user_limits(&self, username: &str) -> Result<Vec<responses::UserLimits>> {
        let response = self.http_get(path!("user-limits", username), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

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

    pub fn clear_vhost_limit(&self, vhost: &str, kind: VirtualHostLimitTarget) -> Result<()> {
        let _response = self.http_delete(
            path!("vhost-limits", vhost, kind),
            Some(StatusCode::NOT_FOUND),
            None,
        )?;
        Ok(())
    }

    pub fn list_all_vhost_limits(&self) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self.http_get("vhost-limits", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_vhost_limits(&self, vhost: &str) -> Result<Vec<responses::VirtualHostLimits>> {
        let response = self.http_get(path!("vhost-limits", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn get_cluster_name(&self) -> Result<responses::ClusterIdentity> {
        let response = self.http_get("cluster-name", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn set_cluster_name(&self, new_name: &str) -> Result<()> {
        let body = json!({"name": new_name});
        let _response = self.http_put("cluster-name", &body, None, None)?;
        Ok(())
    }

    pub fn get_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self.http_get(path!("policies", vhost, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("policies", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("policies", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn declare_policy(&self, params: &PolicyParams) -> Result<()> {
        let _response = self.http_put(
            path!("policies", params.vhost, params.name),
            params,
            None,
            None,
        )?;
        Ok(())
    }

    pub fn delete_policy(&self, vhost: &str, name: &str) -> Result<()> {
        let _response = self.http_delete(
            path!("policies", vhost, name),
            Some(StatusCode::NOT_FOUND),
            None,
        )?;
        Ok(())
    }

    pub fn get_operator_policy(&self, vhost: &str, name: &str) -> Result<responses::Policy> {
        let response = self.http_get(path!("operator-policies", vhost, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_operator_policies(&self) -> Result<Vec<responses::Policy>> {
        let response = self.http_get("operator-policies", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_operator_policies_in(&self, vhost: &str) -> Result<Vec<responses::Policy>> {
        let response = self.http_get(path!("operator-policies", vhost), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn declare_operator_policy(&self, params: &PolicyParams) -> Result<()> {
        let _response = self.http_put(
            path!("operator-policies", params.vhost, params.name),
            params,
            None,
            None,
        )?;
        Ok(())
    }

    pub fn delete_operator_policy(&self, vhost: &str, name: &str) -> Result<()> {
        let _response = self.http_delete(
            path!("operator-policies", vhost, name),
            Some(StatusCode::NOT_FOUND),
            None,
        )?;
        Ok(())
    }

    pub fn list_permissions(&self) -> Result<Vec<responses::Permissions>> {
        let response = self.http_get("permissions", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_permissions_in(&self, vhost: &str) -> Result<Vec<responses::Permissions>> {
        let response = self.http_get(path!("vhosts", vhost, "permissions"), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_permissions_of(&self, user: &str) -> Result<Vec<responses::Permissions>> {
        let response = self.http_get(path!("users", user, "permissions"), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn get_permissions(&self, vhost: &str, user: &str) -> Result<responses::Permissions> {
        let response = self.http_get(path!("permissions", vhost, user), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    //
    // Rebalancing
    //

    pub fn rebalance_queue_leaders(&self) -> Result<()> {
        self.http_post("rebalance/queues", &json!({}), None, None)?;
        Ok(())
    }

    //
    // Definitions

    pub fn export_definitions(&self) -> Result<String> {
        self.export_definitions_as_string()
    }

    pub fn export_definitions_as_string(&self) -> Result<String> {
        let response = self.http_get("definitions", None, None)?;
        let response = response.text()?;
        Ok(response)
    }

    pub fn export_definitions_as_data(&self) -> Result<DefinitionSet> {
        let response = self.http_get("definitions", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn import_definitions(&self, definitions: Value) -> Result<()> {
        self.http_post("definitions", &definitions, None, None)?;
        Ok(())
    }

    //
    // Health Checks
    //

    pub fn health_check_cluster_wide_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/alarms")
    }

    pub fn health_check_local_alarms(&self) -> Result<()> {
        self.health_check_alarms("health/checks/local-alarms")
    }

    pub fn health_check_if_node_is_quorum_critical(&self) -> Result<()> {
        let path = "health/checks/node-is-quorum-critical";
        self.boolean_health_check(path)
    }

    pub fn health_check_port_listener(&self, port: u16) -> Result<()> {
        let path = format!("health/checks/port-listener/{}", port);
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
    // Publish and consume messages
    //

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

    pub fn get_messages(
        &self,
        vhost: &str,
        queue: &str,
        count: u32,
        ack_mode: &str,
    ) -> Result<Vec<responses::GetMessage>> {
        let body = serde_json::json!({
          "count": count,
          "ackmode": ack_mode,
          "encoding": "auto"
        });

        let response = self.http_post(path!("queues", vhost, queue, "get"), &body, None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn overview(&self) -> Result<responses::Overview> {
        let response = self.http_get("overview", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    //
    // Feature flags
    //

    pub fn list_feature_flags(&self) -> Result<FeatureFlagList> {
        let response = self.http_get("feature-flags", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Enables a feature flag.
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

    pub fn list_all_deprecated_features(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    pub fn list_deprecated_features_in_use(&self) -> Result<DeprecatedFeatureList> {
        let response = self.http_get("deprecated-features/used", None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    //
    // OAuth 2 Configuration
    //

    pub fn oauth_configuration(&self) -> Result<OAuthConfiguration> {
        let response = self.http_get("auth", None, None)?;
        let response = response.json()?;

        Ok(response)
    }

    //
    // Implementation
    //

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
    ) -> Result<Vec<responses::BindingInfo>> {
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
