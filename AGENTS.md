# Instructions for AI Agents

## What is This Codebase?

This is a Rust client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/http-api-reference).

## Build System

All the standard Cargo commands apply but with one caveat: make sure to add `--all-features` so that
both async and blocking client are built, tested, linted, and so on.

 * `cargo build --all-features` to build
 * `cargo nextest run --all-features` to run tests
 * `cargo clippy --all-features` to lint
 * `cargo fmt` to reformat
 * `cargo publish` to publish the crate

## The Async and Blocking Clients

This library provides two clients:

 * An async client, `rabbitmq_http_client::api::Client`, defined in @src/api.rs
 * A blocking client, `rabbitmq_http_client::api::BlockingClient`, defined in @src/blocking_api.rs

They have very similar APIs except that all functions of the async client are, well, `async`.

## Key Files

 * Async client: @src/api.rs
 * Blocking client: @src/blocking_api.rs
 * HTTP API request types: @src/requests.rs
 * HTTP API response types: @src/responses.rs
 * Types shared between requests and responses: @src/common.rs
 * Error types: @src/error.rs
 * Definition transformations: @src/transformers.rs

## Test Suite Layout

 * `tests/async*.rs` test modules test the async client
 * `tests/blocking*.rs` test modules test the blocking client
 * `tests/unit*.rs` modules are for unit tests
 * `tests/test_helpers.rs` contains helper functions shared by multiple test modules

## Source of Domain Knowledge

 * [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/http-api-reference)
 * [RabbitMQ Documentation](https://www.rabbitmq.com/docs/)

## Change Log

If asked to perform change log updates, consult and modify @CHANGELOG.md.

## Git Commits

 * Do not commit changes automatically without an explicit permission to do so
 * Never add yourself as a git commit coauthor
