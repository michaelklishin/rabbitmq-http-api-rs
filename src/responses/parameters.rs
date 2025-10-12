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
use std::fmt::Formatter;
use std::ops::Deref;

use crate::commons::VirtualHostName;
use crate::formatting::*;
use serde::de::{MapAccess, SeqAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Serialize};
use serde_json::Map;

#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(transparent)]
pub struct RuntimeParameterValue(pub Map<String, serde_json::Value>);

impl Deref for RuntimeParameterValue {
    type Target = Map<String, serde_json::Value>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for RuntimeParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct GlobalRuntimeParameterValue(pub serde_json::Value);

impl fmt::Display for GlobalRuntimeParameterValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", &self.0)?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct RuntimeParameter {
    pub name: String,
    pub vhost: VirtualHostName,
    pub component: String,
    #[serde(deserialize_with = "deserialize_runtime_parameter_value")]
    pub value: RuntimeParameterValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct RuntimeParameterWithoutVirtualHost {
    pub name: String,
    pub component: String,
    #[serde(deserialize_with = "deserialize_runtime_parameter_value")]
    pub value: RuntimeParameterValue,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct GlobalRuntimeParameter {
    pub name: String,
    pub value: GlobalRuntimeParameterValue,
}

fn deserialize_runtime_parameter_value<'de, D>(
    deserializer: D,
) -> Result<RuntimeParameterValue, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct MapVisitor<T> {
        default: T,
    }

    impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for MapVisitor<T> {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map")
        }

        fn visit_seq<A>(self, _seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            Ok(self.default)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let deserializer = MapAccessDeserializer::new(map);
            let m = Deserialize::deserialize(deserializer)?;
            Ok(m)
        }
    }

    deserializer.deserialize_any(MapVisitor {
        default: RuntimeParameterValue::default(),
    })
}
