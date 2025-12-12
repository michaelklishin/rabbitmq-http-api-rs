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

use crate::{
    commons::PaginationParams,
    path,
    requests::{QueueParams, StreamParams},
    responses,
};
use serde_json::{Map, Value, json};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists all queues and streams across the cluster.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub fn list_queues(&self) -> Result<Vec<responses::QueueInfo>> {
        self.get_api_request("queues")
    }

    /// Lists queues and streams with pagination.
    pub fn list_queues_paged(
        &self,
        params: &PaginationParams,
    ) -> Result<Vec<responses::QueueInfo>> {
        match params.to_query_string() {
            Some(query) => self.get_api_request_with_query("queues", &query),
            None => self.list_queues(),
        }
    }

    /// Lists all queues and streams in the given virtual host.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub fn list_queues_in(&self, virtual_host: &str) -> Result<Vec<responses::QueueInfo>> {
        self.get_api_request(path!("queues", virtual_host))
    }

    /// Lists queues and streams in the given virtual host with pagination.
    pub fn list_queues_in_paged(
        &self,
        virtual_host: &str,
        params: &PaginationParams,
    ) -> Result<Vec<responses::QueueInfo>> {
        match params.to_query_string() {
            Some(query) => self.get_api_request_with_query(path!("queues", virtual_host), &query),
            None => self.list_queues_in(virtual_host),
        }
    }

    /// Lists all queues and streams across the cluster. Compared to [`list_queues`], provides more queue metrics.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub fn list_queues_with_details(&self) -> Result<Vec<responses::DetailedQueueInfo>> {
        self.get_api_request("queues/detailed")
    }

    /// Returns information about a queue or stream.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) to learn more.
    pub fn get_queue_info(&self, virtual_host: &str, name: &str) -> Result<responses::QueueInfo> {
        let response = self.http_get(path!("queues", virtual_host, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Returns information about a stream.
    /// See [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub fn get_stream_info(&self, virtual_host: &str, name: &str) -> Result<responses::QueueInfo> {
        self.get_queue_info(virtual_host, name)
    }

    /// Declares a [queue](https://www.rabbitmq.com/docs/queues).
    ///
    /// If the queue already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub fn declare_queue(&self, vhost: &str, params: &QueueParams<'_>) -> Result<()> {
        self.put_api_request(path!("queues", vhost, params.name), params)
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

    /// Deletes a queue in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent queue
    /// will fail.
    pub fn delete_queue(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(path!("queues", vhost, name), idempotently)
    }

    /// Deletes a stream in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent stream
    /// will fail.
    pub fn delete_stream(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_queue(vhost, name, idempotently)
    }

    /// Removes all messages in "ready for delivery" state from a queue without deleting the queue itself.
    ///
    /// Messages that were delivered but are pending acknowledgement will not be deleted
    /// by purging.
    pub fn purge_queue(&self, virtual_host: &str, name: &str) -> Result<()> {
        let _response =
            self.http_delete(path!("queues", virtual_host, name, "contents"), None, None)?;
        Ok(())
    }

    /// Convenience method: declares a durable quorum queue with no arguments.
    pub fn declare_quorum_queue(&self, vhost: &str, name: &str) -> Result<()> {
        let params = QueueParams::new_quorum_queue(name, None);
        self.declare_queue(vhost, &params)
    }

    /// Convenience method: declares a durable classic queue with no arguments.
    pub fn declare_classic_queue(&self, vhost: &str, name: &str) -> Result<()> {
        let params = QueueParams::new_durable_classic_queue(name, None);
        self.declare_queue(vhost, &params)
    }
}
