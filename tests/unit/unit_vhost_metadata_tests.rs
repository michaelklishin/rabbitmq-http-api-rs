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

use rabbitmq_http_client::responses::VirtualHostMetadata;

#[test]
fn test_unit_vhost_metadata_deserialization_with_protected_from_deletion_true() {
    let json = r#"{
        "tags": null,
        "description": "a protected vhost",
        "default_queue_type": "quorum",
        "protected_from_deletion": true
    }"#;

    let meta: VirtualHostMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.protected_from_deletion, Some(true));
}

#[test]
fn test_unit_vhost_metadata_deserialization_with_protected_from_deletion_false() {
    let json = r#"{
        "tags": null,
        "description": "a regular vhost",
        "default_queue_type": "quorum",
        "protected_from_deletion": false
    }"#;

    let meta: VirtualHostMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.protected_from_deletion, Some(false));
}

#[test]
fn test_unit_vhost_metadata_deserialization_without_protected_from_deletion_field() {
    let json = r#"{
        "tags": null,
        "description": "an older RabbitMQ node without this field",
        "default_queue_type": "quorum"
    }"#;

    let meta: VirtualHostMetadata = serde_json::from_str(json).unwrap();
    assert_eq!(meta.protected_from_deletion, None);
}
