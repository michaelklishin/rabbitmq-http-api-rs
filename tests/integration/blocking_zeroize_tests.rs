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

use rabbitmq_http_client::blocking_api::{Client, ClientBuilder};
use rabbitmq_http_client::commons::Password;

use crate::test_helpers::ENDPOINT;

#[test]
fn test_blocking_client_with_password_type() {
    let password = Password::from("guest");
    let rc = Client::new(ENDPOINT, "guest", password);

    let result = rc.list_nodes();
    assert!(result.is_ok(), "list_nodes failed: {:?}", result.err());

    let nodes = result.unwrap();
    assert!(!nodes.is_empty());
}

#[test]
fn test_blocking_client_with_password_username_and_password() {
    let username = Password::from("guest");
    let password = Password::from("guest");
    let rc = Client::new(ENDPOINT, username, password);

    let result = rc.overview();
    assert!(result.is_ok(), "overview failed: {:?}", result.err());
}

#[test]
fn test_blocking_client_builder_with_password_type() {
    let password = Password::from("guest");
    let rc = ClientBuilder::new()
        .with_endpoint(ENDPOINT)
        .with_basic_auth_credentials("guest", password)
        .build()
        .unwrap();

    let result = rc.list_nodes();
    assert!(result.is_ok(), "list_nodes failed: {:?}", result.err());
}
