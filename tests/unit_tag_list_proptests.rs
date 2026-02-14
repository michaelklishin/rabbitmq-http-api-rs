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
use rabbitmq_http_client::responses::TagList;

fn arb_management_tag() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("management".to_string()),
        Just("monitoring".to_string()),
        Just("policymaker".to_string()),
        Just("administrator".to_string()),
    ]
}

fn arb_non_management_tag() -> impl Strategy<Value = String> {
    "[a-z_]{1,20}".prop_filter("must not be a management tag", |s| {
        !matches!(
            s.as_str(),
            "management" | "monitoring" | "policymaker" | "administrator"
        )
    })
}

proptest! {
    #[test]
    fn prop_any_management_tag_grants_api_access(tag in arb_management_tag()) {
        let tags = TagList::from(vec![tag]);
        prop_assert!(tags.can_access_http_api());
    }

    #[test]
    fn prop_non_management_tags_never_grant_api_access(tag in arb_non_management_tag()) {
        let tags = TagList::from(vec![tag]);
        prop_assert!(!tags.can_access_http_api());
    }

    #[test]
    fn prop_only_administrator_is_administrator(tag in arb_management_tag()) {
        let tags = TagList::from(vec![tag.clone()]);
        prop_assert_eq!(tags.is_administrator(), tag == "administrator");
    }

    #[test]
    fn prop_monitoring_endpoints_require_monitoring_or_admin(tag in arb_management_tag()) {
        let tags = TagList::from(vec![tag.clone()]);
        let expected = tag == "monitoring" || tag == "administrator";
        prop_assert_eq!(tags.can_access_monitoring_endpoints(), expected);
    }

    #[test]
    fn prop_adding_extra_tags_does_not_remove_access(
        management_tag in arb_management_tag(),
        extra_tag in arb_non_management_tag()
    ) {
        let tags = TagList::from(vec![management_tag, extra_tag]);
        prop_assert!(tags.can_access_http_api());
    }
}
