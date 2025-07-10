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
use rabbitmq_http_client::responses::{ClusterDefinitionSet, PolicyDefinitionAndXArgumentsOps};
use rabbitmq_http_client::transformers::{StripCmqKeysFromPolicies, TransformationChain};
use serde_json::json;

#[test]
fn test_unit_strip_cmq_policies_case1() {
    let json = r#"
        {
          "rabbit_version": "3.13.6",
          "rabbitmq_version": "3.13.6",
          "product_name": "RabbitMQ",
          "product_version": "3.13.6",
          "users": [
            {
              "name": "guest",
              "password_hash": "CZGtMFp48hNvlKFZqF/4gKu/i3cMtoSkmgsGTHP07Yi8mCkY",
              "hashing_algorithm": "rabbit_password_hashing_sha256",
              "tags": [
                "administrator"
              ],
              "limits": {}
            }
          ],
          "vhosts": [
            {
              "name": "has_cmq_policies",
              "description": "",
              "tags": [],
              "default_queue_type": "classic",
              "metadata": {
                "description": "",
                "tags": [],
                "default_queue_type": "classic"
              }
            },
            {
              "name": "/",
              "description": "Default virtual host",
              "tags": [],
              "metadata": {
                "description": "Default virtual host",
                "tags": []
              }
            }
          ],
          "permissions": [
            {
              "user": "guest",
              "vhost": "/",
              "configure": ".*",
              "write": ".*",
              "read": ".*"
            },
            {
              "user": "guest",
              "vhost": "has_cmq_policies",
              "configure": ".*",
              "write": ".*",
              "read": ".*"
            }
          ],
          "topic_permissions": [],
          "parameters": [],
          "global_parameters": [
            {
              "name": "internal_cluster_id",
              "value": "rabbitmq-cluster-id-jOfSvUg-P_mpfytlmrwRiA"
            }
          ],
          "policies": [
            {
              "vhost": "has_cmq_policies",
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
              "vhost": "has_cmq_policies",
              "name": "streams",
              "pattern": "^streams",
              "apply-to": "streams",
              "definition": {
                "max-age": "1D"
              },
              "priority": 0
            },
            {
              "vhost": "has_cmq_policies",
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
              "vhost": "has_cmq_policies",
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
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "streams.s1",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "stream"
              }
            },
            {
              "name": "group1.cq.2",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group1.qq.2",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "quorum"
              }
            },
            {
              "name": "group3.cq.2",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group1.qq.1",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "quorum"
              }
            },
            {
              "name": "group2.cq.2",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group2.cq.1",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            },
            {
              "name": "group1.cq.1",
              "vhost": "has_cmq_policies",
              "durable": true,
              "auto_delete": false,
              "arguments": {
                "x-queue-type": "classic"
              }
            }
          ],
          "exchanges": [],
          "bindings": []
        }
    "#;

    let vh = "has_cmq_policies";
    let g1 = "cmq_group1";
    let cq1 = "group1.cq.1";

    let mut defs0: ClusterDefinitionSet = serde_json::from_str(json).unwrap();
    let p0 = defs0.find_policy(vh, g1).unwrap();
    assert_eq!(4, p0.definition.len());
    assert!(p0.definition.has_cmq_keys());

    let chain = TransformationChain {
        chain: vec![Box::new(StripCmqKeysFromPolicies::default())],
    };
    let defs1 = chain.apply(&mut defs0);

    // Three CMQ-related queues were stripped
    let p1 = defs1.find_policy(vh, g1).unwrap();
    assert_eq!(1, p1.definition.len());
    assert!(!p1.definition.has_cmq_keys());
    assert_eq!(
        "queue-version".to_owned(),
        p1.definition_keys().first().unwrap().to_owned()
    );

    let q1 = defs1.find_queue(vh, cq1).unwrap();
    assert_eq!(
        json!(QueueType::Quorum),
        q1.arguments
            .get(X_ARGUMENT_KEY_X_QUEUE_TYPE)
            .unwrap()
            .clone()
    );
}
