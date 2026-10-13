use super::{BillingError, Money};

impl Money {
    /// Parse a JSON-number-style decimal string without an intermediate float or rounding.
    /// # Errors
    /// Rejects noncanonical numeric syntax, scales above nine and i128 overflow.
    pub fn from_decimal(value: &str, currency: String) -> Result<Self, BillingError> {
        if value.is_empty() || value.len() > 128 {
            return Err(BillingError::InvalidAmount);
        }
        let mut pieces = value.split(['e', 'E']);
        let mantissa = pieces.next().ok_or(BillingError::InvalidAmount)?;
        let exponent = match pieces.next() {
            Some(text) if !text.is_empty() && text.len() <= 5 => text
                .parse::<i32>()
                .map_err(|_| BillingError::InvalidAmount)?,
            Some(_) => return Err(BillingError::InvalidAmount),
            None => 0,
        };
        if pieces.next().is_some() {
            return Err(BillingError::InvalidAmount);
        }
        let (negative, unsigned) = mantissa
            .strip_prefix('-')
            .map_or((false, mantissa), |s| (true, s));
        let (whole, fraction) = unsigned
            .split_once('.')
            .map_or((unsigned, None), |(a, b)| (a, Some(b)));
        if whole.is_empty()
            || !whole.bytes().all(|b| b.is_ascii_digit())
            || (whole.len() > 1 && whole.starts_with('0'))
            || fraction.is_some_and(|s| s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(BillingError::InvalidAmount);
        }
        let fraction = fraction.unwrap_or("");
        let mut coefficient = format!("{}{whole}{fraction}", if negative { "-" } else { "" })
            .parse::<i128>()
            .map_err(|_| BillingError::Overflow)?;
        let scale =
            i32::try_from(fraction.len()).map_err(|_| BillingError::InvalidAmount)? - exponent;
        if scale > 9 {
            return Err(BillingError::InvalidAmount);
        }
        let scale = if scale < 0 {
            let magnitude = u32::try_from(-scale).map_err(|_| BillingError::Overflow)?;
            let factor = 10_i128
                .checked_pow(magnitude)
                .ok_or(BillingError::Overflow)?;
            coefficient = coefficient
                .checked_mul(factor)
                .ok_or(BillingError::Overflow)?;
            0
        } else {
            u8::try_from(scale).map_err(|_| BillingError::InvalidAmount)?
        };
        Self::new(coefficient, scale, currency)
    }
}
