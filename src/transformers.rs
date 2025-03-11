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

use crate::commons::QueueType;
use crate::responses::{ClusterDefinitionSet, Policy, PolicyDefinitionOps};

pub trait DefinitionSetTransformer {
    fn transform<'a>(&'a self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet;
}

pub type TransformerFn<T> = Box<dyn Fn(T) -> T>;
pub type TransformerFnOnce<T> = Box<dyn FnOnce(T) -> T>;

pub type TransformerFnMut<T> = Box<dyn FnMut(T) -> T>;

#[derive(Default, Debug)]
pub struct StripCmqKeysFromPolicies {}

impl DefinitionSetTransformer for StripCmqKeysFromPolicies {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        let pf = Box::new(|p: Policy| p.without_cmq_keys());
        let matched_policies = defs.update_policies(pf);

        // for the queue matched by the above policies, inject an "x-queue-type" set to `QueueType::Quorum`
        for mp in matched_policies {
            defs.update_queue_type_of_matching(&mp, QueueType::Quorum)
        }

        defs
    }
}

#[derive(Default, Debug)]
pub struct DropEmptyPolicies {}

impl DefinitionSetTransformer for DropEmptyPolicies {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        let non_empty = defs
            .policies
            .iter()
            .filter(|p| !p.is_empty())
            .cloned()
            .collect::<Vec<_>>();
        defs.policies = non_empty;

        defs
    }
}

//
// Transformation chain
//

pub struct TransformationChain {
    pub chain: Vec<Box<dyn DefinitionSetTransformer>>,
}

#[allow(clippy::single_match)]
impl From<Vec<&str>> for TransformationChain {
    fn from(names: Vec<&str>) -> Self {
        let mut vec: Vec<Box<dyn DefinitionSetTransformer>> = Vec::new();
        dbg!(&names);
        for name in names {
            match name {
                "strip_cmq_keys_from_policies" => {
                    vec.push(Box::new(StripCmqKeysFromPolicies::default()));
                }
                "drop_empty_policies" => {
                    vec.push(Box::new(DropEmptyPolicies::default()));
                }
                _ => (),
            }
        }
        TransformationChain { chain: vec }
    }
}
impl From<Vec<String>> for TransformationChain {
    fn from(names0: Vec<String>) -> Self {
        let names: Vec<&str> = names0.iter().map(|s| s.as_str()).collect();
        TransformationChain::from(names)
    }
}

#[allow(clippy::single_match)]
#[allow(clippy::borrowed_box)]
impl TransformationChain {
    pub fn new(vec: Vec<Box<dyn DefinitionSetTransformer>>) -> TransformationChain {
        TransformationChain { chain: vec }
    }

    pub fn apply<'a>(&'a self, defs: &'a mut ClusterDefinitionSet) -> &'a ClusterDefinitionSet {
        self.chain
            .iter()
            .for_each(|item: &Box<dyn DefinitionSetTransformer>| {
                item.transform(defs);
            });

        defs
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
}
