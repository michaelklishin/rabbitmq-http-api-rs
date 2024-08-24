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

/// The primary API: a blocking HTTP API client
pub mod blocking;
/// Types commonly used by API requests and responses
pub mod commons;
/// Providers password hashing utilities for user pre-seeding.
pub mod password_hashing;
/// Types used to issues API requests (such as `PUT`, `POST`, `DELETE`)
pub mod requests;
/// API response types
pub mod responses;
