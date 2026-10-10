use zixcel_cost::arithmetic::{ArithmeticError, Rounding, round_amount, scaled_cost, sum_costs};

#[test]
fn token_and_fractional_resource_quantities_share_one_integer_calculation() {
    assert_eq!(scaled_cost(800, 2, 1, Rounding::Exact), Ok(1600));
    assert_eq!(scaled_cost(1500, 3, 1000, Rounding::HalfUp), Ok(5));
    assert_eq!(scaled_cost(1500, 3, 1000, Rounding::Truncate), Ok(4));
    assert_eq!(
        scaled_cost(1500, 3, 1000, Rounding::Exact),
        Err(ArithmeticError::FractionalAmount)
    );
    assert_eq!(sum_costs([1600, 200, 480, 200]), Ok(2480));
    assert_eq!(round_amount(2480, 1_000_000, Rounding::HalfUp), Ok(0));
}

#[test]
fn half_up_handles_both_sides_of_boundary_and_odd_scales() {
    assert_eq!(round_amount(499_999, 1_000_000, Rounding::HalfUp), Ok(0));
    assert_eq!(round_amount(500_000, 1_000_000, Rounding::HalfUp), Ok(1));
    assert_eq!(round_amount(1, 3, Rounding::HalfUp), Ok(0));
    assert_eq!(round_amount(2, 3, Rounding::HalfUp), Ok(1));
    assert_eq!(round_amount(u64::MAX, 1, Rounding::HalfUp), Ok(u64::MAX));
}

#[test]
fn wide_intermediates_avoid_false_overflow_and_invalid_totals_reject() {
    assert_eq!(
        scaled_cost(u64::MAX, u64::MAX, u64::MAX, Rounding::Exact),
        Ok(u64::MAX)
    );
    assert_eq!(
        scaled_cost(u64::MAX, 2, 1, Rounding::Exact),
        Err(ArithmeticError::Overflow)
    );
    assert_eq!(
        scaled_cost(0, 0, 0, Rounding::HalfUp),
        Err(ArithmeticError::ZeroScale)
    );
    assert_eq!(sum_costs([u64::MAX, 1]), Err(ArithmeticError::Overflow));
    assert_eq!(sum_costs([]), Ok(0));
}

#[test]
fn rounding_is_applied_to_total_for_display_without_mutating_minor_amounts() {
    let minor = [499_999, 499_999];
    let total = sum_costs(minor).expect("total");
    assert_eq!(round_amount(total, 1_000_000, Rounding::HalfUp), Ok(1));
    assert_eq!(minor, [499_999, 499_999]);
}
