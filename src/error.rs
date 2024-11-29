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
use reqwest::{header::InvalidHeaderValue, StatusCode};
use thiserror::Error;

use crate::responses;

#[derive(Error, Debug)]
pub enum Error<Response> {
    #[error("encountered an error when performing an HTTP request")]
    RequestError(#[from] reqwest::Error),
    #[error("API responded with a client error: status code of {0}: {1}")]
    ClientErrorResponse(StatusCode, Response),
    #[error("API responded with a server error: status code of {0}")]
    ServerErrorResponse(StatusCode, Response),
    #[error("Health check failed")]
    HealthCheckFailed(responses::HealthCheckFailureDetails),
    #[error("Could not find the requested resource")]
    NotFound(),
    #[error("Cannot delete a binding: multiple matching bindings were found, provide additional properties")]
    ManyMatchingBindings(),
    #[error("could not convert provided value into an HTTP header value")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),
    #[error("an unspecified error")]
    Other,
}
