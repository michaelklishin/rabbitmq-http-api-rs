# A Rust Client for the RabbitMQ HTTP API

This is a Rust client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/management#http-api).

This is not an AMQP 0-9-1 client (see [amqprs](https://github.com/gftea/amqprs)), an AMQP 1.0 client (see [fe2o3-amqp](https://github.com/minghuaw/fe2o3-amqp)), or a [RabbitMQ Stream protocol](https://www.rabbitmq.com/docs/streams) client (see [rabbitmq-stream-rust-client](https://github.com/rabbitmq/rabbitmq-stream-rust-client)).

## Project Maturity

This library is reasonably mature.

Before `1.0.0`, breaking API changes can and will be introduced.

## Supported RabbitMQ Series

This library targets RabbitMQ 4.x and 3.13.x.

All older series have [reached End of Life](https://www.rabbitmq.com/release-information).

## Dependency

### Async Client

```toml
rabbitmq_http_client = { version = "0.83.0", features = ["core", "async"] }
```

### Blocking Client

```toml
rabbitmq_http_client = { version = "0.83.0", features = ["core", "blocking"] }
```

### With Tabled Support

```toml
rabbitmq_http_client = { version = "0.83.0", features = ["core", "async", "tabled"] }
rabbitmq_http_client = { version = "0.83.0", features = ["core", "blocking", "tabled"] }
```

### With Zeroize Support

The `zeroize` feature provides a `Password` type that offers [zeroisation](https://en.wikipedia.org/wiki/Zeroisation)
of credentials in memory for security-demanding use cases:

```toml
rabbitmq_http_client = { version = "0.83.0", features = ["core", "async", "zeroize"] }
```

### TLS Backend

By default, this client uses [`native-tls`](https://crates.io/crates/native-tls).
To use [`rustls`](https://crates.io/crates/rustls) instead:

```toml
rabbitmq_http_client = { version = "0.83.0", default-features = false, features = ["core", "async", "rustls"] }
```

Add `hickory-dns` to use the [Hickory DNS](https://github.com/hickory-dns/hickory-dns) resolver.


## Async Client

The async client is in `rabbitmq_http_client::api`.

### Instantiate a Client

```rust
use rabbitmq_http_client::api::Client;

let endpoint = "http://localhost:15672/api";
let rc = Client::new(&endpoint, "username", "password");
```

### Using ClientBuilder

`ClientBuilder#build` validates the endpoint scheme (`http` or `https`)
and returns a `Result`:

```rust
use rabbitmq_http_client::api::ClientBuilder;

let rc = ClientBuilder::new()
    .with_endpoint("http://localhost:15672/api")
    .with_basic_auth_credentials("username", "password")
    .build()?;
```

### ClientBuilder: Production Defaults

`with_recommended_defaults` sets a 60-second request timeout,
3 retry attempts with a 1-second delay, and disables redirects:

```rust
use rabbitmq_http_client::api::ClientBuilder;

let rc = ClientBuilder::new()
    .with_endpoint("http://localhost:15672/api")
    .with_basic_auth_credentials("username", "password")
    .with_recommended_defaults()
    .build()?;
```

### ClientBuilder: Timeouts, Retries, and Redirects

These can be configured individually:

```rust
use std::time::Duration;
use rabbitmq_http_client::api::{ClientBuilder, RetrySettings, RedirectPolicy};

let rc = ClientBuilder::new()
    .with_endpoint("http://localhost:15672/api")
    .with_basic_auth_credentials("username", "password")
    // TCP connection timeout (connection establishment + TLS handshake)
    .with_connect_timeout(Duration::from_secs(10))
    // overall request/response cycle timeout
    .with_request_timeout(Duration::from_secs(30))
    // retry up to 3 times with a 500 ms fixed delay
    .with_retry_settings(RetrySettings {
        max_attempts: 3,
        delay_ms: 500,
    })
    // the RabbitMQ HTTP API does not use redirects;
    // disabling them is an SSRF hardening measure
    .with_redirect_policy(RedirectPolicy::none())
    .build()?;
```

These settings have no effect when a custom HTTP client is passed
via `ClientBuilder#with_client`; configure the custom client directly.

### Reachability Probe

`probe_reachability` checks whether the node is reachable and authentication, authorization
steps succeed.
It returns `ReachabilityProbeOutcome`, not `Result`, because both outcomes are expected:

```rust
use rabbitmq_http_client::api::Client;
use rabbitmq_http_client::responses::ReachabilityProbeOutcome;

let rc = Client::new("http://localhost:15672/api", "username", "password");

match rc.probe_reachability().await {
    ReachabilityProbeOutcome::Reached(details) => {
        println!("Connected as {} in {:?}", details.current_user, details.duration);
    }
    ReachabilityProbeOutcome::Unreachable(error) => {
        eprintln!("Unreachable: {}", error.user_message());
    }
}
```

### List Cluster Nodes

```rust
let nodes = rc.list_nodes().await?;
```

### Get Cluster Name

```rust
let name = rc.get_cluster_name().await?;
```

### Cluster Tags

[Cluster tags](https://www.rabbitmq.com/docs/parameters#cluster-tags) are arbitrary key-value
pairs for the cluster:

```rust
use serde_json::{Map, Value, json};

let tags = rc.get_cluster_tags().await?;

let mut new_tags = Map::<String, Value>::new();
new_tags.insert("environment".to_owned(), json!("production"));
rc.set_cluster_tags(new_tags).await?;

rc.clear_cluster_tags().await?;
```

### Node Memory Footprint

```rust
let footprint = rc.get_node_memory_footprint("rabbit@hostname").await?;
```

Returns a per-category [memory footprint breakdown](https://www.rabbitmq.com/docs/memory-use),
in bytes and as percentages.

### Virtual Host Operations

[Virtual hosts](https://www.rabbitmq.com/docs/vhosts) group and isolate resources.

```rust
let vhosts = rc.list_vhosts().await?;
let vhost = rc.get_vhost("/").await?;
```

### Create Virtual Host

```rust
use rabbitmq_http_client::{commons::QueueType, requests::VirtualHostParams};

let params = VirtualHostParams {
    name: "my-vhost",
    description: Some("Production vhost"),
    tags: Some(vec!["production", "critical"]),
    default_queue_type: Some(QueueType::Quorum),
    tracing: false,
};
rc.create_vhost(&params).await?;
```

### Delete Virtual Host

```rust
rc.delete_vhost("my-vhost", false).await?;
```

### [User](https://www.rabbitmq.com/docs/access-control) Operations

```rust
let users = rc.list_users().await?;
let user = rc.current_user().await?;
```

### Create User

Uses [password hashing](https://www.rabbitmq.com/docs/passwords) with salted SHA-256:

```rust
use rabbitmq_http_client::{password_hashing, requests::UserParams};

let salt = password_hashing::salt();
let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "s3kRe7");

let params = UserParams {
    name: "new-user",
    password_hash: &password_hash,
    tags: "management",
};
rc.create_user(&params).await?;
```

SHA-512 is also supported:

```rust
use rabbitmq_http_client::password_hashing::{self, HashingAlgorithm};

let salt = password_hashing::salt();
let hash = password_hashing::base64_encoded_salted_password_hash_sha512(&salt, "s3kRe7");

// or via the algorithm enum
let hash = HashingAlgorithm::SHA512.salt_and_hash(&salt, "s3kRe7")?;
```

### Delete User

```rust
rc.delete_user("new-user", false).await?;
```

### [Connection](https://www.rabbitmq.com/docs/connections) Operations

```rust
let connections = rc.list_connections().await?;
```

### Close Connection

```rust
rc.close_connection("connection-name", Some("closing for maintenance"), false).await?;
```

### [Queue](https://www.rabbitmq.com/docs/queues) Operations

```rust
let queues = rc.list_queues().await?;
let queues_in_vhost = rc.list_queues_in("/").await?;
let info = rc.get_queue_info("/", "my-queue").await?;
```

Queues can also be listed by type:

```rust
let quorum_queues = rc.list_quorum_queues().await?;
let classic_queues = rc.list_classic_queues_in("/").await?;
let streams = rc.list_streams().await?;
```

### Declare a Classic Queue

```rust
use rabbitmq_http_client::requests::QueueParams;

let params = QueueParams::new_durable_classic_queue("my-queue", None);
rc.declare_queue("/", &params).await?;
```

### Declare a Quorum Queue

[Quorum queues](https://www.rabbitmq.com/docs/quorum-queues) are replicated, data safety-oriented queues based on the Raft consensus algorithm.

```rust
use rabbitmq_http_client::requests::QueueParams;
use serde_json::{Map, Value, json};

let mut args = Map::<String, Value>::new();
args.insert("x-max-length".to_owned(), json!(10_000));
let params = QueueParams::new_quorum_queue("my-qq", Some(args));
rc.declare_queue("/", &params).await?;
```

### Type-Safe Queue Arguments with XArgumentsBuilder

`XArgumentsBuilder` is a type-safe alternative to raw `Map<String, Value>` for
[optional queue arguments](https://www.rabbitmq.com/docs/queues#optional-arguments):

```rust
use rabbitmq_http_client::requests::{QueueParams, XArgumentsBuilder, DeadLetterStrategy};

let args = XArgumentsBuilder::new()
    .max_length(10_000)
    .dead_letter_exchange("my-dlx")
    .dead_letter_strategy(DeadLetterStrategy::AtLeastOnce)
    .delivery_limit(5)
    .single_active_consumer(true)
    .build();

let params = QueueParams::new_quorum_queue("my-qq", args);
rc.declare_queue("/", &params).await?;
```

### Declare a Stream

[Streams](https://www.rabbitmq.com/docs/streams) are persistent, replicated append-only logs with non-destructive consumer semantics.

Using `StreamParams`:

```rust
use rabbitmq_http_client::requests::StreamParams;

let params = StreamParams::with_expiration_and_length_limit("my-stream", "7D", 10_000_000_000);
rc.declare_stream("/", &params).await?;
```

Or using `QueueParams`:

```rust
use rabbitmq_http_client::requests::QueueParams;
use serde_json::{Map, Value, json};

let mut args = Map::<String, Value>::new();
args.insert("x-max-length-bytes".to_owned(), json!(10_000_000));
let params = QueueParams::new_stream("my-stream", Some(args));
rc.declare_queue("/", &params).await?;
```

### Purge Queue

```rust
rc.purge_queue("/", "my-queue").await?;
```

### Delete Queue

```rust
rc.delete_queue("/", "my-queue", false).await?;
```

### Batch Queue Deletion

```rust
rc.delete_queues("/", &["queue-1", "queue-2", "queue-3"], false).await?;
```

### Pagination

Some list operations support pagination:

```rust
use rabbitmq_http_client::commons::PaginationParams;

let page = PaginationParams::first_page(100);
let queues = rc.list_queues_paged(&page).await?;

if let Some(next) = page.next_page() {
    let more_queues = rc.list_queues_paged(&next).await?;
}
```

Paginated variants: `list_queues_paged`, `list_queues_in_paged`, `list_connections_paged`.

### [Exchange](https://www.rabbitmq.com/docs/exchanges) Operations

```rust
let exchanges = rc.list_exchanges().await?;
let exchanges_in_vhost = rc.list_exchanges_in("/").await?;
```

### Declare an Exchange

```rust
use rabbitmq_http_client::{commons::ExchangeType, requests::ExchangeParams};

let params = ExchangeParams {
    name: "my-exchange",
    exchange_type: ExchangeType::Topic,
    durable: true,
    auto_delete: false,
    internal: false,
    arguments: None,
};
rc.declare_exchange("/", &params).await?;
```

### Delete Exchange

```rust
rc.delete_exchange("/", "my-exchange", false).await?;
```

### Binding Operations

Bindings connect exchanges to queues or other exchanges.

```rust
let bindings = rc.list_bindings().await?;
let bindings_in_vhost = rc.list_bindings_in("/").await?;
let queue_bindings = rc.list_queue_bindings("/", "my-queue").await?;
```

### Bind Queue to Exchange

```rust
rc.bind_queue("/", "my-queue", "my-exchange", Some("routing.key"), None).await?;
```

### Bind Exchange to Exchange

```rust
rc.bind_exchange("/", "destination-exchange", "source-exchange", Some("routing.key"), None).await?;
```

### Delete Binding

```rust
use rabbitmq_http_client::{commons::BindingDestinationType, requests::BindingDeletionParams};

let params = BindingDeletionParams {
    virtual_host: "/",
    source: "my-exchange",
    destination: "my-queue",
    destination_type: BindingDestinationType::Queue,
    routing_key: "routing.key",
    arguments: None,
};
rc.delete_binding(&params, false).await?;
```

### [Permission](https://www.rabbitmq.com/docs/access-control) Operations

```rust
use rabbitmq_http_client::requests::Permissions;

let params = Permissions {
    vhost: "/",
    user: "new-user",
    configure: ".*",
    read: ".*",
    write: ".*",
};
rc.declare_permissions(&params).await?;
```

### Policy Operations

[Policies](https://www.rabbitmq.com/docs/policies) dynamically configure queue and exchange properties using pattern matching.

```rust
let policies = rc.list_policies().await?;
let policies_in_vhost = rc.list_policies_in("/").await?;
```

### Declare a Policy

```rust
use rabbitmq_http_client::{commons::PolicyTarget, requests::PolicyParams};
use serde_json::{Map, Value, json};

let mut definition = Map::<String, Value>::new();
definition.insert("max-length".to_owned(), json!(10_000));

let params = PolicyParams {
    vhost: "/",
    name: "my-policy",
    pattern: "^my-.*",
    apply_to: PolicyTarget::Queues,
    priority: 10,
    definition,
};
rc.declare_policy(&params).await?;
```

### Delete Policy

```rust
rc.delete_policy("/", "my-policy", false).await?;
```

### Shovel Operations

[Dynamic shovels](https://www.rabbitmq.com/docs/shovel-dynamic) move messages between queues, potentially across clusters.

```rust
use rabbitmq_http_client::commons::MessageTransferAcknowledgementMode;
use rabbitmq_http_client::requests::{
    Amqp091ShovelParams, Amqp091ShovelSourceParams, Amqp091ShovelDestinationParams,
};

let params = Amqp091ShovelParams {
    vhost: "/",
    name: "my-shovel",
    acknowledgement_mode: MessageTransferAcknowledgementMode::WhenConfirmed,
    reconnect_delay: Some(5),
    source: Amqp091ShovelSourceParams::queue_source(
        "amqp://source-host:5672", "source-queue",
    ),
    destination: Amqp091ShovelDestinationParams::queue_destination(
        "amqp://dest-host:5672", "dest-queue",
    ),
};
rc.declare_amqp091_shovel(params).await?;

let shovels = rc.list_shovels().await?;
let shovels_in_vhost = rc.list_shovels_in("/").await?;
rc.delete_shovel("/", "my-shovel", false).await?;
```

AMQP 1.0 shovels use `declare_amqp10_shovel`.

### Federation Operations

[Federation](https://www.rabbitmq.com/docs/federation) replicates exchanges and queues across clusters.

```rust
use rabbitmq_http_client::requests::FederationUpstreamParams;

let params = FederationUpstreamParams {
    vhost: "/",
    name: "upstream-cluster",
    uri: "amqp://remote-host:5672",
    ..Default::default()
};
rc.declare_federation_upstream(params).await?;

let upstreams = rc.list_federation_upstreams().await?;
let links = rc.list_federation_links().await?;
rc.delete_federation_upstream("/", "upstream-cluster", false).await?;
```

### Runtime Parameters

[Runtime parameters](https://www.rabbitmq.com/docs/parameters) store per-vhost plugin settings
such as federation upstream and shovel configurations.

```rust
use rabbitmq_http_client::requests::RuntimeParameterDefinition;
use serde_json::{Map, Value, json};

// set a max-connections limit on a virtual host
let mut value = Map::<String, Value>::new();
value.insert("max-connections".to_owned(), json!(500));

let param = RuntimeParameterDefinition {
    component: "vhost-limits",
    vhost: "/",
    name: "limits",
    value,
};
rc.upsert_runtime_parameter(&param).await?;

let params = rc.list_runtime_parameters().await?;
rc.clear_runtime_parameter("vhost-limits", "/", "limits", false).await?;
```

### Global Runtime Parameters

Global [runtime parameters](https://www.rabbitmq.com/docs/parameters) are cluster-wide,
not scoped to a virtual host.

```rust
use rabbitmq_http_client::requests::GlobalRuntimeParameterDefinition;
use serde_json::{Map, Value, json};

// tag the cluster with metadata
let mut tags = Map::<String, Value>::new();
tags.insert("region".to_owned(), json!("ca-central-1"));
tags.insert("environment".to_owned(), json!("production"));

let mut value = Map::<String, Value>::new();
value.insert("cluster_tags".to_owned(), json!(tags));

let param = GlobalRuntimeParameterDefinition {
    name: "cluster_tags",
    value,
};
rc.upsert_global_runtime_parameter(&param).await?;

let params = rc.list_global_runtime_parameters().await?;
rc.clear_global_runtime_parameter("cluster_tags").await?;
```

### Definition Operations

[Definitions](https://www.rabbitmq.com/docs/definitions) contain schema, topology, and user metadata for export and import.

```rust
let definitions = rc.export_cluster_wide_definitions().await?;
let vhost_definitions = rc.export_vhost_definitions("/").await?;
```

### Import Definitions

```rust
let defs: serde_json::Value = serde_json::from_str(&definitions)?;
rc.import_definitions(defs).await?;
```

### Definition Transformers

Exported definitions can be transformed before import, e.g. to migrate from
[classic mirrored queues to quorum queues](https://www.rabbitmq.com/blog/2025/07/29/latest-benefits-of-rmq-and-migrating-to-qq-along-the-way):

```rust
use rabbitmq_http_client::transformers::{
    TransformationChain, PrepareForQuorumQueueMigration, DropEmptyPolicies,
};

let mut defs = rc.export_cluster_wide_definitions_as_data().await?;

let chain = TransformationChain::new(vec![
    Box::new(PrepareForQuorumQueueMigration {}),
    Box::new(DropEmptyPolicies {}),
]);
chain.apply(&mut defs);
```

Available transformers:

| Transformer                           | Description                                                                 |
|---------------------------------------|-----------------------------------------------------------------------------|
| `PrepareForQuorumQueueMigration`      | Strips classic mirrored queue policy keys and incompatible x-arguments      |
| `StripCmqKeysFromPolicies`            | Removes only the classic mirrored queue-related keys from policies          |
| `DropEmptyPolicies`                   | Removes policies with empty definitions (use after stripping CMQ keys)      |
| `ExcludeUsers`                        | Removes all users from the definition set                                   |
| `ExcludePermissions`                  | Removes all permissions from the definition set                             |
| `ExcludeRuntimeParameters`            | Removes all runtime parameters from the definition set                      |
| `ExcludePolicies`                     | Removes all policies from the definition set                                |
| `ObfuscateUsernames`                  | Replaces usernames and passwords with dummy values                          |

Virtual host-level equivalents (`PrepareForQuorumQueueMigrationVhost`, `StripCmqKeysFromVhostPolicies`,
`DropEmptyVhostPolicies`) use `VirtualHostTransformationChain`.

### [Health Checks](https://www.rabbitmq.com/docs/monitoring)

```rust
let alarms = rc.health_check_cluster_wide_alarms().await?;
let quorum_critical = rc.health_check_if_node_is_quorum_critical().await?;
let port_check = rc.health_check_port_listener(5672).await?;
let protocol_check = rc.health_check_protocol_listener("amqp").await?;
```

### Feature Flags

[Feature flags](https://www.rabbitmq.com/docs/feature-flags) gate new functionality that requires
cluster-wide coordination.

```rust
let flags = rc.list_feature_flags().await?;
rc.enable_feature_flag("feature_name").await?;
rc.enable_all_stable_feature_flags().await?;
```

### Deprecated Features

```rust
let all = rc.list_all_deprecated_features().await?;
let in_use = rc.list_deprecated_features_in_use().await?;
```

### Rebalance Queue Leaders

Redistributes quorum queue and stream leaders across cluster nodes.

```rust
rc.rebalance_queue_leaders().await?;
```

### Idempotent Deletes

All `delete_*` and `clear_*` functions accept an `idempotently: bool` argument.
When `true`, `404 Not Found` responses are silently ignored:

```rust
// will not fail if the queue does not exist
rc.delete_queue("/", "my-queue", true).await?;
```

### Error Classification

`HttpClientError` has methods for classifying errors:

```rust
match rc.delete_queue("/", "my-queue", false).await {
    Ok(()) => {}
    Err(e) if e.is_not_found() => eprintln!("Queue not found"),
    Err(e) if e.is_unauthorized() => eprintln!("Bad credentials"),
    Err(e) if e.is_connection_error() => eprintln!("Cannot reach node"),
    Err(e) if e.is_timeout() => eprintln!("Request timed out"),
    Err(e) => eprintln!("Error: {}", e.user_message()),
}
```

Predicates: `is_not_found`, `is_already_exists`, `is_unauthorized`, `is_forbidden`,
`is_client_error`, `is_server_error`, `is_connection_error`, `is_timeout`, `is_tls_handshake_error`.

For further detail: `status_code`, `url`, `error_details`.

### URI Builder for Federation and Shovel URIs

`UriBuilder` builds AMQP URIs with [TLS parameters](https://www.rabbitmq.com/docs/ssl)
for [federation](https://www.rabbitmq.com/docs/federation) and [shovel](https://www.rabbitmq.com/docs/shovel) connections:

```rust
use rabbitmq_http_client::uris::{UriBuilder, TlsClientSettings};
use rabbitmq_http_client::commons::TlsPeerVerificationMode;

let uri = UriBuilder::new("amqps://user:pass@remote-host:5671/vhost")
    .unwrap()
    .with_tls_peer_verification(TlsPeerVerificationMode::Enabled)
    .with_ca_cert_file("/path/to/ca_bundle.pem")
    .build()
    .unwrap();

// group TLS settings for reuse across multiple URIs
let tls = TlsClientSettings::with_verification()
    .ca_cert_file("/path/to/ca_bundle.pem")
    .client_cert_file("/path/to/client_cert.pem")
    .client_key_file("/path/to/client_key.pem");

let uri = UriBuilder::new("amqps://user:pass@remote-host:5671/vhost")
    .unwrap()
    .replace(tls)
    .build()
    .unwrap();
```

### Tanzu RabbitMQ: Schema Definition Sync and Warm Standby Replication

Tanzu RabbitMQ [Schema Definition Sync](https://www.rabbitmq.com/docs/definitions#import-on-boot-schema-definition-sync) (SDS)
and Warm Standby Replication (WSR):

```rust
rc.enable_schema_definition_sync_on_node(Some("rabbit@hostname")).await?;
rc.disable_schema_definition_sync_on_node(Some("rabbit@hostname")).await?;
```


## Blocking Client

The blocking client is in `rabbitmq_http_client::blocking_api`. It has the same API as the async client but without `async`/`await`.

### Instantiate a Client

```rust
use rabbitmq_http_client::blocking_api::Client;

let endpoint = "http://localhost:15672/api";
let rc = Client::new(&endpoint, "username", "password");
```

### Using ClientBuilder

```rust
use rabbitmq_http_client::blocking_api::ClientBuilder;

let rc = ClientBuilder::new()
    .with_endpoint("http://localhost:15672/api")
    .with_basic_auth_credentials("username", "password")
    .build()?;
```

### List Cluster Nodes

```rust
let nodes = rc.list_nodes()?;
```

### Virtual Host Operations

```rust
let vhosts = rc.list_vhosts()?;
```

### Create Virtual Host

```rust
use rabbitmq_http_client::{commons::QueueType, requests::VirtualHostParams};

let params = VirtualHostParams {
    name: "my-vhost",
    description: Some("Production vhost"),
    tags: Some(vec!["production"]),
    default_queue_type: Some(QueueType::Quorum),
    tracing: false,
};
rc.create_vhost(&params)?;
```

### Delete Virtual Host

```rust
rc.delete_vhost("my-vhost", false)?;
```

### User Operations

```rust
let users = rc.list_users()?;
```

### Create User

```rust
use rabbitmq_http_client::{password_hashing, requests::UserParams};

let salt = password_hashing::salt();
let password_hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "s3kRe7");

let params = UserParams {
    name: "new-user",
    password_hash: &password_hash,
    tags: "management",
};
rc.create_user(&params)?;
```

### Connection Operations

```rust
let connections = rc.list_connections()?;
```

### Queue Operations

```rust
let queues = rc.list_queues()?;
let queues_in_vhost = rc.list_queues_in("/")?;
```

### Declare a Queue

```rust
use rabbitmq_http_client::requests::QueueParams;

let params = QueueParams::new_durable_classic_queue("my-queue", None);
rc.declare_queue("/", &params)?;
```

### Declare a Quorum Queue

```rust
use rabbitmq_http_client::requests::QueueParams;
use serde_json::{Map, Value, json};

let mut args = Map::<String, Value>::new();
args.insert("x-max-length".to_owned(), json!(10_000));
let params = QueueParams::new_quorum_queue("my-qq", Some(args));
rc.declare_queue("/", &params)?;
```

### Purge Queue

```rust
rc.purge_queue("/", "my-queue")?;
```

### Delete Queue

```rust
rc.delete_queue("/", "my-queue", false)?;
```

### Exchange Operations

```rust
let exchanges = rc.list_exchanges()?;
```

### Declare an Exchange

```rust
use rabbitmq_http_client::{commons::ExchangeType, requests::ExchangeParams};

let params = ExchangeParams {
    name: "my-exchange",
    exchange_type: ExchangeType::Topic,
    durable: true,
    auto_delete: false,
    internal: false,
    arguments: None,
};
rc.declare_exchange("/", &params)?;
```

### Delete Exchange

```rust
rc.delete_exchange("/", "my-exchange", false)?;
```

### Binding Operations

```rust
let bindings = rc.list_bindings()?;
let queue_bindings = rc.list_queue_bindings("/", "my-queue")?;
```

### Bind Queue to Exchange

```rust
rc.bind_queue("/", "my-queue", "my-exchange", Some("routing.key"), None)?;
```

### Permission Operations

```rust
use rabbitmq_http_client::requests::Permissions;

let params = Permissions {
    vhost: "/",
    user: "new-user",
    configure: ".*",
    read: ".*",
    write: ".*",
};
rc.declare_permissions(&params)?;
```

### Policy Operations

```rust
use rabbitmq_http_client::{commons::PolicyTarget, requests::PolicyParams};
use serde_json::{Map, Value, json};

let mut definition = Map::<String, Value>::new();
definition.insert("max-length".to_owned(), json!(10_000));

let params = PolicyParams {
    vhost: "/",
    name: "my-policy",
    pattern: "^my-.*",
    apply_to: PolicyTarget::Queues,
    priority: 10,
    definition,
};
rc.declare_policy(&params)?;
```

### Definition Operations

```rust
let definitions = rc.export_cluster_wide_definitions()?;
```

### Health Checks

```rust
let alarms = rc.health_check_cluster_wide_alarms()?;
```


## TLS Support

This client makes few assumptions about [TLS configuration](https://www.rabbitmq.com/docs/ssl).

Use the [`reqwest::ClientBuilder`](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html) to configure TLS, then pass the client to this library's `ClientBuilder::with_client`:

```rust
use reqwest::blocking::Client as HTTPClient;
use rabbitmq_http_client::blocking_api::ClientBuilder;

// Configure TLS using reqwest's ClientBuilder
let mut b = HTTPClient::builder()
    .user_agent("my-app")
    .min_tls_version(reqwest::tls::Version::TLS_1_2)
    .danger_accept_invalid_certs(false)
    .danger_accept_invalid_hostnames(false);

// Add a CA certificate bundle file to the list of trusted roots
// for x.509 peer verification
b = b.add_root_certificate(ca_certificate);

let httpc = b.build()?;

// Use the TLS port (15671) instead of the default HTTP port (15672)
let endpoint = "https://rabbitmq.example.com:15671/api";

// Pass the pre-configured HTTP client to this library's ClientBuilder
let client = ClientBuilder::new()
    .with_endpoint(endpoint)
    .with_basic_auth_credentials("username", "password")
    .with_client(httpc)
    .build()?;
```

Choose the [certificate store](https://github.com/rustls/rustls-platform-verifier?tab=readme-ov-file#deployment-considerations) for x.509 peer verification and the minimum TLS version.

### Defaults

By default, this client uses [`native-tls`](https://crates.io/crates/native-tls).

Trusted CA certificates are managed via OS-specific mechanisms (Keychain on macOS, `openssl` directories on Linux).


## Combined Examples

### 1. Setting Up an Application Environment

Create an isolated vhost with a dedicated user, topology, and permissions:

```rust
use rabbitmq_http_client::{
    api::Client,
    commons::{ExchangeType, QueueType},
    password_hashing,
    requests::{ExchangeParams, Permissions, QueueParams, UserParams, VirtualHostParams},
};

async fn setup_environment(rc: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // Create vhost with quorum queues as default
    let vh = VirtualHostParams {
        name: "myapp",
        description: Some("MyApp production"),
        tags: Some(vec!["production"]),
        default_queue_type: Some(QueueType::Quorum),
        tracing: false,
    };
    rc.create_vhost(&vh).await?;

    // Create application user
    let salt = password_hashing::salt();
    let hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, "s3cret");
    rc.create_user(&UserParams { name: "myapp", password_hash: &hash, tags: "" }).await?;

    // Grant limited permissions
    rc.declare_permissions(&Permissions {
        vhost: "myapp", user: "myapp",
        configure: "^myapp\\.", read: ".*", write: ".*",
    }).await?;

    // Create exchange and queues
    rc.declare_exchange("myapp", &ExchangeParams {
        name: "myapp.events", exchange_type: ExchangeType::Topic,
        durable: true, auto_delete: false, internal: false, arguments: None,
    }).await?;

    rc.declare_queue("myapp", &QueueParams::new_quorum_queue("myapp.orders", None)).await?;
    rc.bind_queue("myapp", "myapp.orders", "myapp.events", Some("order.#"), None).await?;

    Ok(())
}
```

### 2. Blue-Green Deployment Migration

Set up [queue federation](https://www.rabbitmq.com/docs/federated-queues) for migrating from an old cluster to a new one.
This [RabbitMQ blog post](https://www.rabbitmq.com/blog/2025/07/29/latest-benefits-of-rmq-and-migrating-to-qq-along-the-way) covers the approach in detail.

```rust
use rabbitmq_http_client::{api::Client, commons::PolicyTarget, requests::PolicyParams};
use serde_json::{Map, Value, json};

// green: the new cluster
// blue_uri: e.g., amqp://user:pass@blue-cluster:5672/vhost
async fn setup_federation_for_migration(
    green: &Client,
    blue_uri: &str,
    vhost: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create federation upstream pointing to old cluster
    let upstream_params = rabbitmq_http_client::requests::FederationUpstreamParams {
        vhost,
        name: "blue-cluster",
        uri: blue_uri,
        ack_mode: rabbitmq_http_client::commons::MessageTransferAcknowledgementMode::WhenConfirmed,
        ..Default::default()
    };
    green.declare_federation_upstream(upstream_params).await?;

    // 2. Create blanket policy to federate all queues
    let mut def = Map::<String, Value>::new();
    def.insert("federation-upstream-set".to_owned(), json!("all"));

    green.declare_policy(&PolicyParams {
        vhost,
        name: "migrate-all-queues",
        pattern: ".*",
        apply_to: PolicyTarget::Queues,
        // low priority, so it will not override existing policies
        priority: -10,
        definition: def,
    }).await?;

    // 3. Verify federation links are running
    let links = green.list_federation_links().await?;
    for link in links {
        println!("Federation link: {} ({}, {})", link.upstream, link.typ, link.status);
    }

    Ok(())
}

async fn cleanup_migration(green: &Client, vhost: &str) -> Result<(), Box<dyn std::error::Error>> {
    green.delete_policy(vhost, "migrate-all-queues", true).await?;
    green.delete_federation_upstream(vhost, "blue-cluster", true).await?;
    Ok(())
}
```

### 3. Health Monitoring

```rust
use rabbitmq_http_client::api::Client;

async fn check_health(rc: &Client) -> Result<(), Box<dyn std::error::Error>> {
    // Check for alarms (memory, disk)
    if rc.health_check_cluster_wide_alarms().await.is_err() {
        eprintln!("WARNING: Active alarms in the cluster");
    }

    // Check quorum criticality before maintenance
    if rc.health_check_if_node_is_quorum_critical().await.is_err() {
        eprintln!("WARNING: This node is critical for quorum");
    }

    // Report queue depths
    for q in rc.list_queues().await? {
        if q.message_count > 10_000 {
            println!("{}: {} messages", q.name, q.message_count);
        }
    }

    Ok(())
}
```

### 4. Backup and Restore Definitions

```rust
use rabbitmq_http_client::api::Client;

async fn backup_and_restore(source: &Client, dest: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let defs = source.export_cluster_wide_definitions_as_data().await?;

    println!("Backing up {} vhosts, {} queues, {} exchanges",
        defs.virtual_hosts.len(), defs.queues.len(), defs.exchanges.len());

    dest.import_definitions(serde_json::to_value(&defs)?).await?;
    Ok(())
}
```

### 5. Event Topology with Dead-Lettering

Set up a fan-out pattern with dead-lettering for failed messages:

```rust
use rabbitmq_http_client::{
    api::Client,
    commons::ExchangeType,
    requests::{ExchangeParams, QueueParams},
};
use serde_json::{Map, Value, json};

async fn setup_event_topology(rc: &Client, vhost: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Main event exchange
    rc.declare_exchange(vhost, &ExchangeParams {
        name: "events",
        exchange_type: ExchangeType::Topic,
        durable: true,
        auto_delete: false,
        internal: false,
        arguments: None,
    }).await?;

    // Dead letter exchange for failed messages
    rc.declare_exchange(vhost, &ExchangeParams {
        name: "events.dlx",
        exchange_type: ExchangeType::Fanout,
        durable: true,
        auto_delete: false,
        internal: false,
        arguments: None,
    }).await?;

    // Service queues with dead-lettering
    let services = ["billing", "notifications", "analytics"];
    for svc in services {
        let mut args = Map::<String, Value>::new();
        args.insert("x-dead-letter-exchange".to_owned(), json!("events.dlx"));

        let queue_name = format!("events.{}", svc);
        rc.declare_queue(vhost, &QueueParams::new_quorum_queue(&queue_name, Some(args))).await?;
        rc.bind_queue(vhost, &queue_name, "events", Some(&format!("{}.*", svc)), None).await?;

        // DLQ for each service
        let dlq_name = format!("events.{}.dlq", svc);
        rc.declare_queue(vhost, &QueueParams::new_quorum_queue(&dlq_name, None)).await?;
        rc.bind_queue(vhost, &dlq_name, "events.dlx", None, None).await?;
    }

    Ok(())
}
```

### 6. Pre-Upgrade Health Verification

Before a [rolling upgrade](https://www.rabbitmq.com/docs/upgrade):

```rust
use rabbitmq_http_client::api::Client;

async fn pre_upgrade_checks(rc: &Client) -> Result<bool, Box<dyn std::error::Error>> {
    let mut ready = true;

    // 1. No active alarms
    if rc.health_check_cluster_wide_alarms().await.is_err() {
        eprintln!("BLOCKED: Cluster has active resource alarms");
        ready = false;
    }

    // 2. No nodes being drained
    let nodes = rc.list_nodes().await?;
    for node in &nodes {
        if node.being_drained {
            eprintln!("BLOCKED: Node {} is being drained", node.name);
            ready = false;
        }
    }

    // 3. No quorum-critical nodes
    if rc.health_check_if_node_is_quorum_critical().await.is_err() {
        eprintln!("WARNING: This node is quorum-critical; the upgrade may cause unavailability");
    }

    // 4. Feature flags enabled
    use rabbitmq_http_client::responses::{FeatureFlagState, FeatureFlagStability};
    let flags = rc.list_feature_flags().await?;
    let disabled: Vec<_> = flags.iter()
        .filter(|f| f.state != FeatureFlagState::Enabled && f.stability == FeatureFlagStability::Stable)
        .collect();
    if !disabled.is_empty() {
        eprintln!("WARNING: {} stable feature flags are not enabled", disabled.len());
    }

    // 5. Check for deprecated features in use
    let deprecated = rc.list_deprecated_features_in_use().await?;
    if !deprecated.is_empty() {
        eprintln!("WARNING: {} deprecated features are in use", deprecated.len());
        for feat in &deprecated {
            eprintln!("  - {}", feat.name);
        }
    }

    // 6. Check port listeners
    for port in [5672, 15672, 5552] {
        if rc.health_check_port_listener(port).await.is_err() {
            eprintln!("WARNING: Port {} is not listening", port);
        }
    }

    Ok(ready)
}
```

### 7. Connection and Channel Audit

```rust
use rabbitmq_http_client::api::Client;

async fn audit_connections(rc: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let connections = rc.list_connections().await?;

    // Group by user
    let mut by_user: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for conn in &connections {
        by_user.entry(conn.username.clone()).or_default().push(conn);
    }

    println!("Connection summary ({} total):", connections.len());
    for (user, conns) in &by_user {
        println!("  {}: {} connections", user, conns.len());
    }

    // Find connections with high channel count
    for conn in connections.iter().filter(|c| c.channel_count > 100) {
        println!("WARNING: {} has {} channels (user: {})",
            conn.name, conn.channel_count, conn.username);
    }

    // Find channels with many unacknowledged messages
    let channels = rc.list_channels().await?;
    for ch in channels.iter().filter(|c| c.messages_unacknowledged > 1000) {
        println!("WARNING: {} has {} unacked messages", ch.name, ch.messages_unacknowledged);
    }

    Ok(())
}
```

### 8. Multi-Tenant Setup

Provision isolated environments for multiple tenants:

```rust
use rabbitmq_http_client::{
    api::Client,
    commons::{ExchangeType, QueueType},
    password_hashing,
    requests::{ExchangeParams, Permissions, UserParams, VirtualHostParams},
};

async fn provision_tenant(
    rc: &Client,
    tenant_id: &str,
    admin_password: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let vhost = format!("tenant-{}", tenant_id);
    let admin_user = format!("{}-admin", tenant_id);
    let app_user = format!("{}-app", tenant_id);

    // Create isolated vhost
    rc.create_vhost(&VirtualHostParams {
        name: &vhost,
        description: Some(&format!("Tenant {} environment", tenant_id)),
        tags: Some(vec!["tenant", tenant_id]),
        default_queue_type: Some(QueueType::Quorum),
        tracing: false,
    }).await?;

    // Create admin user with management access
    let salt = password_hashing::salt();
    let hash = password_hashing::base64_encoded_salted_password_hash_sha256(&salt, admin_password);
    rc.create_user(&UserParams {
        name: &admin_user,
        password_hash: &hash,
        tags: "management",
    }).await?;

    rc.declare_permissions(&Permissions {
        vhost: &vhost,
        user: &admin_user,
        configure: ".*",
        read: ".*",
        write: ".*",
    }).await?;

    // Create app user with limited permissions
    let app_hash = password_hashing::base64_encoded_salted_password_hash_sha256(
        &password_hashing::salt(),
        &format!("{}-app-secret", tenant_id),
    );
    rc.create_user(&UserParams {
        name: &app_user,
        password_hash: &app_hash,
        tags: "",
    }).await?;

    // Only allow configuring server-named queues,
    // see https://www.rabbitmq.com/docs/queues#server-named-queues
    rc.declare_permissions(&Permissions {
        vhost: &vhost,
        user: &app_user,
        configure: "^amq\\.gen-.*",
        read: ".*",
        write: ".*",
    }).await?;

    // Standard topology for each tenant
    rc.declare_exchange(&vhost, &ExchangeParams {
        name: "commands",
        exchange_type: ExchangeType::Direct,
        durable: true,
        auto_delete: false,
        internal: false,
        arguments: None,
    }).await?;

    rc.declare_exchange(&vhost, &ExchangeParams {
        name: "events",
        exchange_type: ExchangeType::Topic,
        durable: true,
        auto_delete: false,
        internal: false,
        arguments: None,
    }).await?;

    Ok(())
}
```

### 9. Teardown Test Environment

Clean up after integration tests:

```rust
use rabbitmq_http_client::api::Client;

async fn teardown_test_env(
    rc: &Client,
    vhost: &str,
    users: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    // Delete all queues (bindings are removed automatically)
    if let Ok(queues) = rc.list_queues_in(vhost).await {
        for q in queues {
            let _ = rc.delete_queue(vhost, &q.name, true).await;
        }
    }

    // Delete non-default exchanges
    if let Ok(exchanges) = rc.list_exchanges_in(vhost).await {
        for x in exchanges {
            if !x.name.is_empty() && !x.name.starts_with("amq.") {
                let _ = rc.delete_exchange(vhost, &x.name, true).await;
            }
        }
    }

    // Delete policies
    if let Ok(policies) = rc.list_policies_in(vhost).await {
        for p in policies {
            let _ = rc.delete_policy(vhost, &p.name, true).await;
        }
    }

    // Delete test users
    for user in users {
        let _ = rc.delete_user(user, true).await;
    }

    // Delete the vhost last
    let _ = rc.delete_vhost(vhost, true).await;

    Ok(())
}
```


## License

This crate, rabbitmq-http-api-client-rs, is dual-licensed under
the Apache Software License 2.0 and the MIT license.
