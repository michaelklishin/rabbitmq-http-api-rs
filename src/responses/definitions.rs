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

use crate::commons::{
    BindingDestinationType, OverflowBehavior, PolicyTarget, QueueType, VirtualHostName,
    X_ARGUMENT_KEY_X_OVERFLOW, X_ARGUMENT_KEY_X_QUEUE_TYPE,
};
#[cfg(feature = "tabled")]
use crate::formatting::{display_arg_table, display_option};
use crate::responses::policies::{Policy, PolicyWithoutVirtualHost};
use crate::transformers::{TransformerFn, TransformerFnOnce};
use serde::{Deserialize, Serialize};
use serde_json::{Map, json};
use std::ops::{Deref, DerefMut};
#[cfg(feature = "tabled")]
use tabled::Tabled;

pub trait QueueOps {
    /// Returns the name of the object.
    fn name(&self) -> &str;

    /// Returns the [`QueueType`] applicable to the implementation.
    fn queue_type(&self) -> QueueType;

    /// Returns the policy target kind matching the queue type.
    fn policy_target_type(&self) -> PolicyTarget;

    /// Returns the x-arguments of this object.
    fn x_arguments(&self) -> &XArguments;

    /// Returns true if the queue is server-named.
    /// See the [Queues guide](https://www.rabbitmq.com/docs/queues#server-named-queues) to learn more.
    fn is_server_named(&self) -> bool {
        let name = self.name();
        name.is_empty() || name.starts_with("amq.")
    }
}

pub trait OptionalArgumentSourceOps {
    fn contains_any_keys_of(&self, keys: Vec<&str>) -> bool;

    fn has_cmq_keys(&self) -> bool;

    fn has_quorum_queue_incompatible_keys(&self) -> bool;

    fn is_empty(&self) -> bool;

    fn without_keys(&self, keys: Vec<&str>) -> Self;

    fn without_cmq_keys(&self) -> Self;

    fn without_quorum_queue_incompatible_keys(&self) -> Self;
}

