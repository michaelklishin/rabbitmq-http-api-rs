# A Rust Client for the RabbitMQ HTTP API

This is a Rust client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/management#http-api).

See the [HTTP API reference](https://www.rabbitmq.com/docs/http-api-reference) for the complete list of supported endpoints.

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
rabbitmq_http_client = { version = "0.79.0", features = ["core", "async"] }
```

### Blocking Client

```toml
rabbitmq_http_client = { version = "0.79.0", features = ["core", "blocking"] }
```

### With Tabled Support

```toml
rabbitmq_http_client = { version = "0.79.0", features = ["core", "async", "tabled"] }
rabbitmq_http_client = { version = "0.79.0", features = ["core", "blocking", "tabled"] }
```


## Async Client

The async client is in `rabbitmq_http_client::api`.

### Instantiate a Client

```rust
use rabbitmq_http_client::api::Client;

let endpoint = "http://localhost:15672/api";
let rc = Client::new(&endpoint, "username", "password");
```

### Using ClientBuilder

```rust
use rabbitmq_http_client::api::ClientBuilder;

let rc = ClientBuilder::new()
    .with_endpoint("http://localhost:15672/api")
    .with_basic_auth_credentials("username", "password")
    .build();
```

### List Cluster Nodes

```rust
let nodes = rc.list_nodes().await?;
```

### Get Cluster Name

```rust
let name = rc.get_cluster_name().await?;
```

### Virtual Host Operations

[Virtual hosts](https://www.rabbitmq.com/docs/vhosts) provide logical grouping and separation of resources.

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

### User Operations

See [Access Control](https://www.rabbitmq.com/docs/access-control) for authentication and authorization details.

```rust
let users = rc.list_users().await?;
```

### Create User

Uses [password hashing](https://www.rabbitmq.com/docs/passwords) with salted SHA-256.

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

### Delete User

```rust
rc.delete_user("new-user", false).await?;
```

### Connection Operations

See [Connections](https://www.rabbitmq.com/docs/connections) for lifecycle and monitoring details.

```rust
let connections = rc.list_connections().await?;
```

### Close Connection

```rust
rc.close_connection("connection-name", Some("closing for maintenance"), false).await?;
```

### Queue Operations

See [Queues](https://www.rabbitmq.com/docs/queues) for queue properties and behaviours.

```rust
let queues = rc.list_queues().await?;
let queues_in_vhost = rc.list_queues_in("/").await?;
let info = rc.get_queue_info("/", "my-queue").await?;
```

### Declare a Classic Queue

See [Classic Queues](https://www.rabbitmq.com/docs/classic-queues).

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

### Declare a Stream

[Streams](https://www.rabbitmq.com/docs/streams) are persistent, replicated append-only logs with non-destructive consumer semantics.

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

### Exchange Operations

See [Exchanges](https://www.rabbitmq.com/docs/exchanges) to learn more about the exchange types and their routing semantics.

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

See [Exchange-to-Exchange Bindings](https://www.rabbitmq.com/docs/e2e).

```rust
rc.bind_exchange("/", "destination-exchange", "source-exchange", Some("routing.key"), None).await?;
```

### Delete Binding

```rust
use rabbitmq_http_client::{commons::BindingDestinationType, requests::BindingDeletionParams};

let params = BindingDeletionParams {
    vhost: "/",
    source: "my-exchange",
    destination: "my-queue",
    destination_type: BindingDestinationType::Queue,
    routing_key: "routing.key",
    properties_key: "routing.key",
};
rc.delete_binding(&params, false).await?;
```

### Permission Operations

See [Access Control](https://www.rabbitmq.com/docs/access-control) for permission patterns.

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

### Definition Operations

[Definitions](https://www.rabbitmq.com/docs/definitions) contain schema, topology, and user metadata for export and import.

```rust
let definitions = rc.export_cluster_wide_definitions().await?;
let vhost_definitions = rc.export_vhost_definitions("/").await?;
```

### Import Definitions

```rust
rc.import_definitions(&definitions).await?;
```

### Health Checks

See [Monitoring](https://www.rabbitmq.com/docs/monitoring) for health check details.

```rust
let alarms = rc.health_check_cluster_wide_alarms().await?;
let quorum_critical = rc.health_check_if_node_is_quorum_critical().await?;
let port_check = rc.health_check_port_listener(5672).await?;
```

### Feature Flags

See [Feature Flags](https://www.rabbitmq.com/docs/feature-flags).

```rust
let flags = rc.list_feature_flags().await?;
rc.enable_feature_flag("feature_name").await?;
rc.enable_all_stable_feature_flags().await?;
```

### Rebalance Queue Leaders

Redistributes quorum queue and stream leaders across cluster nodes.

```rust
rc.rebalance_queue_leaders().await?;
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
    .build();
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
    .build();
```

The user chooses the [certificate store](https://github.com/rustls/rustls-platform-verifier?tab=readme-ov-file#deployment-considerations) for x.509 peer verification and the minimum TLS version.

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
See this [RabbitMQ blog post](https://www.rabbitmq.com/blog/2025/07/29/latest-benefits-of-rmq-and-migrating-to-qq-along-the-way) for context.

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
    let upstream_params = rabbitmq_http_client::requests::EnforcedFederationUpstreamParams {
        vhost,
        name: "blue-cluster",
        uri: blue_uri,
        ack_mode: Some(rabbitmq_http_client::commons::MessageTransferAcknowledgementMode::WhenConfirmed),
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
        println!("Federation link: {} -> {} ({})", link.upstream, link.queue, link.status);
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
        if q.messages.unwrap_or(0) > 10_000 {
            println!("{}: {} messages", q.name, q.messages.unwrap_or(0));
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

    dest.import_definitions(&serde_json::to_string(&defs)?).await?;
    Ok(())
}
```

### 5. Event-Driven Architecture Topology

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

Run comprehensive checks before a [rolling upgrade](https://www.rabbitmq.com/docs/upgrade):

```rust
use rabbitmq_http_client::api::Client;

async fn pre_upgrade_checks(rc: &Client) -> Result<bool, Box<dyn std::error::Error>> {
    let mut ready = true;

    // 1. No active alarms
    if rc.health_check_cluster_wide_alarms().await.is_err() {
        eprintln!("BLOCKED: Cluster has active resource alarms");
        ready = false;
    }

    // 2. All nodes running
    let nodes = rc.list_nodes().await?;
    for node in &nodes {
        if !node.running {
            eprintln!("BLOCKED: Node {} is not running", node.name);
            ready = false;
        }
    }

    // 3. No quorum-critical nodes
    if rc.health_check_if_node_is_quorum_critical().await.is_err() {
        eprintln!("WARNING: This node is quorum-critical; the upgrade may cause unavailability");
    }

    // 4. Feature flags enabled
    let flags = rc.list_feature_flags().await?;
    let disabled: Vec<_> = flags.iter()
        .filter(|f| f.state != "enabled" && f.stability == "stable")
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

    // 6. Check protocol listeners
    for port in [5672, 15672, 5552] {
        if rc.health_check_port_listener(port).await.is_err() {
            eprintln!("WARNING: Port {} is not listening", port);
        }
    }

    Ok(ready)
}
```

### 7. Connection and Channel Audit

Monitor connection health and detect issues:

```rust
use rabbitmq_http_client::api::Client;

async fn audit_connections(rc: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let connections = rc.list_connections().await?;

    // Group by user
    let mut by_user: std::collections::HashMap<String, Vec<_>> = std::collections::HashMap::new();
    for conn in &connections {
        by_user.entry(conn.user.clone()).or_default().push(conn);
    }

    println!("Connection summary ({} total):", connections.len());
    for (user, conns) in &by_user {
        println!("  {}: {} connections", user, conns.len());
    }

    // Find connections with high channel count
    for conn in connections.iter().filter(|c| c.channels.unwrap_or(0) > 100) {
        println!("WARNING: {} has {} channels (user: {})",
            conn.name, conn.channels.unwrap_or(0), conn.user);
    }

    // Find idle channels
    let channels = rc.list_channels().await?;
    for ch in channels.iter().filter(|c| c.idle_since.is_some()) {
        println!("Idle channel: {} on {}", ch.name, ch.connection_name);
    }

    Ok(())
}
```

### 8. Multi-Tenant Topology Setup

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
