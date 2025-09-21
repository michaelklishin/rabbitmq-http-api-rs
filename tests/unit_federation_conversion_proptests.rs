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
mod test_helpers;

use proptest::prelude::*;
use rabbitmq_http_client::{
    commons::MessageTransferAcknowledgementMode, requests::FederationResourceCleanupMode,
};

fn arb_ack_mode() -> impl Strategy<Value = MessageTransferAcknowledgementMode> {
    prop_oneof![
        Just(MessageTransferAcknowledgementMode::Immediate),
        Just(MessageTransferAcknowledgementMode::WhenPublished),
        Just(MessageTransferAcknowledgementMode::WhenConfirmed),
    ]
}

fn arb_cleanup_mode() -> impl Strategy<Value = FederationResourceCleanupMode> {
    prop_oneof![
        Just(FederationResourceCleanupMode::Default),
        Just(FederationResourceCleanupMode::Never),
    ]
}

proptest! {
    #[test]
    fn prop_federation_resource_cleanup_mode_conversion(cleanup_mode in arb_cleanup_mode()) {
        let as_str = cleanup_mode.to_string();
        let from_str = FederationResourceCleanupMode::from(as_str.clone());
        prop_assert_eq!(cleanup_mode.clone(), from_str);

        let from_string = FederationResourceCleanupMode::from(as_str);
        prop_assert_eq!(cleanup_mode, from_string);
    }

    #[test]
    fn prop_message_transfer_ack_mode_conversion(ack_mode in arb_ack_mode()) {
        let as_str = ack_mode.as_ref();
        let from_str = MessageTransferAcknowledgementMode::from(as_str);
        prop_assert_eq!(ack_mode.clone(), from_str);

        let as_string = String::from(ack_mode.clone());
        let from_string = MessageTransferAcknowledgementMode::from(as_string);
        prop_assert_eq!(ack_mode.clone(), from_string);

        let display_str = format!("{}", ack_mode);
        prop_assert_eq!(as_str, display_str);
    }

    #[test]
    fn prop_message_transfer_ack_mode_roundtrip(ack_mode in arb_ack_mode()) {
        let as_ref = ack_mode.as_ref();
        let from_ref = MessageTransferAcknowledgementMode::from(as_ref);
        prop_assert_eq!(ack_mode, from_ref);
    }
}
