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

use backtrace::Backtrace;
use rabbitmq_http_client::error::{
    ConversionError, EndpointValidationError, ErrorDetails, HttpClientError,
};
use reqwest::StatusCode;
use reqwest::header::HeaderValue;

#[test]
fn test_user_message_client_error_with_details() {
    let error = HttpClientError::ClientErrorResponse {
        url: None,
        status_code: StatusCode::BAD_REQUEST,
        body: Some("error body".to_string()),
        error_details: Some(ErrorDetails {
            error: Some("bad_request".to_string()),
            reason: Some("Queue name is invalid".to_string()),
        }),
        headers: None,
        backtrace: Backtrace::new(),
    };

    assert_eq!(error.user_message(), "Queue name is invalid");
}

#[test]
fn test_user_message_client_error_with_error_only() {
    let error = HttpClientError::ClientErrorResponse {
        url: None,
        status_code: StatusCode::BAD_REQUEST,
        body: None,
        error_details: Some(ErrorDetails {
            error: Some("bad_request".to_string()),
            reason: None,
        }),
        headers: None,
        backtrace: Backtrace::new(),
    };

    assert_eq!(error.user_message(), "bad_request");
}

#[test]
fn test_user_message_client_error_without_details() {
    let error = HttpClientError::ClientErrorResponse {
        url: None,
        status_code: StatusCode::BAD_REQUEST,
        body: None,
        error_details: None,
        headers: None,
        backtrace: Backtrace::new(),
    };

    assert_eq!(error.user_message(), "Client error: 400 Bad Request");
}

#[test]
fn test_user_message_server_error_with_details() {
    let error = HttpClientError::ServerErrorResponse {
        url: None,
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        body: None,
        error_details: Some(ErrorDetails {
            error: Some("internal_error".to_string()),
            reason: Some("Something went wrong".to_string()),
        }),
        headers: None,
        backtrace: Backtrace::new(),
    };

    assert_eq!(error.user_message(), "Something went wrong");
}

#[test]
fn test_user_message_server_error_without_details() {
    let error = HttpClientError::ServerErrorResponse {
        url: None,
        status_code: StatusCode::INTERNAL_SERVER_ERROR,
        body: None,
        error_details: None,
        headers: None,
        backtrace: Backtrace::new(),
    };

    assert_eq!(
        error.user_message(),
        "Server error: 500 Internal Server Error"
    );
}

#[test]
fn test_user_message_not_found() {
    let error = HttpClientError::NotFound;
    assert_eq!(error.user_message(), "Resource not found");
}

#[test]
fn test_user_message_multiple_matching_bindings() {
    let error = HttpClientError::MultipleMatchingBindings;
    assert_eq!(
        error.user_message(),
        "Multiple matching bindings found, provide additional properties"
    );
}

#[test]
fn test_user_message_unsupported_argument_value() {
    let error = HttpClientError::UnsupportedArgumentValue {
        property: "queue_type".to_string(),
    };
    assert_eq!(
        error.user_message(),
        "Unsupported value for property: queue_type"
    );
}

#[test]
fn test_user_message_missing_property() {
    let error = HttpClientError::MissingProperty {
        argument: "name".to_string(),
    };
    assert_eq!(error.user_message(), "Missing required argument: name");
}

#[test]
fn test_user_message_parsing_error() {
    let error = HttpClientError::ParsingError {
        message: "Invalid JSON".to_string(),
    };
    assert_eq!(error.user_message(), "Parsing error: Invalid JSON");
}

#[test]
fn test_user_message_other() {
    let error = HttpClientError::Other;
    assert_eq!(error.user_message(), "An unspecified error occurred");
}

#[test]
fn test_display_request_error_includes_underlying_error() {
    let err = reqwest::blocking::get("not-a-valid-url").unwrap_err();
    let error = HttpClientError::RequestError {
        error: err,
        backtrace: Backtrace::new(),
    };
    let msg = error.to_string();
    assert!(
        msg.starts_with("encountered an error when performing an HTTP request:"),
        "unexpected message: {}",
        msg
    );
    assert!(
        msg.len() > "encountered an error when performing an HTTP request:".len(),
        "underlying error detail missing: {}",
        msg
    );
}

#[test]
fn test_display_invalid_header_value_includes_underlying_error() {
    let err = HeaderValue::from_bytes(&[0x01]).unwrap_err();
    let error = HttpClientError::InvalidHeaderValue { error: err };
    let msg = error.to_string();
    assert!(
        msg.starts_with("could not convert provided value into an HTTP header value:"),
        "unexpected message: {}",
        msg
    );
    assert!(
        msg.len() > "could not convert provided value into an HTTP header value:".len(),
        "underlying error detail missing: {}",
        msg
    );
}

#[test]
fn test_display_incompatible_body_includes_underlying_error() {
    let error = HttpClientError::IncompatibleBody {
        error: ConversionError::ParsingError {
            message: "unexpected token".to_owned(),
        },
        backtrace: Backtrace::new(),
    };
    let msg = error.to_string();
    assert!(
        msg.contains("unexpected token"),
        "underlying error detail missing: {}",
        msg
    );
}

#[test]
fn test_display_endpoint_validation_client_build_error_includes_source() {
    // An invalid PEM certificate forces reqwest::ClientBuilder::build() to fail
    let bad_cert = reqwest::Certificate::from_pem(b"not a valid certificate");
    if let Err(source) = bad_cert {
        let error = EndpointValidationError::ClientBuildError { source };
        let msg = error.to_string();
        assert!(
            msg.starts_with("failed to build HTTP client:"),
            "unexpected message: {}",
            msg
        );
        assert!(
            msg.len() > "failed to build HTTP client:".len(),
            "underlying error detail missing: {}",
            msg
        );
    } else {
        // If we can't construct the error, at least verify the message format compiles
        // by checking the variant's Display output directly via the error type
        panic!("expected certificate parsing to fail");
    }
}

#[test]
fn test_error_details_from_json() {
    let json = r#"{"error": "bad_request", "reason": "Invalid queue name"}"#;
    let details = ErrorDetails::from_json(json);
    assert!(details.is_some());
    let details = details.unwrap();
    assert_eq!(details.error, Some("bad_request".to_string()));
    assert_eq!(details.reason, Some("Invalid queue name".to_string()));
}

#[test]
fn test_error_details_from_invalid_json() {
    let json = "not json";
    let details = ErrorDetails::from_json(json);
    assert!(details.is_none());
}

#[test]
fn test_error_details_reason_prefers_reason_over_error() {
    let details = ErrorDetails {
        error: Some("generic_error".to_string()),
        reason: Some("Specific reason".to_string()),
    };
    assert_eq!(details.reason(), Some("Specific reason"));
}

#[test]
fn test_error_details_reason_falls_back_to_error() {
    let details = ErrorDetails {
        error: Some("generic_error".to_string()),
        reason: None,
    };
    assert_eq!(details.reason(), Some("generic_error"));
}

#[test]
fn test_error_details_reason_returns_none() {
    let details = ErrorDetails {
        error: None,
        reason: None,
    };
    assert_eq!(details.reason(), None);
}
