# Exact invoice observations

The `billing` module evaluates provider-supplied invoice observations. It does not fetch accounts, prove IAM permissions, pay invoices or convert currencies.

`Money` stores a signed integer coefficient, a scale from zero through nine, and an explicitly declared three-letter uppercase currency code. The wire coefficient is a canonical decimal string, preserving values beyond JavaScript integer precision. Currency validation is syntactic; it does not claim the code is a recognized ISO currency. Credits may be negative. Addition aligns scales using checked integer arithmetic and rejects mixed currencies or overflow.

A `BillingSnapshot` belongs to one explicit provider/account and half-open reporting period. Each invoice has its provider revision, matching period and SHA256 digest of the retained provider source. The caller verifies the source bytes and selects the current revision before constructing the snapshot; this library does not authenticate a digest merely because it is present. Multiple revisions of the same invoice must not be summed together.

The collector sets `complete` only after all required pages are received. Validation rejects partial collections, duplicate invoice IDs, missing evidence, mismatched periods and more than 1024 invoices. For larger datasets, an owner must define an explicit partition/aggregation policy rather than truncate and declare completion.

`evaluate_threshold` compares exact totals with an explicit same-currency threshold and observation age policy (at most 24 hours). Future and stale observations are rejected. An empty complete snapshot is known zero; an incomplete snapshot is unknown. The caller owns provider/account/period selection, polling, persistent observations and change notifications. This module performs no retries or external writes.

```rust
use zixcel_cost::billing::{BillingSnapshot, Money, evaluate_threshold};
let empty = BillingSnapshot {
    provider: "synthetic-cloud".into(), account: "fixture-account".into(),
    period_start_ms: 100, period_end_ms: 200, observed_at_ms: 300,
    complete: true, invoices: vec![],
};
let threshold = Money::new(10000, 2, "USD".into())?;
let result = evaluate_threshold(&empty, &threshold, 350, 100)?;
assert!(!result.threshold_reached);
# Ok::<(), zixcel_cost::billing::BillingError>(())
```

Tests use synthetic accounts/invoices only. Source availability does not mean these new APIs exist in the currently published 0.10.0 archive. Adopt the next reviewed release or an explicitly fixed source revision after verifying its license and dependency lock.

Use `Money::from_decimal` for exact decimal/scientific strings retained by provider report adapters. It accepts JSON numeric syntax and checked i128 arithmetic, rejects unsupported precision/overflow and never passes through a float. Invoice normalization must select the actual amount/currency/period/revision fields; a cost query, account linkage or raw TSV report is not automatically an invoice.

## Bounded observation sessions

`BillingMonitor` binds provider/account/period/threshold once, with explicit `MonitorLimits`: at most 24-hour lifetime, 128 accepted observations and 4096 retained invoice revision entries. It validates freshness and complete scope before updating state. Repeated/reordered current observations are not cumulatively added; each total comes from that complete snapshot. Immutable revision content cannot change, and a previously superseded revision cannot silently become current again. Opaque provider revision names are never sorted to invent a chronology; the collector still selects the authoritative current revision.

`stop`, expiry and backwards session clocks close the session; an older clock cannot reactivate it. Exhausting the observation budget stops it. Capacity, partial input, scope substitution and stale data do not partially publish a new observation. This is an in-memory pure state machine, not a polling daemon, credential authority or payment scheduler. Each external acquisition/write still uses its own approved provider operation. Products own polling/notifications and use managed revision storage for persistence when needed.

## Usage exports

`UsageSnapshot` / `UsageObservation` carry provider-normalized usage report totals,
not issued invoices or payment evidence. `UsageMonitor` has its own typed entry
point and session; it shares scope/freshness, checked totals, immutable revisions,
capacity and commit logic with `BillingMonitor` without converting usage into
`InvoiceObservation`. Do not add both kinds into a single account total.

Provider owners decode vendor bytes and verify their scope, page completeness,
source hashes and stable export revisions before producing a usage snapshot.
A complete table page is not proof that a vendor exported every billed service.
The report owner selects a period/currency and a reviewed source; later export
adjustments are new revisions. The shared library knows no Google schemas or
account credentials. Google-specific decoding and normalization belong in
`zixcel-google`; its synthetic independent consumer must exercise the real
byte decoder, not a manually fabricated shared snapshot.

This 0.11.0 source candidate requires normal package publication acceptance.
No upstream collection, paid SQL query, persistent poll or notification is added.
