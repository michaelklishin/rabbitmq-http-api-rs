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
    path,
    requests::{QueueParams, StreamParams},
    responses,
};
use serde_json::{Map, Value, json};

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: std::fmt::Display,
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    /// Lists all queues and streams across the cluster.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_queues(&self) -> Result<Vec<responses::QueueInfo>> {
        self.get_api_request("queues").await
    }

    /// Lists all queues and streams in the given virtual host.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_queues_in(&self, virtual_host: &str) -> Result<Vec<responses::QueueInfo>> {
        self.get_api_request(path!("queues", virtual_host)).await
    }

    /// Lists all queues and streams across the cluster. Compared to [`list_queues`], provides more queue metrics.
    /// See [Queues Guide](https://www.rabbitmq.com/docs/queues) and [RabbitMQ Streams Guide](https://www.rabbitmq.com/docs/streams) to learn more.
    pub async fn list_queues_with_details(&self) -> Result<Vec<responses::DetailedQueueInfo>> {
        self.get_api_request("queues/detailed").await
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
}
