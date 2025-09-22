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
    blocking_api::Client,
    password_hashing,
    requests::{Permissions, UserParams, VirtualHostParams},
};
use std::cmp::max;

fn arb_username() -> impl Strategy<Value = String> {
    prop::string::string_regex(
        r"rust\.tests\.blocking\.proptest\.permissions\.users\.[a-zA-Z0-9_-]{8,20}",
    )
    .unwrap()
}

fn arb_vhost_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(
        r"rust\.tests\.blocking\.proptest\.permissions\.vhosts\.[a-zA-Z0-9_-]{8,20}",
    )
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
    fn prop_blocking_basic_permission_access(
        (users, vhost_name, (configure, read, write)) in (arb_user_list(), arb_vhost_name(), arb_permission_patterns())
    ) {
        let endpoint = endpoint();
        let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

        let _ = admin_client.delete_vhost(&vhost_name, true);
        for (username, _) in &users {
            let _ = admin_client.delete_user(username, true);
        }

        // Create virtual host
        let vh_params = VirtualHostParams::named(&vhost_name);
        let result1 = admin_client.create_vhost(&vh_params);
        prop_assert!(result1.is_ok(), "Failed to create a virtual host: {result1:?}");

        let _ = admin_client.clear_permissions(&vhost_name, "guest", true);

        for (username, password) in &users {
            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, password);
            let user_params = UserParams {
                name: username,
                password_hash: &password_hash,
                tags: "management", // Required for HTTP API access
            };

            let result2 = admin_client.create_user(&user_params);
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

            let result3 = admin_client.grant_permissions(&permission_params);
            prop_assert!(result3.is_ok(), "Failed to grant permissions to a user {}: {result3:?}", username);
        }

        // Verify permissions were granted correctly
        for (username, _) in users_with_perms {
            let result4 = admin_client.get_permissions(&vhost_name, username);
            prop_assert!(result4.is_ok(), "Failed to get permissions for a user {}: {result4:?}", username);

            let perms = result4.unwrap();
            prop_assert_eq!(&perms.user, username);
            prop_assert_eq!(&perms.vhost, &vhost_name);
            prop_assert_eq!(&perms.configure, configure);
            prop_assert_eq!(&perms.read, read);
            prop_assert_eq!(&perms.write, write);
        }

        // Verify users without permissions don't have access to the vhost
        for (username, _) in users_without_perms {
            let result4 = admin_client.get_permissions(&vhost_name, username);
            prop_assert!(result4.is_err(), "User {} should not have permissions in vhost {}", username, vhost_name);
        }

        // Test listing permissions in the virtual host
        let result5 = admin_client.list_permissions_in(&vhost_name);
        prop_assert!(result5.is_ok(), "Failed to list permissions in a vhost: {result5:?}");

        let vhost_perms = result5.unwrap();
        prop_assert_eq!(vhost_perms.len(), users_with_perms.len(), "Number of permissions should match users with permissions");

        // Test listing permissions for individual users
        for (username, _) in users_with_perms {
            let user_perms_result = admin_client.list_permissions_of(username);
            prop_assert!(user_perms_result.is_ok(), "Failed to list permissions for a user {}: {user_perms_result:?}", username);

            let user_perms = user_perms_result.unwrap();
            let found_perm = user_perms.iter().find(|p| p.vhost == vhost_name);
            prop_assert!(found_perm.is_some(), "User {} should have permission entry for vhost {}", username, vhost_name);
        }

        for (username, _) in users_with_perms {
            let clear_perms_result = admin_client.clear_permissions(&vhost_name, username, true);
            prop_assert!(clear_perms_result.is_ok(), "Failed to clear permissions for a user {}: {clear_perms_result:?}", username);
        }

        for (username, _) in &users {
            let _ = admin_client.delete_user(username, true);
        }
        let _ = admin_client.delete_vhost(&vhost_name, true);
    }

    #[test]
    fn prop_blocking_permission_access_with_multiple_vhosts(
        (users, vhost_names) in (arb_user_list(), prop::collection::vec(arb_vhost_name(), 2..4))
    ) {
        let endpoint = endpoint();
        let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

        for vhost_name in &vhost_names {
            let _ = admin_client.delete_vhost(vhost_name, true);
        }
        for (username, _) in &users {
            let _ = admin_client.delete_user(username, true);
        }

        // Create virtual hosts
        for vhost_name in &vhost_names {
            let vh_params = VirtualHostParams::named(vhost_name);
            let result1 = admin_client.create_vhost(&vh_params);
            prop_assert!(result1.is_ok(), "Failed to create a virtual host {}: {result1:?}", vhost_name);

        let _ = admin_client.clear_permissions(vhost_name, "guest", true);
        }

        for (username, password) in &users {
            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, password);
            let user_params = UserParams {
                name: username,
                password_hash: &password_hash,
                tags: "management",
            };

            let result2 = admin_client.create_user(&user_params);
            prop_assert!(result2.is_ok(), "Failed to create a user {}: {result2:?}", username);
        }

        // Grant each user permissions to different vhosts
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

            let result3 = admin_client.grant_permissions(&permission_params);
            prop_assert!(result3.is_ok(), "Failed to grant permissions to a user {} in vhost {}: {result3:?}", username, vhost_name);
        }

        // Verify users have permissions only in their assigned vhosts
        for (i, (username, _)) in users.iter().enumerate() {
            let assigned_vhost_index = i % vhost_names.len();

            for (j, vhost_name) in vhost_names.iter().enumerate() {
                let result4 = admin_client.get_permissions(vhost_name, username);

                if j == assigned_vhost_index {
                    prop_assert!(result4.is_ok(), "User {} should have permissions in assigned vhost {}", username, vhost_name);
                } else {
                    prop_assert!(result4.is_err(), "User {} should not have permissions in non-assigned vhost {}", username, vhost_name);
                }
            }
        }

        for (username, _) in &users {
            for vhost_name in &vhost_names {
                let _ = admin_client.clear_permissions(vhost_name, username, true);
            }
            let _ = admin_client.delete_user(username, true);
        }
        for vhost_name in &vhost_names {
            let _ = admin_client.delete_vhost(vhost_name, true);
        }
    }

    #[test]
    fn prop_blocking_http_api_access_with_different_user_tags(
        (users, vhost_name, user_tags) in (arb_user_list(), arb_vhost_name(), prop::collection::vec(arb_user_tags(), 2..5))
    ) {
        let endpoint = endpoint();
        let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

        let _ = admin_client.delete_vhost(&vhost_name, true);
        for (username, _) in &users {
            let _ = admin_client.delete_user(username, true);
        }

        // Create virtual host
        let vh_params = VirtualHostParams::named(&vhost_name);
        let result1 = admin_client.create_vhost(&vh_params);
        prop_assert!(result1.is_ok(), "Failed to create a virtual host: {result1:?}");

        let _ = admin_client.clear_permissions(&vhost_name, "guest", true);

        for ((username, password), tags) in users.iter().zip(user_tags.iter()) {
            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, password);
            let user_params = UserParams {
                name: username,
                password_hash: &password_hash,
                tags: &tags,
            };

            let result2 = admin_client.create_user(&user_params);
            prop_assert!(result2.is_ok(), "Failed to create a user {} with tags '{}': {result2:?}", username, tags);

            let permission_params = Permissions {
                user: username,
                vhost: &vhost_name,
                configure: ".*",
                read: ".*",
                write: ".*",
            };

            let result3 = admin_client.grant_permissions(&permission_params);
            prop_assert!(result3.is_ok(), "Failed to grant permissions to a user {}: {result3:?}", username);
        }

        // Test HTTP API access for each user based on their tags
        for ((username, password), tags) in users.iter().zip(user_tags.iter()) {
            let client = Client::new(&endpoint, username, password);

            let should_have_access = tags.contains("management") ||
                                   tags.contains("administrator") ||
                                   tags.contains("policymaker") ||
                                   tags.contains("monitoring");

            let should_have_admin_access = tags.contains("administrator");

            let list_queues_result = client.list_queues_in(&vhost_name);

            if should_have_access {
                prop_assert!(list_queues_result.is_ok(),
                           "User {} with tags '{}' should have HTTP API access to list queues: {list_queues_result:?}",
                           username, tags);
            } else {
                prop_assert!(list_queues_result.is_err(),
                           "User {} with tags '{}' should NOT have HTTP API access to list queues",
                           username, tags);
            }

            let get_vhost_result = client.get_vhost(&vhost_name);

            if should_have_admin_access {
                prop_assert!(get_vhost_result.is_ok(),
                           "User {} with tags '{}' should have administrator access to get vhost info: {get_vhost_result:?}",
                           username, tags);
            } else {
                prop_assert!(get_vhost_result.is_err(),
                           "User {} with tags '{}' should NOT have administrator access to get vhost info",
                           username, tags);
            }

            let list_exchanges_result = client.list_exchanges_in(&vhost_name);

            if should_have_access {
                prop_assert!(list_exchanges_result.is_ok(),
                           "User {} with tags '{}' should have HTTP API access to list exchanges: {list_exchanges_result:?}",
                           username, tags);
            } else {
                prop_assert!(list_exchanges_result.is_err(),
                           "User {} with tags '{}' should NOT have HTTP API access to list exchanges",
                           username, tags);
            }
        }

        for (username, _) in &users {
            let _ = admin_client.clear_permissions(&vhost_name, username, true);
            let _ = admin_client.delete_user(username, true);
        }
        let _ = admin_client.delete_vhost(&vhost_name, true);
    }

    #[test]
    fn prop_blocking_user_tag_enforcement(
        (username, password, base_tags) in (arb_username(), arb_password(), arb_user_tags())
    ) {
        let endpoint = endpoint();
        let admin_client = Client::new(&endpoint, USERNAME, PASSWORD);

        let _ = admin_client.delete_user(&username, true);

        let salt = password_hashing::salt();
        let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &password);
        let user_params = UserParams {
            name: &username,
            password_hash: &password_hash,
            tags: &base_tags,
        };

        let result2 = admin_client.create_user(&user_params);
        prop_assert!(result2.is_ok(), "Failed to create a user with tags '{}': {result2:?}", base_tags);

        let client = Client::new(&endpoint, &username, &password);

        let should_have_access = base_tags.contains("management") ||
                               base_tags.contains("administrator") ||
                               base_tags.contains("policymaker") ||
                               base_tags.contains("monitoring");

        let current_user_result = client.current_user();

        if should_have_access {
            prop_assert!(current_user_result.is_ok(),
                       "User {} with tags '{}' should have HTTP API access to get current user info: {current_user_result:?}",
                       username, base_tags);

            let current_user = current_user_result.unwrap();
            prop_assert_eq!(&current_user.name, &username);
        } else {
            prop_assert!(current_user_result.is_err(),
                       "User {} with tags '{}' should NOT have HTTP API access to get current user info",
                       username, base_tags);
        }

        let list_vhosts_result = client.list_vhosts();

        if should_have_access {
            prop_assert!(list_vhosts_result.is_ok(),
                       "User {} with tags '{}' should have HTTP API access to list vhosts: {list_vhosts_result:?}",
                       username, base_tags);
        } else {
            prop_assert!(list_vhosts_result.is_err(),
                       "User {} with tags '{}' should NOT have HTTP API access to list vhosts",
                       username, base_tags);
        }

        let _ = admin_client.delete_user(&username, true);
    }
}
