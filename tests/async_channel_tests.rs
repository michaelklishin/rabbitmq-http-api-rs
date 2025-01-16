use amqprs::connection::{Connection, OpenConnectionArguments};
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
use rabbitmq_http_client::api::Client;

mod test_helpers;
use crate::test_helpers::{endpoint, hostname, PASSWORD, USERNAME};

#[tokio::test]
async fn test_list_channels() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let ch = conn.open_channel(None).await.unwrap();
    assert!(ch.is_open());

    let result1 = rc.list_channels().await;
    assert!(result1.is_ok(), "list_channels returned {:?}", result1);

    // just to be explicit
    ch.close().await.unwrap();
    conn.clone().close().await.unwrap();
}

#[tokio::test]
async fn test_list_virtual_host_channels() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let ch = conn.open_channel(None).await.unwrap();
    assert!(ch.is_open());

    let vh_name = "/";
    let result1 = rc.list_channels_in(vh_name).await;
    assert!(result1.is_ok(), "list_channels_in returned {:?}", result1);

    // just to be explicit
    ch.close().await.unwrap();
    conn.clone().close().await.unwrap();
}
