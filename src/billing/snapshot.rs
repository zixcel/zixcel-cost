use super::{BillingError, Money, identifier};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// One observed invoice revision. Monetary values are provider supplied, including credits.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InvoiceObservation {
    pub invoice_id: String,
    pub revision: String,
    pub period_start_ms: u64,
    pub period_end_ms: u64,
    pub total: Money,
    pub source_digest: String,
}

/// A complete, bounded observation for one declared provider/account/period.
/// Partial pages cannot be used as a complete threshold observation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BillingSnapshot {
    pub provider: String,
    pub account: String,
    pub period_start_ms: u64,
    pub period_end_ms: u64,
    pub observed_at_ms: u64,
    pub complete: bool,
    pub invoices: Vec<InvoiceObservation>,
}

impl BillingSnapshot {
    /// # Errors
    /// Rejects incomplete scope, duplicate invoices, invalid periods and missing source evidence.
    pub fn validate(&self) -> Result<(), BillingError> {
        validate_scope(
            &self.provider,
            &self.account,
            self.period_start_ms,
            self.period_end_ms,
            self.complete,
            self.invoices.len(),
        )?;
        let mut identities = BTreeSet::new();
        for invoice in &self.invoices {
            validate_identity(
                &invoice.invoice_id,
                &invoice.revision,
                &invoice.source_digest,
            )?;
            if invoice.period_start_ms != self.period_start_ms
                || invoice.period_end_ms != self.period_end_ms
            {
                return Err(BillingError::InvalidObservation);
            }
            if !identities.insert(&invoice.invoice_id) {
                return Err(BillingError::DuplicateInvoice);
            }
        }
        Ok(())
    }
}

pub(super) fn validate_scope(
    provider: &str,
    account: &str,
    start: u64,
    end: u64,
    complete: bool,
    count: usize,
) -> Result<(), BillingError> {
    if !identifier(provider) || !identifier(account) || start >= end || !complete || count > 1024 {
        return Err(BillingError::InvalidObservation);
    }
    Ok(())
}
pub(super) fn validate_identity(
    id: &str,
    revision: &str,
    digest: &str,
) -> Result<(), BillingError> {
    if !identifier(id)
        || !identifier(revision)
        || digest.len() != 64
        || !digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(BillingError::InvalidObservation);
    }
    Ok(())
}
