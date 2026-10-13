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
        if !identifier(&self.provider)
            || !identifier(&self.account)
            || !self.complete
            || self.period_start_ms >= self.period_end_ms
            || self.invoices.len() > 1024
        {
            return Err(BillingError::InvalidObservation);
        }
        let mut identities = BTreeSet::new();
        for invoice in &self.invoices {
            if !identifier(&invoice.invoice_id)
                || !identifier(&invoice.revision)
                || invoice.period_start_ms != self.period_start_ms
                || invoice.period_end_ms != self.period_end_ms
                || invoice.source_digest.len() != 64
                || !invoice
                    .source_digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
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
