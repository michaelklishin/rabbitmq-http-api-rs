// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
use rabbitmq_http_client::{blocking_api::Client, commons::QueueType, requests::VirtualHostParams};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_list_vhosts() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_vhosts();

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().any(|vh| vh.name == "/"))
}

#[test]
fn test_get_vhost() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let name = "/";
    let result = rc.get_vhost(name);

    assert!(result.is_ok());
    let vh = result.unwrap();
    assert!(vh.name == name);
}

#[test]
fn test_create_vhost() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let name = "rust_test_create_vhost";

    let _ = rc.delete_vhost(name, false);

    let result1 = rc.get_vhost(name);
    assert!(result1.is_err());

    let desc = format!("{} description", &name);
    let params = VirtualHostParams {
        name,
        description: Some(&desc),
        tags: Some(vec!["tag-a", "tag-b"]),
        default_queue_type: Some(QueueType::Classic),
        tracing: false,
    };
    let result2 = rc.create_vhost(&params);
    assert!(result2.is_ok());

    let result3 = rc.get_vhost(name);
    assert!(result3.is_ok());
    let vh2 = result3.unwrap();
    assert!(vh2.name == name);

    let _ = rc.delete_vhost(name, false);
}

#[test]
fn test_update_vhost() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let name = "rust_test_update_vhost";

    let _ = rc.delete_vhost(name, false);

    let result1 = rc.get_vhost(name);
    assert!(result1.is_err());

    let desc = format!("{} description", &name);
    let params1 = VirtualHostParams {
        name,
        description: Some(&desc),
        tags: Some(vec!["tag-a", "tag-b"]),
        default_queue_type: Some(QueueType::Classic),
        tracing: false,
    };
    let result2 = rc.create_vhost(&params1);
    assert!(result2.is_ok());

    let alt_desc = "altered description";
    let params2 = VirtualHostParams {
        description: Some(alt_desc),
        ..params1
    };
    let result3 = rc.update_vhost(&params2);
    assert!(result3.is_ok());

    let result4 = rc.get_vhost(name);
    assert!(result4.is_ok());
    let vh = result4.unwrap();
    assert!(vh.description.unwrap() == alt_desc);

    let _ = rc.delete_vhost(name, false);
}

#[test]
fn test_delete_vhost() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let name = "rust_test_delete_vhost";

    let desc = format!("{} description", &name);
    let params = VirtualHostParams {
        name,
        description: Some(&desc),
        tags: Some(vec!["tag-a", "tag-b"]),
        default_queue_type: Some(QueueType::Quorum),
        tracing: false,
    };
    let result1 = rc.create_vhost(&params);
    assert!(result1.is_ok());

    let result2 = rc.get_vhost(name);
    assert!(result2.is_ok());

    let _ = rc.delete_vhost(name, false);
    let result3 = rc.get_vhost(name);
    assert!(result3.is_err());
}
