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
use crate::responses::*;
use serde_json::Map;
use std::fmt;
use std::fmt::Display;

impl Display for ObjectTotals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "connections: {}", self.connections)?;
        writeln!(f, "channels: {}", self.channels)?;
        writeln!(f, "queues: {}", self.queues)?;
        writeln!(f, "exchanges: {}", self.exchanges)?;

        Ok(())
    }
}

impl Display for Rate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:.2}", self.rate)?;
        Ok(())
    }
}

impl Display for QueueTotals {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "all messages: {}", self.messages)?;
        writeln!(
            f,
            "messages ready for dlievery: {}",
            self.messages_ready_for_delivery
        )?;
        writeln!(
            f,
            "messages delivered but unacknowledged by consumer: {}",
            self.messages_delivered_but_unacknowledged_by_consumers
        )?;

        Ok(())
    }
}

impl Display for TagList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_comma_separated_list(f, &self.0)
    }
}

impl Display for TagMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
    }
}

impl Display for DeprecatedFeatureList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for df in &self.0 {
            writeln!(f, "{}", df)?;
        }

        Ok(())
    }
}

impl Display for DeprecatedFeature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: {}", self.name)?;
        writeln!(f, "description: {}", self.description)?;
        writeln!(f, "deprecation_phase: {}", self.deprecation_phase)?;

        Ok(())
    }
}

impl Display for PluginList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_vertical_list_with_bullets(f, &self.0)
    }
}

impl Display for XArguments {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
    }
}

impl Display for FeatureFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: {}", self.name)?;
        writeln!(f, "description: {}", self.description)?;
        writeln!(f, "doc URL: {}", self.doc_url)?;
        writeln!(f, "stability: {}", self.stability)?;
        writeln!(f, "provided by: {}", self.provided_by)?;

        Ok(())
    }
}

impl Display for FeatureFlagList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ff in &self.0 {
            writeln!(f, "{}", ff)?;
        }

        Ok(())
    }
}

impl Display for MessageList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for msg in &self.0 {
            writeln!(f, "{}", msg)?;
        }

        Ok(())
    }
}

impl Display for MessageRouted {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.routed {
            true => write!(f, "Message published and routed successfully"),
            false => write!(f, "Message published but NOT routed"),
        }
    }
}

impl Display for MessageProperties {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_map_as_colon_separated_pairs(f, &self.0)
    }
}

impl Display for GetMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "payload: {}", self.payload)?;
        writeln!(f, "exchange: {}", self.exchange)?;
        writeln!(f, "routing key: {}", self.routing_key)?;
        writeln!(f, "redelivered: {}", self.redelivered)?;
        writeln!(f, "properties: {}", self.properties)?;

        Ok(())
    }
}

#[allow(dead_code)]
pub fn fmt_list_as_json_array(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "[]")
        }
        _ => {
            write!(f, "[")?;
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                write!(f, "{}, ", elem)?;
            }
            write!(f, "{}", last_element)?;
            write!(f, "]")?;
            Ok(())
        }
    }
}

pub fn fmt_comma_separated_list(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "")
        }
        _ => {
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                write!(f, "{}, ", elem)?;
            }
            write!(f, "{}", last_element)?;
            Ok(())
        }
    }
}

pub fn fmt_vertical_list_with_bullets(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "")
        }
        _ => {
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                writeln!(f, "* {}", elem)?;
            }
            write!(f, "* {}", last_element)?;
            Ok(())
        }
    }
}

pub fn fmt_vertical_list_without_bullets(f: &mut fmt::Formatter<'_>, xs: &[String]) -> fmt::Result {
    match xs.len() {
        0 => {
            write!(f, "")
        }
        _ => {
            let mut xs = xs.to_owned();
            let last_element = xs.pop().unwrap();
            for elem in xs {
                writeln!(f, "{}", elem)?;
            }
            write!(f, "{}", last_element)?;
            Ok(())
        }
    }
}

pub fn fmt_map_as_colon_separated_pairs(
    f: &mut fmt::Formatter<'_>,
    xs: &Map<String, serde_json::Value>,
) -> fmt::Result {
    for (k, v) in xs.iter() {
        writeln!(f, "{}: {}", k, v)?;
    }

    Ok(())
}

pub fn display_option<T>(opt: &Option<T>) -> String
where
    T: fmt::Display,
{
    match opt {
        None => "".to_owned(),
        Some(val) => format!("{}", val).to_owned(),
    }
}

pub fn display_arg_table(xs: &XArguments) -> String {
    let mut s = String::new();
    for (k, v) in xs.0.iter() {
        let line = format!("{}: {}\n", k, v);
        s += line.as_str()
    }

    s.clone()
}

pub fn display_tag_map_option(opt: &Option<TagMap>) -> String {
    match opt {
        Some(val) => {
            let mut s = String::new();
            let iter = val.0.clone().into_iter();
            for (k, v) in iter {
                let line = format!("\"{}\": {}\n", k, v);
                s += line.as_str()
            }

            s.clone()
        }
        None => "".to_owned(),
    }
}

pub fn display_tag_list_option(opt: &Option<TagList>) -> String {
    match opt {
        Some(val) => {
            let mut s = String::new();
            let iter = val.0.clone().into_iter();
            for t in iter {
                let line = format!("{}\n", t);
                s += line.as_str()
            }

            s.clone()
        }
        None => "".to_owned(),
    }
}
