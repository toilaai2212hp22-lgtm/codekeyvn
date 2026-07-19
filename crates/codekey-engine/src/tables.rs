//! Lookup tables: base vowels, diacritic variants, tone application.

use crate::tone::Tone;

/// Strip tone from a Vietnamese letter, return (base_with_diacritic, tone).
pub fn strip_tone(ch: char) -> (char, Tone) {
    for &(with_tone, base, tone) in TONE_MAP {
        if with_tone == ch {
            return (base, tone);
        }
    }
    (ch, Tone::None)
}

/// Apply `tone` to a letter that may already have diacritics (but preferably no tone).
pub fn apply_tone(ch: char, tone: Tone) -> Option<char> {
    let (base, _) = strip_tone(ch);
    if tone == Tone::None {
        return Some(base);
    }
    for &(with_tone, b, t) in TONE_MAP {
        if b == base && t == tone {
            return Some(with_tone);
        }
    }
    None
}

/// Whether `ch` is a Vietnamese vowel (with or without tone/diacritic).
pub fn is_vowel(ch: char) -> bool {
    let (base, _) = strip_tone(ch);
    matches!(
        base.to_ascii_lowercase(),
        'a' | 'ă' | 'â' | 'e' | 'ê' | 'i' | 'o' | 'ô' | 'ơ' | 'u' | 'ư' | 'y'
    )
}

/// Whether `ch` is `d` or `đ` (case-insensitive family).
pub fn is_d_family(ch: char) -> bool {
    matches!(ch.to_ascii_lowercase(), 'd' | 'đ')
}

/// Horn / circumflex / breve diacritic transformations on base letters.
/// Returns the transformed char preserving case of `current`.
pub fn apply_diacritic(current: char, kind: Diacritic) -> Option<char> {
    let lower = current.to_ascii_lowercase();
    let (base, tone) = strip_tone(lower);
    let new_base = match (base, kind) {
        ('a', Diacritic::Circumflex) => 'â',
        ('a', Diacritic::Breve) => 'ă',
        ('e', Diacritic::Circumflex) => 'ê',
        ('o', Diacritic::Circumflex) => 'ô',
        ('o', Diacritic::Horn) => 'ơ',
        ('u', Diacritic::Horn) => 'ư',
        // Already has diacritic of same kind → no change (caller may undo)
        ('â', Diacritic::Circumflex) => 'â',
        ('ă', Diacritic::Breve) => 'ă',
        ('ê', Diacritic::Circumflex) => 'ê',
        ('ô', Diacritic::Circumflex) => 'ô',
        ('ơ', Diacritic::Horn) => 'ơ',
        ('ư', Diacritic::Horn) => 'ư',
        // Convert between related forms when re-applying
        ('ă', Diacritic::Circumflex) => 'â',
        ('â', Diacritic::Breve) => 'ă',
        ('ơ', Diacritic::Circumflex) => 'ô',
        ('ô', Diacritic::Horn) => 'ơ',
        _ => return None,
    };
    let with_tone = apply_tone(new_base, tone).unwrap_or(new_base);
    Some(restore_case(with_tone, current))
}

/// d → đ (or reverse for undo checks).
pub fn apply_d_stroke(current: char) -> Option<char> {
    match current {
        'd' => Some('đ'),
        'D' => Some('Đ'),
        'đ' => Some('đ'),
        'Đ' => Some('Đ'),
        _ => None,
    }
}

