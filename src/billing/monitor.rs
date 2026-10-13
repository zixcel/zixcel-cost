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
    if maximum_age_ms == 0
        || maximum_age_ms > 86_400_000
        || snapshot.observed_at_ms > now_ms
        || now_ms - snapshot.observed_at_ms > maximum_age_ms
    {
        return Err(BillingError::StaleObservation);
    }
    let mut total = Money::new(0, threshold.scale(), threshold.currency().to_owned())?;
    for invoice in &snapshot.invoices {
        total = total.checked_add(&invoice.total)?;
    }
    let scale = total.scale().max(threshold.scale());
    let threshold_reached = total.at_scale(scale)? >= threshold.at_scale(scale)?;
    Ok(MonitorResult {
        total,
        threshold_reached,
        observed_at_ms: snapshot.observed_at_ms,
    })
}
