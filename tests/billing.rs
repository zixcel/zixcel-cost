use zixcel_cost::billing::{
    BillingError, BillingSnapshot, InvoiceObservation, Money, evaluate_threshold,
};

fn money(value: i128, scale: u8, currency: &str) -> Money {
    Money::new(value, scale, currency.to_owned()).unwrap()
}
fn snapshot() -> BillingSnapshot {
    BillingSnapshot {
        provider: "synthetic-cloud".to_owned(),
        account: "fixture-account".to_owned(),
        period_start_ms: 100,
        period_end_ms: 200,
        observed_at_ms: 300,
        complete: true,
        invoices: vec![InvoiceObservation {
            invoice_id: "fixture-invoice".to_owned(),
            revision: "rev-2".to_owned(),
            period_start_ms: 100,
            period_end_ms: 200,
            total: money(125, 2, "JPY"),
            source_digest: "a".repeat(64),
        }],
    }
}

#[test]
fn exact_decimal_addition_preserves_credits_without_floats() {
    assert_eq!(
        money(12, 1, "USD")
            .checked_add(&money(-15, 2, "USD"))
            .unwrap(),
        money(105, 2, "USD")
    );
    assert_eq!(
        money(1, 0, "USD").checked_add(&money(1, 0, "JPY")),
        Err(BillingError::CurrencyMismatch)
    );
    assert_eq!(
        money(i128::MAX, 0, "USD").checked_add(&money(1, 1, "USD")),
        Err(BillingError::Overflow)
    );
}

#[test]
fn json_uses_canonical_strings_and_validates_untrusted_input() {
    let input = money(i128::MAX, 9, "USD");
    let encoded = serde_json::to_string(&input).unwrap();
    assert!(encoded.contains(&format!("\"{}\"", i128::MAX)));
    assert_eq!(serde_json::from_str::<Money>(&encoded).unwrap(), input);
    for invalid in [
        r#"{"coefficient":"01","scale":2,"currency":"USD"}"#,
        r#"{"coefficient":"-0","scale":2,"currency":"USD"}"#,
        r#"{"coefficient":"1","scale":10,"currency":"USD"}"#,
        r#"{"coefficient":1,"scale":2,"currency":"USD"}"#,
        r#"{"coefficient":"1","scale":2,"currency":"usd"}"#,
        r#"{"coefficient":"1","scale":2,"currency":"USD","token":"fixture"}"#,
    ] {
        assert!(serde_json::from_str::<Money>(invalid).is_err());
    }
}

#[test]
fn threshold_uses_latest_complete_exact_scope_observation() {
    let value = snapshot();
    let result = evaluate_threshold(&value, &money(1, 0, "JPY"), 350, 100).unwrap();
    assert!(result.threshold_reached);
    assert_eq!(result.total, money(125, 2, "JPY"));
    assert_eq!(
        evaluate_threshold(&value, &money(1, 0, "USD"), 350, 100),
        Err(BillingError::CurrencyMismatch)
    );
    for (now, age) in [(250, 100), (401, 100), (350, 0), (350, 86_400_001)] {
        assert_eq!(
            evaluate_threshold(&value, &money(1, 0, "JPY"), now, age),
            Err(BillingError::StaleObservation)
        );
    }
}

#[test]
fn partial_duplicate_wrong_period_and_unbounded_invoices_are_rejected() {
    let mut value = snapshot();
    value.complete = false;
    assert_eq!(value.validate(), Err(BillingError::InvalidObservation));
    value = snapshot();
    value.invoices.push(value.invoices[0].clone());
    value.invoices[1].revision = "rev-1".to_owned();
    assert_eq!(value.validate(), Err(BillingError::DuplicateInvoice));
    value = snapshot();
    value.invoices[0].period_end_ms = 201;
    assert_eq!(value.validate(), Err(BillingError::InvalidObservation));
    value = snapshot();
    value.invoices = vec![value.invoices[0].clone(); 1025];
    assert_eq!(value.validate(), Err(BillingError::InvalidObservation));
}

#[test]
fn only_complete_empty_snapshot_is_known_zero() {
    let mut value = snapshot();
    value.invoices.clear();
    let result = evaluate_threshold(&value, &money(1, 0, "JPY"), 350, 100).unwrap();
    assert!(!result.threshold_reached);
    assert_eq!(result.total.coefficient(), 0);
    value.complete = false;
    assert!(evaluate_threshold(&value, &money(1, 0, "JPY"), 350, 100).is_err());
}

#[test]
fn source_decimal_lexemes_never_pass_through_floating_point() {
    assert_eq!(
        Money::from_decimal("12345678901234567890.123", "USD".into()).unwrap(),
        money(12_345_678_901_234_567_890_123, 3, "USD")
    );
    assert_eq!(
        Money::from_decimal("-1.25e+2", "USD".into()).unwrap(),
        money(-125, 0, "USD")
    );
    assert_eq!(
        Money::from_decimal("1.25e-2", "USD".into()).unwrap(),
        money(125, 4, "USD")
    );
    assert_eq!(
        Money::from_decimal("-170141183460469231731687303715884105728", "USD".into())
            .unwrap()
            .coefficient(),
        i128::MIN
    );
    for input in [
        "", " 1", "+1", "01", ".1", "1.", "1e", "1e1e1", "1e-10", "NaN",
    ] {
        assert!(Money::from_decimal(input, "USD".into()).is_err(), "{input}");
    }
    assert_eq!(
        Money::from_decimal("1e1000", "USD".into()),
        Err(BillingError::Overflow)
    );
}
