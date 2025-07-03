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
use rabbitmq_http_client::{api::Client, password_hashing, requests::UserParams};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[tokio::test]
async fn test_async_list_users() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_users().await;

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().any(|u| u.name == "guest"))
}

#[tokio::test]
async fn test_async_list_users_without_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let test_name = "test_async_list_users_without_permissions";

    let username = format!("{test_name}.1");
    rc.delete_user(&username, true)
        .await
        .expect("failed to delete a user");

    let salt = password_hashing::salt();
    let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(
        &salt,
        "test_async_list_users_without_permissions.1",
    );
    let params = UserParams {
        name: &username,
        password_hash: &password_hash,
        tags: "",
    };
    rc.create_user(&params)
        .await
        .expect("failed to create a user");

    let result1 = rc.list_users_without_permissions().await;
    assert!(result1.is_ok());
    let vec = result1.unwrap();
    assert!(vec.iter().any(|u| u.name == username));

    rc.delete_user(&username, true)
        .await
        .expect("failed to delete a user");
}

#[tokio::test]
async fn test_async_get_user() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let name = "guest";
    let result = rc.get_user(name).await;

    assert!(result.is_ok());
    let u = result.unwrap();
    assert!(u.name == name);
}

#[tokio::test]
async fn test_async_user_creation() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "rust3_t0p_sEkr37");

    let params = UserParams {
        name: "rust3",
        password_hash: &password_hash,
        tags: "management",
    };
    let result = rc.create_user(&params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_async_user_deletion() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "del3te_me");

    let name = "del3te_me";
    let params = UserParams {
        name,
        password_hash: &password_hash,
        tags: "management",
    };
    let result1 = rc.create_user(&params).await;
    assert!(result1.is_ok());

    let result2 = rc.delete_user(name, false).await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_async_bulk_user_deletion() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "del3te_me");

    let name1 = "del3te_me_1";
    let params1 = UserParams {
        name: name1,
        password_hash: &password_hash,
        tags: "management",
    };
    let result1 = rc.create_user(&params1).await;
    assert!(result1.is_ok());

    let name2 = "del3te_me_2";
    let params2 = UserParams {
        name: name2,
        password_hash: &password_hash,
        tags: "management",
    };
    let result2 = rc.create_user(&params2).await;
    assert!(result2.is_ok());

    let result2 = rc.delete_users(vec![name1, name2]).await;
    assert!(result2.is_ok());
}
