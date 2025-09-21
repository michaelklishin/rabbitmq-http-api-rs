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

#[cfg(feature = "tabled")]
use crate::formatting::display_option;
use serde::{Deserialize, Serialize};
use std::{
    fmt,
    ops::{Deref, DerefMut},
};
#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum DeprecationPhase {
    PermittedByDefault,
    DeniedByDefault,
    Disconnected,
    Removed,
    Undefined,
}

impl From<&str> for DeprecationPhase {
    fn from(value: &str) -> Self {
        match value {
            "permitted_by_default" => DeprecationPhase::PermittedByDefault,
            "denied_by_default" => DeprecationPhase::DeniedByDefault,
            "disconnected" => DeprecationPhase::Disconnected,
            "removed" => DeprecationPhase::Removed,
            _ => DeprecationPhase::Undefined,
        }
    }
}

impl From<String> for DeprecationPhase {
    fn from(value: String) -> Self {
        match value.as_str() {
            "permitted_by_default" => DeprecationPhase::PermittedByDefault,
            "denied_by_default" => DeprecationPhase::DeniedByDefault,
            "disconnected" => DeprecationPhase::Disconnected,
            "removed" => DeprecationPhase::Removed,
            _ => DeprecationPhase::Undefined,
        }
    }
}

impl AsRef<str> for DeprecationPhase {
    fn as_ref(&self) -> &str {
        match self {
            DeprecationPhase::PermittedByDefault => "permitted_by_default",
            DeprecationPhase::DeniedByDefault => "denied_by_default",
            DeprecationPhase::Disconnected => "disconnected",
            DeprecationPhase::Removed => "removed",
            DeprecationPhase::Undefined => "undefined",
        }
    }
}

impl From<DeprecationPhase> for String {
    fn from(value: DeprecationPhase) -> Self {
        match value {
            DeprecationPhase::PermittedByDefault => "permitted_by_default".to_owned(),
            DeprecationPhase::DeniedByDefault => "denied_by_default".to_owned(),
            DeprecationPhase::Disconnected => "disconnected".to_owned(),
            DeprecationPhase::Removed => "removed".to_owned(),
            DeprecationPhase::Undefined => "undefined".to_owned(),
        }
    }
}

impl fmt::Display for DeprecationPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeprecationPhase::PermittedByDefault => writeln!(f, "permitted_by_default")?,
            DeprecationPhase::DeniedByDefault => writeln!(f, "denied_by_default")?,
            DeprecationPhase::Disconnected => writeln!(f, "disconnected")?,
            DeprecationPhase::Removed => writeln!(f, "removed")?,
            DeprecationPhase::Undefined => writeln!(f, "undefined")?,
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct DeprecatedFeature {
    pub name: String,
    #[serde(rename = "desc")]
    pub description: String,
    pub deprecation_phase: DeprecationPhase,
    pub doc_url: String,
    pub provided_by: String,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub state: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct DeprecatedFeatureList(pub Vec<DeprecatedFeature>);

impl DeprecatedFeatureList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.iter().any(|df| df.name == key)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, DeprecatedFeature> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, DeprecatedFeature> {
        self.0.iter_mut()
    }
}

impl Deref for DeprecatedFeatureList {
    type Target = Vec<DeprecatedFeature>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DeprecatedFeatureList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for DeprecatedFeatureList {
    type Item = DeprecatedFeature;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
