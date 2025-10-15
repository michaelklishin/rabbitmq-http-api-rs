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

use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::commons::{PolicyTarget, VirtualHostName};
use crate::formatting::*;
use crate::responses::definitions::{
    NamedPolicyTargetObject, OptionalArgumentSourceOps, XArguments,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, json};

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PolicyDefinition(pub Option<Map<String, serde_json::Value>>);

impl Deref for PolicyDefinition {
    type Target = Option<Map<String, serde_json::Value>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PolicyDefinition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl PolicyDefinition {
    pub const CMQ_KEYS: [&'static str; 6] = [
        "ha-mode",
        "ha-params",
        "ha-promote-on-shutdown",
        "ha-promote-on-failure",
        "ha-sync-mode",
        "ha-sync-batch-size",
    ];
    pub const QUORUM_QUEUE_INCOMPATIBLE_KEYS: [&'static str; 7] = [
        "ha-mode",
        "ha-params",
        "ha-promote-on-shutdown",
        "ha-promote-on-failure",
        "ha-sync-mode",
        "ha-sync-batch-size",
        "queue-mode",
    ];

    pub fn len(&self) -> usize {
        match &self.0 {
            Some(m) => m.len(),
            None => 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_ref().is_none_or(|m| m.is_empty())
    }

    pub fn get(&self, key: &str) -> Option<&serde_json::Value> {
        self.0.as_ref()?.get(key)
    }

    pub fn insert(&mut self, key: String, value: serde_json::Value) -> Option<serde_json::Value> {
        match self.0 {
            Some(ref mut m) => m.insert(key, value),
            None => {
                let mut m = Map::new();
                m.insert(key, value);
                self.0 = Some(m);
                None
            }
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.as_ref().is_some_and(|m| m.contains_key(key))
    }

    pub fn remove(&mut self, key: &str) -> Option<serde_json::Value> {
        self.0.as_mut()?.remove(key)
    }

    pub fn merge(&mut self, other: &PolicyDefinition) {
        match (&mut self.0, &other.0) {
            (Some(m), Some(other_m)) => {
                m.extend(other_m.clone());
            }
            (None, Some(other_m)) => {
                self.0 = Some(other_m.clone());
            }
            _ => {}
        }
    }

    pub fn compare_and_swap_string_argument(
        &mut self,
        argument: &str,
        value: &str,
        new_value: &str,
    ) -> &mut Self {
        if let Some(m) = &self.0
            && let Some(raw_val) = m.get(argument)
            && let Some(s) = raw_val.as_str()
            && s == value
        {
            self.insert(argument.to_owned(), json!(new_value));
        }

        self
    }
}

impl OptionalArgumentSourceOps for PolicyDefinition {
    fn has_cmq_keys(&self) -> bool {
        self.contains_any_keys_of(&Self::CMQ_KEYS)
    }

    fn has_quorum_queue_incompatible_keys(&self) -> bool {
        self.contains_any_keys_of(&Self::QUORUM_QUEUE_INCOMPATIBLE_KEYS)
    }

    fn contains_any_keys_of(&self, keys: &[&str]) -> bool {
        self.0
            .as_ref()
            .is_some_and(|map| map.keys().any(|key| keys.contains(&key.as_str())))
    }

    fn is_empty(&self) -> bool {
        PolicyDefinition::is_empty(self)
    }

    fn without_keys(&self, keys: &[&str]) -> Self {
        match &self.0 {
            Some(m) => {
                let mut nm = m.clone();
                for s in keys {
                    let _ = nm.remove(*s);
                }

                PolicyDefinition(Some(nm))
            }
            None => PolicyDefinition(None),
        }
    }

    fn without_cmq_keys(&self) -> Self {
        self.without_keys(&PolicyDefinition::CMQ_KEYS)
    }

    fn without_quorum_queue_incompatible_keys(&self) -> Self {
        self.without_keys(&PolicyDefinition::QUORUM_QUEUE_INCOMPATIBLE_KEYS)
    }
}

impl fmt::Display for PolicyDefinition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(val) => fmt_map_as_colon_separated_pairs(f, val),
            None => Ok(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct Policy {
    pub name: String,
    pub vhost: VirtualHostName,
    pub pattern: String,
    #[serde(rename(deserialize = "apply-to"))]
    pub apply_to: PolicyTarget,
    pub priority: i16,
    pub definition: PolicyDefinition,
}

impl Policy {
    pub fn insert_definition_key(
        &mut self,
        key: String,
        value: serde_json::Value,
    ) -> Option<serde_json::Value> {
        self.definition.insert(key, value)
    }

    pub fn definition_keys(&self) -> Vec<&str> {
        match &self.definition.0 {
            Some(m) => m.keys().map(AsRef::as_ref).collect(),
            None => vec![],
        }
    }

    pub fn priority(&self) -> &i16 {
        &self.priority
    }

    pub fn vhost(&self) -> &VirtualHostName {
        &self.vhost
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn does_match_object<T>(&self, obj: &T) -> bool
    where
        T: NamedPolicyTargetObject,
    {
        let target = obj.policy_target();

        if !self.apply_to.does_apply_to(target) {
            return false;
        }

        if self.vhost != obj.vhost() {
            return false;
        }

        let re = match regex::Regex::new(&self.pattern) {
            Ok(re) => re,
            Err(_) => return false,
        };

        re.is_match(&obj.name())
    }

    pub fn with_overrides(&self, name: &str, priority: i16, overrides: &PolicyDefinition) -> Self {
        let mut copy = self.clone();
        copy.name = name.to_owned();
        copy.priority = priority;
        copy.definition.merge(overrides);

        copy
    }

    pub fn is_a_name_match(
        pattern: &str,
        apply_to: PolicyTarget,
        name: &str,
        typ: PolicyTarget,
    ) -> bool {
        let matches_kind = apply_to.does_apply_to(typ);

        if let Ok(regex) = regex::Regex::new(pattern) {
            regex.is_match(name) && matches_kind
        } else {
            false
        }
    }

    pub fn does_match_name(&self, vhost: &str, name: &str, typ: PolicyTarget) -> bool {
        Policy::is_a_full_match(&self.vhost, &self.pattern, self.apply_to, vhost, name, typ)
    }

    pub fn is_a_full_match(
        vhost_a: &str,
        pattern: &str,
        apply_to: PolicyTarget,
        vhost_b: &str,
        name: &str,
        typ: PolicyTarget,
    ) -> bool {
        let same_vhost = vhost_a == vhost_b;
        let matches_kind = apply_to.does_apply_to(typ);

        if let Ok(regex) = regex::Regex::new(pattern) {
            regex.is_match(name) && matches_kind && same_vhost
        } else {
            false
        }
    }

    pub fn does_match(&self, predicate: fn(&Policy) -> bool) -> bool {
        predicate(self)
    }

    pub fn contains_any_keys_of(&self, keys: &[&str]) -> bool {
        self.definition.contains_any_keys_of(keys)
    }

    pub fn has_cmq_keys(&self) -> bool {
        self.definition.has_cmq_keys()
    }

    pub fn has_quorum_queue_incompatible_keys(&self) -> bool {
        self.definition.has_quorum_queue_incompatible_keys()
    }

    pub fn is_empty(&self) -> bool {
        self.definition.is_empty()
    }

    pub fn without_keys(&self, keys: &[&str]) -> Self {
        let defs = self.definition.without_keys(keys);

        let mut copy = self.clone();
        copy.definition = defs;

        copy
    }

    pub fn without_cmq_keys(&self) -> Self {
        self.without_keys(&PolicyDefinition::CMQ_KEYS)
    }

    pub fn without_quorum_queue_incompatible_keys(&self) -> Self {
        self.without_keys(&XArguments::QUORUM_QUEUE_INCOMPATIBLE_KEYS)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct PolicyWithoutVirtualHost {
    pub name: String,
    pub pattern: String,
    #[serde(rename(deserialize = "apply-to"))]
    pub apply_to: PolicyTarget,
    pub priority: i16,
    pub definition: PolicyDefinition,
}

impl PolicyWithoutVirtualHost {
    pub fn does_match(&self, name: &str, typ: PolicyTarget) -> bool {
        Policy::is_a_name_match(&self.pattern, self.apply_to, name, typ)
    }

    pub fn contains_any_keys_of(&self, keys: &[&str]) -> bool {
        self.definition.contains_any_keys_of(keys)
    }

    pub fn has_cmq_keys(&self) -> bool {
        self.definition.has_cmq_keys()
    }

    pub fn has_quorum_queue_incompatible_keys(&self) -> bool {
        self.definition.has_quorum_queue_incompatible_keys()
    }

    pub fn is_empty(&self) -> bool {
        self.definition.is_empty()
    }

    pub fn without_keys(&self, keys: &[&str]) -> Self {
        let defs = self.definition.without_keys(keys);

        let mut copy = self.clone();
        copy.definition = defs;

        copy
    }

    pub fn without_cmq_keys(&self) -> Self {
        self.without_keys(&PolicyDefinition::CMQ_KEYS)
    }

    pub fn without_quorum_queue_incompatible_keys(&self) -> Self {
        self.without_keys(&PolicyDefinition::QUORUM_QUEUE_INCOMPATIBLE_KEYS)
    }
}
