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
    commons::{BindingDestinationType, BindingVertex},
    error::Error,
    path,
    requests::BindingDeletionParams,
    requests::XArguments,
    responses::{self, BindingInfo},
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
    /// Use this function for troubleshooting routing of a particular queue.
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
    /// Use this function for troubleshooting routing of a particular exchange.
    pub fn list_exchange_bindings_with_source(
        &self,
        virtual_host: &str,
        exchange: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        self.list_exchange_bindings_with_source_or_destination(
            virtual_host,
            exchange,
            BindingVertex::Source,
        )
    }

    /// Lists all bindings of a specific exchange where it is the destination.
    /// Use this function for troubleshooting routing of a particular exchange.
    pub fn list_exchange_bindings_with_destination(
        &self,
        virtual_host: &str,
        exchange: &str,
    ) -> Result<Vec<responses::BindingInfo>> {
        self.list_exchange_bindings_with_source_or_destination(
            virtual_host,
            exchange,
            BindingVertex::Destination,
        )
    }

    fn list_exchange_bindings_with_source_or_destination(
        &self,
        vhost: &str,
        exchange: &str,
        vertex: BindingVertex,
    ) -> Result<Vec<BindingInfo>> {
        let response = self.http_get(
            path!("exchanges", vhost, exchange, "bindings", vertex),
            None,
            None,
        )?;
        let response = response.json()?;
        Ok(response)
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

    /// Bindings one exchange to another (creates an [exchange-to-exchange binding](https://www.rabbitmq.com/docs/e2e)).
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

    #[allow(clippy::too_many_arguments)]
    pub fn delete_binding(
        &self,
        params: &BindingDeletionParams<'_>,
        idempotently: bool,
    ) -> Result<()> {
        let args = params.arguments.clone().unwrap_or_default();

        // to delete a binding, we need properties, that we can get from the server
        // so we search for the binding before deleting it
        let bindings = match params.destination_type {
            BindingDestinationType::Queue => {
                self.list_queue_bindings(params.virtual_host, params.destination)?
            }
            BindingDestinationType::Exchange => self
                .list_exchange_bindings_with_destination(params.virtual_host, params.destination)?,
        };

        let bs: Vec<&BindingInfo> = bindings
            .iter()
            .filter(|b| {
                b.source == params.source
                    && b.routing_key == params.routing_key
                    && b.arguments.0 == args
            })
            .collect();
        match bs.len() {
            0 => {
                if idempotently {
                    Ok(())
                } else {
                    Err(Error::NotFound)
                }
            }
            1 => {
                let first_key = bs.first().unwrap().properties_key.clone();
                let path_appreviation = params.destination_type.path_appreviation();
                let path = match first_key {
                    Some(pk) => {
                        path!(
                            // /api/bindings/vhost/e/exchange/[eq]/destination/props
                            "bindings",
                            params.virtual_host,
                            "e",
                            params.source,
                            path_appreviation,
                            params.destination,
                            pk.as_str()
                        )
                    }
                    None => {
                        path!(
                            // /api/bindings/vhost/e/exchange/[eq]/destination/
                            "bindings",
                            params.virtual_host,
                            "e",
                            params.source,
                            path_appreviation,
                            params.destination
                        )
                    }
                };
                self.http_delete(&path, None, None)?;
                Ok(())
            }
            _ => Err(Error::MultipleMatchingBindings),
        }
    }
}
