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
            apply_to: policy.apply_to.clone(),
            priority: policy.priority as i32,
            definition: policy.definition.clone().into(),
        }
    }
}
