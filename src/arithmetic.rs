//! Shared integer cost arithmetic; callers own currencies, rates and provenance.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Rounding {
    Exact,
    Truncate,
    HalfUp,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArithmeticError {
    ZeroScale,
    FractionalAmount,
    Overflow,
}

impl std::fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::ZeroScale => "quantity scale must be positive",
            Self::FractionalAmount => "exact calculation would discard a fractional amount",
            Self::Overflow => "cost amount exceeds the supported range",
        })
    }
}
impl std::error::Error for ArithmeticError {}

/// Calculates quantity × rate / quantity scale in caller-defined money units.
/// For milli-units use scale 1000; whole-token quantities use scale 1.
/// The scale and rounding are explicit; there is no currency conversion.
/// # Errors
/// Rejects zero scale, an inexact result under `Exact`, or an amount above `u64`.
pub fn scaled_cost(
    quantity: u64,
    rate: u64,
    quantity_scale: u64,
    rounding: Rounding,
) -> Result<u64, ArithmeticError> {
    if quantity_scale == 0 {
        return Err(ArithmeticError::ZeroScale);
    }
    let product = u128::from(quantity) * u128::from(rate);
    let scale = u128::from(quantity_scale);
    let whole = product / scale;
    let remainder = product % scale;
    let rounded = match rounding {
        Rounding::Exact if remainder != 0 => return Err(ArithmeticError::FractionalAmount),
        Rounding::HalfUp if remainder >= scale - remainder => whole + 1,
        _ => whole,
    };
    u64::try_from(rounded).map_err(|_| ArithmeticError::Overflow)
}

/// Rounds an amount for display using an explicit minor-unit scale.
/// The caller must retain the original amount for later aggregation.
/// # Errors
/// Has the same scale, exactness and range errors as [`scaled_cost`].
pub fn round_amount(
    amount: u64,
    units_per_whole: u64,
    rounding: Rounding,
) -> Result<u64, ArithmeticError> {
    scaled_cost(amount, 1, units_per_whole, rounding)
}

/// Aggregates amounts only after the caller has established matching units.
/// # Errors
/// Rejects overflow without returning a partial total.
pub fn sum_costs(values: impl IntoIterator<Item = u64>) -> Result<u64, ArithmeticError> {
    values.into_iter().try_fold(0_u64, |sum, value| {
        sum.checked_add(value).ok_or(ArithmeticError::Overflow)
    })
}
