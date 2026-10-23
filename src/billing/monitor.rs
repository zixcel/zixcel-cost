use super::{BillingError, BillingSnapshot, Money};

/// A pure observation result. It does not authorize or execute a payment or other write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitorResult {
    pub total: Money,
    pub threshold_reached: bool,
    pub observed_at_ms: u64,
}

/// Evaluate one complete invoice observation without floats, FX or implicit retries.
/// Empty complete snapshots mean a known zero; incomplete snapshots are rejected.
///
/// # Errors
/// Rejects invalid scope, stale/future data, currency mixing and arithmetic overflow.
pub fn evaluate_threshold(
    snapshot: &BillingSnapshot,
    threshold: &Money,
    now_ms: u64,
    maximum_age_ms: u64,
) -> Result<MonitorResult, BillingError> {
    snapshot.validate()?;
    evaluate_totals(
        snapshot.invoices.iter().map(|v| &v.total),
        snapshot.observed_at_ms,
        threshold,
        now_ms,
        maximum_age_ms,
    )
}

pub(super) fn evaluate_totals<'a>(
    values: impl Iterator<Item = &'a Money>,
    observed_at_ms: u64,
    threshold: &Money,
    now_ms: u64,
    maximum_age_ms: u64,
) -> Result<MonitorResult, BillingError> {
    if maximum_age_ms == 0
        || maximum_age_ms > 86_400_000
        || observed_at_ms > now_ms
        || now_ms - observed_at_ms > maximum_age_ms
    {
        return Err(BillingError::StaleObservation);
    }
    let mut total = Money::new(0, threshold.scale(), threshold.currency().to_owned())?;
    for value in values {
        total = total.checked_add(value)?;
    }
    let scale = total.scale().max(threshold.scale());
    let threshold_reached = total.at_scale(scale)? >= threshold.at_scale(scale)?;
    Ok(MonitorResult {
        total,
        threshold_reached,
        observed_at_ms,
    })
}
