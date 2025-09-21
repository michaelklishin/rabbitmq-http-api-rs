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

use crate::responses::RuntimeParameter;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Runtime parameter value map.
///
/// Contains the actual configuration data for a runtime parameter.
/// The structure depends on the component type (federation, shovel, etc.).
pub type RuntimeParameterValue = Map<String, Value>;

/// Represents a [runtime parameter](https://rabbitmq.com/docs/parameters/).
///
/// Runtime parameters are key-value pairs that configure plugin behavior at runtime.
/// The `component` field identifies the plugin (e.g., "federation-upstream", "shovel"),
/// while `name` is the parameter identifier within that component's namespace.
///
/// Common components include:
/// * "federation-upstream": Federation plugin upstreams
/// * "shovel": Dynamic shovel configurations
/// * "mqtt": MQTT plugin parameters
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RuntimeParameterDefinition<'a> {
    pub name: &'a str,
    pub vhost: &'a str,
    pub component: &'a str,
    pub value: RuntimeParameterValue,
}

impl<'a> From<&'a RuntimeParameter> for RuntimeParameterDefinition<'a> {
    fn from(param: &'a RuntimeParameter) -> Self {
        Self {
            name: &param.name,
            vhost: &param.vhost,
            component: &param.component,
            value: param.value.0.clone(),
        }
    }
}

/// Represents a [global runtime parameter](https://rabbitmq.com/docs/parameters/).
///
/// Global parameters apply to the entire RabbitMQ node rather than a specific virtual host.
/// Used for cluster-wide configuration settings.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalRuntimeParameterDefinition<'a> {
    /// Parameter name
    pub name: &'a str,
    /// Parameter value (structure depends on the parameter type)
    pub value: RuntimeParameterValue,
}
