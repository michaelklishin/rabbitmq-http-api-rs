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

use crate::commons::{ChannelUseMode, MessageTransferAcknowledgementMode, QueueType};
use crate::requests::parameters::RuntimeParameterDefinition;
use serde::{Deserialize, Serialize};
use serde_json::{Map, json};
use std::fmt::{Display, Formatter};

/// Controls when federation resources (temporary queues/exchanges) are cleaned up.
///
/// Federation creates temporary resources on the downstream cluster. This enum controls
/// when these resources are removed to prevent resource leaks.
#[derive(Default, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FederationResourceCleanupMode {
    /// Default cleanup behavior: resources are cleaned up when the federation link is stopped
    #[default]
    Default,
    /// Never clean up federation resources automatically (manual cleanup required)
    Never,
}

impl From<&str> for FederationResourceCleanupMode {
    fn from(value: &str) -> Self {
        match value {
            "default" => FederationResourceCleanupMode::Default,
            "never" => FederationResourceCleanupMode::Never,
            _ => FederationResourceCleanupMode::default(),
        }
    }
}

impl From<String> for FederationResourceCleanupMode {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl Display for FederationResourceCleanupMode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FederationResourceCleanupMode::Default => write!(f, "default"),
            FederationResourceCleanupMode::Never => write!(f, "never"),
        }
    }
}

/// [Runtime parameter](https://www.rabbitmq.com/docs/parameters) component name used by federation upstreams.
///
/// This constant is used internally when creating [`RuntimeParameterDefinition`]
/// instances for federation upstreams.
pub const FEDERATION_UPSTREAM_COMPONENT: &str = "federation-upstream";

/// Parameters specific to [queue federation](https://www.rabbitmq.com/docs/federated-queues).
#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct QueueFederationParams<'a> {
    /// Name of the upstream queue to federate from (None uses the same name as downstream)
    pub queue: Option<&'a str>,
    /// Consumer tag for the federation link (None uses auto-generated tag)
    pub consumer_tag: Option<&'a str>,
}

impl<'a> QueueFederationParams<'a> {
    /// Returns queue federation parameters with a specific upstream queue name.
    ///
    /// Use this when the upstream queue has a different name than the downstream queue.
    pub fn new(queue: &'a str) -> Self {
        Self {
            queue: Some(queue),
            consumer_tag: None,
        }
    }

    /// Returns queue federation parameters with both queue name and consumer tag.
    ///
    /// Use this when you need to specify both the upstream queue name and a custom
    /// consumer tag for identification and management purposes.
    pub fn new_with_consumer_tag(queue: &'a str, consumer_tag: &'a str) -> Self {
        Self {
            queue: Some(queue),
            consumer_tag: Some(consumer_tag),
        }
    }
}

/// Parameters specific to [exchange federation](https://www.rabbitmq.com/docs/federated-exchanges).
#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ExchangeFederationParams<'a> {
    /// Name of the upstream exchange to federate from (None uses the same name as downstream)
    pub exchange: Option<&'a str>,
    /// Maximum hops for federation chains to prevent infinite loops
    pub max_hops: Option<u8>,
    /// Queue type for the temporary federation queue
    pub queue_type: QueueType,
    /// Time-to-live for the federation queue in milliseconds
    pub ttl: Option<u32>,
    /// Message TTL for federated messages in milliseconds
    pub message_ttl: Option<u32>,
    /// When to clean up temporary federation resources
    pub resource_cleanup_mode: FederationResourceCleanupMode,
}

impl ExchangeFederationParams<'_> {
    /// Returns exchange federation parameters with the specified queue type.
    ///
    /// The queue type determines the characteristics of the temporary queue used
    /// for the federation link. Use quorum queues for durability or classic for simplicity.
    pub fn new(queue_type: QueueType) -> Self {
        Self {
            exchange: None,
            max_hops: None,
            queue_type,
            ttl: None,
            message_ttl: None,
            resource_cleanup_mode: FederationResourceCleanupMode::default(),
        }
    }
}

/// Matches the default used by the federation plugin.
pub const DEFAULT_FEDERATION_PREFETCH: u32 = 1000;
/// Matches the default used by the federation plugin.
pub const DEFAULT_FEDERATION_RECONNECT_DELAY: u32 = 5;

