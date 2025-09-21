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

use crate::commons::VirtualHostName;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[serde(untagged)]
pub enum HealthCheckFailureDetails {
    AlarmCheck(ClusterAlarmCheckDetails),
    NodeIsQuorumCritical(QuorumCriticalityCheckDetails),
    NoActivePortListener(NoActivePortListenerDetails),
    /// The pre-RabbitMQ 4.1 variant that reports a single missing listener
    NoActiveProtocolListener(NoActiveProtocolListenerDetailsPre41),
    /// The RabbitMQ 4.1+ variant that reports an array of missing listeners
    NoActiveProtocolListeners(NoActiveProtocolListenerDetails41AndLater),
}

impl HealthCheckFailureDetails {
    pub fn reason(&self) -> String {
        match self {
            HealthCheckFailureDetails::AlarmCheck(details) => details.reason.clone(),
            HealthCheckFailureDetails::NodeIsQuorumCritical(details) => details.reason.clone(),
            HealthCheckFailureDetails::NoActivePortListener(details) => details.reason.clone(),
            HealthCheckFailureDetails::NoActiveProtocolListener(details) => details.reason.clone(),
            HealthCheckFailureDetails::NoActiveProtocolListeners(details) => details.reason.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct ClusterAlarmCheckDetails {
    pub reason: String,
    pub alarms: Vec<ResourceAlarm>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct ResourceAlarm {
    pub node: String,
    pub resource: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct QuorumCriticalityCheckDetails {
    pub reason: String,
    pub queues: Vec<QuorumEndangeredQueue>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct NoActivePortListenerDetails {
    pub status: String,
    pub reason: String,
    #[serde(rename(deserialize = "missing"))]
    #[serde(default)]
    pub inactive_port: u16,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct NoActiveProtocolListenerDetailsPre41 {
    pub status: String,
    pub reason: String,
    #[serde(rename(deserialize = "protocols"))]
    pub active_protocols: Vec<String>,
    #[serde(rename(deserialize = "missing"))]
    // Note: switching this to SupportedProtocol will break serde's
    //       detection of various HealthCheckFailureDetails variants since
    //       that enum is untagged
    pub inactive_protocol: String,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct NoActiveProtocolListenerDetails41AndLater {
    pub status: String,
    pub reason: String,
    #[serde(rename(deserialize = "protocols"))]
    pub active_protocols: Vec<String>,
    #[serde(rename(deserialize = "missing"))]
    // Note: switching this to SupportedProtocol will break serde's
    //       detection of various HealthCheckFailureDetails variants since
    //       that enum is untagged
    pub inactive_protocols: Vec<String>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct QuorumEndangeredQueue {
    pub name: String,
    pub readable_name: String,
    #[serde(rename(deserialize = "virtual_host"))]
    pub vhost: VirtualHostName,
    #[serde(rename(deserialize = "type"))]
    pub queue_type: String,
}
