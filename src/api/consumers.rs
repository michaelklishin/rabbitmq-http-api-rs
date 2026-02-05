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

use crate::{path, responses};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists all stream publishers across the cluster.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_stream_publishers(&self) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!("stream", "publishers")).await
    }

    /// Lists stream publishers publishing to the given virtual host.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_stream_publishers_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!("stream", "publishers", virtual_host))
            .await
    }

    /// Lists stream publishers publishing to the given stream.
    /// Useful for detecting publishers that are publishing to a specific stream.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_stream_publishers_of(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!("stream", "publishers", virtual_host, name))
            .await
    }

    /// Lists stream publishers on the given stream connection.
    /// Use this function for inspecting stream publishers on a specific connection.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_stream_publishers_on_connection(
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
        .await
    }

    /// Lists all stream consumers across the cluster.
    ///
    /// Requires the `monitoring` user tag. Does not modify state.
    /// Can be used by restricted monitoring users with the `monitoring` tag and only the `read`, `configure` permissions.
    pub async fn list_stream_consumers(&self) -> Result<Vec<responses::StreamConsumer>> {
        self.get_api_request(path!("stream", "consumers")).await
    }

    /// Lists stream consumers on connections in the given virtual host.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_stream_consumers_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::StreamConsumer>> {
        self.get_api_request(path!("stream", "consumers", virtual_host))
            .await
    }

    /// Lists stream consumers on the given stream connection.
    /// Use this function for inspecting stream consumers on a specific connection.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_stream_consumers_on_connection(
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
        .await
    }

    /// Lists all consumers across the cluster.
    /// See [Consumers Guide](https://www.rabbitmq.com/docs/consumers) to learn more.
    ///
    /// Requires the `management` user tag and the `read` permissions. Does not modify state.
    pub async fn list_consumers(&self) -> Result<Vec<responses::Consumer>> {
        self.get_api_request("consumers").await
    }

    /// Lists all consumers in the given virtual host.
    /// See [Consumers Guide](https://www.rabbitmq.com/docs/consumers) to learn more.
    ///
    /// Requires the `management` user tag and have `read` permissions on the vhost. Does not modify state.
    pub async fn list_consumers_in(&self, virtual_host: &str) -> Result<Vec<responses::Consumer>> {
        self.get_api_request(path!("consumers", virtual_host)).await
    }
}
