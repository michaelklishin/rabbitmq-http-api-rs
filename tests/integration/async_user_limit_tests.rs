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
    api::Client,
    commons::UserLimitTarget,
    password_hashing,
    requests::{EnforcedLimitParams, UserParams},
};

use crate::test_helpers::{PASSWORD, USERNAME, endpoint};

#[tokio::test]
async fn test_async_list_all_user_limits() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "rust3_t0p_sEkr37");

    let params = UserParams {
        name: "test_list_all_user_limits",
        password_hash: &password_hash,
        tags: "management",
    };
    let result1 = rc.create_user(&params).await;
    assert!(result1.is_ok());

    let limit = EnforcedLimitParams::new(UserLimitTarget::MaxChannels, 500);
    let result2 = rc.set_user_limit(params.name, limit).await;
    assert!(result2.is_ok());

    let result3 = rc.list_all_user_limits().await;
    assert!(result3.is_ok());
    let vec = result3.unwrap();
    assert!(vec.iter().any(|it| it.username == params.name));

    let key1 = UserLimitTarget::MaxConnections;
    assert!(
        !vec.iter()
            .any(|it| it.username == params.name && it.limits.get(key1.as_ref()).is_some())
    );
    let key2 = UserLimitTarget::MaxChannels;
    assert!(
        vec.iter()
            .any(|it| it.username == params.name && it.limits.get(key2.as_ref()).is_some())
    );

    rc.delete_user(params.name, false).await.unwrap();
}

#[tokio::test]
async fn test_async_list_user_limits() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let salt = password_hashing::salt();
    let password_hash =
        password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "rust3_t0p_sEkr37");

    let params = UserParams {
        name: "test_list_user_limits",
        password_hash: &password_hash,
        tags: "management",
    };
    let result1 = rc.create_user(&params).await;
    assert!(result1.is_ok());

    let limit = EnforcedLimitParams::new(UserLimitTarget::MaxChannels, 500);
    let result2 = rc.set_user_limit(params.name, limit).await;
    assert!(result2.is_ok());

    let result3 = rc.list_user_limits(params.name).await;
    assert!(result3.is_ok());
    let vec = result3.unwrap();

    let key1 = UserLimitTarget::MaxChannels;
    assert!(
        vec.iter()
            .any(|it| it.username == params.name && it.limits.get(key1.as_ref()).is_some())
    );
    let key2 = UserLimitTarget::MaxConnections;
    assert!(
        !vec.iter()
            .any(|it| it.username == params.name && it.limits.get(key2.as_ref()).is_some())
    );

    rc.delete_user(params.name, false).await.unwrap();
}
