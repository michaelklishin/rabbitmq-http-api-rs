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
use rabbitmq_http_client::requests::{FederationUpstreamParams, QueueFederationParams};
use rabbitmq_http_client::{api::Client, requests::VirtualHostParams};

mod test_helpers;
use crate::test_helpers::{PASSWORD, USERNAME, amqp_endpoint_with_vhost, endpoint};

#[tokio::test]
async fn test_async_declare_a_federation_upstream_with_queue_federation_parameters() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);

    let vh = "rust.http.api.async.test_async_declare_a_federation_upstream_with_queue_federation_parameters";
    let name = "upstream.1";
    let q = "test_async_declare_a_federation_upstream";

    let vh_params = VirtualHostParams::named(vh);
    let result1 = rc.create_vhost(&vh_params).await;
    assert!(result1.is_ok());

    let amqp_endpoint = amqp_endpoint_with_vhost(vh);
    let params = QueueFederationParams::new(q);
    let upstream_params =
        FederationUpstreamParams::new_queue_federation_upstream(vh, name, &amqp_endpoint, params);

    let result2 = rc.declare_federation_upstream(upstream_params).await;
    assert!(result2.is_ok());

    let _ = rc.delete_vhost(vh_params.name, false).await;
}
