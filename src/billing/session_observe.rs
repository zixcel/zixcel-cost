use super::{
    BillingError, BillingMonitor, BillingSnapshot, MonitorEvent, MonitorResult, evaluate_threshold,
};
use sha2::{Digest, Sha256};

pub(super) fn hash<T: serde::Serialize>(value: &T) -> Result<String, BillingError> {
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
        self.admit(
            &snapshot.provider,
            &snapshot.account,
            snapshot.period_start_ms,
            snapshot.period_end_ms,
            snapshot.observed_at_ms,
            now,
        )?;
        let evaluated =
            evaluate_threshold(snapshot, &self.threshold, now, self.limits.maximum_age_ms)?;
        let entries = snapshot
            .invoices
            .iter()
            .map(|v| Ok(((v.invoice_id.clone(), v.revision.clone()), hash(v)?)))
            .collect::<Result<_, BillingError>>()?;
        self.commit(entries, evaluated, now)
    }
    pub(super) fn admit(
        &mut self,
        provider: &str,
        account: &str,
        start: u64,
        end: u64,
        observed: u64,
        now: u64,
    ) -> Result<(), BillingError> {
        if !self.active
            || now < self.started
            || now < self.last_checked_at
            || now >= self.limits.expires_at_ms
        {
            self.stop();
            return Err(BillingError::MonitorInactive);
        }
        if provider != self.scope.provider
            || account != self.scope.account
            || start != self.scope.period_start_ms
            || end != self.scope.period_end_ms
        {
            return Err(BillingError::InvalidObservation);
        }
        if self
            .last_observed_at
            .is_some_and(|previous| observed < previous)
        {
            return Err(BillingError::StaleObservation);
        }
        Ok(())
    }
    pub(super) fn commit(
        &mut self,
        entries: Vec<((String, String), String)>,
        evaluated: MonitorResult,
        now: u64,
    ) -> Result<MonitorEvent, BillingError> {
        let mut current = std::collections::BTreeMap::new();
        let mut new_entries = 0;
        for (key, content) in entries {
            if self.revisions.contains_key(&key)
                && self
                    .latest_revisions
                    .get(&key.0)
                    .is_some_and(|latest| latest != &key.1)
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
        for (id, revision) in current.keys() {
            self.latest_revisions.insert(id.clone(), revision.clone());
        }
        self.revisions.extend(current);
        self.last_content = Some(fingerprint);
        self.last_observed_at = Some(evaluated.observed_at_ms);
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
