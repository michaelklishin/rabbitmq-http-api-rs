# Instructions for AI Agents

## What is This Codebase?

This is a Rust client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/http-api-reference).

## Build System

All the standard Cargo commands apply but with one important detail: make sure to add `--all-features` so that
both the async and blocking client are built, tested, linted, and so on.

 * `cargo build --all-features` to build
 * `cargo nextest run --all-features` to run tests
 * `cargo clippy --all-features` to lint
 * `cargo fmt` to reformat
 * `cargo publish` to publish the crate

Always run `cargo check --all-features` before making changes to verify the codebase compiles cleanly.
If compilation fails, investigate and fix compilation errors before proceeding with any modifications.

## The Async and Blocking Clients

This library provides two clients:

 * An async client, `rabbitmq_http_client::api::Client`, defined in `src/api.rs`
 * A blocking client, `rabbitmq_http_client::api::BlockingClient`, defined in `src/blocking_api.rs`

They have very similar APIs except that all functions of the async client are, well, `async`.

## Key Files

 * Async client: `src/api/*.rs`
 * Blocking client: `src/blocking_api/*.rs`
 * Error types: `src/error.rs` 
 * HTTP API request types: `src/requests/*.rs`
 * HTTP API response types: `src/responses/*.rs`
 * Types shared between requests and responses: `src/commons.rs`
 * Definition transformations: `src/transformers.rs`

## Test Suite Layout

 * `tests/async*.rs` test modules test the async client, use a Tokio runtime and `async` functions
 * `tests/blocking*.rs` test modules test the blocking client and regular (non-`async`) functions
 * `tests/unit*.rs` modules are for unit tests
 * `tests/*proptests.rs` are property-based tests
 * `tests/test_helpers.rs` contains helper functions shared by multiple test modules

## Source of Domain Knowledge

 * [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/http-api-reference)
 * [RabbitMQ Documentation](https://www.rabbitmq.com/docs/)

Treat this documentation as the ultimate first party source of truth.

## Change Log

If asked to perform change log updates, consult and modify `CHANGELOG.md` and stick to its
existing writing style.

## Git Commits

 * Do not commit changes automatically without an explicit permission to do so
 * Never add yourself as a git commit coauthor
