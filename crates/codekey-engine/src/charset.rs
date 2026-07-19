//! Charset helpers (Unicode output for MVP).

/// Output charset. MVP uses Unicode only; hooks for TCVN3/VNI later.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Charset {
    #[default]
    Unicode,
}
