use reqwest::{header::InvalidHeaderValue, StatusCode};
use thiserror::Error;

use crate::responses;

#[derive(Error, Debug)]
pub enum Error<Respone> {
    #[error("encountered an error when performing an HTTP request")]
    RequestError(#[from] reqwest::Error),
    #[error("API responded with a client error: status code of {0}")]
    ClientErrorResponse(StatusCode, Respone),
    #[error("API responded with a server error: status code of {0}")]
    ServerErrorResponse(StatusCode, Respone),
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
