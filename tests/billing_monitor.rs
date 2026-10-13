use zixcel_cost::billing::{BillingError, Money};
#[path = "support/billing_monitor_fixture.rs"]
mod fixture;
use fixture::{monitor, snapshot};

#[test]
fn repeated_observations_are_not_double_counted_and_the_session_stops_at_its_bound() {
    let mut session = monitor(2, 10);
    let first = session.observe(&snapshot(), 350).unwrap();
    assert!(first.changed && first.threshold_reached);
    let second = session.observe(&snapshot(), 350).unwrap();
    assert!(!second.changed);
    assert_eq!(first.total, second.total);
    assert!(!session.is_active());
    assert_eq!(
        session.observe(&snapshot(), 350),
        Err(BillingError::MonitorInactive)
    );
}
#[test]
fn stop_expiry_scope_substitution_and_partial_data_never_create_an_observation() {
    let mut session = monitor(10, 10);
    session.stop();
    assert_eq!(
        session.observe(&snapshot(), 350),
        Err(BillingError::MonitorInactive)
    );
    let mut session = monitor(10, 10);
    assert_eq!(
        session.observe(&snapshot(), 1000),
        Err(BillingError::MonitorInactive)
    );
    let mut wrong = snapshot();
    wrong.account = "other".into();
    assert!(session.observe(&wrong, 350).is_err());
    wrong = snapshot();
    wrong.complete = false;
    assert!(session.observe(&wrong, 350).is_err());
    assert_eq!(session.observations(), 0);
}
#[test]
fn immutable_invoice_revisions_reject_substitution_before_mutating_monitor_state() {
    let mut session = monitor(10, 10);
    session.observe(&snapshot(), 350).unwrap();
    let mut changed = snapshot();
    changed.invoices[0].total = Money::new(999, 2, "USD".into()).unwrap();
    assert_eq!(
        session.observe(&changed, 350),
        Err(BillingError::InconsistentRevision)
    );
    assert_eq!(session.observations(), 1);
    changed.invoices[0].revision = "r2".into();
    assert!(session.observe(&changed, 350).unwrap().changed);
}
#[test]
fn revision_capacity_and_observation_rollback_are_enforced_without_partial_updates() {
    let mut session = monitor(10, 1);
    session.observe(&snapshot(), 350).unwrap();
    let mut changed = snapshot();
    changed.invoices[0].revision = "r2".into();
    assert_eq!(
        session.observe(&changed, 350),
        Err(BillingError::MonitorCapacity)
    );
    assert_eq!(session.observations(), 1);
    changed = snapshot();
    changed.observed_at_ms = 299;
    assert_eq!(
        session.observe(&changed, 350),
        Err(BillingError::StaleObservation)
    );
}

#[test]
fn an_expired_or_rolled_back_session_cannot_be_resurrected_by_an_older_clock() {
    let mut session = monitor(10, 10);
    assert_eq!(
        session.observe(&snapshot(), 1000),
        Err(BillingError::MonitorInactive)
    );
    assert!(!session.is_active());
    assert_eq!(
        session.observe(&snapshot(), 350),
        Err(BillingError::MonitorInactive)
    );
    let mut session = monitor(10, 10);
    session.observe(&snapshot(), 350).unwrap();
    assert_eq!(
        session.observe(&snapshot(), 340),
        Err(BillingError::MonitorInactive)
    );
    assert!(!session.is_active());
}

#[test]
fn previously_superseded_invoice_revisions_do_not_silently_return_as_current() {
    let mut session = monitor(10, 10);
    session.observe(&snapshot(), 350).unwrap();
    let mut corrected = snapshot();
    corrected.invoices[0].revision = "r2".into();
    session.observe(&corrected, 350).unwrap();
    assert_eq!(
        session.observe(&snapshot(), 350),
        Err(BillingError::RevisionRollback)
    );
    assert_eq!(session.observations(), 2);
}
