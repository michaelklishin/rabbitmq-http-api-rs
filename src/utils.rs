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

pub fn percentage(a: u64, b: u64) -> f64 {
    (a as f64 / b as f64) * 100.0
}

pub fn percentage_as_text(a: u64, b: u64) -> String {
    let p = percentage(a, b);
    format!("{p:.2}%")
}

#[macro_export]
macro_rules! path_one_part {
    ($val:expr, $part:literal) => {
        $val.push('/');
        $val.push_str($part);
    };
    ($val:expr, $part:expr) => {
        let part = $part.as_ref();
        let encoded =
            percent_encoding::utf8_percent_encode(part, percent_encoding::NON_ALPHANUMERIC);
        $val.push('/');
        $val.extend(encoded);
    };
}

#[macro_export]
macro_rules! path {
    ($part1:expr, $($part:expr),+) => {{
        let mut url = String::from($part1);
        $(
            $crate::path_one_part!(&mut url, $part);
        )+
        url
    }}
}
