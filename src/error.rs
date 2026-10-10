#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CostError {
    CachedInputExceedsInput,
    InvalidRateCard,
    ArithmeticOverflow,
    Serialization,
}

impl std::fmt::Display for CostError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::CachedInputExceedsInput => "cached input tokens exceed input tokens",
            Self::InvalidRateCard => "invalid rate card",
            Self::ArithmeticOverflow => "cost arithmetic overflow",
            Self::Serialization => "deterministic serialization failed",
        })
    }
}

impl std::error::Error for CostError {}
