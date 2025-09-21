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

//! Async client for the RabbitMQ HTTP API

mod auth;
mod bindings;
mod channels;
mod client;
mod cluster;
mod connections;
mod consumers;
mod definitions;
mod deprecations;
mod exchanges;
mod feature_flags;
mod federation;
mod health_checks;
mod limits;
mod messages;
mod nodes;
mod overview;
mod parameters;
mod permissions;
mod policies;
mod queues_and_streams;
mod rebalancing;
mod shovels;
mod tanzu;
mod users;
mod vhosts;

// Re-export for backwards compatibility.
pub use client::{Client, ClientBuilder, HttpClientError, HttpClientResponse, Result};
