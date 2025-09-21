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

use crate::{path, requests, responses};
use serde_json::json;

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: std::fmt::Display,
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    /// Only use this function in tests and experiments.
    /// Always use a messaging or streaming protocol client for publishing in production.
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

    /// Only use this function in tests and experiments.
    /// Always use a messaging or streaming protocol client for consuming in production.
    pub fn get_messages(
        &self,
        vhost: &str,
        queue: &str,
        count: u32,
        ack_mode: &str,
    ) -> Result<Vec<responses::GetMessage>> {
        let body = json!({
          "count": count,
          "ackmode": ack_mode,
          "encoding": "auto"
        });

        let response = self.http_post(path!("queues", vhost, queue, "get"), &body, None, None)?;
        let response = response.json()?;
        Ok(response)
    }
}
