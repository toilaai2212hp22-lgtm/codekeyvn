//! Lightweight Vietnamese syllable validation (for English auto-restore).

use crate::tables::{is_d_family, is_vowel, strip_tone};

/// Decide whether to discard composed text and emit `raw` keys instead.
///
/// Restore when the user clearly typed English (CamelCase, English clusters,
/// letters j/f/w/z) but Telex/VNI still applied diacritics. Do **not** restore
/// pure-ASCII undo results like `ass` → `as`, or lone `đ` / `â`.
pub fn should_restore_english(composed: &str, raw: &str) -> bool {
    if composed.is_empty() || composed == raw {
        return false;
    }
    // CamelCase product names (CodeKey, iPhone-style already handled elsewhere).
    if is_camel_case(composed) {
        return true;
    }
    // English-only base letters present in composition.
    if has_english_only_base(composed) {
        return true;
    }
    // Has VN marks but fails syllable rules, and is longer than a partial mark.
    if has_viet_diacritic(composed)
        && !is_plausible_vietnamese(composed)
        && !is_partial_viet_atom(composed)
    {
        return true;
    }
    // English consonant clusters even without marks (rare if no marks applied).
    if !has_viet_diacritic(composed) && has_english_cluster(composed) {
        return false; // pure ASCII — keep composed (undo-friendly)
    }
    false
}

fn is_camel_case(word: &str) -> bool {
    let mut seen_lower = false;
    let mut upper_after_lower = false;
    for c in word.chars() {
        if c.is_lowercase() {
            seen_lower = true;
        } else if c.is_uppercase() && seen_lower {
            upper_after_lower = true;
        }
    }
    // Also: CodeKey starts upper, has upper inside
    let uppers = word.chars().filter(|c| c.is_uppercase()).count();
    let letters = word.chars().filter(|c| c.is_alphabetic()).count();
    upper_after_lower || (uppers >= 2 && uppers < letters)
}

fn has_english_only_base(word: &str) -> bool {
    word.chars().any(|c| {
        let (base, _) = strip_tone(c);
        matches!(base.to_ascii_lowercase(), 'j' | 'f' | 'w' | 'z')
    })
}

fn has_english_cluster(word: &str) -> bool {
    let lower: String = word
        .chars()
        .map(|c| strip_tone(c).0.to_ascii_lowercase())
        .collect();
    ["ck", "str", "spr", "scr", "st", "sp", "sk", "sm", "tw", "dw"]
        .iter()
        .any(|b| lower.contains(b))
}

/// Single marked letter or very short VN atom still being typed (`đ`, `â`, `ơ`).
fn is_partial_viet_atom(word: &str) -> bool {
    let letters: Vec<char> = word.chars().filter(|c| c.is_alphabetic()).collect();
    if letters.is_empty() || letters.len() > 2 {
        return false;
    }
    letters.iter().all(|&c| {
        let (base, tone) = strip_tone(c);
        tone != crate::tone::Tone::None
            || matches!(
                base.to_ascii_lowercase(),
                'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư' | 'đ'
            )
    })
}

/// True if `word` looks like a plausible Vietnamese syllable/word piece.
///
/// Used to decide whether to keep diacritics or restore the raw Latin keys
/// (so `CodeKey`, `Facebook`, `email` are not mangled).
pub fn is_plausible_vietnamese(word: &str) -> bool {
    if word.is_empty() {
        return true;
    }

    // Internal capitals (CamelCase) → almost always English product names.
    let mut chars = word.chars();
    let first = chars.next().unwrap();
    if chars.any(|c| c.is_uppercase()) && first.is_uppercase() {
        // Allow all-caps short VN acronyms? Keep strict: mixed/internal upper → EN
        let upper_count = word.chars().filter(|c| c.is_uppercase()).count();
        let letter_count = word.chars().filter(|c| c.is_alphabetic()).count();
        if upper_count >= 2 && upper_count < letter_count {
            return false;
        }
    }

    // Split on non-letters roughly — validate each alphabetic run.
    let mut ok_any = false;
    let mut run = String::new();
    for c in word.chars() {
        if c.is_alphabetic() || is_d_family(c) || is_vowel(c) {
            run.push(c);
        } else if !run.is_empty() {
            if is_plausible_syllable(&run) {
                ok_any = true;
            } else if has_viet_diacritic(&run) {
                // Had diacritics but invalid structure → not VN
                return false;
            } else {
                // plain ascii invalid syllable
                return false;
            }
            run.clear();
        }
    }
    if !run.is_empty() {
        if is_plausible_syllable(&run) {
            ok_any = true;
        } else {
            return false;
        }
    }
    ok_any || word.chars().all(|c| !c.is_alphabetic())
}

fn has_viet_diacritic(s: &str) -> bool {
    s.chars().any(|c| {
        let (base, tone) = strip_tone(c);
        tone != crate::tone::Tone::None
            || matches!(
                base.to_ascii_lowercase(),
                'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư' | 'đ'
            )
    })
}

