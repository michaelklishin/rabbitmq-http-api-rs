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