pub fn undo_d_stroke(current: char) -> Option<char> {
    match current {
        'đ' => Some('d'),
        'Đ' => Some('D'),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Diacritic {
    Circumflex, // â ê ô
    Breve,      // ă
    Horn,       // ơ ư
}

fn restore_case(result: char, original: char) -> char {
    if original.is_uppercase() {
        // Vietnamese uppercase for special letters
        match result {
            'ă' => 'Ă',
            'â' => 'Â',
            'ê' => 'Ê',
            'ô' => 'Ô',
            'ơ' => 'Ơ',
            'ư' => 'Ư',
            'đ' => 'Đ',
            c if c.is_ascii_lowercase() => c.to_ascii_uppercase(),
            // toned lowercase → upper
            c => upper_viet(c).unwrap_or(c),
        }
    } else {
        result
    }
}

fn upper_viet(c: char) -> Option<char> {
    Some(match c {
        'á' => 'Á',
        'à' => 'À',
        'ả' => 'Ả',
        'ã' => 'Ã',
        'ạ' => 'Ạ',
        'ắ' => 'Ắ',
        'ằ' => 'Ằ',
        'ẳ' => 'Ẳ',
        'ẵ' => 'Ẵ',
        'ặ' => 'Ặ',
        'ấ' => 'Ấ',
        'ầ' => 'Ầ',
        'ẩ' => 'Ẩ',
        'ẫ' => 'Ẫ',
        'ậ' => 'Ậ',
        'é' => 'É',
        'è' => 'È',
        'ẻ' => 'Ẻ',
        'ẽ' => 'Ẽ',
        'ẹ' => 'Ẹ',
        'ế' => 'Ế',
        'ề' => 'Ề',
        'ể' => 'Ể',
        'ễ' => 'Ễ',
        'ệ' => 'Ệ',
        'í' => 'Í',
        'ì' => 'Ì',
        'ỉ' => 'Ỉ',
        'ĩ' => 'Ĩ',
        'ị' => 'Ị',
        'ó' => 'Ó',
        'ò' => 'Ò',
        'ỏ' => 'Ỏ',
        'õ' => 'Õ',
        'ọ' => 'Ọ',
        'ố' => 'Ố',
        'ồ' => 'Ồ',
        'ổ' => 'Ổ',
        'ỗ' => 'Ỗ',
        'ộ' => 'Ộ',
        'ớ' => 'Ớ',
        'ờ' => 'Ờ',
        'ở' => 'Ở',
        'ỡ' => 'Ỡ',
        'ợ' => 'Ợ',
        'ú' => 'Ú',
        'ù' => 'Ù',
        'ủ' => 'Ủ',
        'ũ' => 'Ũ',
        'ụ' => 'Ụ',
        'ứ' => 'Ứ',
        'ừ' => 'Ừ',
        'ử' => 'Ử',
        'ữ' => 'Ữ',
        'ự' => 'Ự',
        'ý' => 'Ý',
        'ỳ' => 'Ỳ',
        'ỷ' => 'Ỷ',
        'ỹ' => 'Ỹ',
        'ỵ' => 'Ỵ',
        'ă' => 'Ă',
        'â' => 'Â',
        'ê' => 'Ê',
        'ô' => 'Ô',
        'ơ' => 'Ơ',
        'ư' => 'Ư',
        'đ' => 'Đ',
        _ => return None,
    })
}

/// (with_tone, base_without_tone, tone)
const TONE_MAP: &[(char, char, Tone)] = &[
    // a
    ('á', 'a', Tone::Sac),
    ('à', 'a', Tone::Huyen),
    ('ả', 'a', Tone::Hoi),
    ('ã', 'a', Tone::Nga),
    ('ạ', 'a', Tone::Nang),
    // ă
    ('ắ', 'ă', Tone::Sac),
    ('ằ', 'ă', Tone::Huyen),
    ('ẳ', 'ă', Tone::Hoi),
    ('ẵ', 'ă', Tone::Nga),
    ('ặ', 'ă', Tone::Nang),
    // â
    ('ấ', 'â', Tone::Sac),
    ('ầ', 'â', Tone::Huyen),
    ('ẩ', 'â', Tone::Hoi),
    ('ẫ', 'â', Tone::Nga),
    ('ậ', 'â', Tone::Nang),
    // e
    ('é', 'e', Tone::Sac),
    ('è', 'e', Tone::Huyen),
    ('ẻ', 'e', Tone::Hoi),
    ('ẽ', 'e', Tone::Nga),
    ('ẹ', 'e', Tone::Nang),
    // ê
    ('ế', 'ê', Tone::Sac),
    ('ề', 'ê', Tone::Huyen),
    ('ể', 'ê', Tone::Hoi),
    ('ễ', 'ê', Tone::Nga),
    ('ệ', 'ê', Tone::Nang),
    // i
    ('í', 'i', Tone::Sac),
    ('ì', 'i', Tone::Huyen),
    ('ỉ', 'i', Tone::Hoi),
    ('ĩ', 'i', Tone::Nga),
    ('ị', 'i', Tone::Nang),
    // o
    ('ó', 'o', Tone::Sac),
    ('ò', 'o', Tone::Huyen),
    ('ỏ', 'o', Tone::Hoi),
    ('õ', 'o', Tone::Nga),
    ('ọ', 'o', Tone::Nang),
    // ô
    ('ố', 'ô', Tone::Sac),
    ('ồ', 'ô', Tone::Huyen),
    ('ổ', 'ô', Tone::Hoi),
    ('ỗ', 'ô', Tone::Nga),
    ('ộ', 'ô', Tone::Nang),
    // ơ
    ('ớ', 'ơ', Tone::Sac),
    ('ờ', 'ơ', Tone::Huyen),
    ('ở', 'ơ', Tone::Hoi),
    ('ỡ', 'ơ', Tone::Nga),
    ('ợ', 'ơ', Tone::Nang),
    // u
    ('ú', 'u', Tone::Sac),
    ('ù', 'u', Tone::Huyen),
    ('ủ', 'u', Tone::Hoi),
    ('ũ', 'u', Tone::Nga),
    ('ụ', 'u', Tone::Nang),
    // ư
    ('ứ', 'ư', Tone::Sac),
    ('ừ', 'ư', Tone::Huyen),
    ('ử', 'ư', Tone::Hoi),
    ('ữ', 'ư', Tone::Nga),
    ('ự', 'ư', Tone::Nang),
    // y
    ('ý', 'y', Tone::Sac),
    ('ỳ', 'y', Tone::Huyen),
    ('ỷ', 'y', Tone::Hoi),
    ('ỹ', 'y', Tone::Nga),
    ('ỵ', 'y', Tone::Nang),
    // uppercase
    ('Á', 'A', Tone::Sac),
    ('À', 'A', Tone::Huyen),
    ('Ả', 'A', Tone::Hoi),
    ('Ã', 'A', Tone::Nga),
    ('Ạ', 'A', Tone::Nang),
    ('Ắ', 'Ă', Tone::Sac),
    ('Ằ', 'Ă', Tone::Huyen),
    ('Ẳ', 'Ă', Tone::Hoi),
    ('Ẵ', 'Ă', Tone::Nga),
    ('Ặ', 'Ă', Tone::Nang),
    ('Ấ', 'Â', Tone::Sac),
    ('Ầ', 'Â', Tone::Huyen),
    ('Ẩ', 'Â', Tone::Hoi),
    ('Ẫ', 'Â', Tone::Nga),
    ('Ậ', 'Â', Tone::Nang),
    ('É', 'E', Tone::Sac),
    ('È', 'E', Tone::Huyen),
    ('Ẻ', 'E', Tone::Hoi),
    ('Ẽ', 'E', Tone::Nga),
    ('Ẹ', 'E', Tone::Nang),
    ('Ế', 'Ê', Tone::Sac),
    ('Ề', 'Ê', Tone::Huyen),
    ('Ể', 'Ê', Tone::Hoi),
    ('Ễ', 'Ê', Tone::Nga),
    ('Ệ', 'Ê', Tone::Nang),
    ('Í', 'I', Tone::Sac),
    ('Ì', 'I', Tone::Huyen),
    ('Ỉ', 'I', Tone::Hoi),
    ('Ĩ', 'I', Tone::Nga),
    ('Ị', 'I', Tone::Nang),
    ('Ó', 'O', Tone::Sac),
    ('Ò', 'O', Tone::Huyen),
    ('Ỏ', 'O', Tone::Hoi),
    ('Õ', 'O', Tone::Nga),
    ('Ọ', 'O', Tone::Nang),
    ('Ố', 'Ô', Tone::Sac),
    ('Ồ', 'Ô', Tone::Huyen),
    ('Ổ', 'Ô', Tone::Hoi),
    ('Ỗ', 'Ô', Tone::Nga),
    ('Ộ', 'Ô', Tone::Nang),
    ('Ớ', 'Ơ', Tone::Sac),
    ('Ờ', 'Ơ', Tone::Huyen),
    ('Ở', 'Ơ', Tone::Hoi),
    ('Ỡ', 'Ơ', Tone::Nga),
    ('Ợ', 'Ơ', Tone::Nang),
    ('Ú', 'U', Tone::Sac),
    ('Ù', 'U', Tone::Huyen),
    ('Ủ', 'U', Tone::Hoi),
    ('Ũ', 'U', Tone::Nga),
    ('Ụ', 'U', Tone::Nang),
    ('Ứ', 'Ư', Tone::Sac),
    ('Ừ', 'Ư', Tone::Huyen),
    ('Ử', 'Ư', Tone::Hoi),
    ('Ữ', 'Ư', Tone::Nga),
    ('Ự', 'Ư', Tone::Nang),
    ('Ý', 'Y', Tone::Sac),
    ('Ỳ', 'Y', Tone::Huyen),
    ('Ỷ', 'Y', Tone::Hoi),
    ('Ỹ', 'Y', Tone::Nga),
    ('Ỵ', 'Y', Tone::Nang),
];
