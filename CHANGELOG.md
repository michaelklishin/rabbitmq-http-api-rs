# Rust Client for the RabbitMQ HTTP API Change Log

## v0.18.0  (in development)

No (documented) changes yet.


## v0.17.0  (Jan 27, 2025)

### Enhancements

 * Initial support for Tanzu RabbitMQ Schema Definitions Sync (SDS) operations.

 * Initial support for Tanzu RabbitMQ Warm Standby Replication (WSR) operations.

 * Isolated test suite runs for each client.

   To run only the async client tests, use

   ```bash
   cargo test async --all-features
   ```

   To run only the blocking client tests, use

   ```bash
   cargo test blocking --all-features
   ```

### Bug Fixes

 *  Async `Client#delete_*` functions now correctly handle `NotFound` responses for idempotent deletes


## v0.16.0  (Jan 15, 2025)

### Bug Fixes

 * `api::Client` now computes API endpoint path correctly (a slash was missing)


## v0.15.0  (Jan 5, 2025)

### Enhancements

 * `Client#get_node_memory_footprint` is a new function that returns a [node memory footprint breakdown](https://www.rabbitmq.com/docs/memory-use).
   `responses::NodeMemoryFootprint` and `responses::NodeMemoryBreakdown` are the key types that provide
   access to per-category proportions, both absolute and relative (in percent)


## v0.14.0  (Dec 31, 2024)

### Enhancements

 * New `responses::HealthCheckFailureDetails` variants to accommodate active port and protocol
   listener [health checks](https://www.rabbitmq.com/docs/monitoring#health-checks)

 * New [health check](https://www.rabbitmq.com/docs/monitoring#health-checks) function: `Client#health_check_protocol_listener`


## v0.13.0 (Dec 31, 2024)

### Enhancements

 * New functions for listing [stream](https://www.rabbitmq.com/docs/streams) connections, publishers and consumers: `Client#list_stream_publishers`, `Client#list_stream_publishers_in`, `Client#list_stream_publishers_of`, `Client#list_stream_publishers_on_connection`, `Client#list_stream_consumers`, `Client#list_stream_consumers_in`, `Client#list_stream_consumers_on_connection`, `Client#list_stream_connections`, `Client#list_stream_connections_in`

 * New [health check](https://www.rabbitmq.com/docs/monitoring#health-checks) function: `Client#health_check_port_listener`


## v0.12.0 (Dec 28, 2024)

### Enhancements

 * `Client#list_feature_flags`, `Client#enable_feature_flag`, `Client#enable_all_stable_feature_flags` are three
   new functions for working with [feature flags](https://www.rabbitmq.com/docs/feature-flags)


## v0.11.0 (Dec 28, 2024)

### Enhancements

 * `Client#list_all_deprecated_features` and `Client#list_deprecated_features_in_use`
   are new functions for listing all [deprecated features](https://www.rabbitmq.com/docs/deprecated-features) and only those whose use is
   detected in the cluster.

 * `Client#list_feature_flags` is a new function that lists all [feature flags](https://www.rabbitmq.com/docs/feature-flags)
   in the cluster, including their state and stability.


## v0.10.0 (Dec 27, 2024)

### Dependencies

 * Bumps minimum `reqwest` version to `0.12.11`


## v0.9.0 (Dec 24, 2024)

### Breaking Changes

 * To propagate more request context to the caller,
   `crate::error::Error` was updated to provide a requset URL, a header map,
   and a request body (if available).

   This reason for doing this comes down to how `reqwest`'s `Response` functions
   are designed: the ones that consume and parse the body also consume `self`,
   which means propagating a `Response` to the caller is not very useless at best,
   and arguably is misleading.

### Enhancements

 * `crate::api` now provides an async API. `crate::blocking_api` provides the
   original synchronous API.

   Contributed by @ikrivosheev.

   GitHub issues: [#29](https://github.com/michaelklishin/rabbitmq-http-api-rs/pull/29), [#30](https://github.com/michaelklishin/rabbitmq-http-api-rs/pull/30)

 * `Client#overview` is a new function that corresponds to the `GET /api/overview` API endpoint.

   Contributed by @ikrivosheev.

   GitHub issue: [#31](https://github.com/michaelklishin/rabbitmq-http-api-rs/pull/31)
