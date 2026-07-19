//! Syllable helpers: find which vowel receives the tone mark.
//!
//! Rules follow common UniKey / Vietnamese orthography behaviour:
//! - Marked vowels (ăâêôơư) win, with ơ/ê/ô/â/ă over ư (`người`).
//! - Diphthongs: `ua` → tone on **u** (`của`), but **qu** + a → tone on **a** (`quả`).
//! - `ia` → tone on **i** (`tía`), but **gi** + a → tone on **a** (`giá`).
//! - `oa`/`oe` → usually on **a**/**e** (`hoà`), except when o is the main vowel.

use crate::tables::{is_vowel, strip_tone};

/// Index of the vowel that should carry the tone in `chars`.
pub fn tone_target_index(chars: &[char]) -> Option<usize> {
    if chars.is_empty() {
        return None;
    }

    // 1) Prefer vowels that already have diacritics.
    //    Rank: ơ ê ô â ă first, then ư (so ươ → tone on ơ).
    for wanted in ['ơ', 'ê', 'ô', 'â', 'ă', 'ư'] {
        for (i, &c) in chars.iter().enumerate() {
            let (base, _) = strip_tone(c.to_ascii_lowercase());
            if base == wanted {
                return Some(i);
            }
        }
    }

    let vowels: Vec<(usize, char)> = chars
        .iter()
        .enumerate()
        .filter(|(_, c)| is_vowel(**c))
        .map(|(i, c)| {
            let (base, _) = strip_tone(c.to_ascii_lowercase());
            (i, base)
        })
        .collect();

    if vowels.is_empty() {
        return None;
    }
    if vowels.len() == 1 {
        return Some(vowels[0].0);
    }

    let nucleus: String = vowels.iter().map(|(_, c)| *c).collect();
    let first_vowel_at = vowels[0].0;
    let onset: String = chars[..first_vowel_at]
        .iter()
        .map(|c| c.to_ascii_lowercase())
        .collect();

    // 2) Diphthong / triphthong tables (index into `vowels`, not full buffer).
    let target_in_nucleus: usize = match nucleus.as_str() {
        // ua / ưa: tone on u/ư — except qu* where u is a glide (quả, quá).
        "ua" | "uă" | "uâ" | "ưa" => {
            if is_qu_onset(&onset) {
                1 // a
            } else {
                0 // u / ư  → của, cửa, múa, chúa
            }
        }

        // ia / ya: tone on i/y — except gi* where i is a glide (giá, gìa→già).
        "ia" | "ya" => {
            if is_gi_onset(&onset) {
                1 // a
            } else {
                0 // i  → tía, mía, chìa
            }
        }

        // ie / iê / yê: tone on e/ê (tiếng, yến)
        "ie" | "iê" | "ye" | "yê" => 1,

        // uo / uô / ươ: tone on o/ô/ơ (muốn, người, được)
        "uo" | "uô" | "ươ" | "ưo" => 1,

        // uy: tone on y (thủy, quý) — except quy already covered via qu + y
        "uy" => 1,

        // oa: tone on a (hoà, toà, quả-like without q uses o+a)
        //     "oà" style — a receives tone (modern + our default)
        "oa" | "oă" => 1,

        // oe: tone on e for "khoẻ/khỏe" variants — actually khỏe has tone on o.
        //     Vietnamese: khỏe = o + hỏi + e → tone on o.
        "oe" => 0,

        // Falling diphthongs: tone on first
        "oi" | "ai" | "ei" | "ui" | "ưi" | "ôi" | "ơi" | "ay" | "ây" | "ao" | "au" | "âu"
        | "eo" | "êu" | "iu" | "ưu" | "uu" | "ou" => 0,

        // Triphthongs
        "oai" | "oay" => 1, // a
        "uya" => {
            // "khuya" — tone on a; "chuyện" is different nucleus
            if is_qu_onset(&onset) {
                2
            } else {
                1
            }
        }
        "uyu" => 1,
        "ieu" | "iêu" | "yeu" | "yêu" => 1, // ê
        "uoi" | "uôi" | "ươi" | "ưoi" => 1, // ô/ơ

        // Default: last vowel
        _ => vowels.len() - 1,
    };

    vowels.get(target_in_nucleus).map(|(i, _)| *i)
}

/// Onset is `q` / `qu` (u is glide, not a real vowel for tone).
fn is_qu_onset(onset: &str) -> bool {
    onset == "q" || onset == "qu" || onset.ends_with("q")
}

/// Onset is `g` / `gi` (i is glide in "gia", "giá").
fn is_gi_onset(onset: &str) -> bool {
    onset == "g" || onset == "gi" || onset.ends_with("gi") || onset == "g"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idx(s: &str) -> Option<usize> {
        let v: Vec<char> = s.chars().collect();
        tone_target_index(&v)
    }

    #[test]
    fn cua_tone_on_u() {
        // c u a → tone on u (index 1)
        assert_eq!(idx("cua"), Some(1));
    }

    #[test]
    fn qua_tone_on_a() {
        // q u a → vowels at 1,2 → tone on a
        assert_eq!(idx("qua"), Some(2));
    }

    #[test]
    fn tia_tone_on_i() {
        assert_eq!(idx("tia"), Some(1));
    }

    #[test]
    fn gia_tone_on_a() {
        // g i a → tone on a
        assert_eq!(idx("gia"), Some(2));
    }
}
