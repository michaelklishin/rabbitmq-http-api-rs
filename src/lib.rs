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
#![allow(rustdoc::bare_urls)]
#![allow(clippy::result_large_err)]
//! # Rust client for the RabbitMQ HTTP API
//!
//! This library is a Rust client for the [RabbitMQ HTTP API](https://rabbitmq.com/docs/management/#http-api).
//!
//! It can be used for [monitoring](https://www.rabbitmq.com/docs/monitoring) and automation of provisioning or maintenance of RabbitMQ clusters
//! and topologies used by applications.
//!
//! There are two variants of this client:
//! 1. [`blocking_api::Client`] is the synchronous (blocking) version of the client
//! 2. [`api::Client`] is the async version
//!
//! ## License
//!
//! This library is double licensed under the Apache 2.0 and MIT licenses.
//! This means that the user can choose either of the licenses.

/// The primary API: an async HTTP API client
#[cfg(feature = "async")]
pub mod api;
/// The primary API: a blocking HTTP API client
#[cfg(feature = "blocking")]
pub mod blocking_api;
/// Types common between both API [`requests`] and [`responses`]
pub mod commons;
/// Formatting helpers
pub mod formatting;
/// Provides password hashing utilities for [user pre-seeding](https://www.rabbitmq.com/docs/access-control#seeding)
pub mod password_hashing;
pub mod requests;
pub mod responses;

/// Error types
#[cfg(any(feature = "async", feature = "blocking"))]
pub mod error;

/// Transformers are functions that mutate (transform) [definition sets](https://www.rabbitmq.com/docs/definitions)
pub mod transformers;

/// Builder for the RabbitMQ-specific URIs
pub mod uris;
mod utils;
