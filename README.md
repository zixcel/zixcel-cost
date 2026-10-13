# zixcel-cost

Deterministic integer arithmetic for cost estimates and provider-neutral inference usage accounting. Use the shared arithmetic module for scaled quantities and checked totals, or the token API for a supplied usage record and rate card.

## Install

```toml
[dependencies]
zixcel-cost = "0.10.0"
```

This dependency uses crates.io; no private registry is required.

## Capabilities

- Calculate scaled resource costs with explicit rounding.
- Validate token usage and estimate cached, uncached, output and reasoning costs.
- Preserve reproducible minor-unit totals and estimate digests.
- Bound observation sessions with explicit lifetime/count/revision limits, stop and immutable correction checks.
- Evaluate complete, source-linked invoice observations with exact decimal amounts and explicit freshness/threshold policy (new source API; not in published 0.10.0).

[Invoice observations and example](docs/billing-observations.md) · [Responsibility and source completion](RESPONSIBILITY.md)

## Example

```rust
use zixcel_cost::arithmetic::{Rounding, scaled_cost, sum_costs};

assert_eq!(scaled_cost(1500, 3, 1000, Rounding::HalfUp)?, 5);
assert_eq!(sum_costs([1600, 200, 480, 200])?, 2480);
# Ok::<(), zixcel_cost::arithmetic::ArithmeticError>(())
```

## Features and requirements

Requires Rust 1.97 or newer. The crate is a reusable library, not a command-line application.

## Boundaries

The caller supplies prices, currency or unit conventions, usage and rounding rules. This crate does not fetch rates, convert currencies, perform billing or select a model; provider invoices remain authoritative.

## Development

```sh
cargo fmt --all -- --check
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
```

## Documentation and license

[API documentation](https://docs.rs/zixcel-cost) · [Source](https://github.com/zixcel/zixcel-cost) · [Usage guide](https://github.com/zixcel/zixcel-cost/blob/main/docs/getting-started.md)

Apache-2.0. Retain the package LICENSE and NOTICE; see the source repository for security reporting and contribution guidelines.

[OSS responsibility and source completion](docs/oss-source-contract.md) defines the supported package boundary and required acceptance. It does not declare unexecuted source, distribution or production checks complete.
