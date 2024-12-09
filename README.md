# A Rust Client for the RabbitMQ HTTP API

This is a Rust client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/management#http-api).

This is not an AMQP 0-9-1 client (see [amqprs](https://github.com/gftea/amqprs)), an AMQP 1.0 client (see [fe2o3-amqp](https://github.com/minghuaw/fe2o3-amqp)) or [RabbitMQ Stream protocol client](https://github.com/rabbitmq/rabbitmq-stream-rust-client) library.

## Project Maturity

This library is relatively young, breaking API changes are possible.

## Dependency

### Blocking Client

```toml
rabbitmq_http_client = { git = "https://github.com/michaelklishin/rabbitmq-http-api-rs", features = ["core", "blocking"] }
```

### Async Client

```toml
rabbitmq_http_client = { git = "https://github.com/michaelklishin/rabbitmq-http-api-rs", features = ["core", "async"] }
```

### Blocking Client with Tabled Support

```toml
rabbitmq_http_client = { git = "https://github.com/michaelklishin/rabbitmq-http-api-rs", features = ["core", "blocking", "tabled"] }
```

### Async CLient with Tabled Support

```toml
rabbitmq_http_client = { git = "https://github.com/michaelklishin/rabbitmq-http-api-rs", features = ["core", "async", "tabled"] }
```


## Usage

### Overview

This library offers two client implementations: a blocking one and an async one,
in `rabbitmq_http_client::blocking_api` and `rabbitmq_http_client::api`, respectively.

Both API versions and [`tabled`](https://docs.rs/tabled/latest/tabled/) support are optional features.

### Code Examples

Documentation for async API follows that of the blocking API.

### Blocking API

The examples below do not cover the entire API. Most ``

#### Instantiate a Client

```rust
use rabbitmq_http_client::blocking_api::ClientBuilder;

// a type alias for convenience
type APIClient<'a> = Client<&'a str, &'a str, &'a str>;

let endpoint = "http://localhost:15672/api";
let username = "username";
let password = "password";

let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();

// list cluster nodes
let _ = rc.list_nodes();

// list user connections
let _ = rc.list_connections();

// fetch information and metrics of a specific queue
let _ = rc.get_queue_info("/", "qq.1");
```

#### List Cluster Nodes

```rust
let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();

let _ = rc.list_nodes();
```

#### List Client Connections

```rust
let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();

let _ = rc.list_connections();
```

#### Fetch Metrics of a Specific Queue or Stream

```rust
let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();

// fetch information and metrics of a specific queue or stream
let _ = rc.get_queue_info("/", "qq.1");
```

### Async API

#### Instantiate a Client

```rust
use rabbitmq_http_client::api::ClientBuilder;

// a type alias for convenience
type APIClient<'a> = Client<&'a str, &'a str, &'a str>;

let endpoint = "http://localhost:15672/api";
let username = "username";
let password = "password";
let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();
```

#### List Cluster Members

```rust
let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();

rc.list_nodes().await;
```

#### List Client Connections

```rust
let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();

rc.list_connections().await;
```

#### Fetch Metrics of a Specific Queue or Stream

```rust
let rc = ClientBuilder::new().with_endpoint(&endpoint).with_basic_auth_credentials(&username, &password).build();

rc.get_queue_info("/", "qq.1").await;
```


## License

This crate, rabbitmq-http-api-client-rs, is dual-licensed under
the Apache Software License 2.0 and the MIT license.
