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
    OVERFLOW_REJECT_PUBLISH, OVERFLOW_REJECT_PUBLISH_DLX, OverflowBehavior, QueueType,
    X_ARGUMENT_KEY_X_OVERFLOW,
};
use crate::responses::{
    ClusterDefinitionSet, Policy, PolicyWithoutVirtualHost, QueueDefinition,
    QueueDefinitionWithoutVirtualHost, QueueOps, VirtualHostDefinitionSet, XArguments,
};
use serde_json::json;
use std::collections::HashMap;

use crate::password_hashing;

pub trait DefinitionSetTransformer {
    fn transform<'a>(&'a self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet;
}

pub trait VirtualHostDefinitionSetTransformer {
    fn transform<'a>(
        &'a self,
        defs: &'a mut VirtualHostDefinitionSet,
    ) -> &'a mut VirtualHostDefinitionSet;
}

pub type TransformerFn<T> = Box<dyn Fn(T) -> T>;
pub type TransformerFnOnce<T> = Box<dyn FnOnce(T) -> T>;

pub type TransformerFnMut<T> = Box<dyn FnMut(T) -> T>;

#[derive(Default, Debug)]
pub struct NoOp {}
impl DefinitionSetTransformer for NoOp {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        defs
    }
}

#[derive(Default, Debug)]
pub struct NoOpVhost {}
impl VirtualHostDefinitionSetTransformer for NoOpVhost {
    fn transform<'a>(
        &self,
        defs: &'a mut VirtualHostDefinitionSet,
    ) -> &'a mut VirtualHostDefinitionSet {
        defs
    }
}

#[derive(Default, Debug)]
pub struct PrepareForQuorumQueueMigration {}

impl DefinitionSetTransformer for PrepareForQuorumQueueMigration {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        // remove CMQ-related keys policies
        let f1 = Box::new(|p: Policy| p.without_cmq_keys());
        let matched_policies = defs.update_policies(f1);

        // for the queue matched by the above policies, inject an "x-queue-type" set to `QueueType::Quorum`
        for mp in matched_policies {
            defs.update_queue_type_of_matching(&mp, QueueType::Quorum)
        }

        // remove other policy keys that are not supported by quorum queues
        let f2 = Box::new(|p: Policy| p.without_quorum_queue_incompatible_keys());
        defs.update_policies(f2);

        // Queue x-arguments:
        // replace x-overflow values that are equal to "reject-publish-dlx"
        let f3 = Box::new(|mut qd: QueueDefinition| {
            if let Some(val) = qd.arguments.get(X_ARGUMENT_KEY_X_OVERFLOW)
                && val.as_str().unwrap_or(OVERFLOW_REJECT_PUBLISH) == OVERFLOW_REJECT_PUBLISH_DLX
            {
                qd.arguments.insert(
                    X_ARGUMENT_KEY_X_OVERFLOW.to_owned(),
                    json!(OverflowBehavior::RejectPublish),
                );
            }
            qd
        });
        defs.update_queues(f3);

        // Policy definitions:
        // replace x-overflow values that are equal to "reject-publish-dlx"
        let f4 = Box::new(|mut p: Policy| {
            let key = "overflow";
            if let Some(val) = p.definition.get(key)
                && val.as_str().unwrap_or(OVERFLOW_REJECT_PUBLISH) == OVERFLOW_REJECT_PUBLISH_DLX
            {
                p.definition
                    .insert(key.to_owned(), json!(OverflowBehavior::RejectPublish));
            }
            p
        });
        defs.update_policies(f4);

        // remove CMQ-related x-arguments
        let f5 = Box::new(|mut qd: QueueDefinition| {
            for key in XArguments::CMQ_KEYS {
                qd.arguments.remove(key);
            }
            qd
        });
        defs.queues = defs.queues.iter().map(|q| f5(q.clone())).collect();

        defs
    }
}

#[derive(Default, Debug)]
pub struct PrepareForQuorumQueueMigrationVhost {}

