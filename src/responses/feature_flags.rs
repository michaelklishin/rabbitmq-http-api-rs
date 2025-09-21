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

use serde::{Deserialize, Serialize};
use std::{fmt, ops};
#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlagState {
    Enabled,
    Disabled,
    StateChanging,
    Unavailable,
}

impl From<&str> for FeatureFlagState {
    fn from(value: &str) -> Self {
        match value {
            "enabled" => FeatureFlagState::Enabled,
            "disabled" => FeatureFlagState::Disabled,
            "state_changing" => FeatureFlagState::StateChanging,
            _ => FeatureFlagState::Unavailable,
        }
    }
}

impl From<String> for FeatureFlagState {
    fn from(value: String) -> Self {
        match value.as_str() {
            "enabled" => FeatureFlagState::Enabled,
            "disabled" => FeatureFlagState::Disabled,
            "state_changing" => FeatureFlagState::StateChanging,
            _ => FeatureFlagState::Unavailable,
        }
    }
}

impl AsRef<str> for FeatureFlagState {
    fn as_ref(&self) -> &str {
        match self {
            FeatureFlagState::Enabled => "enabled",
            FeatureFlagState::Disabled => "disabled",
            FeatureFlagState::StateChanging => "state_changing",
            FeatureFlagState::Unavailable => "unavailable",
        }
    }
}

impl From<FeatureFlagState> for String {
    fn from(value: FeatureFlagState) -> Self {
        match value {
            FeatureFlagState::Enabled => "enabled".to_owned(),
            FeatureFlagState::Disabled => "disabled".to_owned(),
            FeatureFlagState::StateChanging => "state_changing".to_owned(),
            FeatureFlagState::Unavailable => "unavailable".to_owned(),
        }
    }
}

impl fmt::Display for FeatureFlagState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureFlagState::Enabled => writeln!(f, "enabled")?,
            FeatureFlagState::Disabled => writeln!(f, "disabled")?,
            FeatureFlagState::StateChanging => writeln!(f, "state_changing")?,
            FeatureFlagState::Unavailable => writeln!(f, "unavailable")?,
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FeatureFlagStability {
    Required,
    Stable,
    Experimental,
}

impl From<&str> for FeatureFlagStability {
    fn from(value: &str) -> Self {
        match value {
            "required" => FeatureFlagStability::Required,
            "stable" => FeatureFlagStability::Stable,
            "experimental" => FeatureFlagStability::Experimental,
            _ => FeatureFlagStability::Stable,
        }
    }
}

impl From<String> for FeatureFlagStability {
    fn from(value: String) -> Self {
        match value.as_ref() {
            "required" => FeatureFlagStability::Required,
            "stable" => FeatureFlagStability::Stable,
            "experimental" => FeatureFlagStability::Experimental,
            _ => FeatureFlagStability::Stable,
        }
    }
}

impl AsRef<str> for FeatureFlagStability {
    fn as_ref(&self) -> &str {
        match self {
            FeatureFlagStability::Required => "required",
            FeatureFlagStability::Stable => "stable",
            FeatureFlagStability::Experimental => "experimental",
        }
    }
}

impl From<FeatureFlagStability> for String {
    fn from(value: FeatureFlagStability) -> Self {
        match value {
            FeatureFlagStability::Required => "required".to_owned(),
            FeatureFlagStability::Stable => "stable".to_owned(),
            FeatureFlagStability::Experimental => "experimental".to_owned(),
        }
    }
}

impl fmt::Display for FeatureFlagStability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FeatureFlagStability::Required => writeln!(f, "required")?,
            FeatureFlagStability::Stable => writeln!(f, "stable")?,
            FeatureFlagStability::Experimental => writeln!(f, "experimental")?,
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct FeatureFlag {
    pub name: String,
    pub state: FeatureFlagState,
    #[serde(rename = "desc")]
    pub description: String,
    pub doc_url: String,
    pub stability: FeatureFlagStability,
    pub provided_by: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct FeatureFlagList(pub Vec<FeatureFlag>);

impl FeatureFlagList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.iter().any(|ff| ff.name == key)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, FeatureFlag> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, FeatureFlag> {
        self.0.iter_mut()
    }
}

impl Deref for FeatureFlagList {
    type Target = Vec<FeatureFlag>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for FeatureFlagList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for FeatureFlagList {
    type Item = FeatureFlag;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
