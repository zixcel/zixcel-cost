#![doc = include_str!("../README.crate.md")]
#![forbid(unsafe_code)]
#![doc = "Shared deterministic integer cost arithmetic and provider-neutral inference accounting."]

use serde::Serialize;
use sha2::{Digest, Sha256};
pub mod arithmetic;
mod error;
mod estimate;
mod rate_card;
mod token_usage;
pub use error::CostError;
pub use estimate::{CostEstimate, estimate};
pub use rate_card::CostRateCard;
pub use token_usage::TokenUsage;

pub const USAGE_SCHEMA_V1: &str = "zixcel://inference-cost/token-usage/v1";
pub const RATE_CARD_SCHEMA_V1: &str = "zixcel://inference-cost/rate-card/v1";
pub const ESTIMATE_SCHEMA_V1: &str = "zixcel://inference-cost/estimate/v1";

/// Conservative byte-based estimate used only when a provider did not return usage.
#[must_use]
pub const fn approximate_tokens_from_bytes(bytes: usize) -> usize {
    bytes.saturating_add(3) / 4
}

#[must_use]
pub const fn approximate_token_count(text: &str) -> usize {
    approximate_tokens_from_bytes(text.len())
}

#[must_use]
pub const fn approximate_bytes_for_tokens(tokens: usize) -> usize {
    tokens.saturating_mul(4)
}

#[must_use]
pub const fn approximate_tokens_from_byte_count(bytes: usize) -> u64 {
    approximate_tokens_from_bytes(bytes) as u64
}

fn digest_json<T: Serialize>(value: &T) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    let mut digest = Sha256::new();
    digest.update(bytes);
    format!("{:x}", digest.finalize())
}

#[cfg(test)]
mod tests;
