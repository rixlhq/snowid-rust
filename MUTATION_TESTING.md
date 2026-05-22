# Mutation and high-load correctness testing

SnowID is an ID generator intended for database-core use. The normal CI suite runs deterministic unit, property, and
stress tests. For Stryker-style mutation testing in Rust, use [`cargo-mutants`](https://mutants.rs/).

## Local mutation testing

Install the tool locally:

```bash
cargo install cargo-mutants
```

Run mutation testing against the generator and tests:

```bash
cargo mutants --package snowid --file src/generator --file src/config --file src/extractor -- --tests
```

The repository also includes a manual GitHub Actions workflow named `mutation` that runs the same check with
`workflow_dispatch`. Use it before release candidates or after large generator changes.

`cargo-mutants` repeatedly rewrites expressions and runs the test suite to check whether the tests catch the injected
bugs. It is intentionally slower than `cargo test`, so keep it as a local or scheduled release gate rather than a fast
pull-request check.

## Property and stress tests

The regular test suite includes property-based tests with `proptest` for invariants that should hold across many input
shapes:

- generated IDs are unique and monotonic
- extracted node and sequence components stay within configured bounds
- logical batch generation preserves ordering across rollover
- strict batch generation never writes more IDs than requested or current wall-clock capacity allows

Run them with the rest of the suite:

```bash
cargo test --release
```

Focused high-load checks are also included:

```bash
cargo test test_high_load_shared_logical_generation --release
cargo test test_concurrent_logical_overflow_lockfree --release
cargo test test_concurrent_logical_batch_generation_lockfree --release
```

These tests exercise one shared generator under heavy logical rollover and concurrent batch generation. They are not a
formal proof of correctness, but they are the practical equivalent of penetration-style load tests for this library's
core failure modes: duplicates, non-monotonic IDs, sequence rollover bugs, and future logical timestamp handling.
