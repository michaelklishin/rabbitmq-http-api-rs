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

use proptest::prelude::*;
use rabbitmq_http_client::commons::{PolicyTarget, QueueType};

fn arb_queue_type() -> impl Strategy<Value = QueueType> {
    prop_oneof![
        Just(QueueType::Classic),
        Just(QueueType::Quorum),
        Just(QueueType::Stream),
        Just(QueueType::Delayed),
    ]
}

fn arb_policy_target() -> impl Strategy<Value = PolicyTarget> {
    prop_oneof![
        Just(PolicyTarget::Queues),
        Just(PolicyTarget::ClassicQueues),
        Just(PolicyTarget::QuorumQueues),
        Just(PolicyTarget::Streams),
        Just(PolicyTarget::Exchanges),
        Just(PolicyTarget::All),
    ]
}

proptest! {
    #[test]
    fn prop_policy_target_from_queue_type_conversion(queue_type in arb_queue_type()) {
        let policy_target = PolicyTarget::from(queue_type.clone());
        match queue_type {
            QueueType::Classic => prop_assert_eq!(policy_target, PolicyTarget::ClassicQueues),
            QueueType::Quorum => prop_assert_eq!(policy_target, PolicyTarget::QuorumQueues),
            QueueType::Stream => prop_assert_eq!(policy_target, PolicyTarget::Streams),
            QueueType::Delayed => prop_assert_eq!(policy_target, PolicyTarget::Queues),
            QueueType::Unsupported(_) => prop_assert_eq!(policy_target, PolicyTarget::Queues),
        }
    }

    #[test]
    fn prop_policy_target_does_apply_to_reflexive(target in arb_policy_target()) {
        prop_assert!(target.does_apply_to(target.clone()));
    }

    #[test]
    fn prop_policy_target_all_applies_to_everything(target in arb_policy_target()) {
        prop_assert!(PolicyTarget::All.does_apply_to(target.clone()));
        prop_assert!(target.does_apply_to(PolicyTarget::All));
    }

}

#[test]
fn test_policy_target_queues_applies_to_queue_types() {
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::ClassicQueues));
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::QuorumQueues));
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::Streams));
    assert!(!PolicyTarget::Queues.does_apply_to(PolicyTarget::Exchanges));
}
