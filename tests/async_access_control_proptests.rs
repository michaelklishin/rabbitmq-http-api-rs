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
use rabbitmq_http_client::{
    api::Client,
    password_hashing,
    requests::{Permissions, UserParams, VirtualHostParams},
};
use std::cmp::max;
use tokio::runtime::Runtime;

fn arb_username() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.permissions\.users\.[a-zA-Z0-9_-]{8,20}")
        .unwrap()
}

fn arb_vhost_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.permissions\.vhosts\.[a-zA-Z0-9_-]{8,20}")
        .unwrap()
}

fn arb_password() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9!@#$%^&*()_+-=]{12,32}").unwrap()
}

fn arb_user_list() -> impl Strategy<Value = Vec<(String, String)>> {
    prop::collection::vec((arb_username(), arb_password()), 2..5)
}

fn arb_permission_patterns() -> impl Strategy<Value = (&'static str, &'static str, &'static str)> {
    prop_oneof![
        Just((".*", ".*", ".*")),
        Just(("", "", "")),
        Just((".*", ".*", "")),
        Just(("", ".*", ".*")),
        Just(("^test.*", ".*", ".*")),
    ]
}

fn arb_user_tags() -> impl Strategy<Value = String> {
    let api_access_tags = vec!["monitoring", "management", "administrator", "policymaker"];
    let no_access_tags = vec!["custom_tag", "worker", "producer", "consumer"];

    prop_oneof![
        Just("".to_string()),
        prop::sample::select(api_access_tags.clone()).prop_map(|s| s.to_string()),
        prop::sample::select(no_access_tags.clone()).prop_map(|s| s.to_string()),
        prop::collection::vec(prop::sample::select(api_access_tags.clone()), 1..3).prop_map(
            |tags| {
                let mut unique_tags: Vec<String> =
                    tags.into_iter().map(|s| s.to_string()).collect();
                unique_tags.sort();
                unique_tags.dedup();
                unique_tags.join(",")
            }
        ),
        prop::collection::vec(prop::sample::select(no_access_tags), 1..3).prop_map(|tags| {
            let mut unique_tags: Vec<String> = tags.into_iter().map(|s| s.to_string()).collect();
            unique_tags.sort();
            unique_tags.dedup();
            unique_tags.join(",")
        }),
        prop::collection::vec(prop::sample::select(api_access_tags), 1..2).prop_flat_map(
            |api_tags| {
                prop::collection::vec(prop::sample::select(vec!["custom_tag", "worker"]), 1..2)
                    .prop_map(move |non_api_tags| {
                        let mut all_tags = api_tags.clone();
                        all_tags.extend(non_api_tags);
                        all_tags.sort();
                        all_tags.dedup();
                        all_tags.join(",")
                    })
            }
        ),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(5))]

    #[test]
    fn prop_async_basic_permission_access(
        (users, vhost_name, (configure, read, write)) in (arb_user_list(), arb_vhost_name(), arb_permission_patterns())
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

            if let Ok(existing_permissions) = admin_client.list_permissions_in(&vhost_name).await {
                for perm in existing_permissions {
                    let _ = admin_client.clear_permissions(&vhost_name, &perm.user, true).await;
                }
            }

            let _ = admin_client.delete_vhost(&vhost_name, true).await;
            for (username, _) in &users {
                let _ = admin_client.delete_user(username, true).await;
            }

            let vh_params = VirtualHostParams::named(&vhost_name);
            let result1 = admin_client.create_vhost(&vh_params).await;
            prop_assert!(result1.is_ok(), "Failed to create a virtual host: {result1:?}");

            let _ = admin_client.clear_permissions(&vhost_name, "guest", true).await;

            for (username, password) in &users {
                let salt = password_hashing::salt();
                let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, password);
                let user_params = UserParams {
                    name: username,
                    password_hash: &password_hash,
                    tags: "management",
                };

                let result2 = admin_client.create_user(&user_params).await;
                prop_assert!(result2.is_ok(), "Failed to create a user {}: {result2:?}", username);
            }

            let split_point = if users.len() == 1 { 1 } else { max(1, users.len() / 2) };
            let (users_with_perms, users_without_perms) = users.split_at(split_point);

            for (username, _) in users_with_perms {
                let permission_params = Permissions {
                    user: username,
                    vhost: &vhost_name,
                    configure,
                    read,
                    write,
                };

                let result3 = admin_client.grant_permissions(&permission_params).await;
                prop_assert!(result3.is_ok(), "Failed to grant permissions to a user {}: {result3:?}", username);
            }

            for (username, _) in users_with_perms {
                let result4 = admin_client.get_permissions(&vhost_name, username).await;
                prop_assert!(result4.is_ok(), "Failed to get permissions for a user {}: {result4:?}", username);

                let perms = result4.unwrap();
                prop_assert_eq!(&perms.user, username);
                prop_assert_eq!(&perms.vhost, &vhost_name);
                prop_assert_eq!(&perms.configure, configure);
                prop_assert_eq!(&perms.read, read);
                prop_assert_eq!(&perms.write, write);
            }

            for (username, _) in users_without_perms {
                let result4 = admin_client.get_permissions(&vhost_name, username).await;
                prop_assert!(result4.is_err(), "User {} should not have permissions in vhost {}", username, vhost_name);
            }

            let result5 = admin_client.list_permissions_in(&vhost_name).await;
            prop_assert!(result5.is_ok(), "Failed to list permissions in a vhost: {result5:?}");

            let vhost_perms = result5.unwrap();
            prop_assert_eq!(vhost_perms.len(), users_with_perms.len(), "Number of permissions should match users with permissions");

            // Test listing permissions for individual users
            for (username, _) in users_with_perms {
                let result6 = admin_client.list_permissions_of(username).await;
                prop_assert!(result6.is_ok(), "Failed to list permissions for a user {}: {result6:?}", username);

                let user_perms = result6.unwrap();
                let found_perm = user_perms.iter().find(|p| p.vhost == vhost_name);
                prop_assert!(found_perm.is_some(), "User {} should have permission entry for vhost {}", username, vhost_name);
            }

            for (username, _) in users_with_perms {
                let result7 = admin_client.clear_permissions(&vhost_name, username, true).await;
                prop_assert!(result7.is_ok(), "Failed to clear permissions for a user {}: {result7:?}", username);
            }

            for (username, _) in &users {
                let _ = admin_client.delete_user(username, true).await;
            }
            let _ = admin_client.delete_vhost(&vhost_name, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_permission_access_with_multiple_vhosts(
        (users, vhost_names) in (arb_user_list(), prop::collection::vec(arb_vhost_name(), 2..4))
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

            for vhost_name in &vhost_names {
                let _ = admin_client.delete_vhost(vhost_name, true).await;
            }
            for (username, _) in &users {
                let _ = admin_client.delete_user(username, true).await;
            }

            for vhost_name in &vhost_names {
                let vh_params = VirtualHostParams::named(vhost_name);
                let result1 = admin_client.create_vhost(&vh_params).await;
                prop_assert!(result1.is_ok(), "Failed to create a virtual host {}: {result1:?}", vhost_name);

            let _ = admin_client.clear_permissions(vhost_name, "guest", true).await;
            }

            for (username, password) in &users {
                let salt = password_hashing::salt();
                let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, password);
                let user_params = UserParams {
                    name: username,
                    password_hash: &password_hash,
                    tags: "management",
                };

                let result2 = admin_client.create_user(&user_params).await;
                prop_assert!(result2.is_ok(), "Failed to create a user {}: {result2:?}", username);
            }

            // Grant each user permission to a different set of virtual hosts
            for (i, (username, _)) in users.iter().enumerate() {
                let vhost_index = i % vhost_names.len();
                let vhost_name = &vhost_names[vhost_index];

                let permission_params = Permissions {
                    user: username,
                    vhost: vhost_name,
                    configure: ".*",
                    read: ".*",
                    write: ".*",
                };

                let result3 = admin_client.grant_permissions(&permission_params).await;
                prop_assert!(result3.is_ok(), "Failed to grant permissions to a user {} in vhost {}: {result3:?}", username, vhost_name);
            }

            // Verify users have permissions only in their assigned vhosts
            for (i, (username, _)) in users.iter().enumerate() {
                let assigned_vhost_index = i % vhost_names.len();

                for (j, vhost_name) in vhost_names.iter().enumerate() {
                    let result4 = admin_client.get_permissions(vhost_name, username).await;

                    if j == assigned_vhost_index {
                        prop_assert!(result4.is_ok(), "User {} should have permissions in assigned vhost {}", username, vhost_name);
                    } else {
                        prop_assert!(result4.is_err(), "User {} should not have permissions in non-assigned vhost {}", username, vhost_name);
                    }
                }
            }

            for (username, _) in &users {
                for vhost_name in &vhost_names {
                    let _ = admin_client.clear_permissions(vhost_name, username, true).await;
                }
                let _ = admin_client.delete_user(username, true).await;
            }
            for vhost_name in &vhost_names {
                let _ = admin_client.delete_vhost(vhost_name, true).await;
            }

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_http_api_access_with_different_user_tags(
        (users, vhost_name, user_tags) in (arb_user_list(), arb_vhost_name(), prop::collection::vec(arb_user_tags(), 2..5))
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = admin_client.delete_vhost(&vhost_name, true).await;
            for (username, _) in &users {
                let _ = admin_client.delete_user(username, true).await;
            }

            let vh_params = VirtualHostParams::named(&vhost_name);
            let result1 = admin_client.create_vhost(&vh_params).await;
            prop_assert!(result1.is_ok(), "Failed to create a virtual host: {result1:?}");

            // Create users with different tags
            for ((username, password), tags) in users.iter().zip(user_tags.iter()) {
                let salt = password_hashing::salt();
                let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, password);
                let user_params = UserParams {
                    name: username,
                    password_hash: &password_hash,
                    tags: &tags,
                };

                let result2 = admin_client.create_user(&user_params).await;
                prop_assert!(result2.is_ok(), "Failed to create a user {} with tags '{}': {result2:?}", username, tags);

                let permission_params = Permissions {
                    user: username,
                    vhost: &vhost_name,
                    configure: ".*",
                    read: ".*",
                    write: ".*",
                };

                let result3 = admin_client.grant_permissions(&permission_params).await;
                prop_assert!(result3.is_ok(), "Failed to grant permissions to a user {}: {result3:?}", username);
            }

            // Test HTTP API access for each user based on their tags
            for ((username, password), tags) in users.iter().zip(user_tags.iter()) {
                let client = Client::new(&endpoint, username, password);

                // Determine if this user should have HTTP API access
                let should_have_access = tags.contains("management") ||
                                       tags.contains("administrator") ||
                                       tags.contains("policymaker") ||
                                       tags.contains("monitoring");

                let should_have_admin_access = tags.contains("administrator");

                // Test listing queues in the virtual host (requires HTTP API access)
                let result8 = client.list_queues_in(&vhost_name).await;

                if should_have_access {
                    prop_assert!(result8.is_ok(),
                               "User {} with tags '{}' should have HTTP API access to list queues: {result8:?}",
                               username, tags);
                } else {
                    prop_assert!(result8.is_err(),
                               "User {} with tags '{}' should NOT have HTTP API access to list queues",
                               username, tags);
                }

                // Test getting virtual host info (requires administrator privileges)
                let result9 = client.get_vhost(&vhost_name).await;

                if should_have_admin_access {
                    prop_assert!(result9.is_ok(),
                               "User {} with tags '{}' should have administrator access to get vhost info: {result9:?}",
                               username, tags);
                } else {
                    prop_assert!(result9.is_err(),
                               "User {} with tags '{}' should NOT have administrator access to get vhost info",
                               username, tags);
                }

                // Test listing exchanges (requires HTTP API access)
                let result10 = client.list_exchanges_in(&vhost_name).await;

                if should_have_access {
                    prop_assert!(result10.is_ok(),
                               "User {} with tags '{}' should have HTTP API access to list exchanges: {result10:?}",
                               username, tags);
                } else {
                    prop_assert!(result10.is_err(),
                               "User {} with tags '{}' should NOT have HTTP API access to list exchanges",
                               username, tags);
                }
            }

            for (username, _) in &users {
                let _ = admin_client.clear_permissions(&vhost_name, username, true).await;
                let _ = admin_client.delete_user(username, true).await;
            }
            let _ = admin_client.delete_vhost(&vhost_name, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_user_tag_enforcement(
        (username, password, base_tags) in (arb_username(), arb_password(), arb_user_tags())
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = admin_client.delete_user(&username, true).await;

            // Create a user with the specified tags
            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &password);
            let user_params = UserParams {
                name: &username,
                password_hash: &password_hash,
                tags: &base_tags,
            };

            let result2 = admin_client.create_user(&user_params).await;
            prop_assert!(result2.is_ok(), "Failed to create a user with tags '{}': {result2:?}", base_tags);

            let client = Client::new(&endpoint, &username, &password);

            let should_have_access = base_tags.contains("management") ||
                                   base_tags.contains("administrator") ||
                                   base_tags.contains("policymaker") ||
                                   base_tags.contains("monitoring");

            let result11 = client.current_user().await;

            if should_have_access {
                prop_assert!(result11.is_ok(),
                           "User {} with tags '{}' should have HTTP API access to get current user info: {result11:?}",
                           username, base_tags);

                let current_user = result11.unwrap();
                prop_assert_eq!(&current_user.name, &username);
            } else {
                prop_assert!(result11.is_err(),
                           "User {} with tags '{}' should NOT have HTTP API access to get current user info",
                           username, base_tags);
            }

            let result12 = client.list_vhosts().await;

            if should_have_access {
                prop_assert!(result12.is_ok(),
                           "User {} with tags '{}' should have HTTP API access to list vhosts: {result12:?}",
                           username, base_tags);
            } else {
                prop_assert!(result12.is_err(),
                           "User {} with tags '{}' should NOT have HTTP API access to list vhosts",
                           username, base_tags);
            }

            let _ = admin_client.delete_user(&username, true).await;

            Ok(())
        })?;
    }
}
