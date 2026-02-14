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

use std::fmt::Display;
use std::time::Instant;

use crate::responses::{ReachabilityProbeDetails, ReachabilityProbeOutcome};

use super::client::Client;

impl<E, U, P> Client<E, U, P>
where
    E: Display,
    U: Display,
    P: Display,
{
    /// Tests whether the node is reachable and the configured credentials are accepted.
    ///
    /// Calls `GET /api/whoami`. Does not check node health, cluster identity,
    /// or resource alarms — the caller can compose those after a `Reached` result.
    pub async fn probe_reachability(&self) -> ReachabilityProbeOutcome {
        let start = Instant::now();

        match self.current_user().await {
            Ok(current_user) => ReachabilityProbeOutcome::Reached(ReachabilityProbeDetails {
                current_user,
                duration: start.elapsed(),
            }),
            Err(e) => ReachabilityProbeOutcome::Unreachable(Box::new(e)),
        }
    }
}
