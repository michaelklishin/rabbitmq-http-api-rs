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
        }
    }
}
