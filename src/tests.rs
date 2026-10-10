use super::*;

#[test]
fn estimates_each_usage_component_without_floating_point() {
    let usage = TokenUsage::new(1_000, 200, 0, 120, 40).expect("usage");
    let rates = CostRateCard::per_token("micro-unit", "model-rate", "2026-09-04", 2, 1, 3, 4, 5);
    let result = estimate(&usage, &rates).expect("estimate");
    assert_eq!(result.input_micro_units, 1_600);
    assert_eq!(result.cached_input_micro_units, 200);
    assert_eq!(result.total_micro_units(), 2_480);
    assert!(!result.billing_authoritative);
}

#[test]
fn rejects_inconsistent_cached_usage() {
    assert_eq!(
        TokenUsage::new(1, 2, 0, 0, 0),
        Err(CostError::CachedInputExceedsInput)
    );
}

#[test]
fn approximation_is_deterministic() {
    assert_eq!(approximate_tokens_from_bytes(5), 2);
    assert_eq!(approximate_bytes_for_tokens(2), 8);
}

#[test]
fn rejects_checked_arithmetic_overflow() {
    let usage = TokenUsage::new(2, 0, 0, 0, 0).expect("usage");
    let rates = CostRateCard::per_token(
        "micro-unit",
        "model-rate",
        "2026-09-04",
        u64::MAX,
        0,
        0,
        0,
        0,
    );
    assert_eq!(estimate(&usage, &rates), Err(CostError::ArithmeticOverflow));
}
