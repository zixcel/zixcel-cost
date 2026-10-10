use crate::{CostError, RATE_CARD_SCHEMA_V1, digest_json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CostRateCard {
    pub schema: String,
    pub unit: String,
    pub source: String,
    pub as_of: String,
    pub input_micro_units_per_token: u64,
    pub cached_input_micro_units_per_token: u64,
    pub cache_write_input_micro_units_per_token: u64,
    pub output_micro_units_per_token: u64,
    pub reasoning_output_micro_units_per_token: u64,
}

impl CostRateCard {
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn per_token(
        unit: impl Into<String>,
        source: impl Into<String>,
        as_of: impl Into<String>,
        input: u64,
        cached_input: u64,
        cache_write_input: u64,
        output: u64,
        reasoning_output: u64,
    ) -> Self {
        Self {
            schema: RATE_CARD_SCHEMA_V1.to_owned(),
            unit: unit.into(),
            source: source.into(),
            as_of: as_of.into(),
            input_micro_units_per_token: input,
            cached_input_micro_units_per_token: cached_input,
            cache_write_input_micro_units_per_token: cache_write_input,
            output_micro_units_per_token: output,
            reasoning_output_micro_units_per_token: reasoning_output,
        }
    }

    /// Validates the identity and provenance fields of the rate card.
    ///
    /// # Errors
    ///
    /// Returns [`CostError::InvalidRateCard`] for an unsupported schema or an
    /// empty unit, source, or timestamp.
    pub fn validate(&self) -> Result<(), CostError> {
        if self.schema != RATE_CARD_SCHEMA_V1
            || self.unit.trim().is_empty()
            || self.source.trim().is_empty()
            || self.as_of.trim().is_empty()
        {
            return Err(CostError::InvalidRateCard);
        }
        Ok(())
    }

    #[must_use]
    pub fn digest(&self) -> String {
        digest_json(self)
    }
}