/// Represents a set of parameters that define a federation upstream
/// and a number of the federation type-specific (exchange, queue) parameters
/// that are associated with an upstream.
///
/// A federation upstream is declared as a runtime parameter,
/// therefore this type implements a conversion that is used
/// by [`crate::api::Client#declare_federation_upstream`] and [`crate::blocking_api::Client#declare_federation_upstream`]
#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct FederationUpstreamParams<'a> {
    pub name: &'a str,
    pub vhost: &'a str,
    pub uri: &'a str,
    pub reconnect_delay: u32,
    pub trust_user_id: bool,
    pub prefetch_count: u32,
    pub ack_mode: MessageTransferAcknowledgementMode,
    pub bind_using_nowait: bool,
    pub channel_use_mode: ChannelUseMode,

    pub queue_federation: Option<QueueFederationParams<'a>>,
    pub exchange_federation: Option<ExchangeFederationParams<'a>>,
}

impl<'a> FederationUpstreamParams<'a> {
    /// Creates a federation upstream that will be used for [queue federation](https://www.rabbitmq.com/docs/federated-queues).
    pub fn new_queue_federation_upstream(
        vhost: &'a str,
        name: &'a str,
        uri: &'a str,
        params: QueueFederationParams<'a>,
    ) -> Self {
        Self {
            vhost,
            name,
            uri,
            ack_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
            reconnect_delay: DEFAULT_FEDERATION_RECONNECT_DELAY,
            trust_user_id: false,
            prefetch_count: DEFAULT_FEDERATION_PREFETCH,
            bind_using_nowait: false,
            channel_use_mode: ChannelUseMode::default(),
            exchange_federation: None,
            queue_federation: Some(params),
        }
    }

    /// Creates a federation upstream that will be used for [exchange federation](https://www.rabbitmq.com/docs/federated-exchanges).
    pub fn new_exchange_federation_upstream(
        vhost: &'a str,
        name: &'a str,
        uri: &'a str,
        params: ExchangeFederationParams<'a>,
    ) -> Self {
        Self {
            vhost,
            name,
            uri,
            ack_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
            reconnect_delay: DEFAULT_FEDERATION_RECONNECT_DELAY,
            trust_user_id: false,
            prefetch_count: DEFAULT_FEDERATION_PREFETCH,
            bind_using_nowait: false,
            channel_use_mode: ChannelUseMode::default(),
            queue_federation: None,
            exchange_federation: Some(params),
        }
    }
}

impl<'a> From<FederationUpstreamParams<'a>> for RuntimeParameterDefinition<'a> {
    fn from(params: FederationUpstreamParams<'a>) -> Self {
        let mut value = Map::new();

        value.insert("uri".to_owned(), json!(params.uri));
        value.insert("prefetch-count".to_owned(), json!(params.prefetch_count));
        value.insert("trust-user-id".to_owned(), json!(params.trust_user_id));
        value.insert("reconnect-delay".to_owned(), json!(params.reconnect_delay));
        value.insert("ack-mode".to_owned(), json!(params.ack_mode));
        value.insert("bind-nowait".to_owned(), json!(params.bind_using_nowait));
        value.insert(
            "channel-use-mode".to_owned(),
            json!(params.channel_use_mode),
        );

        if let Some(qf) = params.queue_federation {
            value.insert("queue".to_owned(), json!(qf.queue));
            if let Some(val) = qf.consumer_tag {
                value.insert("consumer-tag".to_owned(), json!(val));
            }
        }

        if let Some(ef) = params.exchange_federation {
            value.insert("queue-type".to_owned(), json!(ef.queue_type));
            value.insert(
                "resource-cleanup-mode".to_owned(),
                json!(ef.resource_cleanup_mode),
            );

            if let Some(val) = ef.exchange {
                value.insert("exchange".to_owned(), json!(val));
            };
            if let Some(val) = ef.max_hops {
                value.insert("max-hops".to_owned(), json!(val));
            }
            if let Some(val) = ef.ttl {
                value.insert("expires".to_owned(), json!(val));
            }
            if let Some(val) = ef.message_ttl {
                value.insert("message-ttl".to_owned(), json!(val));
            }
        }

        Self {
            name: params.name,
            vhost: params.vhost,
            component: FEDERATION_UPSTREAM_COMPONENT,
            value,
        }
    }
}

