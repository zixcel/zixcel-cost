use crate::{CostError, CostRateCard, ESTIMATE_SCHEMA_V1, TokenUsage};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CostEstimate {
    pub schema: String,
    pub unit: String,
    pub usage_digest: String,
    pub rate_card_digest: String,
    pub input_micro_units: u64,
    pub cached_input_micro_units: u64,
    pub cache_write_input_micro_units: u64,
    pub output_micro_units: u64,
    pub reasoning_output_micro_units: u64,
    pub total_micro_units: u64,
    pub billing_authoritative: bool,
}

impl CostEstimate {
    #[must_use]
    pub const fn total_micro_units(&self) -> u64 {
        self.total_micro_units
    }
}

/// Estimates provider usage using the supplied integer rate card.
///
/// # Errors
///
/// Returns an error when the rate card is invalid or checked arithmetic would
/// overflow.
pub fn estimate(usage: &TokenUsage, rates: &CostRateCard) -> Result<CostEstimate, CostError> {
    rates.validate()?;
    // Provider usage reports `input_tokens` as the total input, while cached
    // input is a billable subset with its own rate. Charge only the uncached
    // remainder at the ordinary input rate; charging the total would count
    // cached tokens twice.
    let uncached_input_tokens = usage
        .input_tokens
        .checked_sub(usage.cached_input_tokens)
        .ok_or(CostError::CachedInputExceedsInput)?;
    let input = multiply(uncached_input_tokens, rates.input_micro_units_per_token)?;
    let cached = multiply(
        usage.cached_input_tokens,
        rates.cached_input_micro_units_per_token,
    )?;
    let cache_write = multiply(
        usage.cache_write_input_tokens,
        rates.cache_write_input_micro_units_per_token,
    )?;
    let output = multiply(usage.output_tokens, rates.output_micro_units_per_token)?;
    let reasoning_output = multiply(
        usage.reasoning_output_tokens,
        rates.reasoning_output_micro_units_per_token,
    )?;
    let total =
        crate::arithmetic::sum_costs([input, cached, cache_write, output, reasoning_output])
            .map_err(|_| CostError::ArithmeticOverflow)?;

    Ok(CostEstimate {
        schema: ESTIMATE_SCHEMA_V1.to_owned(),
        unit: rates.unit.clone(),
        usage_digest: usage.digest(),
        rate_card_digest: rates.digest(),
        input_micro_units: input,
        cached_input_micro_units: cached,
        cache_write_input_micro_units: cache_write,
        output_micro_units: output,
        reasoning_output_micro_units: reasoning_output,
        total_micro_units: total,
        billing_authoritative: false,
    })
}

fn multiply(quantity: u64, rate: u64) -> Result<u64, CostError> {
    crate::arithmetic::scaled_cost(quantity, rate, 1, crate::arithmetic::Rounding::Exact)
        .map_err(|_| CostError::ArithmeticOverflow)
}
