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

use std::time::Duration;

use super::users::CurrentUser;
use crate::error::HttpClientError;

/// Details from a successful reachability probe.
#[derive(Debug, Clone)]
pub struct ReachabilityProbeDetails {
    /// The authenticated user on this node.
    pub current_user: CurrentUser,
    /// Wall-clock time for the probe request.
    pub duration: Duration,
}

/// Outcome of a reachability probe.
#[derive(Debug)]
pub enum ReachabilityProbeOutcome {
    /// The node is reachable and the credentials were accepted.
    Reached(ReachabilityProbeDetails),
    /// Could not reach or authenticate to the node.
    Unreachable(Box<HttpClientError>),
}