/// This auxiliary type is an owned version of FederationUpstreamParams.
/// This is one of the simplest approaches to conversion because
/// of [`FederationUpstreamParams`]'s lifetimes.
#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct OwnedFederationUpstreamParams {
    pub name: String,
    pub vhost: String,
    pub uri: String,
    pub reconnect_delay: u32,
    pub trust_user_id: bool,
    pub prefetch_count: u32,
    pub ack_mode: MessageTransferAcknowledgementMode,
    pub bind_using_nowait: bool,
    pub channel_use_mode: ChannelUseMode,

    pub queue_federation: Option<OwnedQueueFederationParams>,
    pub exchange_federation: Option<OwnedExchangeFederationParams>,
}

/// Owned version of QueueFederationParams
#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct OwnedQueueFederationParams {
    pub queue: Option<String>,
    pub consumer_tag: Option<String>,
}

/// Owned version of ExchangeFederationParams
#[derive(Default, Debug, Serialize, Clone, PartialEq, Eq)]
pub struct OwnedExchangeFederationParams {
    pub exchange: Option<String>,
    pub max_hops: Option<u8>,
    pub queue_type: QueueType,
    pub ttl: Option<u32>,
    pub message_ttl: Option<u32>,
    pub resource_cleanup_mode: FederationResourceCleanupMode,
}

impl From<crate::responses::FederationUpstream> for OwnedFederationUpstreamParams {
    fn from(upstream: crate::responses::FederationUpstream) -> Self {
        // Create queue federation parameters if queue-related fields are present
        let queue_federation = if upstream.queue.is_some() || upstream.consumer_tag.is_some() {
            Some(OwnedQueueFederationParams {
                queue: upstream.queue,
                consumer_tag: upstream.consumer_tag,
            })
        } else {
            None
        };

        // Create exchange federation parameters if exchange-related fields are present
        let exchange_federation = if upstream.exchange.is_some()
            || upstream.max_hops.is_some()
            || upstream.queue_type.is_some()
            || upstream.expires.is_some()
            || upstream.message_ttl.is_some()
            || upstream.resource_cleanup_mode != FederationResourceCleanupMode::default()
        {
            Some(OwnedExchangeFederationParams {
                exchange: upstream.exchange,
                max_hops: upstream.max_hops,
                queue_type: upstream.queue_type.unwrap_or_default(),
                ttl: upstream.expires,
                message_ttl: upstream.message_ttl,
                resource_cleanup_mode: upstream.resource_cleanup_mode,
            })
        } else {
            None
        };

        Self {
            name: upstream.name,
            vhost: upstream.vhost,
            uri: upstream.uri,
            reconnect_delay: upstream
                .reconnect_delay
                .unwrap_or(DEFAULT_FEDERATION_RECONNECT_DELAY),
            trust_user_id: upstream.trust_user_id.unwrap_or(false),
            ack_mode: upstream.ack_mode,
            prefetch_count: upstream
                .prefetch_count
                .unwrap_or(DEFAULT_FEDERATION_PREFETCH),
            bind_using_nowait: upstream.bind_using_nowait,
            channel_use_mode: upstream.channel_use_mode,
            queue_federation,
            exchange_federation,
        }
    }
}

impl<'a> From<&'a OwnedFederationUpstreamParams> for FederationUpstreamParams<'a> {
    fn from(owned: &'a OwnedFederationUpstreamParams) -> Self {
        let queue_federation = owned
            .queue_federation
            .as_ref()
            .map(|qf| QueueFederationParams {
                queue: qf.queue.as_deref(),
                consumer_tag: qf.consumer_tag.as_deref(),
            });

        let exchange_federation =
            owned
                .exchange_federation
                .as_ref()
                .map(|ef| ExchangeFederationParams {
                    exchange: ef.exchange.as_deref(),
                    max_hops: ef.max_hops,
                    queue_type: ef.queue_type.clone(),
                    ttl: ef.ttl,
                    message_ttl: ef.message_ttl,
                    resource_cleanup_mode: ef.resource_cleanup_mode.clone(),
                });

        Self {
            name: &owned.name,
            vhost: &owned.vhost,
            uri: &owned.uri,
            reconnect_delay: owned.reconnect_delay,
            trust_user_id: owned.trust_user_id,
            prefetch_count: owned.prefetch_count,
            ack_mode: owned.ack_mode.clone(),
            bind_using_nowait: owned.bind_using_nowait,
            channel_use_mode: owned.channel_use_mode.clone(),
            queue_federation,
            exchange_federation,
        }
    }
}
