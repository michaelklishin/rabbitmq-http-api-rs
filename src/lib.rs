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
#![allow(rustdoc::bare_urls)]
//! # Rust client for the RabbitMQ HTTP API
//!
//! This library is Rust client for the [RabbitMQ HTTP API](https://rabbitmq.com/docs/management/#http-api).
//!
//! It can be used for [monitoring](https://www.rabbitmq.com/monitoring.html) and automation of provisioning or maintenance of RabbitMQ clusters
//! and topologies used by applications.
//!
//! ## License
//!
//! This library is double licensed under the Apache 2.0 and MIT licenses.
//! This means that the user can choose either of the licenses.

extern crate alloc;
extern crate core;

/// The primary API: a async HTTP API client
#[cfg(feature = "async")]
pub mod api;
/// The primary API: a blocking HTTP API client
#[cfg(feature = "blocking")]
pub mod blocking_api;
/// Types commonly used by API requests and responses
pub mod commons;
/// Providers password hashing utilities for user pre-seeding.
pub mod password_hashing;
/// Types used to issues API requests (such as `PUT`, `POST`, `DELETE`)
pub mod requests;
/// API response types
pub mod responses;

/// Error
#[cfg(any(feature = "async", feature = "blocking"))]
pub mod error;
#[cfg(any(feature = "async", feature = "blocking"))]
mod utils;
