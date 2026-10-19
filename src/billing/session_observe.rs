use super::{BillingError, BillingMonitor, BillingSnapshot, MonitorEvent, evaluate_threshold};
use sha2::{Digest, Sha256};

fn hash<T: serde::Serialize>(value: &T) -> Result<String, BillingError> {
    let bytes = serde_json::to_vec(value).map_err(|_| BillingError::InvalidObservation)?;
    Ok((Sha256::digest(bytes))
        .iter()
        .fold(String::with_capacity(64), |mut encoded, byte| {
            use std::fmt::Write as _;
            write!(encoded, "{byte:02x}").expect("writing to String cannot fail");
            encoded
        }))
}
impl BillingMonitor {
    /// Apply one complete fresh observation at the exact configured scope.
    /// # Errors
    /// Rejects stop/expiry, scope changes, observation rollback, inconsistent revisions and capacity overflow.
    pub fn observe(
        &mut self,
        snapshot: &BillingSnapshot,
        now: u64,
    ) -> Result<MonitorEvent, BillingError> {
        if !self.active
            || now < self.started
            || now < self.last_checked_at
            || now >= self.limits.expires_at_ms
        {
            self.stop();
            return Err(BillingError::MonitorInactive);
        }
        if snapshot.provider != self.scope.provider
            || snapshot.account != self.scope.account
            || snapshot.period_start_ms != self.scope.period_start_ms
            || snapshot.period_end_ms != self.scope.period_end_ms
        {
            return Err(BillingError::InvalidObservation);
        }
        if self
            .last_observed_at
            .is_some_and(|previous| snapshot.observed_at_ms < previous)
        {
            return Err(BillingError::StaleObservation);
        }
        let evaluated =
            evaluate_threshold(snapshot, &self.threshold, now, self.limits.maximum_age_ms)?;
        let mut current = std::collections::BTreeMap::new();
        let mut new_entries = 0;
        for invoice in &snapshot.invoices {
            let key = (invoice.invoice_id.clone(), invoice.revision.clone());
            let content = hash(invoice)?;
            if self.revisions.contains_key(&key)
                && self
                    .latest_revisions
                    .get(&invoice.invoice_id)
                    .is_some_and(|latest| latest != &invoice.revision)
            {
                return Err(BillingError::RevisionRollback);
            }
            match self.revisions.get(&key) {
                Some(previous) if previous != &content => {
                    return Err(BillingError::InconsistentRevision);
                }
                None => new_entries += 1,
                _ => (),
            }
            current.insert(key, content);
        }
        if self.revisions.len() + new_entries > self.limits.maximum_revision_entries {
            return Err(BillingError::MonitorCapacity);
        }
        let fingerprint = hash(&current.iter().collect::<Vec<_>>())?;
        let changed = self.last_content.as_ref() != Some(&fingerprint);
        for invoice in &snapshot.invoices {
            self.latest_revisions
                .insert(invoice.invoice_id.clone(), invoice.revision.clone());
        }
        self.revisions.extend(current);
        self.last_content = Some(fingerprint);
        self.last_observed_at = Some(snapshot.observed_at_ms);
        self.last_checked_at = now;
        self.count += 1;
        if self.count == self.limits.maximum_observations {
            self.stop();
        }
        Ok(MonitorEvent {
            sequence: self.count,
            changed,
            threshold_reached: evaluated.threshold_reached,
            total: evaluated.total,
        })
    }
}
