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

use crate::test_helpers::{PASSWORD, USERNAME, endpoint};
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rabbitmq_http_client::{api::Client, password_hashing, requests::UserParams};
use tokio::runtime::Runtime;

fn arb_username() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.users\.[a-zA-Z0-9_-]{8,20}").unwrap()
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
    fn prop_async_user_create_list_delete(
        (username, password, tags) in arb_user_params()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_user(&username, true).await;

            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &password);
            let params = UserParams {
                name: &username,
                password_hash: &password_hash,
                tags: &tags,
            };

            let result1 = client.create_user(&params).await;
            prop_assert!(result1.is_ok(), "Failed to create user: {result1:?}");


            let result2 = client.list_users().await;
            prop_assert!(result2.is_ok(), "Failed to list users: {result2:?}");

            let users = result2.unwrap();
            let found_user = users.iter().find(|u| u.name == username);
            prop_assert!(found_user.is_some(), "list_users did not include the created user: {}", username);

            let user = found_user.unwrap();
            prop_assert_eq!(&user.name, &username);
            prop_assert_eq!(&user.password_hash, &password_hash);

            let result3 = client.get_user(&username).await;
            prop_assert!(result3.is_ok(), "Failed to get user info: {result3:?}");

            let user_info = result3.unwrap();
            prop_assert_eq!(&user_info.name, &username);
            prop_assert_eq!(&user_info.password_hash, &password_hash);

            let result4 = client.delete_user(&username, false).await;
            prop_assert!(result4.is_ok(), "Failed to delete user: {result4:?}");

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_user_without_permissions(
        (username, password) in (arb_username(), arb_password())
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_user(&username, true).await;

            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &password);
            let params = UserParams {
                name: &username,
                password_hash: &password_hash,
                tags: "",
            };

            let result1 = client.create_user(&params).await;
            prop_assert!(result1.is_ok(), "Failed to create user without permissions: {result1:?}");


            let result2 = client.list_users_without_permissions().await;
            prop_assert!(result2.is_ok(), "Failed to list users without permissions: {result2:?}");

            let users_without_permissions = result2.unwrap();
            let found_user = users_without_permissions.iter().find(|u| u.name == username);
            prop_assert!(found_user.is_some(), "list_users_without_permissions did not include the user: {}", username);

            let _ = client.delete_user(&username, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_user_idempotent_delete(
        username in arb_username()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let result1 = client.delete_user(&username, true).await;
            prop_assert!(result1.is_ok(), "Idempotent delete of non-existent user should succeed: {result1:?}");

            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "testpassword");
            let params = UserParams {
                name: &username,
                password_hash: &password_hash,
                tags: "monitoring",
            };

            let result2 = client.create_user(&params).await;
            prop_assert!(result2.is_ok(), "Failed to create user: {result2:?}");


            let result3 = client.delete_user(&username, true).await;
            prop_assert!(result3.is_ok(), "Failed to delete existing user: {result3:?}");

            let result4 = client.delete_user(&username, true).await;
            prop_assert!(result4.is_ok(), "Second idempotent delete should succeed: {result4:?}");

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_user_bulk_delete(
        usernames in prop::collection::vec(arb_username(), 2..4)
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            for username in &usernames {
                let _ = client.delete_user(username, true).await;
            }

            for username in &usernames {
                let salt = password_hashing::salt();
                let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "testpassword");
                let params = UserParams {
                    name: username,
                    password_hash: &password_hash,
                    tags: "monitoring",
                };

                let result1 = client.create_user(&params).await;
                prop_assert!(result1.is_ok(), "Failed to create user {}: {result1:?}", username);
            }


            let result2 = client.list_users().await;
            prop_assert!(result2.is_ok(), "Failed to list users: {result2:?}");

            let users = result2.unwrap();
            for username in &usernames {
                let found_user = users.iter().any(|u| u.name == *username);
                prop_assert!(found_user, "list_users did not include the created user {}", username);
            }

            let usernames_ref: Vec<&str> = usernames.iter().map(|s| s.as_str()).collect();
            let result3 = client.delete_users(usernames_ref).await;
            prop_assert!(result3.is_ok(), "Failed to bulk delete users: {result3:?}");


            let result4 = client.list_users().await;
            prop_assert!(result4.is_ok(), "Failed to list users after bulk delete: {result4:?}");

            let users_after_delete = result4.unwrap();
            for username in &usernames {
                let found_user = users_after_delete.iter().any(|u| u.name == *username);
                prop_assert!(!found_user, "list_users still includes deleted user {}", username);
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_user_list_consistency(
        usernames in prop::collection::vec(arb_username(), 1..3)
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            for username in &usernames {
                let _ = client.delete_user(username, true).await;
            }

            for username in &usernames {
                let salt = password_hashing::salt();
                let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "testpassword");
                let params = UserParams {
                    name: username,
                    password_hash: &password_hash,
                    tags: "administrator",
                };

                let result1 = client.create_user(&params).await;
                prop_assert!(result1.is_ok(), "Failed to create user {}: {result1:?}", username);
            }


            let result2 = client.list_users().await;
            prop_assert!(result2.is_ok(), "Failed to list users: {result2:?}");

            let users = result2.unwrap();

            for username in &usernames {
                let found_user = users.iter().any(|u| u.name == *username);
                prop_assert!(found_user, "list_users did not include the created user {}", username);
            }

            for username in &usernames {
                let _ = client.delete_user(username, true).await;
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_current_user_info(
        _dummy_input in Just(1u8)
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let result1 = client.current_user().await;
            prop_assert!(result1.is_ok(), "Failed to get current user info: {result1:?}");

            let current_user = result1.unwrap();
            prop_assert_eq!(current_user.name, USERNAME.to_string());

            Ok(())
        })?;
    }
}