/// Represents an object a policy can match: a queue, a stream, an exchange.
pub trait NamedPolicyTargetObject {
    fn vhost(&self) -> String;
    fn name(&self) -> String;
    fn policy_target(&self) -> PolicyTarget;
    fn does_match(&self, policy: &Policy) -> bool;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct XArguments(pub Map<String, serde_json::Value>);

impl XArguments {
    pub const CMQ_KEYS: [&'static str; 6] = [
        "x-ha-mode",
        "x-ha-params",
        "x-ha-promote-on-shutdown",
        "x-ha-promote-on-failure",
        "x-ha-sync-mode",
        "x-ha-sync-batch-size",
    ];
    pub const QUORUM_QUEUE_INCOMPATIBLE_KEYS: [&'static str; 8] = [
        "x-ha-mode",
        "x-ha-params",
        "x-ha-promote-on-shutdown",
        "x-ha-promote-on-failure",
        "x-ha-sync-mode",
        "x-ha-sync-batch-size",
        "x-queue-mode",
        "x-max-priority",
    ];
    pub const X_EXPIRES_KEY: &'static str = "x-expires";
    pub const X_MESSAGE_TTL_KEY: &'static str = "x-message-ttl";
    pub const X_MAX_LENGTH_KEY: &'static str = "x-max-length";
    pub const X_MAX_LENGTH_BYTES_KEY: &'static str = "x-max-length-bytes";

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.get(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn keys(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, key: String, value: serde_json::Value) -> Option<serde_json::Value> {
        self.0.insert(key, value)
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.0.remove(key)
    }

    pub fn merge(&mut self, other: &Self) {
        let mut m: Map<String, serde_json::Value> = self.0.clone();
        m.extend(other.0.clone());

        self.0 = m;
    }
}

impl Deref for XArguments {
    type Target = Map<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for XArguments {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct QueueDefinition {
    pub name: String,
    pub vhost: VirtualHostName,
    pub durable: bool,
    pub auto_delete: bool,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub arguments: XArguments,
}

impl NamedPolicyTargetObject for QueueDefinition {
    fn vhost(&self) -> String {
        self.vhost.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn policy_target(&self) -> PolicyTarget {
        self.policy_target_type()
    }

    fn does_match(&self, policy: &Policy) -> bool {
        policy.does_match_object(self)
    }
}

impl QueueOps for QueueDefinition {
    fn name(&self) -> &str {
        &self.name
    }

    fn queue_type(&self) -> QueueType {
        if let Some((_, val)) = self.arguments.0.get_key_value(X_ARGUMENT_KEY_X_QUEUE_TYPE) {
            val.as_str()
                .map(QueueType::from)
                .unwrap_or(QueueType::default())
        } else {
            QueueType::default()
        }
    }

    fn policy_target_type(&self) -> PolicyTarget {
        PolicyTarget::from(self.queue_type())
    }

    fn x_arguments(&self) -> &XArguments {
        &self.arguments
    }
}

impl OptionalArgumentSourceOps for QueueDefinition {
    fn contains_any_keys_of(&self, keys: Vec<&str>) -> bool {
        self.arguments
            .keys()
            .iter()
            .any(|key| keys.contains(&key.as_str()))
    }

    fn has_cmq_keys(&self) -> bool {
        self.contains_any_keys_of(XArguments::CMQ_KEYS.to_vec())
    }

    fn has_quorum_queue_incompatible_keys(&self) -> bool {
        self.contains_any_keys_of(XArguments::QUORUM_QUEUE_INCOMPATIBLE_KEYS.to_vec())
    }

    fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    fn without_keys(&self, keys: Vec<&str>) -> Self {
        let mut new_args = self.arguments.clone();
        for key in keys {
            new_args.0.remove(key);
        }
        let mut copy = self.clone();
        copy.arguments = new_args;
        copy
    }

    fn without_cmq_keys(&self) -> Self {
        self.without_keys(XArguments::CMQ_KEYS.to_vec())
    }

    fn without_quorum_queue_incompatible_keys(&self) -> Self {
        self.without_keys(XArguments::QUORUM_QUEUE_INCOMPATIBLE_KEYS.to_vec())
    }
}

impl QueueDefinition {
    pub fn update_queue_type(&mut self, typ: QueueType) -> &mut Self {
        self.arguments.remove(X_ARGUMENT_KEY_X_QUEUE_TYPE);
        self.arguments
            .insert(X_ARGUMENT_KEY_X_QUEUE_TYPE.to_owned(), json!(typ));

        self
    }

    pub fn compare_and_swap_string_argument(
        &mut self,
        argument: &str,
        value: &str,
        new_value: &str,
    ) -> &mut Self {
        if let Some(val) = self.arguments.get(argument)
            && let Some(s) = val.as_str()
            && s == value
        {
            self.arguments.insert(argument.to_owned(), json!(new_value));
        }

        self
    }

    pub fn compare_and_swap_overflow_argument(
        &mut self,
        value: OverflowBehavior,
        new_value: OverflowBehavior,
    ) -> &mut Self {
        self.compare_and_swap_string_argument(
            X_ARGUMENT_KEY_X_OVERFLOW,
            value.into(),
            new_value.into(),
        )
    }
}

/// Used in virtual host-specific definitions.
/// The virtual host is omitted so that such objects can
/// be imported into an arbitrary virtual host.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct QueueDefinitionWithoutVirtualHost {
    pub name: String,
    pub durable: bool,
    pub auto_delete: bool,
    #[cfg_attr(feature = "tabled", tabled(skip))]
    pub arguments: XArguments,
}

impl QueueDefinitionWithoutVirtualHost {
    pub fn update_queue_type(&mut self, typ: QueueType) -> &mut Self {
        self.arguments.remove(X_ARGUMENT_KEY_X_QUEUE_TYPE);
        self.arguments
            .insert(X_ARGUMENT_KEY_X_QUEUE_TYPE.to_owned(), json!(typ));

        self
    }
}

impl QueueOps for QueueDefinitionWithoutVirtualHost {
    fn name(&self) -> &str {
        &self.name
    }

    fn queue_type(&self) -> QueueType {
        if let Some((_, val)) = self.arguments.0.get_key_value(X_ARGUMENT_KEY_X_QUEUE_TYPE) {
            val.as_str()
                .map(QueueType::from)
                .unwrap_or(QueueType::default())
        } else {
            QueueType::default()
        }
    }

    fn policy_target_type(&self) -> PolicyTarget {
        PolicyTarget::from(self.queue_type())
    }

    fn x_arguments(&self) -> &XArguments {
        &self.arguments
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct ExchangeInfo {
    pub name: String,
    pub vhost: VirtualHostName,
    #[serde(rename = "type")]
    pub exchange_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    #[cfg_attr(feature = "tabled", tabled(display = "display_arg_table"))]
    pub arguments: XArguments,
}

pub type ExchangeDefinition = ExchangeInfo;

impl NamedPolicyTargetObject for ExchangeDefinition {
    fn vhost(&self) -> String {
        self.vhost.clone()
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn policy_target(&self) -> PolicyTarget {
        PolicyTarget::Exchanges
    }

    fn does_match(&self, policy: &Policy) -> bool {
        policy.does_match_object(self)
    }
}

/// Used in virtual host-specific definitions.
/// The virtual host is omitted so that such objects can
/// be imported into an arbitrary virtual host.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct ExchangeInfoWithoutVirtualHost {
    pub name: String,
    #[serde(rename = "type")]
    pub exchange_type: String,
    pub durable: bool,
    pub auto_delete: bool,
    #[cfg_attr(feature = "tabled", tabled(display = "display_arg_table"))]
    pub arguments: XArguments,
}

pub type ExchangeDefinitionWithoutVirtualHost = ExchangeInfoWithoutVirtualHost;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct BindingInfo {
    pub vhost: VirtualHostName,
    pub source: String,
    pub destination: String,
    pub destination_type: BindingDestinationType,
    pub routing_key: String,
    #[cfg_attr(feature = "tabled", tabled(display = "display_arg_table"))]
    pub arguments: XArguments,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub properties_key: Option<String>,
}

pub type BindingDefinition = BindingInfo;

/// Used in virtual host-specific definitions.
/// The virtual host is omitted so that such objects can
/// be imported into an arbitrary virtual host.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct BindingInfoWithoutVirtualHost {
    pub source: String,
    pub destination: String,
    pub destination_type: BindingDestinationType,
    pub routing_key: String,
    #[cfg_attr(feature = "tabled", tabled(display = "display_arg_table"))]
    pub arguments: XArguments,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub properties_key: Option<String>,
}

pub type BindingDefinitionWithoutVirtualHost = BindingInfoWithoutVirtualHost;

/// Represents definitions of an entire cluster (all virtual hosts).
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct ClusterDefinitionSet {
    #[serde(rename(deserialize = "rabbitmq_version"))]
    pub server_version: Option<String>,
    pub users: Vec<crate::responses::User>,
    #[serde(rename(deserialize = "vhosts"))]
    pub virtual_hosts: Vec<crate::responses::VirtualHost>,
    pub permissions: Vec<crate::responses::Permissions>,

    pub parameters: Vec<crate::responses::RuntimeParameter>,
    pub policies: Vec<Policy>,

    pub queues: Vec<QueueDefinition>,
    pub exchanges: Vec<ExchangeDefinition>,
    pub bindings: Vec<BindingDefinition>,
}

impl ClusterDefinitionSet {
    pub fn find_policy(&self, vhost: &str, name: &str) -> Option<&Policy> {
        self.policies
            .iter()
            .find(|&p| p.vhost == *vhost && p.name == *name)
    }

    pub fn policies_in(&self, vhost: &str) -> Option<&Policy> {
        self.policies.iter().find(|&p| p.vhost == *vhost)
    }

    pub fn find_queue(&self, vhost: &str, name: &str) -> Option<&QueueDefinition> {
        self.queues
            .iter()
            .find(|&q| q.vhost == *vhost && q.name == *name)
    }

    pub fn find_queue_mut(&mut self, vhost: &str, name: &str) -> Option<&mut QueueDefinition> {
        self.queues
            .iter_mut()
            .find(|q| q.vhost == *vhost && q.name == *name)
    }

    pub fn queues_in(&self, vhost: &str) -> Option<&QueueDefinition> {
        self.queues.iter().find(|&q| q.vhost == *vhost)
    }

    pub fn find_exchange(&self, vhost: &str, name: &str) -> Option<&ExchangeDefinition> {
        self.exchanges
            .iter()
            .find(|&x| x.vhost == *vhost && x.name == *name)
    }

    pub fn exchanges_in(&self, vhost: &str) -> Option<&ExchangeDefinition> {
        self.exchanges.iter().find(|&x| x.vhost == *vhost)
    }

    pub fn update_policies(&mut self, f: TransformerFn<Policy>) -> Vec<Policy> {
        let updated = self
            .policies
            .iter()
            .map(|p| f(p.clone()))
            .collect::<Vec<_>>();
        self.policies = updated.clone();

        updated.clone()
    }

    pub fn queues_matching(&self, policy: &Policy) -> Vec<&QueueDefinition> {
        self.queues
            .iter()
            .filter(|&qd| policy.does_match_object(qd))
            .collect()
    }

    pub fn update_queue_type_of_matching(&mut self, policy: &Policy, typ: QueueType) {
        let matches: Vec<(String, String)> = self
            .queues
            .iter()
            .filter(|&qd| policy.does_match_object(&qd.clone()))
            .map(|qd| (qd.vhost.clone(), qd.name.clone()))
            .collect();

        for (vh, qn) in matches.iter() {
            self.update_queue_type(&vh.clone(), &qn.clone(), typ.clone());
        }
    }

    pub fn update_queue_type(
        &mut self,
        vhost: &str,
        name: &str,
        typ: QueueType,
    ) -> Option<QueueDefinition> {
        if let Some(qd) = self.find_queue_mut(vhost, name) {
            let mut args = qd.arguments.clone();
            args.insert(X_ARGUMENT_KEY_X_QUEUE_TYPE.to_owned(), json!(typ.clone()));

            qd.arguments = args;

            Some(qd.clone())
        } else {
            None
        }
    }

    pub fn update_queue(
        &mut self,
        vhost: String,
        name: String,
        f: TransformerFnOnce<QueueDefinition>,
    ) -> Option<QueueDefinition> {
        if let Some(&mut qd) = self
            .queues
            .iter()
            .find(|&q| q.name == name && q.vhost == vhost)
            .as_mut()
        {
            let qd = f(qd.clone());

            Some(qd)
        } else {
            None
        }
    }

    pub fn update_queues(&mut self, f: TransformerFn<QueueDefinition>) -> Vec<QueueDefinition> {
        let updated = self.queues.iter().map(|p| f(p.clone())).collect::<Vec<_>>();
        self.queues = updated.clone();

        updated.clone()
    }
}

/// Represents definitions of a single virtual host.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct VirtualHostDefinitionSet {
    #[serde(rename(deserialize = "rabbitmq_version"))]
    pub server_version: Option<String>,
    /// All virtual host metadata combined
    pub metadata: Option<crate::responses::VirtualHostMetadata>,

    pub parameters: Vec<crate::responses::RuntimeParameterWithoutVirtualHost>,
    pub policies: Vec<PolicyWithoutVirtualHost>,

    pub queues: Vec<QueueDefinitionWithoutVirtualHost>,
    pub exchanges: Vec<ExchangeDefinitionWithoutVirtualHost>,
    pub bindings: Vec<BindingDefinitionWithoutVirtualHost>,
}
