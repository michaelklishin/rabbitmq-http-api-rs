# Contributing

See also [AGENTS.md](./AGENTS.md) for a high-level codebase overview and conventions.

## Running Tests

While tests support the standard `cargo test` option, another option
for running tests is [cargo-nextest](https://nexte.st/).

### `nextest` Test Filters

Key [`nextest` filterset predicates](https://nexte.st/docs/filtersets/reference/):

* `test(pattern)`: matches test names using a substring (e.g. `test(list_nodes)`)
* `binary(pattern)`: matches the test binary name (e.g. `binary(async_tls_tests)`)
* `package(name)`: matches by package (e.g. `package(rabbitmq_http_client)`)
* `test(=exact_name)`: an exact test name match
* `test(/regex/)`: like `test(pattern)` but uses regular expression matching
* Set operations: `expr1 + expr2` (union), `expr1 - expr2` (difference), `not expr` (negation)

For example, use `cargo nextest run --all-features -E 'binary(=test_module_name)'` to run
all tests in a specific module.

### Run All Tests

``` bash
NEXTEST_RETRIES=3 cargo nextest run --all-features
```

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
