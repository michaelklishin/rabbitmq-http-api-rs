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

use crate::{path, requests::ExchangeParams, responses};

use super::client::{Client, Result};

impl<E, U, P> Client<E, U, P>
where
    E: std::fmt::Display,
    U: std::fmt::Display,
    P: std::fmt::Display,
{
    /// Lists all exchanges across the cluster.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub async fn list_exchanges(&self) -> Result<Vec<responses::ExchangeInfo>> {
        let response = self.http_get("exchanges", None, None).await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Lists all exchanges in the given virtual host.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub async fn list_exchanges_in(
        &self,
        virtual_host: &str,
    ) -> Result<Vec<responses::ExchangeInfo>> {
        let response = self
            .http_get(path!("exchanges", virtual_host), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Returns information about an exchange.
    /// See [Exchanges Guide](https://www.rabbitmq.com/docs/exchanges) to learn more.
    pub async fn get_exchange_info(
        &self,
        virtual_host: &str,
        name: &str,
    ) -> Result<responses::ExchangeInfo> {
        let response = self
            .http_get(path!("exchanges", virtual_host, name), None, None)
            .await?;
        let response = response.json().await?;
        Ok(response)
    }

    /// Declares an [exchange](https://www.rabbitmq.com/docs/exchanges).
    ///
    /// If the exchange already exists with different parameters, this operation may fail
    /// unless the parameters are equivalent.
    pub async fn declare_exchange(&self, vhost: &str, params: &ExchangeParams<'_>) -> Result<()> {
        self.put_api_request(path!("exchanges", vhost, params.name), params)
            .await
    }

    /// Deletes an exchange in a specified virtual host.
    ///
    /// Unless `idempotently` is set to `true`, an attempt to delete a non-existent exchange
    /// will fail.
    pub async fn delete_exchange(&self, vhost: &str, name: &str, idempotently: bool) -> Result<()> {
        self.delete_api_request_with_optional_not_found(
            path!("exchanges", vhost, name),
            idempotently,
        )
        .await
    }
}
