use super::Money;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BillingScope {
    pub provider: String,
    pub account: String,
    pub period_start_ms: u64,
    pub period_end_ms: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MonitorLimits {
    pub expires_at_ms: u64,
    pub maximum_observations: u16,
    pub maximum_revision_entries: usize,
    pub maximum_age_ms: u64,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MonitorEvent {
    pub sequence: u16,
    pub changed: bool,
    pub threshold_reached: bool,
    pub total: Money,
}
