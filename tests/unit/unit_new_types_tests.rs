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

use rabbitmq_http_client::commons::{ExchangeType, PolicyTarget, QueueType};
use rabbitmq_http_client::requests::{
    Amqp091ShovelDestinationEndpoint, Amqp091ShovelDestinationParams, Amqp091ShovelSourceEndpoint,
    Amqp091ShovelSourceParams, DeadLetterStrategy, OptionalQueueArgument, OverflowBehavior,
    OwnedAmqp091ShovelDestinationEndpoint, OwnedAmqp091ShovelSourceEndpoint, OwnedExchangeParams,
    OwnedPolicyParams, OwnedQueueParams, OwnedUserParams, PolicyDefinitionBuilder,
    QueueLeaderLocator,
};
use serde_json::json;

mod shovel_endpoint_tests {
    use super::*;

    #[test]
    fn test_queue_source_endpoint_creation() {
        let endpoint = Amqp091ShovelSourceEndpoint::queue("amqp://localhost", "my-queue");

        assert_eq!(endpoint.uri(), "amqp://localhost");
        assert!(!endpoint.is_predeclared());

        if let Amqp091ShovelSourceEndpoint::Queue { queue, .. } = &endpoint {
            assert_eq!(*queue, "my-queue");
        } else {
            panic!("Expected Queue variant");
        }
    }

    #[test]
    fn test_predeclared_queue_source_endpoint() {
        let endpoint =
            Amqp091ShovelSourceEndpoint::predeclared_queue("amqp://localhost", "my-queue");

        assert!(endpoint.is_predeclared());
    }

    #[test]
    fn test_exchange_source_endpoint_creation() {
        let endpoint = Amqp091ShovelSourceEndpoint::exchange(
            "amqp://localhost",
            "my-exchange",
            Some("routing.key"),
        );

        assert_eq!(endpoint.uri(), "amqp://localhost");
        assert!(!endpoint.is_predeclared());

        if let Amqp091ShovelSourceEndpoint::Exchange {
            exchange,
            routing_key,
            ..
        } = &endpoint
        {
            assert_eq!(*exchange, "my-exchange");
            assert_eq!(*routing_key, Some("routing.key"));
        } else {
            panic!("Expected Exchange variant");
        }
    }

    #[test]
    fn test_source_endpoint_to_params_queue() {
        let endpoint = Amqp091ShovelSourceEndpoint::queue("amqp://localhost", "my-queue");
        let params: Amqp091ShovelSourceParams = endpoint.into();

        assert_eq!(params.source_uri, "amqp://localhost");
        assert_eq!(params.source_queue, Some("my-queue"));
        assert_eq!(params.source_exchange, None);
        assert!(!params.predeclared);
    }

    #[test]
    fn test_source_endpoint_to_params_exchange() {
        let endpoint = Amqp091ShovelSourceEndpoint::predeclared_exchange(
            "amqp://localhost",
            "my-exchange",
            Some("routing.key"),
        );
        let params: Amqp091ShovelSourceParams = endpoint.into();

        assert_eq!(params.source_uri, "amqp://localhost");
        assert_eq!(params.source_queue, None);
        assert_eq!(params.source_exchange, Some("my-exchange"));
        assert_eq!(params.source_exchange_routing_key, Some("routing.key"));
        assert!(params.predeclared);
    }

    #[test]
    fn test_destination_endpoint_to_params() {
        let endpoint =
            Amqp091ShovelDestinationEndpoint::queue("amqp://remote-host", "destination-queue");
        let params: Amqp091ShovelDestinationParams = endpoint.into();

        assert_eq!(params.destination_uri, "amqp://remote-host");
        assert_eq!(params.destination_queue, Some("destination-queue"));
        assert_eq!(params.destination_exchange, None);
    }

    #[test]
    fn test_owned_source_endpoint_as_ref() {
        let owned = OwnedAmqp091ShovelSourceEndpoint::queue("amqp://localhost", "my-queue");
        let borrowed = owned.as_ref();

        assert_eq!(borrowed.uri(), "amqp://localhost");
        if let Amqp091ShovelSourceEndpoint::Queue { queue, .. } = borrowed {
            assert_eq!(queue, "my-queue");
        } else {
            panic!("Expected Queue variant");
        }
    }

