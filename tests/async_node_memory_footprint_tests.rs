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

use regex::Regex;

#[tokio::test]
async fn test_async_get_node_memory_footprint() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let nodes = rc.list_nodes().await.unwrap();
    let name = nodes.first().unwrap().name.clone();
    let footprint = &mut rc.get_node_memory_footprint(&name).await.unwrap();

    assert!(footprint.breakdown.total.rss >= 1);
    assert!(footprint.breakdown.total.allocated >= 1);
    assert!(footprint.breakdown.total.used_by_runtime >= 1);
    assert!(footprint.breakdown.grand_total() >= 1);

    assert!(footprint.breakdown.metadata_store >= 1);
    assert!(footprint.breakdown.code >= 1);

    let regex = Regex::new(r"\d+\.\d+%").unwrap();

    let metadata_store_percentage_s = footprint.breakdown.metadata_store_percentage_as_text();
    assert!(regex.is_match(&metadata_store_percentage_s));

    let code_percentage_s = footprint.breakdown.code_percentage_as_text();
    assert!(regex.is_match(&code_percentage_s));
}
