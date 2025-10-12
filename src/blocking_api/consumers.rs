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

    /// Lists stream publishers publishing to the given stream.
    /// Useful for detecting publishers that are publishing to a specific stream.
    pub fn list_stream_publishers_of(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<Vec<responses::StreamPublisher>> {
        self.get_api_request(path!("stream", "publishers", virtual_host, name))
    }

    /// Lists stream publishers on the given stream connection.
    /// Use this function for inspecting stream publishers on a specific connection.
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
    /// Use this function for inspecting stream consumers on a specific connection.
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

    /// Lists all consumers across the cluster.
    /// See [Consumers Guide](https://www.rabbitmq.com/docs/consumers) to learn more.
    pub fn list_consumers(&self) -> Result<Vec<responses::Consumer>> {
        self.get_api_request("consumers")
    }

    /// Lists all consumers in the given virtual host.
    /// See [Consumers Guide](https://www.rabbitmq.com/docs/consumers) to learn more.
    pub fn list_consumers_in(&self, virtual_host: &str) -> Result<Vec<responses::Consumer>> {
        self.get_api_request(path!("consumers", virtual_host))
    }
}
