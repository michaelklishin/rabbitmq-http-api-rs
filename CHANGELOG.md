# Rust Client for the RabbitMQ HTTP API Change Log

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
