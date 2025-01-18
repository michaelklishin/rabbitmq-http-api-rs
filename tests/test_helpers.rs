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

use rabbitmq_http_client::blocking_api::Client as GenericAPIClient;
use std::env;
use std::time::Duration;

use amqprs::channel::BasicPublishArguments;
use amqprs::connection::{Connection, OpenConnectionArguments};
use amqprs::BasicProperties;

pub const ENDPOINT: &str = "http://localhost:15672/api";
pub const USERNAME: &str = "guest";
pub const PASSWORD: &str = "guest";

pub type APIClient<'a> = GenericAPIClient<&'a str, &'a str, &'a str>;

pub fn endpoint() -> String {
    ENDPOINT.to_owned()
}

pub fn hostname() -> String {
    "localhost".to_owned()
}

pub fn await_metric_emission(ms: u64) {
    std::thread::sleep(Duration::from_millis(ms));
}

pub fn await_queue_metric_emission() {
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
}