impl VirtualHostDefinitionSetTransformer for PrepareForQuorumQueueMigrationVhost {
    fn transform<'a>(
        &self,
        defs: &'a mut VirtualHostDefinitionSet,
    ) -> &'a mut VirtualHostDefinitionSet {
        // remove CMQ-related keys policies
        let f1 = Box::new(|p: PolicyWithoutVirtualHost| p.without_cmq_keys());
        let matched_policies: Vec<PolicyWithoutVirtualHost> =
            defs.policies.iter().map(|p| f1(p.clone())).collect();
        defs.policies = matched_policies.clone(); // Update policies in defs

        // for the queue matched by the above policies, inject an "x-queue-type" set to `QueueType::Quorum`
        for mp in matched_policies {
            // Need to iterate through queues and update if they match the policy
            defs.queues.iter_mut().for_each(|qd| {
                if mp.does_match(&qd.name, qd.policy_target_type()) {
                    qd.update_queue_type(QueueType::Quorum);
                }
            });
        }

        // remove other policy keys that are not supported by quorum queues
        let f2 = Box::new(|p: PolicyWithoutVirtualHost| p.without_quorum_queue_incompatible_keys());
        defs.policies = defs.policies.iter().map(|p| f2(p.clone())).collect();

        // Queue x-arguments:
        // replace x-overflow values that are equal to "reject-publish-dlx"
        let f3 = Box::new(|mut qd: QueueDefinitionWithoutVirtualHost| {
            if let Some(val) = qd.arguments.get(X_ARGUMENT_KEY_X_OVERFLOW)
                && val.as_str().unwrap_or(OVERFLOW_REJECT_PUBLISH) == OVERFLOW_REJECT_PUBLISH_DLX
            {
                qd.arguments.insert(
                    X_ARGUMENT_KEY_X_OVERFLOW.to_owned(),
                    json!(OverflowBehavior::RejectPublish),
                );
            }
            qd
        });
        defs.queues = defs.queues.iter().map(|q| f3(q.clone())).collect();

        // Policy definitions:
        // replace x-overflow values that are equal to "reject-publish-dlx"
        let f4 = Box::new(|mut p: PolicyWithoutVirtualHost| {
            let key = "overflow";
            if let Some(val) = p.definition.get(key)
                && val.as_str().unwrap_or(OVERFLOW_REJECT_PUBLISH) == OVERFLOW_REJECT_PUBLISH_DLX
            {
                p.definition
                    .insert(key.to_owned(), json!(OverflowBehavior::RejectPublish));
            }
            p
        });
        defs.policies = defs.policies.iter().map(|p| f4(p.clone())).collect();

        // remove CMQ-related x-arguments
        let f5 = Box::new(|mut qd: QueueDefinitionWithoutVirtualHost| {
            for key in XArguments::CMQ_KEYS {
                qd.arguments.remove(key);
            }
            qd
        });
        defs.queues = defs.queues.iter().map(|q| f5(q.clone())).collect();

        defs
    }
}

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
pub struct StripCmqKeysFromVhostPolicies {}

impl VirtualHostDefinitionSetTransformer for StripCmqKeysFromVhostPolicies {
    fn transform<'a>(
        &self,
        defs: &'a mut VirtualHostDefinitionSet,
    ) -> &'a mut VirtualHostDefinitionSet {
        let pf = Box::new(|p: PolicyWithoutVirtualHost| p.without_cmq_keys());
        let matched_policies: Vec<PolicyWithoutVirtualHost> =
            defs.policies.iter().map(|p| pf(p.clone())).collect();
        defs.policies = matched_policies.clone(); // Update policies in defs

        // for the queue matched by the above policies, inject an "x-queue-type" set to `QueueType::Quorum`
        for mp in matched_policies {
            defs.queues.iter_mut().for_each(|qd| {
                if mp.does_match(&qd.name, qd.policy_target_type()) {
                    qd.update_queue_type(QueueType::Quorum);
                }
            });
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

#[derive(Default, Debug)]
pub struct DropEmptyVhostPolicies {}

impl VirtualHostDefinitionSetTransformer for DropEmptyVhostPolicies {
    fn transform<'a>(
        &self,
        defs: &'a mut VirtualHostDefinitionSet,
    ) -> &'a mut VirtualHostDefinitionSet {
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

#[derive(Default, Debug)]
pub struct ObfuscateUsernames {}

impl DefinitionSetTransformer for ObfuscateUsernames {
    fn transform<'a>(&'a self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        // keeps track of name changes
        let mut obfuscated = HashMap::<String, String>::new();
        let mut i = 1u16;
        for u in &defs.users {
            let new_name = format!("obfuscated-user-{i}");
            i += 1;
            obfuscated.insert(u.name.clone(), new_name.clone());
        }
        let obfuscated2 = obfuscated.clone();

        let updated_users = defs.users.clone().into_iter().map(|u| {
            let new_name = obfuscated.get(&u.name).unwrap_or(&u.name).as_str();
            let salt = password_hashing::salt();
            let new_password = format!("password-{i}");
            let hash =
                password_hashing::base64_encoded_salted_password_hash_sha256(&salt, &new_password);

            u.with_name(new_name.to_owned()).with_password_hash(hash)
        });

        let updated_permissions = defs.permissions.clone().into_iter().map(|p| {
            let new_new = obfuscated2.get(&p.user).unwrap_or(&p.user).as_str();

            p.with_username(new_new)
        });

        defs.users = updated_users.collect::<Vec<_>>();
        defs.permissions = updated_permissions.collect::<Vec<_>>();

        defs
    }
}

