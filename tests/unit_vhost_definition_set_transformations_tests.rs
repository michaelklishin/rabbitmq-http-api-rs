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

use rabbitmq_http_client::commons::{QueueType, X_ARGUMENT_KEY_X_QUEUE_TYPE};
use rabbitmq_http_client::responses::{OptionalArgumentSourceOps, VirtualHostDefinitionSet};
use rabbitmq_http_client::transformers::{
    StripCmqKeysFromVhostPolicies, VirtualHostTransformationChain,
};
use serde_json::json;

#[test]
fn test_unit_vhost_strip_cmq_policies_case1() {
    let json = r#"
        {
          "rabbit_version": "3.13.6",
          "rabbitmq_version": "3.13.6",
          "product_name": "RabbitMQ",
          "product_version": "3.13.6",
          "metadata": {
            "description": "",
            "tags": [],
            "default_queue_type": "classic"
          },
          "parameters": [],
          "policies": [
            {
              "name": "cmq_group3",
              "pattern": "^group3",
              "apply-to": "all",
              "definition": {
                "ha-mode": "all",
                "ha-promote-on-failure": "always",
                "ha-promote-on-shutdown": "always"
              },
              "priority": 0
            },
            {
              "name": "streams",
              "pattern": "^streams",
              "apply-to": "streams",
              "definition": {
                "max-age": "1D"
              },
              "priority": 0
            },
            {
              "name": "cmq_group2",
              "pattern": "^group2\\.",
              "apply-to": "classic_queues",
              "definition": {
                "ha-mode": "exactly",
                "ha-params": 2
              },
              "priority": 0
            },
            {
              "name": "cmq_group1",
              "pattern": "^group1\\.cq",
              "apply-to": "queues",
              "definition": {
                "ha-mode": "all",
                "ha-promote-on-failure": "always",
                "ha-sync-mode": "automatic",
                "queue-version": 2
              },
              "priority": 0
            }
          ],
          "queues": [
            {
              "name": "group3.cq.1",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "streams.s1",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "stream"
              }
            },
            {
              "name": "group1.cq.2",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group1.qq.2",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "quorum"
              }
            },
            {
              "name": "group3.cq.2",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group1.qq.1",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "quorum"
              }
            },
            {
              "name": "group2.cq.2",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group2.cq.1",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group1.cq.1",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            }
          ],
          "exchanges": [],
          "bindings": []
        }"#;

    let g1 = "cmq_group1";
    let cq1 = "group1.cq.1";

    let mut defs0: VirtualHostDefinitionSet = serde_json::from_str(json).unwrap();
    let p0 = defs0.policies.iter().find(|&p| p.name == g1).unwrap();
    assert_eq!(4, p0.definition.len());
    assert!(p0.definition.has_cmq_keys());

    let chain = VirtualHostTransformationChain {
        chain: vec![Box::new(StripCmqKeysFromVhostPolicies::default())],
    };
    let defs1 = chain.apply(&mut defs0);

    // Three CMQ-related keys were stripped
    let p1 = defs1.policies.iter().find(|&p| p.name == g1).unwrap();
    assert_eq!(1, p1.definition.len());
    assert!(!p1.definition.has_cmq_keys());
    assert_eq!(
        String::from("queue-version"),
        String::from(
            p1.definition
                .0
                .as_ref()
                .unwrap()
                .keys()
                .next()
                .unwrap()
                .as_str()
        )
    );

    let q1 = defs1.queues.iter().find(|&q| q.name == cq1).unwrap();
    assert_eq!(
        json!(QueueType::Quorum),
        q1.arguments
            .get(X_ARGUMENT_KEY_X_QUEUE_TYPE)
            .unwrap()
            .clone()
    );
}
