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

use crate::test_helpers::{PASSWORD, USERNAME, async_await_metric_emission, endpoint};
use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rabbitmq_http_client::{
    api::Client,
    commons::{ExchangeType, PolicyTarget, QueueType},
    password_hashing,
    requests::{
        ExchangeParams, Permissions, PolicyParams, QueueParams, UserParams, VirtualHostParams,
    },
};
use serde_json::{Map, json};
use tokio::runtime::Runtime;

fn arb_vhost_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.definitions\.vhosts\.[a-zA-Z0-9_-]{8,20}")
        .unwrap()
}

fn arb_username() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.definitions\.users\.[a-zA-Z0-9_-]{8,20}")
        .unwrap()
}

fn arb_password() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9!@#$%^&*()_+-=]{12,32}").unwrap()
}

fn arb_queue_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.definitions\.queues\.[a-zA-Z0-9_-]{8,20}")
        .unwrap()
}

fn arb_exchange_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(
        r"rust\.tests\.proptest\.definitions\.exchanges\.[a-zA-Z0-9_-]{8,20}",
    )
    .unwrap()
}

fn arb_policy_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.definitions\.policies\.[a-zA-Z0-9_-]{8,20}")
        .unwrap()
}

fn arb_exchange_type() -> impl Strategy<Value = ExchangeType> {
    prop_oneof![
        Just(ExchangeType::Direct),
        Just(ExchangeType::Fanout),
        Just(ExchangeType::Topic),
        Just(ExchangeType::Headers),
    ]
}

fn arb_user_tags() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("".to_string()),
        Just("administrator".to_string()),
        Just("monitoring".to_string()),
        Just("management".to_string()),
    ]
}

fn arb_queue_type() -> impl Strategy<Value = QueueType> {
    prop_oneof![
        Just(QueueType::Classic),
        Just(QueueType::Quorum),
        Just(QueueType::Stream),
    ]
}

