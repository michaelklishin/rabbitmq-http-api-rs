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

use crate::{commons::PaginationParams, path, requests::ExchangeParams, responses};

use super::client::{Client, Result};
use std::fmt::Display;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Lists all exchanges across the cluster.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub fn list_exchanges(&self) -> Result<Vec<responses::ExchangeInfo>> {
        self.get_api_request("exchanges")
    }

    /// Lists exchanges with pagination.
    pub fn list_exchanges_paged(
        &self,
        params: &PaginationParams,
    ) -> Result<Vec<responses::ExchangeInfo>> {
        match params.to_query_string() {
            Some(query) => self.get_paginated_api_request("exchanges", &query),
            None => self.list_exchanges(),
        }
    }

    /// Lists all exchanges in the given virtual host.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub fn list_exchanges_in(&self, virtual_host: &str) -> Result<Vec<responses::ExchangeInfo>> {
        self.get_api_request(path!("exchanges", virtual_host))
    }

    /// Lists exchanges in the given virtual host with pagination.
    pub fn list_exchanges_in_paged(
        &self,
        virtual_host: &str,
        params: &PaginationParams,
    ) -> Result<Vec<responses::ExchangeInfo>> {
        match params.to_query_string() {
            Some(query) => self.get_paginated_api_request(path!("exchanges", virtual_host), &query),
            None => self.list_exchanges_in(virtual_host),
        }
    }

    /// Returns information about an exchange.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub fn get_exchange_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::ExchangeInfo> {
        let response = self.http_get(path!("exchanges", virtual_host, name), None, None)?;
        let response = response.json()?;
        Ok(response)
    }

    /// Declares an [exchange](https://www.rabbitmq.com/docs/exchanges).
    ///
    /// If the exchange already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub fn declare_exchange(&self, vhost: &str, params: &ExchangeParams<'_>) -> Result<()> {
        self.put_api_request(path!("exchanges", vhost, params.name), params)
    }

    /// Deletes an exchange in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent exchange
    /// will fail.
    pub fn delete_exchange(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("exchanges", vhost, name),
            idempotently,
        )
    }

    /// Deletes multiple exchanges in a specified virtual host.
    ///
    /// When `idempotently` is true, non-existent exchanges are silently skipped.
    /// When `idempotently` is false, the operation fails on the first non-existent exchange.
    pub fn delete_exchanges(&self, vhost: &str, names: &[&str], idempotently: bool) -> Result<()> {
        for name in names {
            self.delete_exchange(vhost, name, idempotently)?;
        }
        Ok(())
    }
}
