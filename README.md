# zixcel-inference-cost

Calculate an inference-cost estimate from explicit usage and price inputs.

## What you can do

- Validate supplied usage and rates.
- Return reproducible calculated costs.

## Current scope

Prices and usage are caller inputs. The package does not retrieve prices, bill an account or purchase capacity.

This is the reviewed first-publication candidate for `zixcel-inference-cost` 0.10.0. Published availability must be verified separately; this documentation does not claim the release is complete.

## Getting started

Install Rust 1.97 or newer and make the declared dependencies available. The reviewed crate uses crates.io dependencies. Run from this repository:

```sh
cargo test --locked
```

## Examples and interface details

## Usage

```rust
use zixcel_inference_cost::{CostRateCard, TokenUsage, estimate};

let usage = TokenUsage::new(1_000, 200, 0, 120, 40)?;
let rates = CostRateCard::per_token("micro-unit", "model-rate", "2026-09-04", 2, 1, 3, 4, 5);
let result = estimate(&usage, &rates)?;
// 800 uncached input × 2 + 200 cached × 1 + 120 output × 4 + 40 reasoning × 5.
assert_eq!(result.total_micro_units(), 2_480);
# Ok::<(), zixcel_inference_cost::CostError>(())
```

The rates are informational estimates and must not silently control model
selection. Provider billing remains authoritative at the provider boundary.

## Documentation and source

[Interface reference](docs/interface-reference.md)

[Usage guide](docs/getting-started.md)

[Schemas](schemas) · [Implementation and public interfaces](src) · [Verification cases](tests) · [Contributing](CONTRIBUTING.md) · [Security reporting](SECURITY.md) · [License](LICENSE) · [Attribution notices](NOTICE)
