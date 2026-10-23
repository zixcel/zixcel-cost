//! Usage evidence is deliberately distinct from issued invoice observations.
use super::{BillingError, BillingMonitor, BillingScope, Money, MonitorEvent, MonitorLimits};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageObservation {
    pub observation_id: String,
    pub revision: String,
    pub total: Money,
    pub source_digest: String,
}

/// One complete usage report for an explicitly selected account and period.
/// This type carries no claim of invoice issuance or payment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UsageSnapshot {
    pub provider: String,
    pub account: String,
    pub period_start_ms: u64,
    pub period_end_ms: u64,
    pub observed_at_ms: u64,
    pub complete: bool,
    pub observations: Vec<UsageObservation>,
}

impl UsageSnapshot {
    /// # Errors
    /// Rejects incomplete/invalid scope, duplicate identities and missing evidence.
    pub fn validate(&self) -> Result<(), BillingError> {
        super::snapshot::validate_scope(
            &self.provider,
            &self.account,
            self.period_start_ms,
            self.period_end_ms,
            self.complete,
            self.observations.len(),
        )?;
        let mut identities = std::collections::BTreeSet::new();
        for item in &self.observations {
            super::snapshot::validate_identity(
                &item.observation_id,
                &item.revision,
                &item.source_digest,
            )?;
            if !identities.insert(&item.observation_id) {
                return Err(BillingError::DuplicateObservation);
            }
        }
        Ok(())
    }
}

/// A usage-only facade over the shared bounded revision monitor.
/// There is no conversion to an invoice or shared invoice/usage session.
pub struct UsageMonitor(BillingMonitor);
impl UsageMonitor {
    /// # Errors
    /// Rejects invalid scope and unbounded sessions, as for `BillingMonitor`.
    pub fn new(
        scope: BillingScope,
        threshold: Money,
        limits: MonitorLimits,
        now: u64,
    ) -> Result<Self, BillingError> {
        BillingMonitor::new(scope, threshold, limits, now).map(Self)
    }
    /// # Errors
    /// Rejects incomplete/stale evidence, mixed currencies and revision substitution.
    pub fn observe(
        &mut self,
        snapshot: &UsageSnapshot,
        now: u64,
    ) -> Result<MonitorEvent, BillingError> {
        self.0.admit(
            &snapshot.provider,
            &snapshot.account,
            snapshot.period_start_ms,
            snapshot.period_end_ms,
            snapshot.observed_at_ms,
            now,
        )?;
        snapshot.validate()?;
        let evaluated = super::monitor::evaluate_totals(
            snapshot.observations.iter().map(|v| &v.total),
            snapshot.observed_at_ms,
            &self.0.threshold,
            now,
            self.0.limits.maximum_age_ms,
        )?;
        let entries = snapshot
            .observations
            .iter()
            .map(|v| {
                Ok((
                    (v.observation_id.clone(), v.revision.clone()),
                    super::session_observe::hash(v)?,
                ))
            })
            .collect::<Result<_, BillingError>>()?;
        self.0.commit(entries, evaluated, now)
    }
    pub fn stop(&mut self) {
        self.0.stop();
    }
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.0.is_active()
    }
    #[must_use]
    pub const fn observations(&self) -> u16 {
        self.0.observations()
    }
}
