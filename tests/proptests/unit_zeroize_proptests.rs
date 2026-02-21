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

use proptest::prelude::*;
use proptest::test_runner::Config as ProptestConfig;
use rabbitmq_http_client::commons::Password;
use rabbitmq_http_client::requests::users::OwnedUserParams;
use rabbitmq_http_client::requests::users::UserParams;
use rabbitmq_http_client::responses::TagList;
use rabbitmq_http_client::responses::users::User;
use zeroize::Zeroize;

fn arb_name() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9_.-]{1,64}").unwrap()
}

fn arb_password_hash() -> impl Strategy<Value = String> {
    prop::string::string_regex(r"[a-zA-Z0-9+/=]{8,128}").unwrap()
}

fn arb_tag() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("administrator".to_string()),
        Just("monitoring".to_string()),
        Just("management".to_string()),
        Just("policymaker".to_string()),
        Just("".to_string()),
    ]
}

fn arb_tag_list() -> impl Strategy<Value = Vec<String>> {
    prop::collection::vec(arb_tag(), 0..=5)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn prop_owned_user_params_zeroize_clears_all_fields(
        name in arb_name(),
        hash in arb_password_hash(),
        tag in arb_tag()
    ) {
        let mut params = OwnedUserParams::new(name.clone(), hash.clone(), tag.clone());

        prop_assert_eq!(&params.name, &name);
        prop_assert_eq!(&params.password_hash, &hash);
        prop_assert_eq!(&params.tags, &tag);

        params.zeroize();

        prop_assert!(params.name.is_empty());
        prop_assert!(params.password_hash.is_empty());
        prop_assert!(params.tags.is_empty());
    }

    #[test]
    fn prop_user_response_zeroize_clears_all_fields(
        name in arb_name(),
        hash in arb_password_hash(),
        tags in arb_tag_list()
    ) {
        let mut user = User {
            name: name.clone(),
            tags: TagList(tags.clone()),
            password_hash: hash.clone(),
        };

        prop_assert_eq!(&user.name, &name);
        prop_assert_eq!(&user.password_hash, &hash);

        user.zeroize();

        prop_assert!(user.name.is_empty());
        prop_assert!(user.password_hash.is_empty());
        prop_assert!(user.tags.is_empty());
    }

    #[test]
    fn prop_tag_list_zeroize_clears_all_entries(tags in arb_tag_list()) {
        let mut tag_list = TagList(tags);
        tag_list.zeroize();
        prop_assert!(tag_list.is_empty());
    }

    #[test]
    fn prop_zeroize_is_idempotent_on_owned_user_params(
        name in arb_name(),
        hash in arb_password_hash(),
        tag in arb_tag()
    ) {
        let mut params = OwnedUserParams::new(name, hash, tag);
        params.zeroize();
        params.zeroize();

        prop_assert!(params.name.is_empty());
        prop_assert!(params.password_hash.is_empty());
        prop_assert!(params.tags.is_empty());
    }

    #[test]
    fn prop_zeroize_is_idempotent_on_user(
        name in arb_name(),
        hash in arb_password_hash(),
        tags in arb_tag_list()
    ) {
        let mut user = User {
            name,
            tags: TagList(tags),
            password_hash: hash,
        };
        user.zeroize();
        user.zeroize();

        prop_assert!(user.name.is_empty());
        prop_assert!(user.password_hash.is_empty());
        prop_assert!(user.tags.is_empty());
    }

    #[test]
    fn prop_clone_then_zeroize_leaves_clone_intact(
        name in arb_name(),
        hash in arb_password_hash(),
        tag in arb_tag()
    ) {
        let mut original = OwnedUserParams::new(name.clone(), hash.clone(), tag.clone());
        let cloned = original.clone();

        original.zeroize();

        prop_assert!(original.name.is_empty());
        prop_assert!(original.password_hash.is_empty());
        prop_assert_eq!(&cloned.name, &name);
        prop_assert_eq!(&cloned.password_hash, &hash);
        prop_assert_eq!(&cloned.tags, &tag);
    }

    #[test]
    fn prop_user_clone_then_zeroize_leaves_clone_intact(
        name in arb_name(),
        hash in arb_password_hash(),
        tags in arb_tag_list()
    ) {
        let mut original = User {
            name: name.clone(),
            tags: TagList(tags.clone()),
            password_hash: hash.clone(),
        };
        let cloned = original.clone();

        original.zeroize();

        prop_assert!(original.name.is_empty());
        prop_assert!(original.password_hash.is_empty());
        prop_assert_eq!(&cloned.name, &name);
        prop_assert_eq!(&cloned.password_hash, &hash);
    }

    #[test]
    fn prop_owned_user_params_from_user_params_then_zeroize(
        name in arb_name(),
        hash in arb_password_hash(),
        tag in arb_tag()
    ) {
        let borrowed = UserParams::new(&name, &hash, &tag);
        let mut owned = OwnedUserParams::from(borrowed);

        prop_assert_eq!(&owned.name, &name);
        prop_assert_eq!(&owned.password_hash, &hash);

        owned.zeroize();

        prop_assert!(owned.name.is_empty());
        prop_assert!(owned.password_hash.is_empty());
        prop_assert!(owned.tags.is_empty());
    }

    #[test]
    fn prop_password_display_is_transparent(
        value in prop::string::string_regex(r"[a-zA-Z0-9!@#$%^&*()]{1,64}").unwrap()
    ) {
        let pw = Password::new(value.clone());
        prop_assert_eq!(format!("{}", pw), value);
    }

    #[test]
    fn prop_password_zeroize_clears_value(
        value in prop::string::string_regex(r"[a-zA-Z0-9!@#$%^&*()]{1,64}").unwrap()
    ) {
        let mut pw = Password::new(value);
        pw.zeroize();
        prop_assert_eq!(format!("{}", pw), "");
    }

    #[test]
    fn prop_password_clone_then_zeroize_leaves_clone_intact(
        value in prop::string::string_regex(r"[a-zA-Z0-9!@#$%^&*()]{1,64}").unwrap()
    ) {
        let mut original = Password::new(value.clone());
        let cloned = original.clone();

        original.zeroize();

        prop_assert_eq!(format!("{}", original), "");
        prop_assert_eq!(format!("{}", cloned), value);
    }

    #[test]
    fn prop_password_from_string_matches_from_str(
        value in prop::string::string_regex(r"[a-zA-Z0-9]{1,32}").unwrap()
    ) {
        let from_str = Password::from(value.as_str());
        let from_string = Password::from(value.clone());

        prop_assert_eq!(format!("{}", from_str), format!("{}", from_string));
    }
}
