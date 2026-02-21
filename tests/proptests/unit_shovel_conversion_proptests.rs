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
use rabbitmq_http_client::{
    commons::MessageTransferAcknowledgementMode,
    responses::{MessagingProtocol, ShovelPublishingState, ShovelState, ShovelType},
};

fn arb_ack_mode() -> impl Strategy<Value = MessageTransferAcknowledgementMode> {
    prop_oneof![
        Just(MessageTransferAcknowledgementMode::Immediate),
        Just(MessageTransferAcknowledgementMode::WhenPublished),
        Just(MessageTransferAcknowledgementMode::WhenConfirmed),
    ]
}

fn arb_messaging_protocol() -> impl Strategy<Value = MessagingProtocol> {
    prop_oneof![
        Just(MessagingProtocol::Amqp091),
        Just(MessagingProtocol::Amqp10),
        Just(MessagingProtocol::Local),
    ]
}

fn arb_shovel_type() -> impl Strategy<Value = ShovelType> {
    prop_oneof![Just(ShovelType::Dynamic), Just(ShovelType::Static),]
}

fn arb_shovel_state() -> impl Strategy<Value = ShovelState> {
    prop_oneof![
        Just(ShovelState::Starting),
        Just(ShovelState::Running),
        Just(ShovelState::Terminated),
        Just(ShovelState::Unknown),
    ]
}

fn arb_shovel_publishing_state() -> impl Strategy<Value = ShovelPublishingState> {
    prop_oneof![
        Just(ShovelPublishingState::Running),
        Just(ShovelPublishingState::Blocked),
        Just(ShovelPublishingState::Unknown),
    ]
}

proptest! {
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
    fn prop_messaging_protocol_conversion(protocol in arb_messaging_protocol()) {
        let as_string = String::from(protocol.clone());
        let from_string = MessagingProtocol::from(as_string.clone());
        prop_assert_eq!(protocol.clone(), from_string);

        let display_str = format!("{}", protocol);
        // Display
        match protocol {
            MessagingProtocol::Amqp091 => prop_assert_eq!(display_str, "AMQP 0-9-1"),
            MessagingProtocol::Amqp10 => prop_assert_eq!(display_str, "AMQP 1.0"),
            MessagingProtocol::Local => prop_assert_eq!(display_str, "Local"),
        }

        // String serialization
        match protocol {
            MessagingProtocol::Amqp091 => prop_assert_eq!(as_string, "amqp091"),
            MessagingProtocol::Amqp10 => prop_assert_eq!(as_string, "amqp10"),
            MessagingProtocol::Local => prop_assert_eq!(as_string, "local"),
        }
    }

    #[test]
    fn prop_shovel_type_conversion(shovel_type in arb_shovel_type()) {
        let as_string = String::from(shovel_type.clone());
        let from_string = ShovelType::from(as_string.clone());
        prop_assert_eq!(shovel_type.clone(), from_string);

        let display_str = format!("{}", shovel_type);
        prop_assert_eq!(as_string, display_str);
    }

    #[test]
    fn prop_shovel_state_conversion(state in arb_shovel_state()) {
        let as_string = String::from(state.clone());
        let from_string = ShovelState::from(as_string.clone());
        prop_assert_eq!(state.clone(), from_string);

        let display_str = format!("{}", state);
        prop_assert_eq!(as_string, display_str);
    }

    #[test]
    fn prop_shovel_publishing_state_conversion(pub_state in arb_shovel_publishing_state()) {
        let as_string = String::from(pub_state.clone());
        let from_string = ShovelPublishingState::from(as_string.clone());
        prop_assert_eq!(pub_state.clone(), from_string);

        let display_str = format!("{}", pub_state);
        prop_assert_eq!(as_string, display_str);
    }

    #[test]
    fn prop_message_transfer_ack_mode_roundtrip(ack_mode in arb_ack_mode()) {
        let as_ref = ack_mode.as_ref();
        let from_ref = MessageTransferAcknowledgementMode::from(as_ref);
        prop_assert_eq!(ack_mode.clone(), from_ref);

        let to_string = ack_mode.to_string();
        let from_string = MessageTransferAcknowledgementMode::from(to_string);
        prop_assert_eq!(ack_mode, from_string);
    }

    #[test]
    fn prop_messaging_protocol_roundtrip(protocol in arb_messaging_protocol()) {
        let as_string = String::from(protocol.clone());
        let from_string = MessagingProtocol::from(as_string);
        prop_assert_eq!(protocol, from_string);
    }

    #[test]
    fn prop_shovel_enums_roundtrip(
        shovel_type in arb_shovel_type(),
        state in arb_shovel_state(),
        pub_state in arb_shovel_publishing_state()
    ) {
        // Test ShovelType roundtrip
        let string_from_type = String::from(shovel_type.clone());
        let type_from_string = ShovelType::from(string_from_type);
        prop_assert_eq!(shovel_type, type_from_string);

        // Test ShovelState roundtrip
        let string_from_state = String::from(state.clone());
        let state_from_string = ShovelState::from(string_from_state);
        prop_assert_eq!(state, state_from_string);

        // Test ShovelPublishingState roundtrip
        let string_from_pub_state = String::from(pub_state.clone());
        let pub_state_from_string = ShovelPublishingState::from(string_from_pub_state);
        prop_assert_eq!(pub_state, pub_state_from_string);
    }
}
