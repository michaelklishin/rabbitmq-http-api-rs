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

use crate::responses;
use thiserror::Error;

use backtrace::Backtrace;
use reqwest::{
    StatusCode, Url,
    header::{HeaderMap, InvalidHeaderValue},
};
use serde::Deserialize;

#[derive(Error, Debug)]
pub enum ConversionError {
    #[error("Unsupported argument value for property (field) {property}")]
    UnsupportedPropertyValue { property: String },
    #[error("Missing the required argument")]
    MissingProperty { argument: String },
    #[error("Could not parse a value: {message}")]
    ParsingError { message: String },
    #[error("Invalid type: expected {expected}")]
    InvalidType { expected: String },
}

/// The API returns JSON with "error" and "reason" fields in error responses.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ErrorDetails {
    /// Generic error type, e.g., "bad_request"
    pub error: Option<String>,
    /// Detailed reason for the error
    pub reason: Option<String>,
}

impl ErrorDetails {
    pub fn from_json(body: &str) -> Option<Self> {
        serde_json::from_str(body).ok()
    }

    /// `reason` (typically more detailed) over `error` (generic).
    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref().or(self.error.as_deref())
    }
}

#[derive(Error, Debug)]
pub enum Error<U, S, E, BT> {
    #[error("API responded with a client error: status code of {status_code}")]
    ClientErrorResponse {
        url: Option<U>,
        status_code: S,
        body: Option<String>,
        error_details: Option<ErrorDetails>,
        headers: Option<HeaderMap>,
        backtrace: BT,
    },
    #[error("API responded with a server error: status code of {status_code}")]
    ServerErrorResponse {
        url: Option<U>,
        status_code: S,
        body: Option<String>,
        error_details: Option<ErrorDetails>,
        headers: Option<HeaderMap>,
        backtrace: BT,
    },
    #[error("Health check failed")]
    HealthCheckFailed {
        path: String,
        details: responses::HealthCheckFailureDetails,
        status_code: S,
    },
    #[error("API responded with a 404 Not Found")]
    NotFound,
    #[error(
        "Cannot delete a binding: multiple matching bindings were found, provide additional properties"
    )]
    MultipleMatchingBindings,
    #[error("could not convert provided value into an HTTP header value")]
    InvalidHeaderValue { error: InvalidHeaderValue },
    #[error("Unsupported argument value for property (field) {property}")]
    UnsupportedArgumentValue { property: String },
    #[error("Missing required argument")]
    MissingProperty { argument: String },
    #[error("Response is incompatible with the target data type")]
    IncompatibleBody {
        error: ConversionError,
        backtrace: BT,
    },
    #[error("Could not parse a value: {message}")]
    ParsingError { message: String },
    #[error("encountered an error when performing an HTTP request")]
    RequestError { error: E, backtrace: BT },
    #[error("an unspecified error")]
    Other,
}

#[allow(unused)]
pub type HttpClientError = Error<Url, StatusCode, reqwest::Error, Backtrace>;

impl From<reqwest::Error> for HttpClientError {
    fn from(req_err: reqwest::Error) -> Self {
        match req_err.status() {
            None => HttpClientError::RequestError {
                error: req_err,
                backtrace: Backtrace::new(),
            },
            Some(status_code) => {
                if status_code.is_client_error() {
                    return HttpClientError::ClientErrorResponse {
                        url: req_err.url().cloned(),
                        status_code,
                        body: None,
                        error_details: None,
                        headers: None,
                        backtrace: Backtrace::new(),
                    };
                };

                if status_code.is_server_error() {
                    return HttpClientError::ServerErrorResponse {
                        url: req_err.url().cloned(),
                        status_code,
                        body: None,
                        error_details: None,
                        headers: None,
                        backtrace: Backtrace::new(),
                    };
                };

                HttpClientError::RequestError {
                    error: req_err,
                    backtrace: Backtrace::new(),
                }
            }
        }
    }
}

impl From<InvalidHeaderValue> for HttpClientError {
    fn from(err: InvalidHeaderValue) -> Self {
        HttpClientError::InvalidHeaderValue { error: err }
    }
}

impl From<ConversionError> for HttpClientError {
    fn from(value: ConversionError) -> Self {
        match value {
            ConversionError::UnsupportedPropertyValue { property } => {
                HttpClientError::UnsupportedArgumentValue { property }
            }
            ConversionError::MissingProperty { argument } => {
                HttpClientError::MissingProperty { argument }
            }
            ConversionError::ParsingError { message } => HttpClientError::ParsingError { message },
            ConversionError::InvalidType { expected } => HttpClientError::ParsingError {
                message: format!("invalid type: expected {expected}"),
            },
        }
    }
}

impl HttpClientError {
    /// Returns true if the error is a 404 Not Found response.
    pub fn is_not_found(&self) -> bool {
        matches!(self, HttpClientError::NotFound)
            || matches!(
                self,
                HttpClientError::ClientErrorResponse { status_code, .. }
                if *status_code == StatusCode::NOT_FOUND
            )
    }

    /// Returns true if the error indicates a resource already exists (409 Conflict).
    pub fn is_already_exists(&self) -> bool {
        matches!(
            self,
            HttpClientError::ClientErrorResponse { status_code, .. }
            if *status_code == StatusCode::CONFLICT
        )
    }

