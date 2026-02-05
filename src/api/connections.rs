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

use crate::{commons::PaginationParams, path, responses};
use reqwest::{
    StatusCode,
    header::{HeaderMap, HeaderValue},
};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists all AMQP 1.0 and 0-9-1 client connections across the cluster.
    /// See [Connections Guide](https://www.rabbitmq.com/docs/connections) to learn more.
    ///
    /// Requires the `monitoring` user tag for all connections, or `management` for own connections only. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_connections(&self) -> Result<Vec<responses::Connection>> {
        self.get_api_request("connections").await
    }

    /// Lists connections with pagination.
    ///
    /// Requires the `monitoring` user tag for all connections, or `management` for own connections only. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_connections_paged(
        &self,
        params: &PaginationParams,
    ) -> Result<Vec<responses::Connection>> {
        match params.to_query_string() {
            Some(query) => self.get_paginated_api_request("connections", &query).await,
            None => self.list_connections().await,
        }
    }

    /// Returns information about a connection.
    ///
    /// Connection name is usually obtained from `crate::responses::Connection` or `crate::responses::UserConnection`,
    /// e.g. via `Client#list_connections`, `Client#list_connections_in`, `Client#list_user_connections`.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn get_connection_info(&self, name: &str) -> Result<responses::Connection> {
        self.get_api_request(path!("connections", name)).await
    }

    /// Returns information about a stream connection.
    ///
    /// Connection name is usually obtained from `crate::responses::Connection` or `crate::responses::UserConnection`,
    /// e.g. via `Client#list_stream_connections`, `Client#list_stream_connections_in`, `Client#list_user_connections`.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
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
    ///
    /// Requires the `administrator` user tag for other users' connections, or `management` for own connections.
    pub async fn close_connection(
        &self,
        name: &str,
        reason: Option<&str>,
        idempotently: bool,
    ) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };

        let mut headers = HeaderMap::new();
        if let Some(value) = reason {
            let hdr = HeaderValue::from_str(value)?;
            headers.insert("X-Reason", hdr);
        }

        self.http_delete_with_headers(path!("connections", name), headers, excludes, None)
            .await?;

        Ok(())
    }

    /// Closes all connections for a user with an optional reason.
    ///
    /// The reason will be passed on in the connection error to the client and will be logged on the RabbitMQ end.
    ///
    /// This is en equivalent of listing all connections of a user with `Client#list_user_connections` and then
    /// closing them one by one.
    ///
    /// Requires the `administrator` user tag.
    pub async fn close_user_connections(
        &self,
        username: &str,
        reason: Option<&str>,
        idempotently: bool,
    ) -> Result<()> {
        let excludes = if idempotently {
            Some(StatusCode::NOT_FOUND)
        } else {
            None
        };

        let mut headers = HeaderMap::new();
        if let Some(value) = reason {
            let hdr = HeaderValue::from_str(value)?;
            headers.insert("X-Reason", hdr);
        }

        self.http_delete_with_headers(
            path!("connections", "username", username),
            headers,
            excludes,
            None,
        )
        .await?;

        Ok(())
    }

    /// Lists all connections in the given virtual host.
    /// See [Connections Guide](https://www.rabbitmq.com/docs/connections) to learn more.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_connections_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::Connection>> {
        self.get_api_request(path!("vhosts", virtual_host, "connections"))
            .await
    }

    /// Lists all connections of a specific user.
    /// See [Connection Guide](https://www.rabbitmq.com/docs/connections) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_user_connections(
        &self,
        username: &str,
    ) -> Result<Vec<responses::UserConnection>> {
        self.get_api_request(path!("connections", "username", username))
            .await
    }

    /// Lists all RabbitMQ Stream Protocol client connections across the cluster.
    /// See [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_stream_connections(&self) -> Result<Vec<responses::Connection>> {
        self.get_api_request("stream/connections").await
    }

    /// Lists RabbitMQ Stream Protocol client connections in the given virtual host.
    /// See [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_stream_connections_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::Connection>> {
        self.get_api_request(path!("stream", "connections", virtual_host))
            .await
    }
}
