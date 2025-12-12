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

use rabbitmq_http_client::commons::{DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE, PaginationParams};

#[test]
fn test_constants() {
    assert_eq!(DEFAULT_PAGE_SIZE, 100);
    assert_eq!(MAX_PAGE_SIZE, 500);
}

#[test]
fn test_new() {
    let params = PaginationParams::new(2, 50);
    assert_eq!(params.page, Some(2));
    assert_eq!(params.page_size, Some(50));
}

#[test]
fn test_new_at_max_page_size() {
    let params = PaginationParams::new(1, MAX_PAGE_SIZE);
    assert_eq!(params.page_size, Some(MAX_PAGE_SIZE));
}

#[test]
#[should_panic(expected = "exceeds MAX_PAGE_SIZE")]
fn test_new_exceeds_max_page_size() {
    let _ = PaginationParams::new(1, MAX_PAGE_SIZE + 1);
}

#[test]
fn test_first_page() {
    let params = PaginationParams::first_page(25);
    assert_eq!(params.page, Some(1));
    assert_eq!(params.page_size, Some(25));
}

#[test]
fn test_first_page_default() {
    let params = PaginationParams::first_page_default();
    assert_eq!(params.page, Some(1));
    assert_eq!(params.page_size, Some(DEFAULT_PAGE_SIZE));
}

#[test]
fn test_default() {
    let params = PaginationParams::default();
    assert_eq!(params.page, None);
    assert_eq!(params.page_size, None);
}

#[test]
fn test_to_query_string_both() {
    let params = PaginationParams::new(3, 100);
    assert_eq!(
        params.to_query_string(),
        Some("page=3&page_size=100".to_string())
    );
}

#[test]
fn test_to_query_string_page_only() {
    let params = PaginationParams {
        page: Some(5),
        page_size: None,
    };
    assert_eq!(params.to_query_string(), Some("page=5".to_string()));
}

#[test]
fn test_to_query_string_page_size_only() {
    let params = PaginationParams {
        page: None,
        page_size: Some(200),
    };
    assert_eq!(params.to_query_string(), Some("page_size=200".to_string()));
}

#[test]
fn test_to_query_string_none() {
    let params = PaginationParams::default();
    assert_eq!(params.to_query_string(), None);
}

#[test]
fn test_clone() {
    let params = PaginationParams::new(2, 50);
    let cloned = params.clone();
    assert_eq!(params, cloned);
}

#[test]
fn test_debug() {
    let params = PaginationParams::new(1, 100);
    let debug = format!("{:?}", params);
    assert!(debug.contains("PaginationParams"));
    assert!(debug.contains("page"));
    assert!(debug.contains("page_size"));
}
