# Responsibility and source completion

This repository owns deterministic, provider-neutral cost arithmetic. Token usage/rate-card estimation and exact decimal invoice observations share checked numeric primitives; they do not share product role policy or provider API catalogs.

The public `billing` module preserves signed amounts, declared currencies, observation scope, invoice revisions and source evidence. It validates complete bounded observations and computes threshold results without floats or implicit FX. Providers own authenticated data collection and normalization; products own persistence, schedules, approvals and actual payment/administrative operations.

## Completion criteria

- Public numeric units, rounding and overflow behavior are explicit and tested.
- Money survives wire serialization without integer precision loss; malformed input, unknown fields and currency mixing are rejected.
- Invoices cannot be double-counted across revisions or accepted from partial, stale or incorrectly scoped observations.
- Examples and tests use synthetic input and require no private registry, account credentials or network access.
- Independent consumers can use the documented public API with the supported Rust version and official registry dependencies.
- New APIs are versioned and reviewed before release; existing immutable 0.10.0 artifacts are not treated as containing unreleased source.

This crate does not establish effective IAM, authenticate invoice evidence, infer tax policy, maintain a monitoring daemon or execute billing writes. These remain explicit caller/provider responsibilities. See [billing observations](docs/billing-observations.md), `README.md`, `SECURITY.md`, `LICENSE` and `NOTICE`.
