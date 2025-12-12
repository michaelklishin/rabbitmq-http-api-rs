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
fn test_new_clamps_to_max_page_size() {
    let params = PaginationParams::new(1, MAX_PAGE_SIZE + 100);
    assert_eq!(params.page_size, Some(MAX_PAGE_SIZE));
}

#[test]
fn test_is_last_page_handles_zero_page_size() {
    let params = PaginationParams {
        page: Some(1),
        page_size: Some(0),
    };
    let results: Vec<i32> = vec![1, 2, 3];
    // With page_size=0, is_last_page should treat it as page_size=1
    // Since 3 results >= 1, this is not the last page
    assert!(!params.is_last_page(&results));

    let single: Vec<i32> = vec![];
    assert!(params.is_last_page(&single));
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

#[test]
fn test_next_page() {
    let params = PaginationParams::new(1, 50);
    let next = params.next_page();
    assert!(next.is_some());
    let next = next.unwrap();
    assert_eq!(next.page, Some(2));
    assert_eq!(next.page_size, Some(50));
}

#[test]
fn test_next_page_preserves_size() {
    let params = PaginationParams::new(5, 200);
    let next = params.next_page().unwrap();
    assert_eq!(next.page, Some(6));
    assert_eq!(next.page_size, Some(200));
}

#[test]
fn test_next_page_none_when_uninitialized() {
    let params = PaginationParams::default();
    assert!(params.next_page().is_none());
}

#[test]
fn test_current_page() {
    let params = PaginationParams::new(3, 100);
    assert_eq!(params.current_page(), Some(3));
}

#[test]
fn test_current_page_none() {
    let params = PaginationParams::default();
    assert_eq!(params.current_page(), None);
}

#[test]
fn test_page_size_accessor() {
    let params = PaginationParams::new(1, 75);
    assert_eq!(params.page_size(), Some(75));
}

#[test]
fn test_page_size_accessor_none() {
    let params = PaginationParams::default();
    assert_eq!(params.page_size(), None);
}

#[test]
fn test_is_last_page_true_when_fewer_results() {
    let params = PaginationParams::new(1, 100);
    let results: Vec<i32> = vec![1, 2, 3];
    assert!(params.is_last_page(&results));
}

#[test]
fn test_is_last_page_false_when_full_page() {
    let params = PaginationParams::new(1, 100);
    let results: Vec<i32> = (0..100).collect();
    assert!(!params.is_last_page(&results));
}

#[test]
fn test_is_last_page_true_when_empty() {
    let params = PaginationParams::new(1, 100);
    let results: Vec<i32> = vec![];
    assert!(params.is_last_page(&results));
}

#[test]
fn test_is_last_page_uses_default_page_size() {
    let params = PaginationParams {
        page: Some(1),
        page_size: None,
    };
    let results: Vec<i32> = (0..99).collect();
    assert!(params.is_last_page(&results));
}

#[test]
fn test_pagination_loop_pattern() {
    let mut params = PaginationParams::first_page(10);
    let mut all_items: Vec<i32> = Vec::new();

    for page_num in 1..=4 {
        let results: Vec<i32> = if page_num < 4 {
            (0..10).collect()
        } else {
            vec![1, 2, 3]
        };

        all_items.extend(&results);

        if params.is_last_page(&results) {
            break;
        }
        params = params.next_page().unwrap();
    }

    assert_eq!(all_items.len(), 33);
}
