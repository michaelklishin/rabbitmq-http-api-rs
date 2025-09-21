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

//! Types in this module are used to represent API responses, such as [`QueueDefinition`], [`PolicyDefinition`],
//! [`User`], [`VirtualHost`], [`Shovel`] or [`FederationLink`].

use std::fmt;
use std::ops::{Deref, DerefMut};

use crate::formatting::*;
use serde::{
    Deserialize, Serialize,
    de::{MapAccess, Visitor},
};
use serde_json::Map;

#[cfg(feature = "tabled")]
use tabled::Tabled;

pub mod federation;
pub use federation::{FederationLink, FederationType, FederationUpstream};

pub mod shovel;
pub use shovel::{Shovel, ShovelPublishingState, ShovelState, ShovelType};

pub mod feature_flags;
pub use feature_flags::{FeatureFlag, FeatureFlagList, FeatureFlagStability, FeatureFlagState};

pub mod deprecations;
pub use deprecations::{DeprecatedFeature, DeprecatedFeatureList, DeprecationPhase};

pub mod health_checks;
pub use health_checks::{
    ClusterAlarmCheckDetails, HealthCheckFailureDetails, NoActivePortListenerDetails,
    NoActiveProtocolListenerDetails41AndLater, NoActiveProtocolListenerDetailsPre41,
    QuorumCriticalityCheckDetails, QuorumEndangeredQueue, ResourceAlarm,
};

pub mod parameters;
pub use parameters::{
    GlobalRuntimeParameter, GlobalRuntimeParameterValue, RuntimeParameter, RuntimeParameterValue,
    RuntimeParameterWithoutVirtualHost,
};

pub mod policies;
pub use policies::{Policy, PolicyDefinition, PolicyWithoutVirtualHost};

pub mod definitions;
pub use definitions::{
    BindingDefinition, BindingDefinitionWithoutVirtualHost, BindingInfo,
    BindingInfoWithoutVirtualHost, ClusterDefinitionSet, ExchangeDefinition,
    ExchangeDefinitionWithoutVirtualHost, ExchangeInfo, ExchangeInfoWithoutVirtualHost,
    NamedPolicyTargetObject, OptionalArgumentSourceOps, QueueDefinition,
    QueueDefinitionWithoutVirtualHost, QueueOps, VirtualHostDefinitionSet, XArguments,
};

pub mod tanzu;
pub use tanzu::{
    HostnamePortPairs, MessagingProtocol, OperatingMode, SchemaDefinitionSyncState,
    SchemaDefinitionSyncStatus, WarmStandbyReplicationInVirtualHost,
    WarmStandbyReplicationLinkStateOnDownstream, WarmStandbyReplicationState,
    WarmStandbyReplicationStateOnUpstream, WarmStandbyReplicationStatus,
};

pub mod vhosts;
pub use vhosts::{EnforcedLimits, VirtualHost, VirtualHostLimits, VirtualHostMetadata};

pub mod connections;
pub use connections::{
    ClientCapabilities, ClientProperties, Connection, ConnectionDetails, UserConnection,
};

pub mod channels;
pub use channels::{Channel, ChannelDetails, ChannelState};

pub mod cluster;
pub use cluster::{
    AuthenticationAttemptStatistics, ChurnRates, ClusterIdentity, ClusterNode, ClusterTags,
    GarbageCollectionDetails, Listener, NodeList, NodeMemoryBreakdown, NodeMemoryFootprint,
    NodeMemoryTotals, Overview,
};

pub mod permissions;
pub use permissions::{Permissions, TopicPermission};

pub mod queues_and_streams;
pub use queues_and_streams::{
    DetailedQueueInfo, NameAndVirtualHost, QueueInfo, StreamConsumer, StreamPublisher,
};

pub mod consumers;
pub use consumers::Consumer;

pub mod users;
pub use users::{CurrentUser, User, UserLimits};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct TagList(pub Vec<String>);

impl TagList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.iter().any(|s| s == key)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, String> {
        self.0.iter_mut()
    }
}

