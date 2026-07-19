//! Tone marks (dấu thanh).

/// Vietnamese tone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Tone {
    #[default]
    None,
    /// sắc (´) — Telex `s`, VNI `1`
    Sac,
    /// huyền (`) — Telex `f`, VNI `2`
    Huyen,
    /// hỏi (˘) — Telex `r`, VNI `3`
    Hoi,
    /// ngã (~) — Telex `x`, VNI `4`
    Nga,
    /// nặng (.) — Telex `j`, VNI `5`
    Nang,
}

impl Tone {
    pub fn from_telex(c: char) -> Option<Self> {
        match c.to_ascii_lowercase() {
            's' => Some(Self::Sac),
            'f' => Some(Self::Huyen),
            'r' => Some(Self::Hoi),
            'x' => Some(Self::Nga),
            'j' => Some(Self::Nang),
            'z' => Some(Self::None), // remove tone
            _ => None,
        }
    }

    pub fn from_vni(c: char) -> Option<Self> {
        match c {
            '1' => Some(Self::Sac),
            '2' => Some(Self::Huyen),
            '3' => Some(Self::Hoi),
            '4' => Some(Self::Nga),
            '5' => Some(Self::Nang),
            '0' => Some(Self::None),
            _ => None,
        }
    }
}