    #[test]
    fn test_owned_destination_endpoint_as_ref() {
        let owned = OwnedAmqp091ShovelDestinationEndpoint::exchange(
            "amqp://localhost",
            "my-exchange",
            Some("key".to_string()),
        );
        let borrowed = owned.as_ref();

        if let Amqp091ShovelDestinationEndpoint::Exchange {
            exchange,
            routing_key,
            ..
        } = borrowed
        {
            assert_eq!(exchange, "my-exchange");
            assert_eq!(routing_key, Some("key"));
        } else {
            panic!("Expected Exchange variant");
        }
    }
}

mod owned_params_tests {
    use super::*;

    #[test]
    fn test_owned_policy_params_creation() {
        let definition = PolicyDefinitionBuilder::new()
            .message_ttl(60000)
            .max_length(10000)
            .build();

        let params = OwnedPolicyParams::new("/", "my-policy", "^test\\.", definition)
            .apply_to(PolicyTarget::Queues)
            .priority(10);

        assert_eq!(params.vhost, "/");
        assert_eq!(params.name, "my-policy");
        assert_eq!(params.pattern, "^test\\.");
        assert_eq!(params.apply_to, PolicyTarget::Queues);
        assert_eq!(params.priority, 10);
    }

    #[test]
    fn test_owned_policy_params_as_ref() {
        let definition = PolicyDefinitionBuilder::new().max_length(1000).build();

        let owned = OwnedPolicyParams::new("/", "test-policy", ".*", definition);
        let borrowed = owned.as_ref();

        assert_eq!(borrowed.vhost, "/");
        assert_eq!(borrowed.name, "test-policy");
    }

    #[test]
    fn test_owned_queue_params_quorum_queue() {
        let params = OwnedQueueParams::new_quorum_queue("my-quorum-queue", None);

        assert_eq!(params.name, "my-quorum-queue");
        assert_eq!(params.queue_type, QueueType::Quorum);
        assert!(params.durable);
        assert!(!params.auto_delete);
    }

    #[test]
    fn test_owned_queue_params_with_arguments() {
        let params = OwnedQueueParams::new_quorum_queue("my-queue", None)
            .with_message_ttl(60000)
            .with_max_length(10000)
            .with_dead_letter_exchange("dlx");

        let args = params.arguments.as_ref().unwrap();
        assert_eq!(args.get("x-message-ttl"), Some(&json!(60000)));
        assert_eq!(args.get("x-max-length"), Some(&json!(10000)));
        assert_eq!(args.get("x-dead-letter-exchange"), Some(&json!("dlx")));
    }

    #[test]
    fn test_owned_queue_params_as_ref() {
        let owned = OwnedQueueParams::new_stream("my-stream", None);
        let borrowed = owned.as_ref();

        assert_eq!(borrowed.name, "my-stream");
        assert_eq!(borrowed.queue_type, QueueType::Stream);
    }

    #[test]
    fn test_owned_exchange_params_durable_topic() {
        let params = OwnedExchangeParams::durable_topic("my-topic-exchange", None);

        assert_eq!(params.name, "my-topic-exchange");
        assert_eq!(params.exchange_type, ExchangeType::Topic);
        assert!(params.durable);
        assert!(!params.auto_delete);
    }

    #[test]
    fn test_owned_exchange_params_as_ref() {
        let owned = OwnedExchangeParams::durable_direct("my-direct", None);
        let borrowed = owned.as_ref();

        assert_eq!(borrowed.name, "my-direct");
        assert_eq!(borrowed.exchange_type, ExchangeType::Direct);
    }

    #[test]
    fn test_owned_user_params_administrator() {
        let params = OwnedUserParams::administrator("admin", "hashed_password");

        assert_eq!(params.name, "admin");
        assert_eq!(params.tags, "administrator");
    }

