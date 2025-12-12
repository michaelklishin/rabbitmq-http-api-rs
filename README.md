# A Rust Client for the RabbitMQ HTTP API

This is a Rust client for the [RabbitMQ HTTP API](https://www.rabbitmq.com/docs/management#http-api).

This is not an AMQP 0-9-1 client (see [amqprs](https://github.com/gftea/amqprs)), an AMQP 1.0 client (see [fe2o3-amqp](https://github.com/minghuaw/fe2o3-amqp)) or [RabbitMQ Stream protocol client](https://github.com/rabbitmq/rabbitmq-stream-rust-client) library.

## Project Maturity

This library is reasonably mature.

Before `1.0.0`, breaking API changes are can and will be introduced.

## Supported RabbitMQ Series

This library targets RabbitMQ 4.x and 3.13.x.

All older series have [reached End of Life](https://www.rabbitmq.com/release-information).

## Dependency

### Blocking Client

```toml
rabbitmq_http_client = { version = "0.70.0", features = ["core", "blocking"] }
```

### Async Client

```toml
rabbitmq_http_client = { version = "0.70.0", features = ["core", "async"] }
```

### Blocking Client with Tabled Support

```toml
rabbitmq_http_client = { version = "0.70.0", features = ["core", "blocking", "tabled"] }
```

### Async Client with Tabled Support

```toml
rabbitmq_http_client = { version = "0.70.0", features = ["core", "async", "tabled"] }
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


## TLS Support

This client library was designed to make very few assumptions about how the underlying
HTTP client, `reqwest`, is set up, in particular when it comes to TLS and crypto.

To use TLS connections to the HTTP API, use the [`ClientBuilder`](https://docs.rs/reqwest/latest/reqwest/struct.ClientBuilder.html)
interface in `reqwest` to configure TLS, then combine the constructed client
with this library's `blocking_api::ClientBuilder` or `api::ClientBuilder`'s `with_client` function:

```rust
use reqwest::blocking::Client as HTTPClient;

// this is reqwest's `ClientBuilder`
let mut b = HTTPClient::builder()
    .user_agent("my-app")
    .min_tls_version(reqwest::tls::Version::TLS_1_2)
    .danger_accept_invalid_certs(false)
    .danger_accept_invalid_hostnames(false);

// add a CA certificate bundle file to the list of trusted roots
// for x.509 peer verification
b.add_root_certificate(ca_certificate_path);

let httpc = b.build()
let username = "example";
let password = "ex4mple $eKr37;

// make sure the endpoint uses TLS and the correct port for TLS-enabled connections
let endpoint = "https://example.domain:15671/api"

// this is this library's `ClientBuilder`
let client = ClientBuilder::new()
    .with_endpoint(endpoint)
    .with_basic_auth_credentials(username, password)
    // pass in the pre-configured HTTP client
    .with_client(httpc)
    .build();
```

This design decision means that with this HTTP API client, it's up to the user to make
some key TLS-related choices. For example, [what certificate store to use](https://github.com/rustls/rustls-platform-verifier?tab=readme-ov-file#deployment-considerations) for x.509 peer verification,
what the acceptable minimum TLS version should be, and so on.

### Defaults

By default this client will use [`native-tls`](https://crates.io/crates/native-tls).

This means that the default list of trusted CA certificates (roots) is managed via the OS-specific mechanisms
such as the Keychain on macOS or the local `openssl` version and its standard directories for trusted (root) CA certificates.


## License

This crate, rabbitmq-http-api-client-rs, is dual-licensed under
the Apache Software License 2.0 and the MIT license.
