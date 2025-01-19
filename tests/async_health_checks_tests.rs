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
use rabbitmq_http_client::{api::Client, commons::SupportedProtocol};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[tokio::test]
async fn test_async_health_check_cluster_wide_alarms() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_cluster_wide_alarms().await;
    assert!(result1.is_ok());
}

#[tokio::test]
async fn test_async_health_check_local_alarms() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_local_alarms().await;
    assert!(result1.is_ok());
}

#[tokio::test]
async fn test_async_health_check_node_is_quorum_critical() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_if_node_is_quorum_critical().await;
    assert!(result1.is_ok());
}

#[tokio::test]
async fn test_async_health_check_port_listener_succeeds() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_port_listener(15672).await;
    assert!(result1.is_ok());
}

#[tokio::test]
async fn test_async_health_check_port_listener_fails() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.health_check_port_listener(15679).await;
    assert!(result1.is_err());
}

#[tokio::test]
async fn test_async_health_check_protocol_listener_succeeds() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc
        .health_check_protocol_listener(SupportedProtocol::HTTP)
        .await;
    assert!(result1.is_ok());

    let result2 = rc
        .health_check_protocol_listener(SupportedProtocol::AMQP)
        .await;
    assert!(result2.is_ok());

    let result3 = rc
        .health_check_protocol_listener(SupportedProtocol::Stream)
        .await;
    assert!(result3.is_ok());
}

#[tokio::test]
async fn test_async_health_check_protocol_listener_fails() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc
        .health_check_protocol_listener(SupportedProtocol::Other("https/non-existent".to_owned()))
        .await;
    assert!(result1.is_err());

    let result2 = rc
        .health_check_protocol_listener(SupportedProtocol::STOMPOverWebsocketsWithTLS)
        .await;
    assert!(result2.is_err());
}
