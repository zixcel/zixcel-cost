use super::BillingError;
use serde::{Deserialize, Serialize};

/// Exact signed decimal amount. Currency is a declared code, not an FX policy.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "WireMoney", into = "WireMoney")]
pub struct Money {
    coefficient: i128,
    scale: u8,
    currency: String,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireMoney {
    coefficient: String,
    scale: u8,
    currency: String,
}

impl TryFrom<WireMoney> for Money {
    type Error = BillingError;
    fn try_from(value: WireMoney) -> Result<Self, Self::Error> {
        let number = value
            .coefficient
            .parse::<i128>()
            .map_err(|_| BillingError::InvalidAmount)?;
        if number.to_string() != value.coefficient {
            return Err(BillingError::InvalidAmount);
        }
        Self::new(number, value.scale, value.currency)
    }
}

impl From<Money> for WireMoney {
    fn from(value: Money) -> Self {
        Self {
            coefficient: value.coefficient.to_string(),
            scale: value.scale,
            currency: value.currency,
        }
    }
}

impl Money {
    /// # Errors
    /// Rejects scales above nine and codes other than three uppercase ASCII letters.
    pub fn new(coefficient: i128, scale: u8, currency: String) -> Result<Self, BillingError> {
        if scale > 9
            || currency.len() != 3
            || !currency.bytes().all(|byte| byte.is_ascii_uppercase())
        {
            return Err(BillingError::InvalidAmount);
        }
        Ok(Self {
            coefficient,
            scale,
            currency,
        })
    }

    #[must_use]
    pub const fn coefficient(&self) -> i128 {
        self.coefficient
    }
    #[must_use]
    pub const fn scale(&self) -> u8 {
        self.scale
    }
    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// # Errors
    /// Rejects currency mixing and checked decimal arithmetic overflow.
    pub fn checked_add(&self, other: &Self) -> Result<Self, BillingError> {
        if self.currency != other.currency {
            return Err(BillingError::CurrencyMismatch);
        }
        let scale = self.scale.max(other.scale);
        let left = self.at_scale(scale)?;
        let right = other.at_scale(scale)?;
        Self::new(
            left.checked_add(right).ok_or(BillingError::Overflow)?,
            scale,
            self.currency.clone(),
        )
    }

    pub(super) fn at_scale(&self, scale: u8) -> Result<i128, BillingError> {
        let factor = 10_i128.pow(u32::from(scale - self.scale));
        self.coefficient
            .checked_mul(factor)
            .ok_or(BillingError::Overflow)
    }
}
