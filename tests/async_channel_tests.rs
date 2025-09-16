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
use amqprs::{channel::ConfirmSelectArguments, connection::{Connection, OpenConnectionArguments}};
use rabbitmq_http_client::api::Client;
use std::time::Duration;

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint, hostname};

#[tokio::test]
async fn test_async_list_channels() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let ch = conn.open_channel(None).await.unwrap();
    assert!(ch.is_open());

    let result1 = rc.list_channels().await;
    assert!(result1.is_ok(), "list_channels returned {result1:?}");

    // just to be explicit
    ch.close().await.unwrap();
    conn.clone().close().await.unwrap();
}

#[tokio::test]
async fn test_async_list_virtual_host_channels() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let ch = conn.open_channel(None).await.unwrap();
    assert!(ch.is_open());

    let vh_name = "/";
    let result1 = rc.list_channels_in(vh_name).await;
    assert!(result1.is_ok(), "list_channels_in returned {result1:?}");

    // just to be explicit
    ch.close().await.unwrap();
    conn.clone().close().await.unwrap();
}

#[tokio::test]
async fn test_async_list_channels_on_connection() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let ch = conn.open_channel(None).await.unwrap();
    assert!(ch.is_open());

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let connections = rc.list_connections().await.unwrap();
    let first = connections.first().unwrap();

    let result1 = rc.list_channels_on(&first.name).await;
    assert!(result1.is_ok(), "list_channels_on returned {result1:?}");

    let channels = result1.unwrap();
    assert_eq!(1, channels.len());

    // just to be explicit
    ch.close().await.unwrap();
    conn.clone().close().await.unwrap();
}

#[tokio::test]
async fn test_async_get_channel_info() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let ch = conn.open_channel(None).await.unwrap();
    assert!(ch.is_open());
    let _ = ch.confirm_select(ConfirmSelectArguments::default()).await;

    tokio::time::sleep(Duration::from_millis(1000)).await;

    let channels = rc.list_channels().await.unwrap();
    assert!(!channels.is_empty(), "Expected at least one channel");

    let first_channel = channels.first().unwrap();

    // Note: the HTTP API uses a string channel name, not the numeric ID
    let result1 = rc.get_channel_info(&first_channel.name).await;
    assert!(result1.is_ok(), "get_channel_info returned {result1:?}");

    let ch_details = result1.unwrap();
    assert_eq!(ch_details.vhost, "/");
    assert_eq!(ch_details.consumer_count, 0, "Expected 0 consumers");

    // just to be explicit
    ch.close().await.unwrap();
    conn.clone().close().await.unwrap();
}
