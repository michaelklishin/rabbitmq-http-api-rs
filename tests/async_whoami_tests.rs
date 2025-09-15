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
use rabbitmq_http_client::api::Client;

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, endpoint};

//
// GET /api/whoami
//

#[tokio::test]
pub async fn test_async_current_user() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let result = rc.current_user().await;
    assert!(result.is_ok());

    let current_user = result.unwrap();
    assert_eq!(current_user.name, USERNAME);
    assert!(!current_user.tags.is_empty());
}