    #[test]
    fn test_owned_user_params_as_ref() {
        let owned = OwnedUserParams::monitoring("monitor_user", "hash");
        let borrowed = owned.as_ref();

        assert_eq!(borrowed.name, "monitor_user");
        assert_eq!(borrowed.tags, "monitoring");
    }
}

mod queue_argument_tests {
    use super::*;

    #[test]
    fn test_queue_argument_to_x_arguments_empty() {
        let args: Vec<OptionalQueueArgument> = vec![];
        let x_args = OptionalQueueArgument::to_x_arguments(args);

        assert!(x_args.is_none());
    }

    #[test]
    fn test_queue_argument_message_ttl() {
        let args = vec![OptionalQueueArgument::MessageTtl(60000)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-message-ttl"), Some(&json!(60000)));
    }

    #[test]
    fn test_queue_argument_queue_ttl() {
        let args = vec![OptionalQueueArgument::QueueTtl(300000)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-expires"), Some(&json!(300000)));
    }

    #[test]
    fn test_queue_argument_max_length() {
        let args = vec![OptionalQueueArgument::MaxLength(10000)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-max-length"), Some(&json!(10000)));
    }

    #[test]
    fn test_queue_argument_dead_letter_exchange() {
        let args = vec![OptionalQueueArgument::DeadLetterExchange("dlx".to_string())];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-dead-letter-exchange"), Some(&json!("dlx")));
    }

    #[test]
    fn test_queue_argument_overflow_drop_head() {
        let args = vec![OptionalQueueArgument::Overflow(OverflowBehavior::DropHead)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-overflow"), Some(&json!("drop-head")));
    }

    #[test]
    fn test_queue_argument_overflow_reject_publish() {
        let args = vec![OptionalQueueArgument::Overflow(
            OverflowBehavior::RejectPublish,
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-overflow"), Some(&json!("reject-publish")));
    }

    #[test]
    fn test_queue_argument_quorum_initial_group_size() {
        let args = vec![OptionalQueueArgument::QuorumInitialGroupSize(5)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-quorum-initial-group-size"), Some(&json!(5)));
    }

    #[test]
    fn test_queue_argument_single_active_consumer() {
        let args = vec![OptionalQueueArgument::SingleActiveConsumer(true)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-single-active-consumer"), Some(&json!(true)));
    }

    #[test]
    fn test_queue_argument_custom() {
        let args = vec![OptionalQueueArgument::Custom(
            "x-custom-arg".to_string(),
            json!("custom-value"),
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-custom-arg"), Some(&json!("custom-value")));
    }

    #[test]
    fn test_queue_argument_multiple() {
        let args = vec![
            OptionalQueueArgument::MessageTtl(60000),
            OptionalQueueArgument::MaxLength(10000),
            OptionalQueueArgument::DeadLetterExchange("dlx".to_string()),
            OptionalQueueArgument::DeadLetterRoutingKey("dlx.key".to_string()),
        ];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-message-ttl"), Some(&json!(60000)));
        assert_eq!(x_args.get("x-max-length"), Some(&json!(10000)));
        assert_eq!(x_args.get("x-dead-letter-exchange"), Some(&json!("dlx")));
        assert_eq!(
            x_args.get("x-dead-letter-routing-key"),
            Some(&json!("dlx.key"))
        );
    }

    #[test]
    fn test_queue_argument_max_length_bytes() {
        let args = vec![OptionalQueueArgument::MaxLengthBytes(1_000_000)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-max-length-bytes"), Some(&json!(1_000_000)));
    }

    #[test]
    fn test_queue_argument_dead_letter_strategy() {
        let args = vec![OptionalQueueArgument::DeadLetterStrategy(
            DeadLetterStrategy::AtLeastOnce,
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(
            x_args.get("x-dead-letter-strategy"),
            Some(&json!("at-least-once"))
        );
    }

    #[test]
    fn test_queue_argument_dead_letter_strategy_at_most_once() {
        let args = vec![OptionalQueueArgument::DeadLetterStrategy(
            DeadLetterStrategy::AtMostOnce,
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(
            x_args.get("x-dead-letter-strategy"),
            Some(&json!("at-most-once"))
        );
    }

    #[test]
    fn test_queue_argument_overflow_reject_publish_dlx() {
        let args = vec![OptionalQueueArgument::Overflow(
            OverflowBehavior::RejectPublishDlx,
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-overflow"), Some(&json!("reject-publish-dlx")));
    }

    #[test]
    fn test_queue_argument_max_priority() {
        let args = vec![OptionalQueueArgument::MaxPriority(10)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-max-priority"), Some(&json!(10)));
    }

    #[test]
    fn test_queue_argument_quorum_target_group_size() {
        let args = vec![OptionalQueueArgument::QuorumTargetGroupSize(5)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-quorum-target-group-size"), Some(&json!(5)));
    }

    #[test]
    fn test_queue_argument_queue_leader_locator_balanced() {
        let args = vec![OptionalQueueArgument::QueueLeaderLocator(
            QueueLeaderLocator::Balanced,
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(
            x_args.get("x-queue-leader-locator"),
            Some(&json!("balanced"))
        );
    }

    #[test]
    fn test_queue_argument_queue_leader_locator_client_local() {
        let args = vec![OptionalQueueArgument::QueueLeaderLocator(
            QueueLeaderLocator::ClientLocal,
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(
            x_args.get("x-queue-leader-locator"),
            Some(&json!("client-local"))
        );
    }

    #[test]
    fn test_queue_argument_delivery_limit() {
        let args = vec![OptionalQueueArgument::DeliveryLimit(3)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-delivery-limit"), Some(&json!(3)));
    }

    #[test]
    fn test_queue_argument_initial_cluster_size() {
        let args = vec![OptionalQueueArgument::InitialClusterSize(3)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-initial-cluster-size"), Some(&json!(3)));
    }

    #[test]
    fn test_queue_argument_max_age() {
        let args = vec![OptionalQueueArgument::MaxAge("7D".to_string())];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-max-age"), Some(&json!("7D")));
    }

    #[test]
    fn test_queue_argument_stream_max_segment_size_bytes() {
        let args = vec![OptionalQueueArgument::StreamMaxSegmentSizeBytes(
            500_000_000,
        )];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(
            x_args.get("x-stream-max-segment-size-bytes"),
            Some(&json!(500_000_000))
        );
    }

    #[test]
    fn test_queue_argument_stream_filter_size_bytes() {
        let args = vec![OptionalQueueArgument::StreamFilterSizeBytes(32)];
        let x_args = OptionalQueueArgument::to_x_arguments(args).unwrap();

        assert_eq!(x_args.get("x-stream-filter-size-bytes"), Some(&json!(32)));
    }
}

mod error_helper_tests {
    use rabbitmq_http_client::error::HttpClientError;
    use reqwest::StatusCode;

    fn create_client_error(status: StatusCode) -> HttpClientError {
        HttpClientError::ClientErrorResponse {
            url: None,
            status_code: status,
            body: None,
            error_details: None,
            headers: None,
            backtrace: backtrace::Backtrace::new(),
        }
    }

    fn create_server_error(status: StatusCode) -> HttpClientError {
        HttpClientError::ServerErrorResponse {
            url: None,
            status_code: status,
            body: None,
            error_details: None,
            headers: None,
            backtrace: backtrace::Backtrace::new(),
        }
    }

    #[test]
    fn test_is_not_found_with_not_found_variant() {
        let err = HttpClientError::NotFound;
        assert!(err.is_not_found());
    }

    #[test]
    fn test_is_not_found_with_404_status() {
        let err = create_client_error(StatusCode::NOT_FOUND);
        assert!(err.is_not_found());
    }

    #[test]
    fn test_is_not_found_with_other_status() {
        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert!(!err.is_not_found());
    }

    #[test]
    fn test_is_already_exists() {
        let err = create_client_error(StatusCode::CONFLICT);
        assert!(err.is_already_exists());
    }

    #[test]
    fn test_is_already_exists_false() {
        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert!(!err.is_already_exists());
    }

    #[test]
    fn test_is_unauthorized_401() {
        let err = create_client_error(StatusCode::UNAUTHORIZED);
        assert!(err.is_unauthorized());
    }

    #[test]
    fn test_is_forbidden_403() {
        let err = create_client_error(StatusCode::FORBIDDEN);
        assert!(err.is_forbidden());
    }

    #[test]
    fn test_is_unauthorized_false() {
        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert!(!err.is_unauthorized());
    }

    #[test]
    fn test_is_forbidden_false() {
        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert!(!err.is_forbidden());
    }

    #[test]
    fn test_is_client_error() {
        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert!(err.is_client_error());
        assert!(!err.is_server_error());
    }

    #[test]
    fn test_is_server_error() {
        let err = create_server_error(StatusCode::INTERNAL_SERVER_ERROR);
        assert!(err.is_server_error());
        assert!(!err.is_client_error());
    }

    #[test]
    fn test_status_code_from_client_error() {
        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert_eq!(err.status_code(), Some(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn test_status_code_from_server_error() {
        let err = create_server_error(StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(err.status_code(), Some(StatusCode::INTERNAL_SERVER_ERROR));
    }

    #[test]
    fn test_status_code_from_not_found() {
        let err = HttpClientError::NotFound;
        assert_eq!(err.status_code(), Some(StatusCode::NOT_FOUND));
    }

    #[test]
    fn test_status_code_from_other_error() {
        let err = HttpClientError::MultipleMatchingBindings;
        assert_eq!(err.status_code(), None);
    }

    // Happy path coverage (transport errors that return true) requires a real
    // `reqwest::Error` and is therefore covered by integration tests.

    #[test]
    fn test_not_found_is_not_a_connection_error() {
        let err = HttpClientError::NotFound;
        assert!(!err.is_connection_error());
        assert!(!err.is_timeout());
        assert!(!err.is_tls_handshake_error());
    }

    #[test]
    fn test_client_error_is_not_a_connection_error() {
        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert!(!err.is_connection_error());
        assert!(!err.is_timeout());
        assert!(!err.is_tls_handshake_error());
    }

    #[test]
    fn test_server_error_is_not_a_connection_error() {
        let err = create_server_error(StatusCode::INTERNAL_SERVER_ERROR);
        assert!(!err.is_connection_error());
        assert!(!err.is_timeout());
        assert!(!err.is_tls_handshake_error());
    }

    #[test]
    fn test_as_reqwest_error_returns_none_for_non_request_errors() {
        let err = HttpClientError::NotFound;
        assert!(err.as_reqwest_error().is_none());

        let err = create_client_error(StatusCode::BAD_REQUEST);
        assert!(err.as_reqwest_error().is_none());

        let err = create_server_error(StatusCode::INTERNAL_SERVER_ERROR);
        assert!(err.as_reqwest_error().is_none());
    }
}

mod client_builder_tests {
    use rabbitmq_http_client::blocking_api::ClientBuilder;
    use std::time::Duration;

    #[test]
    fn test_default_builder() {
        let _client = ClientBuilder::new().build().unwrap();
    }

    #[test]
    fn test_recommended_defaults() {
        let _client = ClientBuilder::new()
            .with_endpoint("http://localhost:15672/api")
            .with_basic_auth_credentials("user", "pass")
            .with_recommended_defaults()
            .build()
            .unwrap();
    }

    #[test]
    fn test_recommended_defaults_can_be_overridden() {
        let _client = ClientBuilder::new()
            .with_recommended_defaults()
            .with_request_timeout(Duration::from_secs(120))
            .build()
            .unwrap();
    }

    #[test]
    fn test_with_connect_timeout() {
        let _client = ClientBuilder::new()
            .with_connect_timeout(Duration::from_secs(5))
            .build()
            .unwrap();
    }

    #[test]
    fn test_with_both_timeouts() {
        let _client = ClientBuilder::new()
            .with_connect_timeout(Duration::from_secs(5))
            .with_request_timeout(Duration::from_secs(30))
            .build()
            .unwrap();
    }

    #[test]
    fn test_connect_timeout_preserved_across_with_endpoint() {
        let _client = ClientBuilder::new()
            .with_connect_timeout(Duration::from_secs(5))
            .with_endpoint("http://localhost:15672/api")
            .build()
            .unwrap();
    }

    #[test]
    fn test_connect_timeout_preserved_across_with_credentials() {
        let _client = ClientBuilder::new()
            .with_connect_timeout(Duration::from_secs(5))
            .with_basic_auth_credentials("user", "pass")
            .build()
            .unwrap();
    }
}

mod enum_conversion_tests {
    use super::*;

    #[test]
    fn test_overflow_behavior_from_str_drop_head() {
        assert_eq!(
            OverflowBehavior::from("drop-head"),
            OverflowBehavior::DropHead
        );
    }

    #[test]
    fn test_overflow_behavior_from_str_reject_publish() {
        assert_eq!(
            OverflowBehavior::from("reject-publish"),
            OverflowBehavior::RejectPublish
        );
    }

    #[test]
    fn test_overflow_behavior_from_str_reject_publish_dlx() {
        assert_eq!(
            OverflowBehavior::from("reject-publish-dlx"),
            OverflowBehavior::RejectPublishDlx
        );
    }

    #[test]
    fn test_overflow_behavior_from_str_unknown_defaults_to_drop_head() {
        assert_eq!(
            OverflowBehavior::from("unknown"),
            OverflowBehavior::DropHead
        );
        assert_eq!(OverflowBehavior::from(""), OverflowBehavior::DropHead);
    }

    #[test]
    fn test_overflow_behavior_to_str() {
        let s: &str = OverflowBehavior::DropHead.into();
        assert_eq!(s, "drop-head");
        let s: &str = OverflowBehavior::RejectPublish.into();
        assert_eq!(s, "reject-publish");
        let s: &str = OverflowBehavior::RejectPublishDlx.into();
        assert_eq!(s, "reject-publish-dlx");
    }

    #[test]
    fn test_dead_letter_strategy_from_str_at_most_once() {
        assert_eq!(
            DeadLetterStrategy::from("at-most-once"),
            DeadLetterStrategy::AtMostOnce
        );
    }

    #[test]
    fn test_dead_letter_strategy_from_str_at_least_once() {
        assert_eq!(
            DeadLetterStrategy::from("at-least-once"),
            DeadLetterStrategy::AtLeastOnce
        );
    }

    #[test]
    fn test_dead_letter_strategy_from_str_unknown_defaults_to_at_most_once() {
        assert_eq!(
            DeadLetterStrategy::from("unknown"),
            DeadLetterStrategy::AtMostOnce
        );
        assert_eq!(DeadLetterStrategy::from(""), DeadLetterStrategy::AtMostOnce);
    }

    #[test]
    fn test_dead_letter_strategy_to_str() {
        let s: &str = DeadLetterStrategy::AtMostOnce.into();
        assert_eq!(s, "at-most-once");
        let s: &str = DeadLetterStrategy::AtLeastOnce.into();
        assert_eq!(s, "at-least-once");
    }

    #[test]
    fn test_queue_leader_locator_from_str_balanced() {
        assert_eq!(
            QueueLeaderLocator::from("balanced"),
            QueueLeaderLocator::Balanced
        );
    }

    #[test]
    fn test_queue_leader_locator_from_str_client_local() {
        assert_eq!(
            QueueLeaderLocator::from("client-local"),
            QueueLeaderLocator::ClientLocal
        );
    }

    #[test]
    fn test_queue_leader_locator_from_str_unknown_defaults_to_balanced() {
        assert_eq!(
            QueueLeaderLocator::from("unknown"),
            QueueLeaderLocator::Balanced
        );
        assert_eq!(QueueLeaderLocator::from(""), QueueLeaderLocator::Balanced);
    }

    #[test]
    fn test_queue_leader_locator_to_str() {
        let s: &str = QueueLeaderLocator::Balanced.into();
        assert_eq!(s, "balanced");
        let s: &str = QueueLeaderLocator::ClientLocal.into();
        assert_eq!(s, "client-local");
    }
}
