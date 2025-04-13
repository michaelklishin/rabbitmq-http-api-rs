use std::time::Duration;
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
use amqprs::connection::{Connection, OpenConnectionArguments};
use rabbitmq_http_client::api::Client;
use rabbitmq_http_client::requests::VirtualHostParams;

mod test_helpers;
use crate::test_helpers::{endpoint, hostname, PASSWORD, USERNAME};

#[tokio::test]
async fn test_async_list_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let result1 = rc.list_connections().await;
    assert!(result1.is_ok(), "list_connections returned {:?}", result1);

    conn.close().await.unwrap();
}

#[tokio::test]
async fn test_async_list_user_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_user_connections(USERNAME).await;
    assert!(
        result1.is_ok(),
        "list_user_connections returned {:?}",
        result1
    );
}

#[tokio::test]
async fn test_async_list_virtual_host_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust/http/api/async/test_list_virtual_host_connections";
    rc.delete_vhost(vh, true).await.unwrap();

    let vh_params = VirtualHostParams::named(vh);
    rc.create_vhost(&vh_params).await.unwrap();

    let result1 = rc.list_connections_in(vh).await;
    assert!(
        result1.is_ok(),
        "list_connections_in returned {:?}",
        result1
    );

    rc.delete_vhost(vh, true).await.unwrap();
}

#[tokio::test]
async fn test_async_list_stream_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_stream_connections().await;
    assert!(
        result1.is_ok(),
        "list_stream_connections returned {:?}",
        result1
    );
}

#[tokio::test]
async fn test_async_list_virtual_host_stream_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let result1 = rc.list_stream_connections_in(vh_name).await;
    assert!(
        result1.is_ok(),
        "list_stream_connections returned {:?}",
        result1
    );
}

#[tokio::test]
async fn test_async_close_user_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let result1 = rc
        .close_user_connections(
            USERNAME,
            Some("closed in test_async_close_user_connections"),
        )
        .await;
    assert!(
        result1.is_ok(),
        "close_user_connections returned {:?}",
        result1
    );

    tokio::time::sleep(Duration::from_millis(40)).await;
    assert!(!conn.is_open());
}
