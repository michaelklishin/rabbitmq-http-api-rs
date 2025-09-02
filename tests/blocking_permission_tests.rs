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
use rabbitmq_http_client::requests::VirtualHostParams;
use rabbitmq_http_client::responses;
use rabbitmq_http_client::blocking_api::Client;
use rabbitmq_http_client::requests::Permissions;

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint};

#[test]
fn test_blocking_list_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_permissions");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let result = rc.list_permissions();
    assert!(result.is_ok());

    let vec = result.unwrap();
    assert!(vec.iter().any(|p| p
        == &responses::Permissions {
            user: "guest".to_owned(),
            vhost: "test_list_permissions".to_owned(),
            configure: ".*".to_owned(),
            read: ".*".to_owned(),
            write: ".*".to_owned(),
        }));

    rc.delete_vhost(vh_params.name, false).unwrap();
}

#[test]
fn test_blocking_list_permissions_in() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_permissions_in");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let result = rc.list_permissions_in("test_list_permissions_in");
    assert!(result.is_ok(), "list_permissions_in returned {result:?}");

    let vec = result.unwrap();
    assert!(vec.iter().any(|p| p
        == &responses::Permissions {
            user: "guest".to_owned(),
            vhost: "test_list_permissions_in".to_owned(),
            configure: ".*".to_owned(),
            read: ".*".to_owned(),
            write: ".*".to_owned(),
        }));

    rc.delete_vhost(vh_params.name, false).unwrap();
}

#[test]
fn test_blocking_list_permissions_of() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_permissions_of");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let result = rc.list_permissions_of("guest");
    assert!(result.is_ok(), "list_permissions_of returned {result:?}");

    let vec = result.unwrap();
    assert!(vec.iter().any(|p| p
        == &responses::Permissions {
            user: "guest".to_owned(),
            vhost: "test_list_permissions_of".to_owned(),
            configure: ".*".to_owned(),
            read: ".*".to_owned(),
            write: ".*".to_owned(),
        }));

    rc.delete_vhost(vh_params.name, false).unwrap();
}

#[test]
fn test_blocking_get_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_get_permissions");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let result2 = rc.get_permissions("test_get_permissions", "guest");
    assert!(result2.is_ok(), "list_permissions_of returned {result2:?}");

    let result3 = result2.unwrap();
    assert_eq!(
        result3,
        responses::Permissions {
            user: "guest".to_owned(),
            vhost: "test_get_permissions".to_owned(),
            configure: ".*".to_owned(),
            read: ".*".to_owned(),
            write: ".*".to_owned(),
        }
    );

    rc.delete_vhost(vh_params.name, false).unwrap();
}

#[test]
fn test_blocking_list_topic_permissions_of() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_topic_permissions_of");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let result = rc.list_topic_permissions_of("guest");
    assert!(result.is_ok(), "list_topic_permissions_of returned {result:?}");

    rc.delete_vhost(vh_params.name, false).unwrap();
}

#[test]
fn test_blocking_grant_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_grant_permissions");
    let _ = rc.delete_vhost(vh_params.name, false);
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let permissions_to_grant = Permissions {
        user: "guest",
        vhost: vh_params.name,
        configure: ".*",
        read: ".*",
        write: ".*",
    };
    let result = rc.grant_permissions(&permissions_to_grant);
    assert!(result.is_ok(), "grant_permissions returned {result:?}");

    let result2 = rc.get_permissions(vh_params.name, "guest");
    assert!(result2.is_ok(), "get_permissions_of returned {result2:?}");

    let result3 = result2.unwrap();
    assert_eq!(
        result3,
        responses::Permissions {
            user: "guest".to_owned(),
            vhost: vh_params.name.to_owned(),
            configure: ".*".to_owned(),
            read: ".*".to_owned(),
            write: ".*".to_owned(),
        }
    );

    let result4 = rc.clear_permissions(vh_params.name, "guest", false);
    assert!(result4.is_ok(), "clear_permissions returned {result4:?}");

    let result5 = rc.get_permissions(vh_params.name, "guest");
    assert!(result5.is_err(), "permissions found after deletion");

    rc.delete_vhost(vh_params.name, false).unwrap();
}
