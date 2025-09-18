# Rust Client for the RabbitMQ HTTP API Change Log

## v0.52.0 (in development)

### Enhancements

 * `VirtualHostDefinitionSetTransformer` trait and `VirtualHostTransformationChain` for transforming virtual host-specific definition sets
 * Virtual host equivalents of cluster-wide transformers: `PrepareForQuorumQueueMigrationVhost`, `StripCmqKeysFromVhostPolicies`, `DropEmptyVhostPolicies`
 * `PrepareForQuorumQueueMigration` and `PrepareForQuorumQueueMigrationVhost` transformers now also strip CMQ-related x-arguments
   (such as `x-ha-mode`) from queues definitions, as they won't pass validation on RabbitMQ `4.x`

## v0.51.0 (Sep 16, 2025)

### Enhancements

 * `responses::TagList`, `responses::PluginList`, `responses::XArguments`, `responses::NodeList`, `responses::MessageList`,`responses::FeatureFlagList`,  
   `responses::DeprecatedFeatureList` now all implement `Deref` and `DerefMut`
 * `responses::Channel#state` now uses an enum, `responses::ChannelState`, instead of a string.
 * `Client#enable_vhost_deletion_protection` [protects](https://www.rabbitmq.com/docs/vhosts#deletion-protection) a virtual host from deletion (using the `POST /api/vhosts/{vhost}/deletion/protection` endpoint).
 * `Client#disable_vhost_deletion_protection` lifts [deletion protection](https://www.rabbitmq.com/docs/vhosts#deletion-protection) (using the `DELETE /api/vhosts/{vhost}/deletion/protection` endpoint).
 * `Client#auth_attempts_statistics` is a new function providing support for the `GET /api/auth/attempts/{node}` endpoint.
 * `Client#list_topic_permissions` is a new function that provides support for the `GET /api/topic-permissions` endpoint.
 * `Client#list_topic_permissions_in` is a new function that provides support for the `GET /api/vhosts/{vhost}/topic-permissions` endpoint.
 * `Client#get_topic_permissions_of` is a new function that provides support for the `GET /api/topic-permissions/{vhost}/{user}` endpoint.
 * `Client#list_channels_on` is a new function that provides support for the `GET /api/connections/{connection}/channels` endpoint.
 * `Client#get_channel_info` returns information about a specific channel.
 * `Client#current_user` is a new function providing support for the `GET /api/whoami` endpoint.

### Bug Fixes

 * `Client#delete_binding` could panic if the optional `x-arguments` value was passed in as `None` 
 * `api::Client#enable_schema_definition_sync_on_node` was unintentionally named `enable_schema_definition_sync_one_node`

## v0.50.0 (Sep 13, 2025)

### Breaking Changes

 * `blocking_api::Client#enable_schema_definition_sync`, `blocking_api::Client#disable_schema_definition_sync` were
   removed in favor of `blocking_api::Client#enable_schema_definition_sync_on_node` and `blocking_api::Client#disable_schema_definition_sync_on_node`
   that accept an `Option<&str>` for name, like in `api::Client`. `None` means "on all cluster nodes".

### Enhancements

 * Adopted a few type aliases: `common::Username`, `common::VirtualHostName`, `common::PermissionPattern`, `common::ChannelId`.
 * `responses::TagList`, `responses::PluginList`, `responses::NodeList`, `responses::FeatureFlagList`, `responses::DeprecatedFeatureList`
   now implement `contains`.


## v0.49.0 (Sep 07, 2025)

### Enhancements

 * `responses::TagList`, `responses::PluginList`, `responses::FeatureFlagList`, `responses::DeprecatedFeatureList`,
   `responses::MessageList` now all have `len` and `is_empty` methods.
 * `responses::TagList`, `responses::PluginList`, `responses::NodeList`, `responses::FeatureFlagList`, `responses::DeprecatedFeatureList`,
   `responses::MessageList` now all implement `IntoIterator`.


## v0.48.0 (Sep 07, 2025)

* `responses::NodeList#len` is now properly accompanied by `responses::NodeList#is_empty`.


## v0.47.0 (Sep 07, 2025)

### Enhancements

 * `responses::ClusterIdentity` now implements `ToString`, `From<String>`, and `From<&str>`.
 * `responses::NodeList#len` is a new function that returns the length of the list.


## v0.46.0 (Sep 06, 2025)

### Enhancements

 * `QueueOps#x_arguments`: returns the optional arguments of an object (`XArguments`).
 * `QueueOps#has_queue_ttl_arg`: returns true if optional arguments of an object include a queue TTL argument (`"x-expires"`).
 * `QueueOps#has_message_ttl_arg`: returns true if optional arguments of an object include a message TTL argument (`"x-message-ttl"`).
 * `QueueOps#has_length_limit_in_messages`: returns true if optional arguments of an object include a queue length limit in messages (`"x-max-length"`).
 * `QueueOps#has_length_limit_in_bytes`: returns true if optional arguments of an object include a queue length limit in bytes (`"x-max-length-bytes"`).
 * `QueueOps#is_server_named`: returns true if an object's name suggests it is a server-named (system) entity.



## v0.45.0 (Sep 01, 2025)

### Enhancements

 * New struct, `requests::TopicPermissions`, for declaring topic permissions.
 * New struct, `responses::TopicPermission` for representing topic permissions.
 * `Client#list_topic_permissions_of` is a new function that lists topic permissions of a user.
 * `Client#grant_permissions` is now an alias for `Client#declare_permissions`, providing an easier to remember alias.
 * `Client#declare_topic_permissions` is a new function that sets topic permissions for a user
 * `Client#clear_topic_permissions` is a new function to clear topic permissions for a user.


## v0.44.0 (Aug 25, 2025)

### Enhancements

 * `responses::MessagingProtocol` was updated to include a new variant, `MessagingProtocol::Local`,
    that represents a "protocol" used by local shovels in RabbitMQ 4.2.0 and later.


## v0.43.0 (Aug 19, 2025)

### Enhancements

 * `responses::VirtualHost#metadata` is now optional. It was introduced in RabbitMQ `3.8.0` in 2019
   but for various reasons can be missing in versions up to `3.11.0` or so.

   This can be considered a RabbitMQ `3.10.x` (that reached EOL in late 2023) compatibility fix.
   Responsible adults don't run EOL versions of RabbitMQ, of course.


## v0.42.0 (Aug 14, 2025)

### Bug Fixes

 * `responses::Shovel.vhost` is now an option because this field will be missing for
   static shovels.


## v0.41.0 (Aug 11, 2025)

### Enhancements

 * `Client#list_shovels_in` is a new function that returns a list of shovels in a specific virtual host
 * `ShovelState` now includes one more state, `ShovelState::Terminated`


## v0.40.0 (Jul 17, 2025)

### Enhancements

 * Several structs in `responses` now implement `Default`, in particular for deserialization,
   and can handle the cases with certain metrics missing at request time on a freshly booted
   RabbitMQ node


## v0.39.0 (Jul 14, 2025)

### Enhancements

 * Support for a new deprecated feature flag state column, introduced in [rabbitmq/rabbitmq-server#14227](https://github.com/rabbitmq/rabbitmq-server/pull/14227)
 * `Client#declare_policies` and `Client#declare_operator_policies` are two new helper functions for declaring multiple policies at once.
    Note that both functions will still issue the same number of API requests, so it only exists for convenience
 * `Client#delete_policies_in` and `Client#delete_operator_policies_in` are two new helper functions for deleting multiple policies at once.
    Note that both functions will still issue the same number of API requests, so it only exists for convenience

## v0.38.0 (Jul 11, 2025)

### Enhancements

 * Introduce `password_hashing::HashingAlgorithm` with two variants, SHA-256 and SHA-512
 * Support for SHA-512 hashing of salted passwords


## v0.37.0 (Jul 11, 2025)

### Breaking Changes

 * `responses::PolicyDefinitionOps` was extended and renamed to `responses::OptionalArgumentSourceOps`

### Enhancements

 * New `DefinitionSetTransformer`: `PrepareForQuorumQueueMigration`.

   This one not only strips off the CMQ-related keys
   but also handles an incompatible `"overflow"`/`"x-overflow"` key value
   and `"queue-mode"`/`"x-queue-mode"` keys, both not supported
   by quorum queues.

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
