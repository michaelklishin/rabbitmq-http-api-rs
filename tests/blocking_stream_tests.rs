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
use rabbitmq_http_client::{blocking_api::Client, requests::StreamParams};
use serde_json::{json, Map, Value};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_blocking_declare_stream() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.blocking.stream.792879823479";
    let expiration = "24h";

    let _ = rc.delete_stream(vhost, name, false);

    let result1 = rc.get_stream_info(vhost, name);
    assert!(result1.is_err());

    let mut map = Map::<String, Value>::new();
    map.insert("x-initial-cluster-size".to_owned(), json!(3));
    let optional_args = Some(map);

    let params = StreamParams {
        name,
        expiration,
        max_length_bytes: None,
        max_segment_length_bytes: None,
        arguments: optional_args,
    };

    let result2 = rc.declare_stream(vhost, &params);
    assert!(result2.is_ok(), "declare_stream returned {:?}", result2);

    let _ = rc.delete_stream(vhost, name, false);
}

#[test]
fn test_blocking_delete_stream() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let name = "rust.tests.blocking.stream.67816479475";

    let _ = rc.delete_stream(vhost, name, false);

    let result1 = rc.get_stream_info(vhost, name);
    assert!(result1.is_err());

    let params = StreamParams::new(name, "7D");

    let result2 = rc.declare_stream(vhost, &params);
    assert!(result2.is_ok(), "declare_stream returned {:?}", result2);

    rc.delete_stream(vhost, name, false).unwrap();
    let result3 = rc.get_stream_info(vhost, name);
    assert!(result3.is_err());
}
