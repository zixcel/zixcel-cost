//! Provider-neutral invoice observations and exact arithmetic. No account access or payment execution.
mod decimal;
mod money;
mod monitor;
mod session;
mod session_models;
mod session_observe;
mod snapshot;
mod usage;
pub use money::Money;
pub use monitor::{MonitorResult, evaluate_threshold};
pub use session::BillingMonitor;
pub use session_models::{BillingScope, MonitorEvent, MonitorLimits};
pub use snapshot::{BillingSnapshot, InvoiceObservation};
pub use usage::{UsageMonitor, UsageObservation, UsageSnapshot};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BillingError {
    InvalidAmount,
    CurrencyMismatch,
    Overflow,
    InvalidObservation,
    DuplicateInvoice,
    DuplicateObservation,
    StaleObservation,
    MonitorInactive,
    MonitorCapacity,
    InconsistentRevision,
    RevisionRollback,
}

impl std::fmt::Display for BillingError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidAmount => "invalid exact decimal amount",
            Self::CurrencyMismatch => "billing currencies differ",
            Self::Overflow => "billing arithmetic overflow",
            Self::InvalidObservation => "invalid or incomplete billing observation",
            Self::DuplicateObservation => "duplicate usage observation identity",
            Self::DuplicateInvoice => "duplicate invoice identity",
            Self::MonitorInactive => "billing monitor is stopped or expired",
            Self::MonitorCapacity => "billing monitor revision capacity exceeded",
            Self::RevisionRollback => "previously superseded invoice revision returned",
            Self::InconsistentRevision => "invoice revision content changed",
            Self::StaleObservation => "stale or future billing observation",
        })
    }
}
impl std::error::Error for BillingError {}

pub(super) fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value.trim() == value
        && !value.chars().any(char::is_control)
}