#[derive(Default, Debug)]
pub struct ExcludeUsers {}

impl DefinitionSetTransformer for ExcludeUsers {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        defs.users = Vec::new();
        defs
    }
}

#[derive(Default, Debug)]
pub struct ExcludePermissions {}

impl DefinitionSetTransformer for ExcludePermissions {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        defs.permissions = Vec::new();
        defs
    }
}

#[derive(Default, Debug)]
pub struct ExcludeRuntimeParameters {}

impl DefinitionSetTransformer for ExcludeRuntimeParameters {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        defs.parameters = Vec::new();
        defs
    }
}

#[derive(Default, Debug)]
pub struct ExcludePolicies {}

impl DefinitionSetTransformer for ExcludePolicies {
    fn transform<'a>(&self, defs: &'a mut ClusterDefinitionSet) -> &'a mut ClusterDefinitionSet {
        defs.policies = Vec::new();
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
        for name in names {
            match name {
                "prepare_for_quorum_queue_migration" => {
                    vec.push(Box::new(PrepareForQuorumQueueMigration::default()));
                }
                "strip_cmq_keys_from_policies" => {
                    vec.push(Box::new(StripCmqKeysFromPolicies::default()));
                }
                "drop_empty_policies" => {
                    vec.push(Box::new(DropEmptyPolicies::default()));
                }
                "obfuscate_usernames" => {
                    vec.push(Box::new(ObfuscateUsernames::default()));
                }
                "exclude_users" => {
                    vec.push(Box::new(ExcludeUsers::default()));
                }
                "exclude_permissions" => {
                    vec.push(Box::new(ExcludePermissions::default()));
                }
                "exclude_runtime_parameters" => {
                    vec.push(Box::new(ExcludeRuntimeParameters::default()));
                }
                "exclude_policies" => {
                    vec.push(Box::new(ExcludePolicies::default()));
                }
                _ => {
                    vec.push(Box::new(NoOp::default()));
                }
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

pub struct VirtualHostTransformationChain {
    pub chain: Vec<Box<dyn VirtualHostDefinitionSetTransformer>>,
}

impl VirtualHostTransformationChain {
    pub fn new(
        vec: Vec<Box<dyn VirtualHostDefinitionSetTransformer>>,
    ) -> VirtualHostTransformationChain {
        VirtualHostTransformationChain { chain: vec }
    }

    pub fn apply<'a>(
        &'a self,
        defs: &'a mut VirtualHostDefinitionSet,
    ) -> &'a VirtualHostDefinitionSet {
        self.chain.iter().for_each(
            #[allow(clippy::borrowed_box)]
            |item: &Box<dyn VirtualHostDefinitionSetTransformer>| {
                item.transform(defs);
            },
        );
        defs
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chain.is_empty()
    }
}

impl From<Vec<&str>> for VirtualHostTransformationChain {
    fn from(names: Vec<&str>) -> Self {
        let mut vec: Vec<Box<dyn VirtualHostDefinitionSetTransformer>> = Vec::new();
        for name in names {
            match name {
                "prepare_for_quorum_queue_migration" => {
                    vec.push(Box::new(PrepareForQuorumQueueMigrationVhost::default()));
                }
                "strip_cmq_keys_from_policies" => {
                    vec.push(Box::new(StripCmqKeysFromVhostPolicies::default()));
                }
                "drop_empty_policies" => {
                    vec.push(Box::new(DropEmptyVhostPolicies::default()));
                }
                _ => {
                    vec.push(Box::new(NoOpVhost::default()));
                }
            }
        }
        VirtualHostTransformationChain { chain: vec }
    }
}

impl From<Vec<String>> for VirtualHostTransformationChain {
    fn from(names0: Vec<String>) -> Self {
        let names: Vec<&str> = names0.iter().map(|s| s.as_str()).collect();
        VirtualHostTransformationChain::from(names)
    }
}
