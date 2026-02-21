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

use rabbitmq_http_client::responses::Overview;

#[test]
fn test_unit_overview_has_jit_enabled_with_jit() {
    let overview = Overview {
        erlang_full_version: "Erlang/OTP 26 [erts-14.2.5.11] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1] [jit]".to_string(),
        ..Default::default()
    };

    assert!(overview.has_jit_enabled());
}

#[test]
fn test_unit_overview_has_jit_enabled_without_jit() {
    let overview = Overview {
        erlang_full_version: "Erlang/OTP 26 [erts-14.2.5.11] [source] [64-bit] [smp:10:10] [ds:10:10:10] [async-threads:1]".to_string(),
        ..Default::default()
    };

    assert!(!overview.has_jit_enabled());
}
