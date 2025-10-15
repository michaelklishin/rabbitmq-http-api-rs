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
use crate::commons::PolicyTarget;
use crate::responses::{Policy, PolicyDefinition as PolDef};
use serde::Serialize;
use serde_json::{Map, Value};

pub type PolicyDefinition = Map<String, Value>;

impl From<PolDef> for PolicyDefinition {
    fn from(policy: PolDef) -> Self {
        match policy.0 {
            None => {
                let empty: Map<String, Value> = Map::new();
                empty
            }
            Some(value) => value,
        }
    }
}

#[derive(Serialize, Debug)]
pub struct PolicyParams<'a> {
    pub vhost: &'a str,
    pub name: &'a str,
    pub pattern: &'a str,
    #[serde(rename(serialize = "apply-to"))]
    pub apply_to: PolicyTarget,
    pub priority: i32,
    pub definition: PolicyDefinition,
}

impl<'a> From<&'a Policy> for PolicyParams<'a> {
    fn from(policy: &'a Policy) -> Self {
        PolicyParams {
            vhost: &policy.vhost,
            name: &policy.name,
            pattern: &policy.pattern,
            apply_to: policy.apply_to,
            priority: policy.priority as i32,
            definition: policy.definition.clone().into(),
        }
    }
}
