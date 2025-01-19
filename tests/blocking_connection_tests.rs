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
use rabbitmq_http_client::blocking_api::Client;
use rabbitmq_http_client::requests::VirtualHostParams;

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_connections();
    assert!(result1.is_ok(), "list_connections returned {:?}", result1);
}

#[test]
fn test_list_user_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_user_connections(USERNAME);
    assert!(
        result1.is_ok(),
        "list_user_connections returned {:?}",
        result1
    );
}

#[test]
fn test_list_virtual_host_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust/http/api/blocking/test_list_virtual_host_connections";
    let _ = rc.delete_vhost(vh, true).unwrap();

    let vh_params = VirtualHostParams::named(vh);
    rc.create_vhost(&vh_params).unwrap();

    let result1 = rc.list_connections_in(vh);
    assert!(
        result1.is_ok(),
        "list_connections_in returned {:?}",
        result1
    );

    let _ = rc.delete_vhost(vh, true).unwrap();
}

#[test]
fn test_list_stream_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result1 = rc.list_stream_connections();
    assert!(
        result1.is_ok(),
        "list_stream_connections returned {:?}",
        result1
    );
}

#[test]
fn test_list_virtual_host_stream_connections() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_name = "/";
    let result1 = rc.list_stream_connections_in(vh_name);
    assert!(
        result1.is_ok(),
        "list_stream_connections returned {:?}",
        result1
    );
}
