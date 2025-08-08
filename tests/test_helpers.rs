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
#![allow(dead_code)]

use rabbitmq_http_client::blocking_api::Client as BlockingClient;
use std::env;
use std::time::Duration;

use amqprs::BasicProperties;
use amqprs::channel::BasicPublishArguments;
use amqprs::connection::{Connection, OpenConnectionArguments};
use rabbitmq_http_client::api::Client as AsyncClient;
use regex::Regex;
use serde_json::{Map, Value, json};
use tokio::time;
//
// Common
//

pub const ENDPOINT: &str = "http://localhost:15672/api";
pub const USERNAME: &str = "guest";
pub const PASSWORD: &str = "guest";

pub const AMQP_ENDPOINT: &str = "amqp://localhost:5672";

pub type APIClient<'a> = BlockingClient<&'a str, &'a str, &'a str>;

pub fn endpoint() -> String {
    ENDPOINT.to_owned()
}

pub fn hostname() -> String {
    "localhost".to_owned()
}

pub fn amqp_endpoint() -> String {
    AMQP_ENDPOINT.to_owned()
}

pub fn amqp_endpoint_with_vhost(name: &str) -> String {
    format!("{AMQP_ENDPOINT}/{name}").to_owned()
}

pub fn amqp10_endpoint_with_vhost(name: &str) -> String {
    format!("{AMQP_ENDPOINT}?hostname='vhost:{name}'").to_owned()
}

//
// Blocking client tests
//

pub fn testing_against_3_13_x() -> bool {
    testing_against_series("^3.13")
}

pub fn testing_against_4_0_x() -> bool {
    testing_against_series("^4.0")
}

pub fn testing_against_4_1_x() -> bool {
    testing_against_series("^4.1")
}

pub fn testing_against_4_2_x() -> bool {
    testing_against_series("^4.2")
}

pub fn testing_against_series(series: &str) -> bool {
    let endpoint = endpoint();
    let rc = BlockingClient::new(&endpoint, USERNAME, PASSWORD);

    let regex = Regex::new(series).unwrap();
    regex.is_match(&rc.server_version().unwrap())
}

pub fn testing_against_version(series: &str) -> bool {
    let endpoint = endpoint();
    let rc = BlockingClient::new(&endpoint, USERNAME, PASSWORD);

    &rc.server_version().unwrap() == series
}

pub fn await_metric_emission(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub fn await_queue_metric_emission() {
    let delay = env::var("TEST_STATS_DELAY").unwrap_or("500".to_owned());
    await_metric_emission(delay.parse::<u64>().unwrap());
}

//
// Async client tests
//

pub async fn async_testing_against_3_13_x() -> bool {
    async_testing_against_series("^3.13").await
}

pub async fn async_testing_against_4_0_x() -> bool {
    async_testing_against_series("^4.0").await
}

pub async fn async_testing_against_4_1_x() -> bool {
    async_testing_against_series("^4.1").await
}

pub async fn async_testing_against_4_2_x() -> bool {
    async_testing_against_series("^4.2").await
}

pub async fn async_testing_against_series(series: &str) -> bool {
    let endpoint = endpoint();
    let rc = AsyncClient::new(&endpoint, USERNAME, PASSWORD);

    let regex = Regex::new(series).unwrap();
    regex.is_match(&rc.server_version().await.unwrap())
}

pub async fn async_testing_against_version(series: &str) -> bool {
    let endpoint = endpoint();
    let rc = AsyncClient::new(&endpoint, USERNAME, PASSWORD);

    &rc.server_version().await.unwrap() == series
}

pub async fn async_await_metric_emission(ms: u64) {
    time::sleep(Duration::from_millis(ms)).await;
}

pub async fn async_await_queue_metric_emission() {
    let delay = env::var("TEST_STATS_DELAY").unwrap_or("500".to_owned());
    await_metric_emission(delay.parse::<u64>().unwrap());
}

pub async fn generate_activity() {
    let args = OpenConnectionArguments::new(&hostname(), 5672, USERNAME, PASSWORD);
    let conn = Connection::open(&args).await.unwrap();
    assert!(conn.is_open());

    let ch = conn.open_channel(None).await.unwrap();
    assert!(ch.is_open());

    let payload = String::from("a dummy message").into_bytes();
    let args = BasicPublishArguments::new("amq.fanout", "");
    // we do not use publisher confirms here because the goal is
    // merely to force the node to serve some channel and connection metrics
    // which would exist in any practically useful cluster
    for _ in 0..1000 {
        ch.basic_publish(BasicProperties::default(), payload.clone(), args.clone())
            .await
            .unwrap()
    }

    async_await_queue_metric_emission().await;

    conn.close().await.unwrap()
}

//
// Metadata, runtime parameters
//

pub fn cluster_tags(tags: Map<String, Value>) -> Map<String, Value> {
    let mut val = Map::<String, Value>::new();
    val.insert(String::from("cluster_tags"), json!(tags));
    val
}
