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

use rabbitmq_http_client::responses::{
    NamedPolicyTargetObject, OptionalArgumentSourceOps, QueueDefinition, QueueOps,
};

#[test]
fn test_unit_queue_definition_without_quorum_queue_incompatible_keys_case1() {
    let input = r#"{
        "name": "cq.1",
        "vhost": "/",
        "durable": true,
        "auto_delete": false,
        "arguments": {
          "x-queue-type": "quorum",
          "x-queue-mode": "lazy",
          "x-max-priority": 11
        }
    }"#;
    let k1 = "x-queue-type".to_owned();
    let k2 = "x-queue-mode".to_owned();
    let k3 = "x-max-priority".to_owned();

    let def1 = serde_json::from_str::<QueueDefinition>(input).unwrap();

    let def2 = def1.without_quorum_queue_incompatible_keys();

    assert!(def1.arguments.contains_key(&k1));
    assert!(def1.arguments.contains_key(&k2));
    assert!(def1.arguments.contains_key(&k3));
    
    assert!(def2.arguments.contains_key(&k1));
    assert!(!def2.arguments.contains_key(&k2));
    assert!(!def2.arguments.contains_key(&k3));
}
