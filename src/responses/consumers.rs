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

use crate::responses::{
    ChannelDetails, definitions::XArguments, deserialize_object_that_may_be_empty,
    queues_and_streams::NameAndVirtualHost,
};
use serde::Deserialize;

#[cfg(feature = "tabled")]
use std::borrow::Cow;
#[cfg(feature = "tabled")]
use tabled::Tabled;

#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Consumer {
    pub consumer_tag: String,
    pub active: bool,
    #[serde(rename(deserialize = "ack_required"))]
    pub manual_ack: bool,
    pub prefetch_count: u32,
    pub exclusive: bool,
    pub arguments: XArguments,
    #[serde(rename(deserialize = "consumer_timeout"))]
    pub delivery_ack_timeout: u64,
    pub queue: NameAndVirtualHost,

    #[serde(deserialize_with = "deserialize_object_that_may_be_empty")]
    pub channel_details: Option<ChannelDetails>,
}

#[cfg(feature = "tabled")]
impl Tabled for Consumer {
    const LENGTH: usize = 9;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        let mut fds: Vec<Cow<'static, str>> = Vec::with_capacity(Self::LENGTH);
        let qinfo = &self.queue;
        fds.push(Cow::Owned(qinfo.vhost.clone()));
        fds.push(Cow::Owned(qinfo.name.clone()));
        fds.push(Cow::Owned(self.consumer_tag.clone()));
        fds.push(Cow::Owned(self.manual_ack.to_string()));
        fds.push(Cow::Owned(self.prefetch_count.to_string()));
        fds.push(Cow::Owned(self.active.to_string()));
        fds.push(Cow::Owned(self.exclusive.to_string()));
        fds.push(Cow::Owned(self.arguments.to_string()));
        fds.push(Cow::Owned(self.delivery_ack_timeout.to_string()));

        fds
    }

    fn headers() -> Vec<Cow<'static, str>> {
        let mut hds: Vec<Cow<'static, str>> = Vec::with_capacity(Self::LENGTH);
        hds.push(Cow::Borrowed("vhost"));
        hds.push(Cow::Borrowed("queue"));
        hds.push(Cow::Borrowed("consumer_tag"));
        hds.push(Cow::Borrowed("manual_ack"));
        hds.push(Cow::Borrowed("prefetch_count"));
        hds.push(Cow::Borrowed("active"));
        hds.push(Cow::Borrowed("exclusive"));
        hds.push(Cow::Borrowed("arguments"));
        hds.push(Cow::Borrowed("delivery_ack_timeout"));

        hds
    }
}