/// Rule-based single-syllable check (consonant cluster + nucleus + coda).
fn is_plausible_syllable(syl: &str) -> bool {
    let s: String = syl
        .chars()
        .map(|c| strip_tone(c).0.to_ascii_lowercase())
        .collect();

    if s.is_empty() || s.len() > 8 {
        return false;
    }

    // Must contain a vowel to be a VN syllable (except standalone non-letter).
    let has_vowel = s.chars().any(|c| {
        matches!(
            c,
            'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
        )
    });
    if !has_vowel {
        // đ alone, or consonant-only → not a finished VN syllable
        return false;
    }

    // Quick reject: illegal consonant sequences common in English
    let lower = s.as_str();
    for bad in ["ck", "th", "st", "sp", "sk", "sm", "sn", "sw", "tw", "dw", "qu", "x"] {
        // "th" and "qu" ARE valid in Vietnamese (th, qu). Keep them.
        let _ = bad;
    }
    // English-only clusters
    for bad in ["ck", "st", "sp", "sk", "sm", "sn", "sw", "tw", "dw", "str", "spr", "scr"] {
        if lower.contains(bad) {
            return false;
        }
    }
    if lower.contains('j') || lower.contains('f') || lower.contains('z') || lower.contains('w') {
        // j/f/z/w not in modern VN alphabet as base letters
        return false;
    }

    // Onset + rhyme split: find first vowel
    let bytes: Vec<char> = lower.chars().collect();
    let mut i = 0;
    while i < bytes.len() && !is_vowel_base(bytes[i]) {
        i += 1;
    }
    if i == bytes.len() {
        return false;
    }
    let onset: String = bytes[..i].iter().collect();
    let rest: String = bytes[i..].iter().collect();

    if !valid_onset(&onset) {
        return false;
    }
    valid_rhyme(&rest)
}

fn is_vowel_base(c: char) -> bool {
    matches!(
        c,
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
    )
}

fn valid_onset(onset: &str) -> bool {
    matches!(
        onset,
        "" | "b"
            | "c"
            | "ch"
            | "d"
            | "đ"
            | "g"
            | "gh"
            | "gi"
            | "h"
            | "k"
            | "kh"
            | "l"
            | "m"
            | "n"
            | "ng"
            | "ngh"
            | "nh"
            | "p"
            | "ph"
            | "q"
            | "qu"
            | "r"
            | "s"
            | "t"
            | "th"
            | "tr"
            | "v"
            | "x"
    )
}

fn valid_rhyme(rhyme: &str) -> bool {
    if rhyme.is_empty() {
        return false;
    }
    // nucleus: one or more vowels, then optional coda
    let chars: Vec<char> = rhyme.chars().collect();
    let mut n = 0;
    while n < chars.len() && is_vowel_base(chars[n]) {
        n += 1;
    }
    if n == 0 || n > 3 {
        return false;
    }
    let nucleus: String = chars[..n].iter().collect();
    let coda: String = chars[n..].iter().collect();

    if !valid_nucleus(&nucleus) {
        return false;
    }
    valid_coda(&coda)
}

fn valid_nucleus(n: &str) -> bool {
    matches!(
        n,
        "a" | "ă"
            | "â"
            | "e"
            | "ê"
            | "i"
            | "o"
            | "ô"
            | "ơ"
            | "u"
            | "ư"
            | "y"
            | "ai"
            | "ao"
            | "au"
            | "ay"
            | "âu"
            | "ây"
            | "eo"
            | "êu"
            | "ia"
            | "iê"
            | "iu"
            | "oa"
            | "oă"
            | "oe"
            | "oi"
            | "ôi"
            | "ơi"
            | "ua"
            | "uâ"
            | "uê"
            | "ui"
            | "uo"
            | "uô"
            | "uy"
            | "ưa"
            | "ưi"
            | "ươ"
            | "ưu"
            | "ya"
            | "yê"
            | "yêu"
            | "iêu"
            | "oai"
            | "oay"
            | "uôi"
            | "ươi"
            | "uya"
            | "uyu"
            | "ươu"
    ) || {
        // allow simple 1-2 letter nuclei with marked vowels
        n.chars().all(is_vowel_base) && n.chars().count() <= 3
    }
}

fn valid_coda(c: &str) -> bool {
    matches!(
        c,
        "" | "c" | "ch" | "m" | "n" | "ng" | "nh" | "p" | "t" | "y" | "u" | "i" | "o"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vn_words_ok() {
        assert!(is_plausible_vietnamese("Việt"));
        assert!(is_plausible_vietnamese("người"));
        assert!(is_plausible_vietnamese("xin"));
        assert!(is_plausible_vietnamese("chào"));
        assert!(is_plausible_vietnamese("đúng"));
        assert!(is_plausible_vietnamese("tôi"));
    }

    #[test]
    fn english_rejected() {
        assert!(!is_plausible_vietnamese("CodeKey"));
        assert!(!is_plausible_vietnamese("Facebook"));
        assert!(!is_plausible_vietnamese("email"));
        assert!(!is_plausible_vietnamese("class"));
        assert!(!is_plausible_vietnamese("strong"));
    }
}
