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

use serde::Deserialize;
use std::fmt;
#[cfg(feature = "tabled")]
use tabled::Tabled;

use crate::commons::{
    ChannelUseMode, MessageTransferAcknowledgementMode, QueueType, VirtualHostName,
};
use crate::error::ConversionError;
use crate::formatting::*;
use crate::requests::federation::FederationResourceCleanupMode;
use crate::responses::RuntimeParameter;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)]
pub struct FederationUpstream {
    pub name: String,
    pub vhost: VirtualHostName,
    pub uri: String,
    pub ack_mode: MessageTransferAcknowledgementMode,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub prefetch_count: Option<u32>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub trust_user_id: Option<bool>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub reconnect_delay: Option<u32>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub queue: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub consumer_tag: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub exchange: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub max_hops: Option<u8>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub queue_type: Option<QueueType>,
    #[cfg_attr(
        feature = "tabled",
        tabled(display = "display_option", rename = "expires (queue TTL)")
    )]
    pub expires: Option<u32>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub message_ttl: Option<u32>,
    pub resource_cleanup_mode: FederationResourceCleanupMode,
    pub bind_using_nowait: bool,
    pub channel_use_mode: ChannelUseMode,
}

impl TryFrom<RuntimeParameter> for FederationUpstream {
    type Error = ConversionError;

    fn try_from(param: RuntimeParameter) -> Result<Self, Self::Error> {
        let values = &param.value.0;

        let uri = values
            .get("uri")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ConversionError::MissingProperty {
                argument: "uri".to_string(),
            })?
            .to_string();

        let ack_mode = values
            .get("ack-mode")
            .and_then(|v| v.as_str())
            .map(MessageTransferAcknowledgementMode::from)
            .unwrap_or_default();
        let prefetch_count = values
            .get("prefetch-count")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let trust_user_id = values.get("trust-user-id").and_then(|v| v.as_bool());
        let bind_using_nowait = values
            .get("bind-nowait")
            .and_then(|v| v.as_bool())
            .unwrap_or_default();
        let reconnect_delay = values
            .get("reconnect-delay")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let queue = values
            .get("queue")
            .and_then(|v| v.as_str())
            .map(String::from);
        let consumer_tag = values
            .get("consumer-tag")
            .and_then(|v| v.as_str())
            .map(String::from);
        let exchange = values
            .get("exchange")
            .and_then(|v| v.as_str())
            .map(String::from);
        let max_hops = values
            .get("max-hops")
            .and_then(|v| v.as_u64())
            .map(|v| v as u8);
        let queue_type = values
            .get("queue-type")
            .and_then(|v| v.as_str())
            .map(QueueType::from);
        let expires = values
            .get("expires")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        let message_ttl = values
            .get("message-ttl")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);

        let resource_cleanup_mode = values
            .get("resource-cleanup-mode")
            .and_then(|v| v.as_str())
            .map(FederationResourceCleanupMode::from)
            .unwrap_or_default();
        let channel_use_mode = values
            .get("channel-use-mode")
            .and_then(|v| v.as_str())
            .map(ChannelUseMode::from)
            .unwrap_or_default();

        Ok(FederationUpstream {
            name: param.name,
            vhost: param.vhost,
            uri,
            ack_mode,
            prefetch_count,
            trust_user_id,
            reconnect_delay,
            queue,
            consumer_tag,
            exchange,
            max_hops,
            queue_type,
            expires,
            message_ttl,
            resource_cleanup_mode,
            bind_using_nowait,
            channel_use_mode,
        })
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum FederationType {
    #[default]
    Exchange,
    Queue,
}

impl fmt::Display for FederationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FederationType::Exchange => write!(f, "exchange"),
            FederationType::Queue => write!(f, "queue"),
        }
    }
}

impl From<String> for FederationType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "exchange" => FederationType::Exchange,
            "queue" => FederationType::Queue,
            _ => FederationType::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct FederationLink {
    pub node: String,
    pub vhost: VirtualHostName,
    pub id: String,
    pub uri: String,
    pub status: String,
    #[serde(rename = "type")]
    #[cfg_attr(feature = "tabled", tabled(rename = "type"))]
    pub typ: FederationType,
    pub upstream: String,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub consumer_tag: Option<String>,
}
