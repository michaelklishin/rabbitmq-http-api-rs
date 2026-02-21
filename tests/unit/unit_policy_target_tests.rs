use rabbitmq_http_client::commons::PolicyTarget;

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

#[test]
fn test_unit_policy_target_does_apply_to() {
    // "all" matches everything
    assert!(PolicyTarget::All.does_apply_to(PolicyTarget::All));
    assert!(PolicyTarget::All.does_apply_to(PolicyTarget::ClassicQueues));
    assert!(PolicyTarget::All.does_apply_to(PolicyTarget::QuorumQueues));
    assert!(PolicyTarget::All.does_apply_to(PolicyTarget::Queues));
    assert!(PolicyTarget::All.does_apply_to(PolicyTarget::Streams));
    assert!(PolicyTarget::All.does_apply_to(PolicyTarget::Exchanges));

    // "queues" matches all queue types plus streams plus "all"
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::All));
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::ClassicQueues));
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::QuorumQueues));
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::Queues));
    assert!(PolicyTarget::Queues.does_apply_to(PolicyTarget::Streams));

    assert!(!PolicyTarget::Queues.does_apply_to(PolicyTarget::Exchanges));

    // "streams" only matches "streams" and "all"
    assert!(PolicyTarget::Streams.does_apply_to(PolicyTarget::All));
    assert!(PolicyTarget::Streams.does_apply_to(PolicyTarget::Streams));
    assert!(!PolicyTarget::Streams.does_apply_to(PolicyTarget::QuorumQueues));
    assert!(!PolicyTarget::Streams.does_apply_to(PolicyTarget::Queues));
    assert!(!PolicyTarget::Streams.does_apply_to(PolicyTarget::ClassicQueues));
    assert!(!PolicyTarget::Streams.does_apply_to(PolicyTarget::Exchanges));

    // "exchanges" only matches "exchanges" and "all"
    assert!(PolicyTarget::Exchanges.does_apply_to(PolicyTarget::All));
    assert!(PolicyTarget::Exchanges.does_apply_to(PolicyTarget::Exchanges));
    assert!(!PolicyTarget::Exchanges.does_apply_to(PolicyTarget::QuorumQueues));
    assert!(!PolicyTarget::Exchanges.does_apply_to(PolicyTarget::Queues));
    assert!(!PolicyTarget::Exchanges.does_apply_to(PolicyTarget::ClassicQueues));
    assert!(!PolicyTarget::Exchanges.does_apply_to(PolicyTarget::Streams));
}
