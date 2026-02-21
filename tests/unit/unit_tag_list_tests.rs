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

use rabbitmq_http_client::responses::TagList;

#[test]
fn test_administrator_implies_all_access() {
    let tags = TagList::from(vec!["administrator".to_string()]);
    assert!(tags.can_access_http_api());
    assert!(tags.is_administrator());
    assert!(tags.can_access_monitoring_endpoints());
}

#[test]
fn test_monitoring_has_monitoring_and_management_access() {
    let tags = TagList::from(vec!["monitoring".to_string()]);
    assert!(tags.can_access_http_api());
    assert!(!tags.is_administrator());
    assert!(tags.can_access_monitoring_endpoints());
}

#[test]
fn test_policymaker_has_management_access_only() {
    let tags = TagList::from(vec!["policymaker".to_string()]);
    assert!(tags.can_access_http_api());
    assert!(!tags.is_administrator());
    assert!(!tags.can_access_monitoring_endpoints());
}

#[test]
fn test_management_has_management_access_only() {
    let tags = TagList::from(vec!["management".to_string()]);
    assert!(tags.can_access_http_api());
    assert!(!tags.is_administrator());
    assert!(!tags.can_access_monitoring_endpoints());
}

#[test]
fn test_empty_tags_have_no_access() {
    let tags = TagList::from(vec![]);
    assert!(!tags.can_access_http_api());
    assert!(!tags.is_administrator());
    assert!(!tags.can_access_monitoring_endpoints());
}

#[test]
fn test_custom_tag_has_no_access() {
    let tags = TagList::from(vec!["custom_app_tag".to_string()]);
    assert!(!tags.can_access_http_api());
    assert!(!tags.is_administrator());
    assert!(!tags.can_access_monitoring_endpoints());
}

#[test]
fn test_multiple_tags_combine() {
    let tags = TagList::from(vec!["custom_tag".to_string(), "monitoring".to_string()]);
    assert!(tags.can_access_http_api());
    assert!(tags.can_access_monitoring_endpoints());
    assert!(!tags.is_administrator());
}

#[test]
fn test_from_vec_string() {
    let tags = TagList::from(vec!["management".to_string()]);
    assert_eq!(tags.len(), 1);
    assert!(tags.contains("management"));
}
