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
use rabbitmq_http_client::commons::ExchangeType;
use rabbitmq_http_client::requests;
use rabbitmq_http_client::requests::ExchangeParams;
use rabbitmq_http_client::requests::Permissions;
use rabbitmq_http_client::requests::VirtualHostParams;
use rabbitmq_http_client::responses;

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint};

#[tokio::test]
async fn test_async_list_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_permissions");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let result = rc.list_permissions().await;
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

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_list_permissions_in() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_permissions_in");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let result = rc.list_permissions_in("test_list_permissions_in").await;
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

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_list_permissions_of() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_permissions_of");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let result = rc.list_permissions_of("guest").await;
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

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_get_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_get_permissions");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let result2 = rc.get_permissions("test_get_permissions", "guest").await;
    assert!(result2.is_ok(), "list_permissions_of returned {result2:?}");

    let permissions = result2.unwrap();
    assert_eq!(
        permissions,
        responses::Permissions {
            user: "guest".to_owned(),
            vhost: "test_get_permissions".to_owned(),
            configure: ".*".to_owned(),
            read: ".*".to_owned(),
            write: ".*".to_owned(),
        }
    );

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_grant_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_grant_permissions");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let permissions_to_grant = Permissions {
        user: "guest",
        vhost: vh_params.name,
        configure: ".*",
        read: ".*",
        write: ".*",
    };
    let result = rc.grant_permissions(&permissions_to_grant).await;
    assert!(result.is_ok(), "grant_permissions returned {result:?}");

    let result2 = rc.get_permissions(vh_params.name, "guest").await;
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

    let result4 = rc.clear_permissions(vh_params.name, "guest", false).await;
    assert!(result4.is_ok(), "clear_permissions returned {result4:?}");

    let result5 = rc.get_permissions(vh_params.name, "guest").await;
    assert!(result5.is_err(), "permissions found after deletion");

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_list_topic_permissions_of() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_topic_permissions_of");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let result = rc.list_topic_permissions_of("guest").await;
    assert!(
        result.is_ok(),
        "list_topic_permissions_of returned {result:?}"
    );

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_list_topic_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_topic_permissions");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let x_name = "test_async_list_topic_permissions.alt.topic";
    let tx_params = ExchangeParams {
        name: x_name,
        exchange_type: ExchangeType::Topic,
        durable: false,
        auto_delete: false,
        arguments: None,
    };
    let result2 = rc.declare_exchange(&vh_params.name, &tx_params).await;
    assert!(result2.is_ok());

    let params = requests::TopicPermissions {
        user: "guest",
        vhost: vh_params.name,
        exchange: x_name,
        read: ".*",
        write: ".*",
    };
    let result3 = rc.declare_topic_permissions(&params).await;
    assert!(
        result3.is_ok(),
        "declare_topic_permissions returned {result3:?}"
    );

    let result4 = rc.list_topic_permissions().await;
    assert!(result4.is_ok());
    let permissions = result4.unwrap();
    assert!(permissions.iter().any(|p| p.vhost == vh_params.name));

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_list_topic_permissions_in_vhost() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_async_list_topic_permissions_in_vhost");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let x_name = "test_async_list_topic_permissions_in_vhost.alt.topic";
    let tx_params = ExchangeParams {
        name: x_name,
        exchange_type: ExchangeType::Topic,
        durable: false,
        auto_delete: false,
        arguments: None,
    };
    let result2 = rc.declare_exchange(&vh_params.name, &tx_params).await;
    assert!(result2.is_ok());

    let topic_permissions_grant_params = requests::TopicPermissions {
        user: "guest",
        vhost: vh_params.name,
        exchange: x_name,
        read: ".*",
        write: ".*",
    };
    let result3 = rc
        .declare_topic_permissions(&topic_permissions_grant_params)
        .await;
    assert!(
        result3.is_ok(),
        "declare_topic_permissions returned {result3:?}"
    );

    let result4 = rc.list_topic_permissions_in(vh_params.name).await;
    assert!(result4.is_ok());
    let permissions = result4.unwrap();
    assert!(permissions.iter().any(|p| p.vhost == vh_params.name));

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_get_topic_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_async_get_topic_permissions");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let x_name = "test_async_get_topic_permissions.alt.topic";
    let tx_params = ExchangeParams {
        name: x_name,
        exchange_type: ExchangeType::Topic,
        durable: false,
        auto_delete: false,
        arguments: None,
    };
    let result2 = rc.declare_exchange(&vh_params.name, &tx_params).await;
    assert!(result2.is_ok());

    let params = requests::TopicPermissions {
        user: "guest",
        vhost: vh_params.name,
        exchange: x_name,
        read: ".*",
        write: ".*",
    };
    let result3 = rc.declare_topic_permissions(&params).await;
    assert!(
        result3.is_ok(),
        "declare_topic_permissions returned {result3:?}"
    );

    let result4 = rc.get_topic_permissions_of(vh_params.name, "guest").await;
    assert!(result4.is_ok());
    let permissions = result4.unwrap();
    assert_eq!(permissions.vhost, vh_params.name);
    assert_eq!(permissions.user, "guest");
    assert_eq!(permissions.exchange, x_name);
    assert_eq!(permissions.read, ".*");
    assert_eq!(permissions.write, ".*");

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_declare_topic_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_declare_topic_permissions");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let params = requests::TopicPermissions {
        user: "guest",
        vhost: vh_params.name,
        exchange: "amq.topic",
        read: ".*",
        write: ".*",
    };
    let result = rc.declare_topic_permissions(&params).await;
    assert!(
        result.is_ok(),
        "declare_topic_permissions returned {result:?}"
    );

    // Verify that the topic permissions are set
    let topic_permissions = rc.list_topic_permissions_of("guest").await.unwrap();
    assert!(topic_permissions.iter().any(|p| p.vhost == vh_params.name
        && p.exchange == "amq.topic"
        && p.read == ".*"
        && p.write == ".*"));

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_clear_topic_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_clear_topic_permissions");
    let _ = rc.delete_vhost(vh_params.name, false).await;
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let params = requests::TopicPermissions {
        user: "guest",
        vhost: vh_params.name,
        exchange: "amq.topic",
        read: ".*",
        write: ".*",
    };
    let result = rc.declare_topic_permissions(&params).await;
    assert!(
        result.is_ok(),
        "declare_topic_permissions returned {result:?}"
    );

    // Verify that the topic permissions are set
    let topic_permissions = rc.list_topic_permissions_of("guest").await.unwrap();
    assert!(topic_permissions.iter().any(|p| p.vhost == vh_params.name
        && p.exchange == "amq.topic"
        && p.read == ".*"
        && p.write == ".*"));

    let result2 = rc
        .clear_topic_permissions(vh_params.name, "guest", false)
        .await;
    assert!(
        result2.is_ok(),
        "clear_topic_permissions returned {result2:?}"
    );

    // Verify that the topic permissions are cleared
    let topic_permissions_after_clear = rc.list_topic_permissions_of("guest").await.unwrap();
    assert!(
        !topic_permissions_after_clear
            .iter()
            .any(|p| p.vhost == vh_params.name
                && p.exchange == "amq.topic"
                && p.read == ".*"
                && p.write == ".*")
    );

    rc.delete_vhost(vh_params.name, false).await.unwrap();
}
