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
use crate::responses;
use reqwest::header::InvalidHeaderValue;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error<R, S, E, BT> {
    #[error("API responded with a client error: status code of {status_code}")]
    ClientErrorResponse {
        status_code: S,
        response: Option<R>,
        backtrace: BT,
    },
    #[error("API responded with a server error: status code of {status_code}")]
    ServerErrorResponse {
        status_code: S,
        response: Option<R>,
        backtrace: BT,
    },
    #[error("Health check failed")]
    HealthCheckFailed {
        path: String,
        details: responses::HealthCheckFailureDetails,
        status_code: S,
    },
    #[error("Could not find the requested resource")]
    NotFound,
    #[error("Cannot delete a binding: multiple matching bindings were found, provide additional properties")]
    MultipleMatchingBindings,
    #[error("could not convert provided value into an HTTP header value")]
    InvalidHeaderValue { error: InvalidHeaderValue },
    #[error("encountered an error when performing an HTTP request")]
    RequestError { error: E, backtrace: BT },
    #[error("an unspecified error")]
    Other,
}
