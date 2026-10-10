use crate::{CostError, digest_json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub cache_write_input_tokens: u64,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
}

impl TokenUsage {
    /// Creates a usage record, rejecting an impossible cached-input count.
    ///
    /// # Errors
    ///
    /// Returns [`CostError::CachedInputExceedsInput`] when cached input is
    /// greater than total input.
    pub fn new(
        input_tokens: u64,
        cached_input_tokens: u64,
        cache_write_input_tokens: u64,
        output_tokens: u64,
        reasoning_output_tokens: u64,
    ) -> Result<Self, CostError> {
        if cached_input_tokens > input_tokens {
            return Err(CostError::CachedInputExceedsInput);
        }
        Ok(Self {
            input_tokens,
            cached_input_tokens,
            cache_write_input_tokens,
            output_tokens,
            reasoning_output_tokens,
        })
    }

    #[must_use]
    pub fn digest(&self) -> String {
        digest_json(self)
    }
}