impl Deref for TagList {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TagList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for TagList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub struct PluginList(pub Vec<String>);

impl PluginList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn contains(&self, key: &str) -> bool {
        self.0.iter().any(|s| s == key)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, String> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, String> {
        self.0.iter_mut()
    }
}

impl Deref for PluginList {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PluginList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for PluginList {
    type Item = String;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Represents a number of key OAuth 2 configuration settings.
#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct OAuthConfiguration {
    pub oauth_enabled: bool,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub oauth_client_id: Option<String>,
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub oauth_provider_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
#[allow(dead_code)]
pub struct GetMessage {
    pub payload_bytes: u32,
    pub redelivered: bool,
    pub exchange: String,
    pub routing_key: String,
    pub message_count: u32,
    #[serde(deserialize_with = "deserialize_message_properties")]
    pub properties: MessageProperties,
    pub payload: String,
    pub payload_encoding: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(transparent)]
pub struct MessageList(pub Vec<GetMessage>);

impl MessageList {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for MessageList {
    type Target = Vec<GetMessage>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for MessageList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl IntoIterator for MessageList {
    type Item = GetMessage;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[allow(clippy::partialeq_ne_impl)]
impl PartialEq for MessageList {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }

    fn ne(&self, other: &Self) -> bool {
        self.0.ne(&other.0)
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct MessageRouted {
    pub routed: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct MessageProperties(pub Map<String, serde_json::Value>);

#[derive(Debug, Deserialize, Clone, PartialEq, PartialOrd, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct Rate {
    pub rate: f64,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct ObjectTotals {
    pub connections: u64,
    pub channels: u64,
    pub queues: u64,
    pub exchanges: u64,
    pub consumers: u64,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[serde(default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct QueueTotals {
    pub messages: u64,
    #[serde(rename = "messages_ready", default)]
    pub messages_ready_for_delivery: u64,
    #[serde(rename = "messages_unacknowledged", default)]
    pub messages_delivered_but_unacknowledged_by_consumers: u64,
    pub messages_details: Rate,
    #[serde(rename = "messages_ready_details", default)]
    pub messages_ready_for_delivery_details: Rate,
    #[serde(rename = "messages_unacknowledged_details", default)]
    pub messages_delivered_but_unacknowledged_by_consumers_details: Rate,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Default)]
#[cfg_attr(feature = "tabled", derive(Tabled))]
pub struct MessageStats {
    /// Consumer delivery rate plus polling (via 'basic.get') rate
    #[serde(rename = "deliver_get_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub delivery_details: Option<Rate>,
    #[serde(rename = "publish_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub publishing_details: Option<Rate>,

    #[serde(rename = "deliver_no_ack_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub delivery_with_automatic_acknowledgement_details: Option<Rate>,
    #[serde(rename = "redeliver_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub redelivery_details: Option<Rate>,

    #[serde(rename = "confirm_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub publisher_confirmation_details: Option<Rate>,
    #[serde(rename = "ack_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub consumer_acknowledgement_details: Option<Rate>,

    #[serde(rename = "drop_unroutable_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub unroutable_dropped_message_details: Option<Rate>,
    #[serde(rename = "return_unroutable_details", default)]
    #[cfg_attr(feature = "tabled", tabled(display = "display_option"))]
    pub unroutable_returned_message_details: Option<Rate>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(transparent)]
pub struct TagMap(pub Map<String, serde_json::Value>);

//
// Implementation
//

fn deserialize_map_or_seq<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    struct MapVisitor<T> {
        default: T,
    }

    impl<'de, T: serde::Deserialize<'de>> Visitor<'de> for MapVisitor<T> {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map")
        }

        fn visit_seq<A>(self, _seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            // Treat a sequence as the default for the type.
            Ok(self.default)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let deserializer = serde::de::value::MapAccessDeserializer::new(map);
            let m = Deserialize::deserialize(deserializer)?;
            Ok(m)
        }
    }

    deserializer.deserialize_any(MapVisitor {
        default: T::default(),
    })
}

fn deserialize_message_properties<'de, D>(deserializer: D) -> Result<MessageProperties, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_map_or_seq::<MessageProperties, D>(deserializer)
}

pub fn deserialize_object_that_may_be_empty<'de, D, T>(
    deserializer: D,
) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    #[derive(Deserialize)]
    #[serde(
        untagged,
        deny_unknown_fields,
        expecting = "object, empty object or null"
    )]
    enum Helper<T> {
        Data(T),
        Empty {},
        Null,
    }
    match Helper::deserialize(deserializer) {
        Ok(Helper::Data(data)) => Ok(Some(data)),
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}
