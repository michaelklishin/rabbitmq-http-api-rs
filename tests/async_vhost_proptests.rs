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
use rabbitmq_http_client::{api::Client, commons::QueueType, requests::VirtualHostParams};
use tokio::runtime::Runtime;

fn arb_vhost_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"rust\.tests\.proptest\.vhosts\.[a-zA-Z0-9_-]{8,20}").unwrap()
}

fn arb_vhost_description() -> impl Strategy<Value = Option<String>> {
    prop_oneof![
        Just(None),
        prop::string::string_regex(r"[a-zA-Z0-9 _.-]{10,50}")
            .unwrap()
            .prop_map(Some),
    ]
}

fn arb_vhost_tags() -> impl Strategy<Value = Option<Vec<String>>> {
    prop_oneof![
        Just(None),
        prop::collection::vec(
            prop::string::string_regex(r"[a-zA-Z0-9_-]{3,15}").unwrap(),
            0..3
        )
        .prop_map(Some),
    ]
}

fn arb_default_queue_type() -> impl Strategy<Value = Option<QueueType>> {
    prop_oneof![
        Just(None),
        Just(Some(QueueType::Classic)),
        Just(Some(QueueType::Quorum)),
    ]
}

fn arb_vh_params() -> impl Strategy<
    Value = (
        String,
        Option<String>,
        Option<Vec<String>>,
        Option<QueueType>,
        bool,
    ),
> {
    (
        arb_vhost_name(),
        arb_vhost_description(),
        arb_vhost_tags(),
        arb_default_queue_type(),
        any::<bool>(), // tracing
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10))]

    #[test]
    fn prop_async_vhost_create_list_delete(
        (name, description, tags, default_queue_type, tracing) in arb_vh_params()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_vhost(&name, true).await;

            let tags_ref: Option<Vec<&str>> = tags.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());
            let params = VirtualHostParams {
                name: &name,
                description: description.as_deref(),
                tags: tags_ref,
                default_queue_type,
                tracing,
            };

            let result1 = client.create_vhost(&params).await;
            prop_assert!(result1.is_ok(), "Failed to create a virtual host: {result1:?}");


            let result2 = client.list_vhosts().await;
            prop_assert!(result2.is_ok(), "Failed to list virtual hosts: {result2:?}");

            let vhosts = result2.unwrap();
            let found_vhost = vhosts.iter().find(|v| v.name == name);
            prop_assert!(found_vhost.is_some(), "list_vhosts did not include the created vhost: {}", name);

            let vhost = found_vhost.unwrap();
            prop_assert_eq!(&vhost.name, &name);

            let result3 = client.get_vhost(&name).await;
            prop_assert!(result3.is_ok(), "Failed to get virtual host info: {result3:?}");

            let vhost_info = result3.unwrap();
            prop_assert_eq!(&vhost_info.name, &name);

            if let Some(ref desc) = description {
                prop_assert_eq!(vhost_info.description.as_ref(), Some(desc));
            }

            let result4 = client.delete_vhost(&name, false).await;
            prop_assert!(result4.is_ok(), "Failed to delete a virtual host: {result4:?}");

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_vhost_update_operations(
        (name, initial_desc, updated_desc) in (
            arb_vhost_name(),
            arb_vhost_description(),
            arb_vhost_description()
        )
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_vhost(&name, true).await;

            let initial_params = VirtualHostParams {
                name: &name,
                description: initial_desc.as_deref(),
                tags: None,
                default_queue_type: None,
                tracing: false,
            };

            let result1 = client.create_vhost(&initial_params).await;
            prop_assert!(result1.is_ok(), "Failed to create initial vhost: {result1:?}");


            let updated_params = VirtualHostParams {
                name: &name,
                description: updated_desc.as_deref(),
                tags: Some(vec!["updated", "test"]),
                default_queue_type: Some(QueueType::Quorum),
                tracing: true,
            };

            let result2 = client.update_vhost(&updated_params).await;
            prop_assert!(result2.is_ok(), "Failed to update a virtual host: {result2:?}");


            let result3 = client.get_vhost(&name).await;
            prop_assert!(result3.is_ok(), "Failed to get Update virtual host info: {result3:?}");

            let updated_vhost = result3.unwrap();
            if let Some(ref desc) = updated_desc {
                prop_assert_eq!(updated_vhost.description.as_ref(), Some(desc));
            }

            if let Some(ref tags) = updated_vhost.tags {
                let tag_strings: Vec<String> = tags.iter().cloned().collect();
                prop_assert!(tag_strings.contains(&"updated".to_string()), "The newly added virtual host tag wasn't found");
                prop_assert!(tag_strings.contains(&"test".to_string()), "The newly added virtual host tag wasn't found");
            } else {
                prop_assert!(false, "Update virtual host should have tags but none were found");
            }

            let _ = client.delete_vhost(&name, true).await;

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_vhost_deletion_protection(
        name in arb_vhost_name()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let _ = client.delete_vhost(&name, true).await;

            let params = VirtualHostParams::named(&name);
            let result1 = client.create_vhost(&params).await;
            prop_assert!(result1.is_ok(), "Failed to create a virtual host: {result1:?}");


            let result2 = client.enable_vhost_deletion_protection(&name).await;
            prop_assert!(result2.is_ok(), "Failed to enable deletion protection: {result2:?}");

            let result3 = client.delete_vhost(&name, false).await;
            prop_assert!(result3.is_err(), "Attempts at deleting a protected virtual host should fail");

            let result4 = client.disable_vhost_deletion_protection(&name).await;
            prop_assert!(result4.is_ok(), "Failed to disable deletion protection: {result4:?}");

            let result5 = client.delete_vhost(&name, false).await;
            prop_assert!(result5.is_ok(), "Failed to delete a virtual host after disabling protection: {result5:?}");

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_vhost_idempotent_delete(
        name in arb_vhost_name()
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            let result1 = client.delete_vhost(&name, true).await;
            prop_assert!(result1.is_ok(), "Idempotent delete of non-existent vhost should succeed: {result1:?}");

            let params = VirtualHostParams::named(&name);
            let result2 = client.create_vhost(&params).await;
            prop_assert!(result2.is_ok(), "Failed to create a virtual host: {result2:?}");


            let result3 = client.delete_vhost(&name, true).await;
            prop_assert!(result3.is_ok(), "Failed to delete existing vhost: {result3:?}");

            let result4 = client.delete_vhost(&name, true).await;
            prop_assert!(result4.is_ok(), "Second idempotent delete should succeed: {result4:?}");

            Ok(())
        })?;
    }

    #[test]
    fn prop_async_vhost_list_consistency(
        names in prop::collection::vec(arb_vhost_name(), 1..3)
    ) {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let endpoint = endpoint();
            let client = Client::new(&endpoint, USERNAME, PASSWORD);

            for name in &names {
                let _ = client.delete_vhost(name, true).await;
            }

            for name in &names {
                let params = VirtualHostParams::named(name);
                let result1 = client.create_vhost(&params).await;
                prop_assert!(result1.is_ok(), "Failed to create a virtual host {}: {result1:?}", name);
            }


            let result2 = client.list_vhosts().await;
            prop_assert!(result2.is_ok(), "Failed to list virtual hosts: {result2:?}");

            let vhosts = result2.unwrap();

            for name in &names {
                let found_vhost = vhosts.iter().any(|v| v.name == *name);
                prop_assert!(found_vhost, "list_vhosts did not include the created vhost {}", name);
            }

            for name in &names {
                let _ = client.delete_vhost(name, true).await;
            }

            Ok(())
        })?;
    }
}
