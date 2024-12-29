# Rust Client for the RabbitMQ HTTP API Change Log

# v0.13.0 (in development)

No (documented) changes yet.


# v0.12.0 (Dec 28, 2024)

### Enhancements

 * `Client#list_feature_flags`, `Client#enable_feature_flag`, `Client#enable_all_stable_feature_flags` are three
   new functions for working with [feature flags](https://www.rabbitmq.com/docs/feature-flags)

# v0.11.0 (Dec 28, 2024)

### Enhancements

 * `Client#list_all_deprecated_features` and `Client#list_deprecated_features_in_use`
   are new functions for listing all [deprecated features](https://www.rabbitmq.com/docs/deprecated-features) and only those whose use is
   detected in the cluster.

 * `Client#list_feature_flags` is a new function that lists all [feature flags](https://www.rabbitmq.com/docs/feature-flags)
   in the cluster, including their state and stability.


# v0.10.0 (Dec 27, 2024)

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
