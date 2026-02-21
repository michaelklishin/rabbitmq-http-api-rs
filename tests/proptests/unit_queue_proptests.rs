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
use rabbitmq_http_client::commons::QueueType;

proptest! {
    #[test]
    fn test_queue_type_from_str_known_types(s in "[Cc]lassic|[Qq]uorum|[Ss]tream|[Dd]elayed") {
        let queue_type = QueueType::from(s.as_str());
        match s.to_ascii_lowercase().as_str() {
            "classic" => prop_assert_eq!(queue_type, QueueType::Classic),
            "quorum" => prop_assert_eq!(queue_type, QueueType::Quorum),
            "stream" => prop_assert_eq!(queue_type, QueueType::Stream),
            "delayed" => prop_assert_eq!(queue_type, QueueType::Delayed),
            _ => unreachable!()
        }
    }

    #[test]
    fn test_queue_type_from_str_case_insensitive(
        prefix in "[a-zA-Z]*",
        core in "classic|quorum|stream|delayed",
        suffix in "[a-zA-Z]*"
    ) {
        // Only test if the string exactly matches one of the known types
        let test_str = format!("{}{}{}", prefix, core, suffix);
        let queue_type = QueueType::from(test_str.as_str());

        if test_str.to_ascii_lowercase() == core {
            match core.as_str() {
                "classic" => prop_assert_eq!(queue_type, QueueType::Classic),
                "quorum" => prop_assert_eq!(queue_type, QueueType::Quorum),
                "stream" => prop_assert_eq!(queue_type, QueueType::Stream),
                "delayed" => prop_assert_eq!(queue_type, QueueType::Delayed),
                _ => unreachable!()
            }
        } else {
            prop_assert_eq!(queue_type, QueueType::Unsupported(test_str));
        }
    }

    #[test]
    fn test_queue_type_from_str_unsupported(s in "[^csqd][a-zA-Z0-9_-]*|[a-zA-Z0-9_-]*[^cseqmrayuolnd][a-zA-Z0-9_-]*") {
        let lower = s.to_ascii_lowercase();
        if !["classic", "quorum", "stream", "delayed"].contains(&lower.as_str()) {
            let queue_type = QueueType::from(s.as_str());
            prop_assert_eq!(queue_type, QueueType::Unsupported(s));
        }
    }

    #[test]
    fn test_queue_type_roundtrip_conversion(s in "classic|quorum|stream|delayed") {
        let queue_type = QueueType::from(s.as_str());
        let back_to_string = String::from(queue_type.clone());
        prop_assert_eq!(back_to_string, s.clone());

        let as_ref_str = queue_type.as_ref();
        prop_assert_eq!(as_ref_str, s);
    }

    #[test]
    fn test_queue_type_display_matches_string_conversion(queue_type in prop_oneof![
        Just(QueueType::Classic),
        Just(QueueType::Quorum),
        Just(QueueType::Stream),
        Just(QueueType::Delayed),
        "[a-zA-Z0-9_-]+".prop_map(QueueType::Unsupported)
    ]) {
        let display_str = format!("{}", queue_type);
        let string_conv = String::from(queue_type);
        prop_assert_eq!(display_str, string_conv);
    }
}
