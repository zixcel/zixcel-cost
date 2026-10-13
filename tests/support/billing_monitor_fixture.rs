use zixcel_cost::billing::{
    BillingMonitor, BillingScope, BillingSnapshot, InvoiceObservation, Money, MonitorLimits,
};
pub fn snapshot() -> BillingSnapshot {
    BillingSnapshot {
        provider: "fixture".into(),
        account: "synthetic".into(),
        period_start_ms: 100,
        period_end_ms: 200,
        observed_at_ms: 300,
        complete: true,
        invoices: vec![InvoiceObservation {
            invoice_id: "invoice".into(),
            revision: "r1".into(),
            period_start_ms: 100,
            period_end_ms: 200,
            total: Money::new(150, 2, "USD".into()).unwrap(),
            source_digest: "a".repeat(64),
        }],
    }
}
pub fn monitor(maximum_observations: u16, entries: usize) -> BillingMonitor {
    BillingMonitor::new(
        BillingScope {
            provider: "fixture".into(),
            account: "synthetic".into(),
            period_start_ms: 100,
            period_end_ms: 200,
        },
        Money::new(100, 2, "USD".into()).unwrap(),
        MonitorLimits {
            expires_at_ms: 1000,
            maximum_observations,
            maximum_revision_entries: entries,
            maximum_age_ms: 100,
        },
        250,
    )
    .unwrap()
}