    /// Returns true if the error is a 401 Unauthorized response (not authenticated).
    pub fn is_unauthorized(&self) -> bool {
        matches!(
            self,
            HttpClientError::ClientErrorResponse { status_code, .. }
            if *status_code == StatusCode::UNAUTHORIZED
        )
    }

    /// Returns true if the error is a 403 Forbidden response (authenticated but not authorized).
    pub fn is_forbidden(&self) -> bool {
        matches!(
            self,
            HttpClientError::ClientErrorResponse { status_code, .. }
            if *status_code == StatusCode::FORBIDDEN
        )
    }

    /// Returns true if the error is a client error (4xx status code).
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            HttpClientError::ClientErrorResponse { .. } | HttpClientError::NotFound
        )
    }

    /// Returns true if the error is a server error (5xx status code).
    pub fn is_server_error(&self) -> bool {
        matches!(self, HttpClientError::ServerErrorResponse { .. })
    }

    /// Returns the HTTP status code, if available.
    pub fn status_code(&self) -> Option<StatusCode> {
        match self {
            HttpClientError::ClientErrorResponse { status_code, .. } => Some(*status_code),
            HttpClientError::ServerErrorResponse { status_code, .. } => Some(*status_code),
            HttpClientError::HealthCheckFailed { status_code, .. } => Some(*status_code),
            HttpClientError::NotFound => Some(StatusCode::NOT_FOUND),
            _ => None,
        }
    }

    /// Returns the URL that caused the error, if available.
    pub fn url(&self) -> Option<&Url> {
        match self {
            HttpClientError::ClientErrorResponse { url, .. } => url.as_ref(),
            HttpClientError::ServerErrorResponse { url, .. } => url.as_ref(),
            HttpClientError::RequestError { error, .. } => error.url(),
            _ => None,
        }
    }

    /// Returns the error details from the API response, if available.
    pub fn error_details(&self) -> Option<&ErrorDetails> {
        match self {
            HttpClientError::ClientErrorResponse { error_details, .. } => error_details.as_ref(),
            HttpClientError::ServerErrorResponse { error_details, .. } => error_details.as_ref(),
            _ => None,
        }
    }

    /// Returns true if the error is a connection failure
    /// that is NOT a TLS handshake error (a hostname resolution failure, TCP connection refused, etc).
    pub fn is_connection_error(&self) -> bool {
        matches!(
            self,
            HttpClientError::RequestError { error, .. }
            if error.is_connect() && !self.is_tls_handshake_error()
        )
    }

    /// Returns true if the error is a request timeout.
    pub fn is_timeout(&self) -> bool {
        matches!(
            self,
            HttpClientError::RequestError { error, .. }
            if error.is_timeout()
        )
    }

    /// Returns true if the error is likely a TLS/SSL handshake failure.
    /// Uses a heuristic on the error chain since reqwest doesn't expose
    /// a first-class TLS error type.
    ///
    /// The patterns below are tested against rustls debug output.
    /// native-tls may produce different casing; most cases are still
    /// caught because the error chain typically includes at least one
    /// of the lowercase or variant-name matches.
    pub fn is_tls_handshake_error(&self) -> bool {
        match self {
            HttpClientError::RequestError { error, .. } if error.is_connect() => {
                let debug = format!("{error:?}");
                debug.contains("certificate")
                    || debug.contains("CertificateRequired")
                    || debug.contains("HandshakeFailure")
                    || debug.contains("InvalidCertificate")
                    || debug.contains("tls")
                    || debug.contains("TLS")
                    || debug.contains("ssl")
                    || debug.contains("SSL")
            }
            _ => false,
        }
    }

    /// Returns the underlying `reqwest::Error` for transport-level errors.
    pub fn as_reqwest_error(&self) -> Option<&reqwest::Error> {
        match self {
            HttpClientError::RequestError { error, .. } => Some(error),
            _ => None,
        }
    }

    /// Returns a user-friendly error message, preferring API-provided details when available.
    pub fn user_message(&self) -> String {
        match self {
            HttpClientError::ClientErrorResponse {
                error_details,
                status_code,
                ..
            } => {
                if let Some(details) = error_details
                    && let Some(reason) = details.reason()
                {
                    return reason.to_owned();
                }
                format!("Client error: {status_code}")
            }
            HttpClientError::ServerErrorResponse {
                error_details,
                status_code,
                ..
            } => {
                if let Some(details) = error_details
                    && let Some(reason) = details.reason()
                {
                    return reason.to_owned();
                }
                format!("Server error: {status_code}")
            }
            HttpClientError::HealthCheckFailed { details, .. } => {
                format!("Health check failed: {}", details.reason())
            }
            HttpClientError::NotFound => "Resource not found".to_owned(),
            HttpClientError::MultipleMatchingBindings => {
                "Multiple matching bindings found, provide additional properties".to_owned()
            }
            HttpClientError::InvalidHeaderValue { .. } => "Invalid header value".to_owned(),
            HttpClientError::UnsupportedArgumentValue { property } => {
                format!("Unsupported value for property: {property}")
            }
            HttpClientError::MissingProperty { argument } => {
                format!("Missing required argument: {argument}")
            }
            HttpClientError::IncompatibleBody { error, .. } => {
                format!("Response parsing error: {error}")
            }
            HttpClientError::ParsingError { message } => format!("Parsing error: {message}"),
            HttpClientError::RequestError { error, .. } => {
                format!("Request error: {error}")
            }
            HttpClientError::Other => "An unspecified error occurred".to_owned(),
        }
    }
}