fn arb_policy_target() -> impl Strategy<Value = PolicyTarget> {
    prop_oneof![
        Just(PolicyTarget::Queues),
        Just(PolicyTarget::ClassicQueues),
        Just(PolicyTarget::QuorumQueues),
        Just(PolicyTarget::Streams),
        Just(PolicyTarget::Exchanges),
        Just(PolicyTarget::All),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(8))]

    #[test]
    fn prop_async_export_import_cluster_definitions_roundtrip(
        vhost_name in arb_vhost_name(),
        queue_name in arb_queue_name(),
        exchange_name in arb_exchange_name(),
        exchange_type in arb_exchange_type(),
        queue_type in arb_queue_type()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_vhost(&vhost_name, true).await;

            let vh_params = VirtualHostParams::named(&vhost_name);
            let create_vhost_result = client.create_vhost(&vh_params).await;
            prop_assert!(create_vhost_result.is_ok(), "Failed to create a virtual host: {create_vhost_result:?}");

            let xp = ExchangeParams::new(&exchange_name, exchange_type, true, false, None);
            let declare_exchange_result = client.declare_exchange(&vhost_name, &xp).await;
            prop_assert!(declare_exchange_result.is_ok(), "Failed to declare an exchange: {declare_exchange_result:?}");

            let mut queue_args = Map::new();
            queue_args.insert("x-queue-type".to_string(), json!(queue_type.to_string()));
            let qp = QueueParams::new(&queue_name, queue_type, true, false, Some(queue_args));
            let declare_queue_result = client.declare_queue(&vhost_name, &qp).await;
            prop_assert!(declare_queue_result.is_ok(), "Failed to declare a queue: {declare_queue_result:?}");

            async_await_metric_emission(20).await;

            let bind_result = client.bind_queue(&vhost_name, &queue_name, &exchange_name, None, None).await;
            prop_assert!(bind_result.is_ok(), "Failed to bind a queue: {bind_result:?}");

            async_await_metric_emission(100).await;

            let export_result = client.export_cluster_wide_definitions_as_data().await;
            prop_assert!(export_result.is_ok(), "Failed to export the definitions: {export_result:?}");

            let exported_defs = export_result.unwrap();

            let vhost_found = exported_defs.virtual_hosts.iter().any(|v| v.name == vhost_name);
            prop_assert!(vhost_found, "Virtual host not found in exported definitions");

            let queue_found = exported_defs.queues.iter().any(|q| q.name == queue_name && q.vhost == vhost_name);
            prop_assert!(queue_found, "Exported definitions do not contain an expected queue");

            let exchange_found = exported_defs.exchanges.iter().any(|x| x.name == exchange_name && x.vhost == vhost_name);
            prop_assert!(exchange_found, "Exported definitions do not contain an expected exchange");

            let binding_found = exported_defs.bindings.iter().any(|b|
                b.destination == queue_name &&
                b.source == exchange_name &&
                b.vhost == vhost_name
            );
            prop_assert!(binding_found, "Binding not found in exported definitions");

            let _ = client.delete_vhost(&vhost_name, true).await;

            let import_vhost_result = client.create_vhost(&vh_params).await;
            prop_assert!(import_vhost_result.is_ok(), "Failed to recreate a virtual host for import: {import_vhost_result:?}");

            let vhost_def = json!({
                "queues": exported_defs.queues.iter()
                    .filter(|q| q.vhost == vhost_name)
                    .map(|q| json!({
                        "name": q.name,
                        "durable": q.durable,
                        "auto_delete": q.auto_delete,
                        "arguments": q.arguments
                    }))
                    .collect::<Vec<_>>(),
                "exchanges": exported_defs.exchanges.iter()
                    .filter(|x| x.vhost == vhost_name && !x.name.starts_with("amq."))
                    .map(|x| json!({
                        "name": x.name,
                        "type": x.exchange_type,
                        "durable": x.durable,
                        "auto_delete": x.auto_delete,
                        "arguments": x.arguments
                    }))
                    .collect::<Vec<_>>(),
                "bindings": exported_defs.bindings.iter()
                    .filter(|b| b.vhost == vhost_name)
                    .map(|b| json!({
                        "source": b.source,
                        "destination": b.destination,
                        "destination_type": b.destination_type,
                        "routing_key": b.routing_key,
                        "arguments": b.arguments
                    }))
                    .collect::<Vec<_>>()
            });

            let import_result = client.import_vhost_definitions(&vhost_name, vhost_def).await;
            prop_assert!(import_result.is_ok(), "Failed to import the virtual host definitions: {import_result:?}");

            async_await_metric_emission(100).await;

            let get_queue_result = client.get_queue_info(&vhost_name, &queue_name).await;
            prop_assert!(get_queue_result.is_ok(), "Imported queue was not found: {get_queue_result:?}");

            let get_exchange_result = client.get_exchange_info(&vhost_name, &exchange_name).await;
            prop_assert!(get_exchange_result.is_ok(), "Imported exchange was not found: {get_exchange_result:?}");

            let _ = client.delete_vhost(&vhost_name, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_export_import_vhost_definitions_with_users_and_permissions(
        vhost_name in arb_vhost_name(),
        username in arb_username(),
        password in arb_password(),
        user_tags in arb_user_tags()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_user(&username, true).await;
            let _ = client.delete_vhost(&vhost_name, true).await;

            let vh_params = VirtualHostParams::named(&vhost_name);
            let create_vhost_result = client.create_vhost(&vh_params).await;
            prop_assert!(create_vhost_result.is_ok(), "Failed to create a virtual host: {create_vhost_result:?}");

            let salt = password_hashing::salt();
            let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &password);
            let user_params = UserParams {
                name: &username,
                password_hash: &password_hash,
                tags: &user_tags,
            };

            let create_user_result = client.create_user(&user_params).await;
            prop_assert!(create_user_result.is_ok(), "Failed to create a user: {create_user_result:?}");

            let permissions = Permissions {
                vhost: &vhost_name,
                user: &username,
                configure: ".*",
                write: ".*",
                read: ".*",
            };
            let set_permissions_result = client.declare_permissions(&permissions).await;
            prop_assert!(set_permissions_result.is_ok(), "Failed to set the permissions: {set_permissions_result:?}");

            let export_result = client.export_cluster_wide_definitions_as_data().await;
            prop_assert!(export_result.is_ok(), "Failed to export the definitions: {export_result:?}");

            let exported_defs = export_result.unwrap();

            let user_found = exported_defs.users.iter().any(|u| u.name == username);
            prop_assert!(user_found, "User not found in exported definitions");

            let permission_found = exported_defs.permissions.iter().any(|p|
                p.user == username && p.vhost == vhost_name
            );
            prop_assert!(permission_found, "Permission not found in exported definitions");

            let _ = client.delete_user(&username, true).await;
            let _ = client.delete_vhost(&vhost_name, true).await;

            let cluster_def = json!({
                "users": exported_defs.users.iter()
                    .filter(|u| u.name == username)
                    .map(|u| json!({
                        "name": u.name,
                        "password_hash": u.password_hash,
                        "tags": u.tags
                    }))
                    .collect::<Vec<_>>(),
                "vhosts": [json!({
                    "name": vhost_name
                })],
                "permissions": exported_defs.permissions.iter()
                    .filter(|p| p.user == username && p.vhost == vhost_name)
                    .map(|p| json!({
                        "user": p.user,
                        "vhost": p.vhost,
                        "configure": p.configure,
                        "write": p.write,
                        "read": p.read
                    }))
                    .collect::<Vec<_>>()
            });

            let import_result = client.import_cluster_wide_definitions(cluster_def).await;
            prop_assert!(import_result.is_ok(), "Failed to import the cluster definitions: {import_result:?}");

            let get_user_result = client.get_user(&username).await;
            prop_assert!(get_user_result.is_ok(), "Imported user was not found: {get_user_result:?}");

            let get_vhost_result = client.get_vhost(&vhost_name).await;
            prop_assert!(get_vhost_result.is_ok(), "Imported vhost was not found: {get_vhost_result:?}");

            let get_permissions_result = client.get_permissions(&vhost_name, &username).await;
            prop_assert!(get_permissions_result.is_ok(), "Imported permissions were not found: {get_permissions_result:?}");

            let _ = client.delete_user(&username, true).await;
            let _ = client.delete_vhost(&vhost_name, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_export_import_definitions_with_policies(
        vhost_name in arb_vhost_name(),
        policy_name in arb_policy_name(),
        queue_name in arb_queue_name(),
        policy_target in arb_policy_target()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_vhost(&vhost_name, true).await;

            let vh_params = VirtualHostParams::named(&vhost_name);
            let create_vhost_result = client.create_vhost(&vh_params).await;
            prop_assert!(create_vhost_result.is_ok(), "Failed to create a virtual host: {create_vhost_result:?}");

            let mut policy_def = Map::new();
            policy_def.insert("max-length".to_string(), json!(1000));
            policy_def.insert("message-ttl".to_string(), json!(60000));

            let policy_params = PolicyParams {
                vhost: &vhost_name,
                name: &policy_name,
                pattern: &format!("{}.*", queue_name),
                apply_to: policy_target,
                priority: 1,
                definition: policy_def,
            };

            let declare_policy_result = client.declare_policy(&policy_params).await;
            prop_assert!(declare_policy_result.is_ok(), "Failed to declare a policy: {declare_policy_result:?}");

            let export_result = client.export_vhost_definitions_as_data(&vhost_name).await;
            prop_assert!(export_result.is_ok(), "Failed to export definitions of a specific virtual host: {export_result:?}");

            let exported_defs = export_result.unwrap();

            let policy_found = exported_defs.policies.iter().any(|p| p.name == policy_name);
            prop_assert!(policy_found, "Policy not found in exported definitions");

            let _ = client.delete_policy(&vhost_name, &policy_name).await;

            let vhost_def = json!({
                "policies": exported_defs.policies.iter()
                    .filter(|p| p.name == policy_name)
                    .map(|p| json!({
                        "name": p.name,
                        "pattern": p.pattern,
                        "apply-to": p.apply_to,
                        "priority": p.priority,
                        "definition": p.definition
                    }))
                    .collect::<Vec<_>>()
            });

            let import_result = client.import_vhost_definitions(&vhost_name, vhost_def).await;
            prop_assert!(import_result.is_ok(), "Failed to import the virtual host definitions: {import_result:?}");

            let get_policy_result = client.get_policy(&vhost_name, &policy_name).await;
            prop_assert!(get_policy_result.is_ok(), "Imported policy was not found: {get_policy_result:?}");

            let _ = client.delete_vhost(&vhost_name, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_definition_consistency_across_formats(
        vhost_name in arb_vhost_name()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_vhost(&vhost_name, true).await;

            let vh_params = VirtualHostParams::named(&vhost_name);
            let create_vhost_result = client.create_vhost(&vh_params).await;
            prop_assert!(create_vhost_result.is_ok(), "Failed to create a virtual host: {create_vhost_result:?}");

            let export_string_result = client.export_vhost_definitions_as_string(&vhost_name).await;
            prop_assert!(export_string_result.is_ok(), "Failed to export as a string: {export_string_result:?}");

            let export_data_result = client.export_vhost_definitions_as_data(&vhost_name).await;
            prop_assert!(export_data_result.is_ok(), "Failed to export as data structure: {export_data_result:?}");

            let string_def = export_string_result.unwrap();
            let data_def = export_data_result.unwrap();

            prop_assert!(!string_def.is_empty(), "String export should not be empty");

            let parsed_string: serde_json::Value = serde_json::from_str(&string_def)
                .expect("Failed to parse exported string as JSON");

            let data_as_json = serde_json::to_value(&data_def)
                .expect("Failed to convert data to JSON");

            if let (Some(string_exchanges), Some(data_exchanges)) = (
                parsed_string.get("exchanges").and_then(|v| v.as_array()),
                data_as_json.get("exchanges").and_then(|v| v.as_array())
            ) {
                prop_assert_eq!(string_exchanges.len(), data_exchanges.len(),
                    "Exchange count mismatch between string and data exports");
            }

            let _ = client.delete_vhost(&vhost_name, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_policy_target_conversion_and_matching(
        vhost_name in arb_vhost_name(),
        policy_name in arb_policy_name(),
        queue_name in arb_queue_name(),
        exchange_name in arb_exchange_name(),
        policy_target in arb_policy_target(),
        queue_type in arb_queue_type()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_vhost(&vhost_name, true).await;

            let vh_params = VirtualHostParams::named(&vhost_name);
            let create_vhost_result = client.create_vhost(&vh_params).await;
            prop_assert!(create_vhost_result.is_ok(), "Failed to create a virtual host: {create_vhost_result:?}");

            let mut policy_def = Map::new();
            policy_def.insert("max-length".to_string(), json!(1000));

            let pattern = "rust\\.tests\\.proptest\\.definitions\\..*";
            let policy_params = PolicyParams {
                vhost: &vhost_name,
                name: &policy_name,
                pattern,
                apply_to: policy_target.clone(),
                priority: 1,
                definition: policy_def,
            };

            let declare_policy_result = client.declare_policy(&policy_params).await;
            prop_assert!(declare_policy_result.is_ok(), "Failed to declare a policy: {declare_policy_result:?}");

            let get_policy_result = client.get_policy(&vhost_name, &policy_name).await;
            prop_assert!(get_policy_result.is_ok(), "Failed to get policy: {get_policy_result:?}");

            let retrieved_policy = get_policy_result.unwrap();
            prop_assert_eq!(&retrieved_policy.apply_to, &policy_target, "Policy target mismatch after roundtrip");
            prop_assert_eq!(&retrieved_policy.name, &policy_name, "Policy name mismatch");
            prop_assert_eq!(&retrieved_policy.vhost, &vhost_name, "Policy vhost mismatch");

            let policy_target_from_queue_type = PolicyTarget::from(queue_type.clone());
            let queue_type_matches = match (policy_target.clone(), queue_type.clone()) {
                (PolicyTarget::ClassicQueues, QueueType::Classic) => true,
                (PolicyTarget::QuorumQueues, QueueType::Quorum) => true,
                (PolicyTarget::Streams, QueueType::Stream) => true,
                (PolicyTarget::Queues, QueueType::Classic | QueueType::Quorum | QueueType::Stream) => true,
                (PolicyTarget::All, _) => true,
                _ => false,
            };

            if queue_type_matches {
                let mut queue_args = Map::new();
                queue_args.insert("x-queue-type".to_string(), json!(queue_type.to_string()));
                let qp = QueueParams::new(&queue_name, queue_type, true, false, Some(queue_args));
                let declare_queue_result = client.declare_queue(&vhost_name, &qp).await;
                prop_assert!(declare_queue_result.is_ok(), "Failed to declare a queue: {declare_queue_result:?}");

                async_await_metric_emission(20).await;

                let list_result = client.list_queues_in(&vhost_name).await;
                prop_assert!(list_result.is_ok(), "Failed to list queues: {list_result:?}");

                let queues = list_result.unwrap();
                let found_queue = queues.iter().find(|q| q.name == queue_name);
                prop_assert!(found_queue.is_some(), "Declared queue was not found in list");

                let queue = found_queue.unwrap();
                prop_assert!(retrieved_policy.does_match_name(&vhost_name, &queue.name, policy_target_from_queue_type),
                    "Policy should match the created queue based on policy target");
            }

            if policy_target == PolicyTarget::Exchanges || policy_target == PolicyTarget::All {
                let xp = ExchangeParams::durable_fanout(&exchange_name, None);
                let declare_exchange_result = client.declare_exchange(&vhost_name, &xp).await;
                prop_assert!(declare_exchange_result.is_ok(), "Failed to declare an exchange: {declare_exchange_result:?}");

                prop_assert!(retrieved_policy.does_match_name(&vhost_name, &exchange_name, PolicyTarget::Exchanges),
                    "Policy should match the created exchange when targeting exchanges");
            }

            let _ = client.delete_vhost(&vhost_name, true).await;

            Ok(())
        })?;
    }
}
