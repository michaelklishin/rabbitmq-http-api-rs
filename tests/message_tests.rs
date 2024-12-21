// Copyright (C) 2023-2024 RabbitMQ Core Team (teamrabbitmq@gmail.com)
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
    requests::{self, QueueParams},
    responses::{GetMessage, MessageProperties, MessageRouted},
};
use serde_json::{json, Map, Value};

mod test_helpers;
use crate::test_helpers::{endpoint, PASSWORD, USERNAME};

#[test]
fn test_publish_and_get() {
    let endpoint = endpoint();
    let rc = Client::new(&endpoint, USERNAME, PASSWORD);
    let vhost = "/";
    let queue = "rust.tests.cq.publish_and_get";

    let _ = rc.delete_queue(vhost, queue, false);

    let params = QueueParams::new_durable_classic_queue(queue, None);
    let result2 = rc.declare_queue(vhost, &params);
    assert!(result2.is_ok(), "declare_queue returned {:?}", result2);

    let result3 = rc.publish_message(
        vhost,
        "",
        queue,
        "rust test 1",
        requests::MessageProperties::default(),
    );
    assert!(result3.is_ok(), "get_messages returned {:?}", result3);
    assert_eq!(result3.unwrap(), MessageRouted { routed: true });

    let mut props = Map::<String, Value>::new();
    props.insert(String::from("timestamp"), json!(123456789));
    let result4 = rc.publish_message(vhost, "", queue, "rust test 2", props.clone());
    assert!(result4.is_ok(), "get_messages returned {:?}", result4);
    assert_eq!(result4.unwrap(), MessageRouted { routed: true });

    let result5 = rc.get_messages(vhost, queue, 1, "ack_requeue_false");
    assert!(result5.is_ok(), "get_messages returned {:?}", result5);

    let result6 = result5.unwrap();
    assert_eq!(
        result6,
        [GetMessage {
            payload_bytes: 11,
            redelivered: false,
            exchange: "".to_owned(),
            routing_key: "rust.tests.cq.publish_and_get".to_owned(),
            message_count: 1,
            properties: MessageProperties::default(),
            payload: "rust test 1".to_owned(),
            payload_encoding: "string".to_owned()
        }]
    );

    let result7 = rc.get_messages(vhost, queue, 1, "ack_requeue_false");
    assert!(result7.is_ok(), "get_messages returned {:?}", result7);

    let props = rabbitmq_http_client::responses::MessageProperties(props);
    let result8 = result7.unwrap();
    assert_eq!(
        result8,
        [GetMessage {
            payload_bytes: 11,
            redelivered: false,
            exchange: "".to_owned(),
            routing_key: "rust.tests.cq.publish_and_get".to_owned(),
            message_count: 0,
            properties: props,
            payload: "rust test 2".to_owned(),
            payload_encoding: "string".to_owned()
        }]
    );

    rc.delete_queue(vhost, queue, false).unwrap();
}
