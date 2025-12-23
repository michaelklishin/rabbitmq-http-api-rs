# Contributing

## Running Tests

While tests support the standard `cargo test` option, another option
for running tests is [cargo-nextest](https://nexte.st/).

### Run All Tests

``` bash
NEXTEST_RETRIES=3 cargo nextest run --all-features
```

### Run All Async Client Tests

```bash
cargo test async --all-features
```

### Run All Sync Client Tests

```bash
cargo test blocking --all-features
```

### Run a Specific Test

``` bash
NEXTEST_RETRIES=3 cargo nextest run -E "test(test_list_all_vhost_limits)"
```

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
cargo nextest run -E 'binary(async_tls_tests)' --run-ignored=only --all-features

cargo nextest run -E 'binary(blocking_tls_tests)' --run-ignored=only --all-features
```
