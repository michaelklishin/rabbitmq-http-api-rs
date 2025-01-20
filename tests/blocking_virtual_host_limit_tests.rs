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
use rabbitmq_http_client::{
    blocking_api::Client,
    commons::VirtualHostLimitTarget,
    requests::{EnforcedLimitParams, VirtualHostParams},
};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_blocking_list_all_vhost_limits() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_all_vhost_limits");
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let limit = EnforcedLimitParams::new(VirtualHostLimitTarget::MaxQueues, 500);
    let result2 = rc.set_vhost_limit(vh_params.name, limit);
    assert!(result2.is_ok());

    let result3 = rc.list_all_vhost_limits();
    assert!(result3.is_ok());
    let vec = result3.unwrap();
    assert!(vec.iter().any(|it| it.vhost == vh_params.name));

    let key1 = VirtualHostLimitTarget::MaxConnections;
    assert!(!vec
        .iter()
        .any(|it| it.vhost == vh_params.name && it.limits.get(key1.as_ref()).is_some()));
    let key2 = VirtualHostLimitTarget::MaxQueues;
    assert!(vec
        .iter()
        .any(|it| it.vhost == vh_params.name && it.limits.get(key2.as_ref()).is_some()));

    rc.delete_vhost(vh_params.name, false).unwrap();
}

#[test]
fn test_blocking_list_vhost_limits() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh_params = VirtualHostParams::named("test_list_vhost_limits");
    let result1 = rc.create_vhost(&vh_params);
    assert!(result1.is_ok());

    let limit = EnforcedLimitParams::new(VirtualHostLimitTarget::MaxConnections, 500);
    let result2 = rc.set_vhost_limit(vh_params.name, limit);
    assert!(result2.is_ok());

    let result3 = rc.list_vhost_limits(vh_params.name);
    assert!(result3.is_ok());
    let vec = result3.unwrap();

    let key1 = VirtualHostLimitTarget::MaxConnections;
    assert!(vec
        .iter()
        .any(|it| it.vhost == vh_params.name && it.limits.get(key1.as_ref()).is_some()));
    let key2 = VirtualHostLimitTarget::MaxQueues;
    assert!(!vec
        .iter()
        .any(|it| it.vhost == vh_params.name && it.limits.get(key2.as_ref()).is_some()));

    rc.delete_vhost(vh_params.name, false).unwrap();
}
