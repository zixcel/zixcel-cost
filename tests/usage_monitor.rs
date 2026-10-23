use zixcel_cost::billing::{
    BillingError, BillingScope, Money, MonitorLimits, UsageMonitor, UsageObservation, UsageSnapshot,
};
fn snapshot() -> UsageSnapshot {
    UsageSnapshot {
        provider: "fixture-export".into(),
        account: "synthetic".into(),
        period_start_ms: 100,
        period_end_ms: 200,
        observed_at_ms: 300,
        complete: true,
        observations: vec![UsageObservation {
            observation_id: "table-month".into(),
            revision: "r1".into(),
            total: Money::from_decimal("9007199254740993.000000001", "USD".into())
                .unwrap()
                .checked_add(&Money::from_decimal("-0.000000001", "USD".into()).unwrap())
                .unwrap(),
            source_digest: "a".repeat(64),
        }],
    }
}
fn monitor(capacity: usize) -> UsageMonitor {
    UsageMonitor::new(
        BillingScope {
            provider: "fixture-export".into(),
            account: "synthetic".into(),
            period_start_ms: 100,
            period_end_ms: 200,
        },
        Money::new(100, 0, "USD".into()).unwrap(),
        MonitorLimits {
            expires_at_ms: 1000,
            maximum_observations: 10,
            maximum_revision_entries: capacity,
            maximum_age_ms: 100,
        },
        300,
    )
    .unwrap()
}
#[test]
fn usage_has_exact_credits_and_idempotent_totals_without_invoice_conversion() {
    let mut m = monitor(10);
    let first = m.observe(&snapshot(), 350).unwrap();
    assert_eq!(first.total.coefficient(), 9_007_199_254_740_993_000_000_000);
    assert_eq!(first.total.scale(), 9);
    assert!(first.threshold_reached);
    assert!(!m.observe(&snapshot(), 350).unwrap().changed);
    let mut corrected = snapshot();
    corrected.observations[0].revision = "r2".into();
    corrected.observations[0].total = Money::new(-5, 2, "USD".into()).unwrap();
    assert_eq!(m.observe(&corrected, 350).unwrap().total.coefficient(), -5);
    assert_eq!(
        m.observe(&snapshot(), 350),
        Err(BillingError::RevisionRollback)
    );
    assert!(
        !serde_json::to_string(&snapshot())
            .unwrap()
            .contains("invoice")
    );
}
#[test]
fn refusal_keeps_last_complete_revision_and_observation_count() {
    let mut m = monitor(10);
    let original = snapshot();
    m.observe(&original, 350).unwrap();
    let mut cases = vec![];
    let mut s = original.clone();
    s.complete = false;
    cases.push(s);
    let mut s = original.clone();
    s.account = "other".into();
    cases.push(s);
    let mut s = original.clone();
    s.period_end_ms = 201;
    cases.push(s);
    let mut s = original.clone();
    s.observed_at_ms = 299;
    cases.push(s);
    let mut s = original.clone();
    s.observed_at_ms = 351;
    cases.push(s);
    let mut s = original.clone();
    s.observations[0].source_digest = "invalid".into();
    cases.push(s);
    let mut s = original.clone();
    s.observations.push(s.observations[0].clone());
    cases.push(s);
    let mut s = original.clone();
    s.observations[0].total = Money::new(1, 0, "JPY".into()).unwrap();
    cases.push(s);
    let mut s = original.clone();
    s.observations[0].source_digest = "b".repeat(64);
    cases.push(s);
    for s in cases {
        assert!(m.observe(&s, 350).is_err());
        assert_eq!(m.observations(), 1);
    }
    assert!(!m.observe(&original, 350).unwrap().changed);
}
#[test]
fn usage_revision_capacity_and_expiry_are_shared_bounded_state() {
    let mut m = monitor(1);
    m.observe(&snapshot(), 350).unwrap();
    let mut s = snapshot();
    s.observations[0].revision = "r2".into();
    assert_eq!(m.observe(&s, 350), Err(BillingError::MonitorCapacity));
    assert_eq!(m.observations(), 1);
    assert_eq!(m.observe(&s, 1000), Err(BillingError::MonitorInactive));
    assert!(!m.is_active());
    assert_eq!(m.observe(&s, 350), Err(BillingError::MonitorInactive));
}
