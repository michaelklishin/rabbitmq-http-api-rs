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

mod test_helpers;

use crate::test_helpers::{PASSWORD, USERNAME, endpoint};
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rabbitmq_http_client::{blocking_api::Client, password_hashing, requests::UserParams};

fn arb_username() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.blocking\.proptest\.users\.[a-zA-Z0-9_-]{8,20}")
        .unwrap()
}

fn arb_password() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9!@#$%^&*()_+-=]{12,32}").unwrap()
}

fn arb_user_tags() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("administrator".to_string()),
        Just("monitoring".to_string()),
        Just("management".to_string()),
        Just("administrator,monitoring".to_string()),
        Just("monitoring,management".to_string()),
        Just("administrator,management".to_string()),
        Just("administrator,monitoring,management".to_string()),
    ]
}

fn arb_user_params() -> impl Strategy<Value = (String, String, String)> {
    (arb_username(), arb_password(), arb_user_tags())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prop_blocking_user_create_list_delete(
        (username, password, tags) in arb_user_params()
    ) {
        let endpoint = endpoint();
        let client = Client::new(&endpoint, USERNAME, PASSWORD);

        let _ = client.delete_user(&username, true);

        let salt = password_hashing::salt();
        let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &password);
        let params = UserParams {
            name: &username,
            password_hash: &password_hash,
            tags: &tags,
        };

        let create_result = client.create_user(&params);
        prop_assert!(create_result.is_ok(), "Failed to create user: {create_result:?}");


        let list_result = client.list_users();
        prop_assert!(list_result.is_ok(), "Failed to list users: {list_result:?}");

        let users = list_result.unwrap();
        let found_user = users.iter().find(|u| u.name == username);
        prop_assert!(found_user.is_some(), "list_users did not include the created user: {}", username);

        let user = found_user.unwrap();
        prop_assert_eq!(&user.name, &username);
        prop_assert_eq!(&user.password_hash, &password_hash);

        let get_result = client.get_user(&username);
        prop_assert!(get_result.is_ok(), "Failed to get user info: {get_result:?}");

        let user_info = get_result.unwrap();
        prop_assert_eq!(&user_info.name, &username);
        prop_assert_eq!(&user_info.password_hash, &password_hash);

        let delete_result = client.delete_user(&username, false);
        prop_assert!(delete_result.is_ok(), "Failed to delete user: {delete_result:?}");
    }

    #[test]
    fn prop_blocking_user_without_permissions(
        (username, password) in (arb_username(), arb_password())
    ) {
        let endpoint = endpoint();
        let client = Client::new(&endpoint, USERNAME, PASSWORD);

        let _ = client.delete_user(&username, true);

        let salt = password_hashing::salt();
        let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &password);
        let params = UserParams {
            name: &username,
            password_hash: &password_hash,
            tags: "",
        };

        let create_result = client.create_user(&params);
        prop_assert!(create_result.is_ok(), "Failed to create user without permissions: {create_result:?}");


        let list_without_permissions_result = client.list_users_without_permissions();
        prop_assert!(list_without_permissions_result.is_ok(), "Failed to list users without permissions: {list_without_permissions_result:?}");

        let users_without_permissions = list_without_permissions_result.unwrap();
        let found_user = users_without_permissions.iter().find(|u| u.name == username);
        prop_assert!(found_user.is_some(), "list_users_without_permissions did not include the user: {}", username);

        let _ = client.delete_user(&username, true);
    }

    #[test]
    fn prop_blocking_user_idempotent_delete(
        username in arb_username()
    ) {
        let endpoint = endpoint();
        let client = Client::new(&endpoint, USERNAME, PASSWORD);

        let delete_nonexistent_result = client.delete_user(&username, true);
        prop_assert!(delete_nonexistent_result.is_ok(), "Idempotent delete of non-existent user should succeed: {delete_nonexistent_result:?}");

        let salt = password_hashing::salt();
        let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "testpassword");
        let params = UserParams {
            name: &username,
            password_hash: &password_hash,
            tags: "monitoring",
        };

        let create_result = client.create_user(&params);
        prop_assert!(create_result.is_ok(), "Failed to create user: {create_result:?}");


        let delete_existing_result = client.delete_user(&username, true);
        prop_assert!(delete_existing_result.is_ok(), "Failed to delete existing user: {delete_existing_result:?}");

        let delete_again_result = client.delete_user(&username, true);
        prop_assert!(delete_again_result.is_ok(), "Second idempotent delete should succeed: {delete_again_result:?}");
    }

    #[test]
    fn prop_blocking_user_bulk_delete(
        usernames in prop::collection::vec(arb_username(), 2..5)
    ) {
        let endpoint = endpoint();
        let client = Client::new(&endpoint, USERNAME, PASSWORD);

        for username in &usernames {
            let _ = client.delete_user(username, true);
        }

        for username in &usernames {
            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "testpassword");
            let params = UserParams {
                name: username,
                password_hash: &password_hash,
                tags: "monitoring",
            };

            let create_result = client.create_user(&params);
            prop_assert!(create_result.is_ok(), "Failed to create user {}: {create_result:?}", username);
        }


        let list_result = client.list_users();
        prop_assert!(list_result.is_ok(), "Failed to list users: {list_result:?}");

        let users = list_result.unwrap();
        for username in &usernames {
            let found_user = users.iter().any(|u| u.name == *username);
            prop_assert!(found_user, "list_users did not include the created user {}", username);
        }

        let usernames_ref: Vec<&str> = usernames.iter().map(|s| s.as_str()).collect();
        let bulk_delete_result = client.delete_users(usernames_ref);
        prop_assert!(bulk_delete_result.is_ok(), "Failed to bulk delete users: {bulk_delete_result:?}");


        let list_after_delete_result = client.list_users();
        prop_assert!(list_after_delete_result.is_ok(), "Failed to list users after bulk delete: {list_after_delete_result:?}");

        let users_after_delete = list_after_delete_result.unwrap();
        for username in &usernames {
            let found_user = users_after_delete.iter().any(|u| u.name == *username);
            prop_assert!(!found_user, "list_users still includes deleted user {}", username);
        }
    }

    #[test]
    fn prop_blocking_user_list_consistency(
        usernames in prop::collection::vec(arb_username(), 1..5)
    ) {
        let endpoint = endpoint();
        let client = Client::new(&endpoint, USERNAME, PASSWORD);

        for username in &usernames {
            let _ = client.delete_user(username, true);
        }

        for username in &usernames {
            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "testpassword");
            let params = UserParams {
                name: username,
                password_hash: &password_hash,
                tags: "administrator",
            };

            let create_result = client.create_user(&params);
            prop_assert!(create_result.is_ok(), "Failed to create user {}: {create_result:?}", username);
        }


        let list_result = client.list_users();
        prop_assert!(list_result.is_ok(), "Failed to list users: {list_result:?}");

        let users = list_result.unwrap();

        for username in &usernames {
            let found_user = users.iter().any(|u| u.name == *username);
            prop_assert!(found_user, "list_users did not include the created user {}", username);
        }

        for username in &usernames {
            let _ = client.delete_user(username, true);
        }
    }

    #[test]
    fn prop_blocking_current_user_info(
        _dummy_input in Just(1u8)
    ) {
        let endpoint = endpoint();
        let client = Client::new(&endpoint, USERNAME, PASSWORD);

        let current_user_result = client.current_user();
        prop_assert!(current_user_result.is_ok(), "Failed to get current user info: {current_user_result:?}");

        let current_user = current_user_result.unwrap();
        prop_assert_eq!(current_user.name, USERNAME.to_string());
    }
}
