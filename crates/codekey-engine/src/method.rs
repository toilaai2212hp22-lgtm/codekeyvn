//! Typing methods (Telex, VNI, …).

/// Vietnamese input method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InputMethod {
    /// Telex: `aa`→â, `aw`→ă, `s/f/r/x/j` tones, `dd`→đ, `w`→ư/ơ.
    #[default]
    Telex,
    /// VNI: digits `1–5` tones, `6–9` diacritics, `0` clear tone.
    Vni,
}

impl InputMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Telex => "telex",
            Self::Vni => "vni",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_lowercase().as_str() {
            "telex" | "tx" => Some(Self::Telex),
            "vni" => Some(Self::Vni),
            _ => None,
        }
    }
}

impl std::fmt::Display for InputMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
