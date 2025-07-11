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
use rabbitmq_http_client::{blocking_api::Client, password_hashing, requests::UserParams};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_blocking_list_users() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let result = rc.list_users();

    assert!(result.is_ok());
    let vec = result.unwrap();
    assert!(vec.iter().any(|u| u.name == "guest"))
}

#[test]
fn test_blocking_list_users_without_permissions() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let test_name = "test_blocking_list_users_without_permissions";

    let username = format!("{test_name}.1");
    rc.delete_user(&username, true)
        .expect("failed to delete a user");

    let salt = password_hashing::salt();
    let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(
        &salt,
        "test_blocking_list_users_without_permissions.1",
    );
    let params = UserParams {
        name: &username,
        password_hash: &password_hash,
        tags: "",
    };
    rc.create_user(&params).expect("failed to create a user");

    let result1 = rc.list_users_without_permissions();
    assert!(result1.is_ok());
    let vec = result1.unwrap();
    assert!(vec.iter().any(|u| u.name == username));

    rc.delete_user(&username, true)
        .expect("failed to delete a user");
}

#[test]
fn test_blocking_get_user() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let name = "guest";
    let result = rc.get_user(name);

    assert!(result.is_ok());
    let u = result.unwrap();
    assert!(u.name == name);
}

#[test]
fn test_blocking_user_creation_using_default_hashing_algorithm() {
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
    let result = rc.create_user(&params);
    assert!(result.is_ok());

    rc.delete_user(&params.name, true)
        .expect("failed to delete a user");
}

#[test]
fn test_blocking_user_creation_using_sha512() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha512(&salt, "rust4_t0p_sEkr37");

    let params = UserParams {
        name: "rust4",
        password_hash: &password_hash,
        tags: "management",
    };
    let result = rc.create_user(&params);
    assert!(result.is_ok());

    rc.delete_user(&params.name, true)
        .expect("failed to delete a user");
}

#[test]
fn test_blocking_user_deletion() {
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
    let result1 = rc.create_user(&params);
    assert!(result1.is_ok());

    let result2 = rc.delete_user(name, false);
    assert!(result2.is_ok());
}

#[test]
fn test_blocking_bulk_user_deletion() {
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
    let result1 = rc.create_user(&params1);
    assert!(result1.is_ok());

    let name2 = "del3te_me_2";
    let params2 = UserParams {
        name: name2,
        password_hash: &password_hash,
        tags: "management",
    };
    let result2 = rc.create_user(&params2);
    assert!(result2.is_ok());

    let result2 = rc.delete_users(vec![name1, name2]);
    assert!(result2.is_ok());
}
