# Contributing

See also [AGENTS.md](./AGENTS.md) for a high-level codebase overview and conventions.

## Running Tests

Most tests require a locally running RabbitMQ node. The easiest way to get one is via Docker.

### Prerequisites

Install [cargo-nextest](https://nexte.st/) if you don't have it:

```bash
cargo install cargo-nextest
```

### Step 1: Start RabbitMQ

```bash
docker run -d --name rabbitmq \
  -p 15672:15672 \
  -p 5672:5672 \
  rabbitmq:4.1-management
```

Wait for the node to boot:

```bash
sleep 15
```

### Step 2: Pre-configure the Node

Run the setup script using the Docker exec variant of `rabbitmqctl`:

```bash
RUST_HTTP_API_CLIENT_RABBITMQCTL=DOCKER:rabbitmq bin/ci/before_build.sh
```

This enables the required plugins (management, shovel, federation, stream), creates test users,
sets up the `rust/http/api/client` vhost, sets the cluster name, and enables all feature flags.

Wait for the changes to apply:

```bash
sleep 10
```

### Step 3: Run All Tests

```bash
NEXTEST_RETRIES=3 cargo nextest run --all-features
```

`NEXTEST_RETRIES=3` retries each failing test up to 3 times. This is recommended because some
tests depend on management plugin stats that can lag slightly behind the actual broker state.

### Stopping the Node

```bash
docker stop rabbitmq && docker rm rabbitmq
```

---

## `nextest` Test Filters

Key [`nextest` filterset predicates](https://nexte.st/docs/filtersets/reference/):

* `test(pattern)`: matches test names using a substring (e.g. `test(list_nodes)`)
* `binary(pattern)`: matches the test binary name (e.g. `binary(integration)`, `binary(unit)`, `binary(proptests)`)
* `package(name)`: matches by package (e.g. `package(rabbitmq_http_client)`)
* `test(=exact_name)`: an exact test name match
* `test(/regex/)`: like `test(pattern)` but uses regular expression matching
* Set operations: `expr1 + expr2` (union), `expr1 - expr2` (difference), `not expr` (negation)

For example, use `cargo nextest run --all-features -E 'binary(=test_module_name)'` to run
all tests in a specific module.

### Run All Async Client Tests

```bash
cargo nextest run --all-features -E 'binary(~async)'
```

### Run All Blocking Client Tests

```bash
cargo nextest run --all-features -E 'binary(~blocking)'
```

### Run All Unit Tests

```bash
cargo nextest run --all-features -E 'binary(~unit)'
```

### Run All Tests in a Specific Module

Use the `binary(=name)` predicate with the test module name:

```bash
cargo nextest run --all-features -E 'binary(=unit_endpoint_validation_tests)'
```

### Run a Specific Test by Name

Use the `test(=name)` predicate for an exact match:

``` bash
cargo nextest run --all-features -E 'test(=test_list_all_vhost_limits)'
```

Or use `test(~substring)` for a substring match:

```bash
cargo nextest run --all-features -E 'test(~list_vhost)'
```

### Run Property-based Tests

```bash
cargo nextest run --all-features -E 'test(~prop_)'
```

See the [nextest filtersets documentation](https://nexte.st/docs/selecting/) for more
filter expression predicates and operators.

---

## Running TLS Integration Tests

TLS-enabled tests require a set of certificate and private key pairs, plus
a preconfigured RabbitMQ node. As such, they are excluded from standard `cargo nextest` runs.

To run these tests, generate certificates using [tls-gen](https://github.com/rabbitmq/tls-gen):

```shell
cd /path/to/tls-gen/basic && make CN=localhost && make alias-leaf-artifacts
```

Then [configure](https://www.rabbitmq.com/docs/management#single-listener-https) RabbitMQ management plugin to use TLS.

Next, export the `TLS_CERTS_DIR` environment variable:

```shell
export TLS_CERTS_DIR=/path/to/tls-gen/basic/result
```

Finally, run the tests in question only:

```shell
cargo nextest run --all-features --run-ignored=only -E 'binary(=async_tls_tests)'

cargo nextest run --all-features --run-ignored=only -E 'binary(=blocking_tls_tests)'
```
