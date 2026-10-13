use super::{BillingError, BillingScope, Money, MonitorLimits, identifier};
use std::collections::BTreeMap;

/// Pure bounded observation lifecycle. It has no scheduler, credentials or payment authority.
pub struct BillingMonitor {
    pub(super) scope: BillingScope,
    pub(super) threshold: Money,
    pub(super) limits: MonitorLimits,
    pub(super) started: u64,
    pub(super) last_checked_at: u64,
    pub(super) active: bool,
    pub(super) count: u16,
    pub(super) latest_revisions: BTreeMap<String, String>,
    pub(super) revisions: BTreeMap<(String, String), String>,
    pub(super) last_content: Option<String>,
    pub(super) last_observed_at: Option<u64>,
}
impl BillingMonitor {
    /// # Errors
    /// Rejects missing scope, unbounded/expired sessions and invalid freshness budgets.
    pub fn new(
        scope: BillingScope,
        threshold: Money,
        limits: MonitorLimits,
        now: u64,
    ) -> Result<Self, BillingError> {
        if !identifier(&scope.provider)
            || !identifier(&scope.account)
            || scope.period_start_ms >= scope.period_end_ms
            || limits.expires_at_ms <= now
            || limits.expires_at_ms - now > 86_400_000
            || limits.maximum_observations == 0
            || limits.maximum_observations > 128
            || limits.maximum_revision_entries == 0
            || limits.maximum_revision_entries > 4096
            || limits.maximum_age_ms == 0
            || limits.maximum_age_ms > 86_400_000
        {
            return Err(BillingError::InvalidObservation);
        }
        Ok(Self {
            scope,
            threshold,
            limits,
            started: now,
            last_checked_at: now,
            active: true,
            count: 0,
            latest_revisions: BTreeMap::new(),
            revisions: BTreeMap::new(),
            last_content: None,
            last_observed_at: None,
        })
    }
    pub fn stop(&mut self) {
        self.active = false;
    }
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }
    #[must_use]
    pub const fn observations(&self) -> u16 {
        self.count
    }
}
