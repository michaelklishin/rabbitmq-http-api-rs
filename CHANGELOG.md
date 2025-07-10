# Rust Client for the RabbitMQ HTTP API Change Log

## v0.37.0 (in development)

### Breaking Changes

 * `responses::PolicyDefinitionOps` was extended and renamed to `responses::OptionalArgumentSourceOps`

### Enhancements

 * `responses::OptionalArgumentSourceOps` now supports more operations on [optional queue arguments](https://www.rabbitmq.com/docs/queues#optional-arguments)
    of `responses::QueueDefinition` as well as policy definitions (`responses::PolicyDefinition`, `responses::Policy`)


## v0.36.0 (Jul 4, 2025)

### Enhancements

 * `response::Connection` now can represent direct connections,
   a special kind of connections supported by the Erlang AMQP 0-9-1 client,
   that shovels and federation links use when connecting to the local
   node.

   GitHub issues: [rabbitmq/rabbitmqadmin-ng#68](https://github.com/rabbitmq/rabbitmqadmin-ng/issues/68), [#61](https://github.com/michaelklishin/rabbitmq-http-api-rs/pull/61)


## v0.35.0 (Jun 28, 2025)

### Enhancements

 * `ClientCapabilities` fields now default to `false` when not provided in
    the API response.

    GitHub issue: [#56](https://github.com/michaelklishin/rabbitmq-http-api-rs/issues/56).


## v0.34.0 (Jun 12, 2025)

### Upgrades

 * `tabled` was upgraded to `0.20.0`


## v0.33.0 (Jun 10, 2025)

### Bug Fixes

 * `NoActiveProtocolListenerDetails` was split into `NoActiveProtocolListenerDetailsPre41` and `NoActiveProtocolListenerDetails41AndLater`
    to support `GET /api/health/checks/protocol-listener/{protocols}` responses of both RabbitMQ 4.0.x and 4.1.x.

    Relevant RabbitMQ change: [rabbitmq/rabbitmq-server#13871](https://github.com/rabbitmq/rabbitmq-server/pull/13871).


## v0.32.0 (Jun 6, 2025)

### Enhancements

 * Minor helper functions, such as `Policy#with_overrides` and `PolicyDefinition#merge`


## v0.31.0 (May 16, 2025)

### Enhancements

 * `PolicyDefinition#insert` and `Policy#insert_definition_key` are new functions for adding or updating
   policy definition key-value pairs
 * `responses::Policy` now can be converted to `requests::PolicyParams` for easier policy definition
    updates
 * More flexible use of optional `reqwest` features.

   Contributed by @ikrivosheev.

   GitHub issue: [#53](https://github.com/michaelklishin/rabbitmq-http-api-rs/pull/53).


## v0.30.0 (May 6, 2025)

### Enhancements

 * `Client#list_global_runtime_parameters`, `Client#get_global_runtime_parameter`, `Client#upsert_global_runtime_parameter`, `Client#clear_global_runtime_parameter`
   are new functions for working with [global runtime parameters](https://www.rabbitmq.com/docs/parameters)
 * `Client#get_cluster_tags`, `Client#set_cluser_tags`, `Client#clear_cluster_tags` are new functions for operations on [cluster tags](https://www.rabbitmq.com/docs/parameters#cluster-tags)


## v0.29.0 (Apr 14, 2025)

### Breaking Changes

 * `PolicyDefinition` and specifically `requests::PolicyParams.definition` is now a `Map<String, Value>`
   and not an `Option<Map<String, Value>>`. When creating a policy, the definition cannot be missing or blank,
   otherwise it would not pass server validation.


## v0.28.0 (Mar 23, 2025)

### Enhancements

 * Federation support. Key API elements: `FederationUpstreamParams`, `QueueFederationParams`, `ExchangeFederationParams`,
   `FederationUpstream`, `FederationLink`, `Client#declare_federation_upstream_with_parameters`, `Client#declare_federation_upstream`, `Client#delete_federation_upstream`, `Client#list_federation_upstreams`, `Client#list_federation_links`

 * New definition set transformations that include certain parts of the definition set:
   `exclude_users`, `exclude_permissions`, `exclude_runtime_parameters`, `exclude_policies`


## v0.27.0 (Mar 11, 2025)

### Enhancements

 * `ClusterDefinitionSet` transformations are maturing.

   There are two `transformations::DefinitionSetTransformer`s
   available: one that removes classic queue mirroring-related (from the 3.13.x era) policy keys, and another
   that removes policies with empty definitions.

   The two are supposed to be used together.


## v0.26.0  (Mar 10, 2025)

### Enhancements

 * `ClientBuilder<E, U, P>` now has a default type parameter value.

   Contributed by @ikrivosheev.

   GitHub issue: [#46](https://github.com/michaelklishin/rabbitmq-http-api-rs/pull/46)

 * `QueueOps`, `NamedPolicyTargetObject` are two new traits that allow
   key queue properties to be accessed on several structs that semantically represent
   a queue, either directly or in an exported set of definitions

 * `QueueType::Unsupported(String)` is a new queue type variant

 * Initial functions for mutating certain parts of `ClusterDefinitionSet`s


## v0.25.0  (Mar 3, 2025)

### Enhancements

 * `PolicyTarget#does_apply_to` is a new function that allow for `PolicyTarget`
   equivalence comparison. For example, `PolicyTarget::QuorumQueues` is a subset of `PolicyTarget::Queues` but `PolicyTarget::QuorumQueues` is not


## v0.24.0  (Mar 2, 2025)

### Enhancements

 * `Client#declare_amqp10_shovel` is a new function that declares a dynamic shovel
   where both source and destination use AMQP 1.0

 * Both `Amqp091ShovelSourceParams` and `Amqp091ShovelDestinationParams` now support a new boolean option, `predeclared`,
   that enables [either or both sides to rely on a pre-declared topology](https://www.rabbitmq.com/docs/shovel-dynamic#predeclared-topology)




## v0.23.0  (Feb 24, 2025)

### Breaking Changes

 * `RuntimeParameterDefinition#name`, `RuntimeParameterDefinition#vhost`, and `RuntimeParameterDefinition#component` types changed from `String` to `&str`

### Enhancements

 * `Client#declare_amqp091_shovel` is a new function that declares a dynamic shovel
    where both source and destination use AMQP 0-9-1

 * `Client#delete_shovel` is a new function for deleting shovels


## v0.22.0 (Feb 8, 2025)

### Enhancements

 * `Client#import_vhost_definitions` is a new function that imports virtual host-specific
   definition files (as opposed to cluster-wide ones) into the target virtual host

 * `Client#import_cluster_wide_definitions` is an alias to `Client#import_definitions`
   to better reflect what it does


## v0.21.0 (Feb 8, 2025)

### Enhancements

 * `responses::VirtualHostDefinitionSet` is an equivalent of `responses::ClusterDefinitionSet` but adapted
   for the specific of virtual host-specific definitions, namely the fact that they do not contain
   virtual hosts, users, or permissions, and objects such as queues or bindings do not have the
   virtual host field to make it possible to import them into a virtual host with any name

 * `Client#export_vhost_definitions`, `Client#export_vhost_definitions_as_string` and
   `Client#export_vhost_definitions_as_data` are new functions that export virtual host-specific
    definitions (as opposed to cluster-wide ones)

### Breaking Changes

 * `responses::DefinitionSet` was renamed to `responses::ClusterDefinitionSet` to
   differentiate it from virtual host-specific definitions, which are from now on
   represented by `responses::VirtualHostDefinitionSet`

 * `Client#export_definitions` was renamed to `Client#export_cluster_wide_definitions`
 * `Client#export_definitions_as_string` was renamed to `Client#export_cluster_wide_definitions_as_string`
 * `Client#export_definitions_as_data` was renamed to `Client#export_cluster_wide_definitions_as_data`


## v0.20.0  (Feb 2, 2025)

### Enhancements

 * `Client#.close_user_connections` is a new function that closes all connections
    that authenticated with a specific username


## v0.19.0 (Feb 1, 2025)

### Refactoring

* `Client#disable_schema_definition_sync` was renamed to `Client#disable_schema_definition_sync_on_node`.

  This breaking change only applies to a function specific to
  Tanzu RabbitMQ 4.1, a series currently in development.

* `Client#enable_schema_definition_sync` was renamed to `Client#enable_schema_definition_sync_on_node`.

  This breaking change only applies to a function specific to
  Tanzu RabbitMQ 4.1, a series currently in development.

### Enhancements

* `Client#disable_schema_definition_sync` now disables SDS on all cluster nodes.

  This function is specific to Tanzu RabbitMQ 4.1, a series currently in development.

* `Client#enable_schema_definition_sync` now enables SDS on all cluster nodes.

  This function is specific to Tanzu RabbitMQ 4.1, a series currently in development.


## v0.18.0  (Feb 1, 2025)

### Bug Fixes

 * `responses::Connection#channel_max` is now an `Option<u16>` because
   this metric wont' be available for, say, RabbitMQ Stream Protocol
   connections

### Enhancements

 * New convenience functions for working with streams: `Client#get_stream_info`, `Client#delete_stream`

 * `Client#declare_stream` and `requests::StreamParams` for convenient
   stream declaration


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
