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

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists all channels across the cluster.
    /// See [Channels Guide](https://www.rabbitmq.com/docs/channels) to learn more.
    ///
    /// Requires the `monitoring` user tag for all channels, or `management` for own channels only. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_channels(&self) -> Result<Vec<responses::Channel>> {
        self.get_api_request("channels").await
    }

    /// Lists channels with pagination.
    ///
    /// Requires the `monitoring` user tag for all channels, or `management` for own channels only. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_channels_paged(
        &self,
        params: &PaginationParams,
    ) -> Result<Vec<responses::Channel>> {
        match params.to_query_string() {
            Some(query) => self.get_paginated_api_request("channels", &query).await,
            None => self.list_channels().await,
        }
    }

    /// Lists all channels in the given virtual host.
    /// See [Channels Guide](https://www.rabbitmq.com/docs/channels) to learn more.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_channels_in(&self, virtual_host: &str) -> Result<Vec<responses::Channel>> {
        self.get_api_request(path!("vhosts", virtual_host, "channels"))
            .await
    }

    /// Lists channels in the given virtual host with pagination.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_channels_in_paged(
        &self,
        virtual_host: &str,
        params: &PaginationParams,
    ) -> Result<Vec<responses::Channel>> {
        match params.to_query_string() {
            Some(query) => {
                self.get_paginated_api_request(path!("vhosts", virtual_host, "channels"), &query)
                    .await
            }
            None => self.list_channels_in(virtual_host).await,
        }
    }

    /// Lists all channels on a given AMQP 0-9-1 connection.
    /// See [Channels Guide](https://www.rabbitmq.com/docs/channels) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_channels_on(&self, connection_name: &str) -> Result<Vec<responses::Channel>> {
        self.get_api_request(path!("connections", connection_name, "channels"))
            .await
    }

    /// Returns information about a specific channel.
    ///
    /// Unlike AMQP 0-9-1, HTTP API identifies channels by a string identifier instead of a numeric ID.
    ///
    /// Channel name is usually obtained from `crate::responses::Channel`,
    /// e.g. via `Client#list_channels`, `Client#list_channels_in`, `Client#list_channels_on`.
    /// See [Channels Guide](https://www.rabbitmq.com/docs/channels) to learn more.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn get_channel_info<S: AsRef<str>>(
        &self,
        channel_name: S,
    ) -> Result<responses::Channel> {
        self.get_api_request(path!("channels", channel_name.as_ref()))
            .await
    }
}
