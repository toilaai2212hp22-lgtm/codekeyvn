//! Incremental composition engine.

use crate::method::InputMethod;
use crate::syllable::tone_target_index;
use crate::tables::{
    apply_d_stroke, apply_diacritic, apply_tone, is_d_family, is_vowel, strip_tone, undo_d_stroke,
    Diacritic,
};
use crate::tone::Tone;

/// Result of feeding one key into the engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyResult {
    /// Preedit text updated (still composing this word).
    Update,
    /// Character was appended as-is (letter that starts/continues buffer).
    Append,
    /// Key did not belong to current composition — commit buffer + pass key.
    CommitAndPass,
    /// Key committed the buffer (space, punctuation, etc.) and may be included.
    Commit,
    /// Backspace consumed inside preedit (or emptied buffer).
    Backspace,
    /// Key ignored (e.g. disabled).
    Ignored,
}

/// Engine behaviour flags.
#[derive(Debug, Clone, Copy)]
pub struct EngineOptions {
    /// On commit, if the word is not a plausible VN syllable, restore raw keys
    /// (prevents `Facebook` / `class` from keeping wrong diacritics).
    pub auto_restore_english: bool,
}

impl Default for EngineOptions {
    fn default() -> Self {
        Self {
            auto_restore_english: true,
        }
    }
}

/// Stateful Vietnamese input engine (one composition session / word).
#[derive(Debug, Clone)]
pub struct Engine {
    method: InputMethod,
    /// Composed characters currently shown as preedit.
    buffer: Vec<char>,
    /// Raw key history (typed keys, including tone/diacritic keys consumed).
    raw: Vec<char>,
    enabled: bool,
    options: EngineOptions,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(InputMethod::Telex)
    }
}

impl Engine {
    pub fn new(method: InputMethod) -> Self {
        Self::with_options(method, EngineOptions::default())
    }

    pub fn with_options(method: InputMethod, options: EngineOptions) -> Self {
        Self {
            method,
            buffer: Vec::new(),
            raw: Vec::new(),
            enabled: true,
            options,
        }
    }

    pub fn method(&self) -> InputMethod {
        self.method
    }

    pub fn set_method(&mut self, method: InputMethod) {
        self.method = method;
        self.reset();
    }

    pub fn options_mut(&mut self) -> &mut EngineOptions {
        &mut self.options
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset();
        }
    }

    pub fn toggle(&mut self) -> bool {
        self.enabled = !self.enabled;
        if !self.enabled {
            self.reset();
        }
        self.enabled
    }

    /// Current preedit text.
    pub fn preedit(&self) -> String {
        self.buffer.iter().collect()
    }

    /// Raw key sequence for the current composition.
    pub fn raw_text(&self) -> String {
        self.raw.iter().collect()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.raw.clear();
    }

    /// Take and clear current composition (commit), applying English restore if needed.
    pub fn commit_text(&mut self) -> String {
        let composed = self.preedit();
        let raw: String = self.raw.iter().collect();
        self.reset();
        if self.options.auto_restore_english
            && crate::validate::should_restore_english(&composed, &raw)
        {
            return raw;
        }
        composed
    }

    /// Transform a whole string offline (for CLI / batch).
    pub fn transform(method: InputMethod, input: &str) -> String {
        let mut eng = Self::new(method);
        let mut out = String::new();
        for ch in input.chars() {
            match eng.feed(ch) {
                KeyResult::Update | KeyResult::Append | KeyResult::Backspace => {}
                KeyResult::Commit => {
                    // Separator committed the word; append separator itself.
                    out.push_str(&eng.commit_text());
                    out.push(ch);
                }
                KeyResult::CommitAndPass => {
                    out.push_str(&eng.commit_text());
                    // Try to start a new composition with this key (e.g. letter after flush).
                    match eng.feed(ch) {
                        KeyResult::Update | KeyResult::Append | KeyResult::Backspace => {}
                        KeyResult::Commit => {
                            out.push_str(&eng.commit_text());
                            out.push(ch);
                        }
                        KeyResult::CommitAndPass | KeyResult::Ignored => out.push(ch),
                    }
                }
                KeyResult::Ignored => out.push(ch),
            }
        }
        out.push_str(&eng.commit_text());
        out
    }

    /// Feed one character (letter, digit, tone key, space, …).
    pub fn feed(&mut self, ch: char) -> KeyResult {
        if !self.enabled {
            return KeyResult::Ignored;
        }

        if ch == '\u{8}' || ch == '\x7f' {
            return self.backspace();
        }

        // Telex shortcuts must not be treated as punctuation separators.
        let telex_shortcut = matches!(self.method, InputMethod::Telex)
            && matches!(ch, '[' | ']' | '{' | '}');

        // Word separators → commit
        if !telex_shortcut
            && (ch == ' ' || ch == '\n' || ch == '\t' || is_word_separator(ch))
        {
            if self.buffer.is_empty() {
                return KeyResult::CommitAndPass;
            }
            return KeyResult::Commit;
        }

        match self.method {
            InputMethod::Telex => self.feed_telex(ch),
            InputMethod::Vni => self.feed_vni(ch),
        }
    }

    pub fn backspace(&mut self) -> KeyResult {
        if self.buffer.is_empty() {
            return KeyResult::CommitAndPass;
        }
        self.buffer.pop();
        self.raw.pop();
        KeyResult::Backspace
    }

    fn feed_telex(&mut self, ch: char) -> KeyResult {
        let lower = ch.to_ascii_lowercase();

        // Tone keys
        if let Some(tone) = Tone::from_telex(lower) {
            if self.buffer.is_empty() {
                self.push_raw(ch);
                return KeyResult::Append;
            }
            // z with no tone → append z
            if lower == 'z' {
                if !self.has_any_tone() {
                    self.push_raw(ch);
                    return KeyResult::Append;
                }
            }
            return self.apply_tone_key(tone, ch);
        }

        // dd → đ
        if lower == 'd' {
            if let Some(last) = self.buffer.last().copied() {
                if is_d_family(last) && last.to_ascii_lowercase() == 'd' {
                    // undo if already đ? second d on d → đ; third → dd
                    if matches!(last, 'd' | 'D') {
                        if let Some(mapped) = apply_d_stroke(last) {
                            *self.buffer.last_mut().unwrap() = mapped;
                            self.raw.push(ch);
                            return KeyResult::Update;
                        }
                    }
                } else if matches!(last, 'đ' | 'Đ') {
                    // third d: undo to dd
                    if let Some(plain) = undo_d_stroke(last) {
                        *self.buffer.last_mut().unwrap() = plain;
                        self.buffer.push(ch);
                        self.raw.push(ch);
                        return KeyResult::Update;
                    }
                }
            }
            self.push_raw(ch);
            return KeyResult::Append;
        }

        // Double vowel circumflex: aa ee oo — only if the *last* char is that vowel
        // (so "Code"+"Key" does not turn the first `e` into `ê`).
        if matches!(lower, 'a' | 'e' | 'o') {
            if let Some(idx) = self.last_char_vowel_matching(lower) {
                let cur = self.buffer[idx];
                let (base, _tone) = strip_tone(cur);
                let base_l = base.to_ascii_lowercase();
                let plain = match base_l {
                    'â' => 'a',
                    'ê' => 'e',
                    'ô' => 'o',
                    x => x,
                };
                if plain == lower {
                    if is_circumflex_base(base_l) {
                        if let Some(restored) = remove_circumflex(cur) {
                            self.buffer[idx] = restored;
                            self.buffer.push(ch);
                            self.raw.push(ch);
                            return KeyResult::Update;
                        }
                    } else if let Some(mapped) = apply_diacritic(cur, Diacritic::Circumflex) {
                        self.buffer[idx] = mapped;
                        self.raw.push(ch);
                        return KeyResult::Update;
                    }
                }
            }
        }

        // aw → ă, ow → ơ, uw → ư, w alone → ư
        if lower == 'w' {
            if let Some(idx) = self.find_horn_or_breve_target() {
                let cur = self.buffer[idx];
                let (base, _) = strip_tone(cur);
                let bl = base.to_ascii_lowercase();
                // undo if already ă/ơ/ư
                if matches!(bl, 'ă' | 'ơ' | 'ư') {
                    if let Some(plain) = remove_horn_breve(cur) {
                        self.buffer[idx] = plain;
                        self.buffer.push(ch);
                        self.raw.push(ch);
                        return KeyResult::Update;
                    }
                }
                let kind = if bl == 'a' || bl == 'ă' {
                    Diacritic::Breve
                } else {
                    Diacritic::Horn
                };
                if let Some(mapped) = apply_diacritic(cur, kind) {
                    self.buffer[idx] = mapped;
                    self.raw.push(ch);
                    return KeyResult::Update;
                }
            }
            // no target: insert ư
            let u = if ch.is_uppercase() { 'Ư' } else { 'ư' };
            self.buffer.push(u);
            self.raw.push(ch);
            return KeyResult::Append;
        }

        // [ → ư, ] → ơ (UniKey shortcuts)
        if ch == '[' || ch == '{' {
            let u = if ch == '{' { 'Ư' } else { 'ư' };
            self.buffer.push(u);
            self.raw.push(ch);
            return KeyResult::Append;
        }
        if ch == ']' || ch == '}' {
            let o = if ch == '}' { 'Ơ' } else { 'ơ' };
            self.buffer.push(o);
            self.raw.push(ch);
            return KeyResult::Append;
        }

        // Normal letter
        if ch.is_alphabetic() || is_vowel(ch) || is_d_family(ch) {
            self.push_raw(ch);
            return KeyResult::Append;
        }

        // Unknown: commit and pass
        if !self.buffer.is_empty() {
            KeyResult::CommitAndPass
        } else {
            KeyResult::CommitAndPass
        }
    }

    fn feed_vni(&mut self, ch: char) -> KeyResult {
        // Tones 0-5
        if let Some(tone) = Tone::from_vni(ch) {
            if self.buffer.is_empty() {
                self.push_raw(ch);
                return KeyResult::Append;
            }
            return self.apply_tone_key(tone, ch);
        }

        // Diacritics 6-9
        match ch {
            '6' => {
                // circumflex on a/e/o
                if let Some(idx) = self.find_vni_circumflex_target() {
                    let cur = self.buffer[idx];
                    let (base, _) = strip_tone(cur);
                    let bl = base.to_ascii_lowercase();
                    if matches!(bl, 'â' | 'ê' | 'ô') {
                        if let Some(plain) = remove_circumflex(cur) {
                            self.buffer[idx] = plain;
                            self.buffer.push(ch);
                            self.raw.push(ch);
                            return KeyResult::Update;
                        }
                    }
                    if let Some(mapped) = apply_diacritic(cur, Diacritic::Circumflex) {
                        self.buffer[idx] = mapped;
                        self.raw.push(ch);
                        return KeyResult::Update;
                    }
                }
                self.push_raw(ch);
                KeyResult::Append
            }
            '7' => {
                // horn on o/u
                if let Some(idx) = self.find_vni_horn_target() {
                    let cur = self.buffer[idx];
                    let (base, _) = strip_tone(cur);
                    let bl = base.to_ascii_lowercase();
                    if matches!(bl, 'ơ' | 'ư') {
                        if let Some(plain) = remove_horn_breve(cur) {
                            self.buffer[idx] = plain;
                            self.buffer.push(ch);
                            self.raw.push(ch);
                            return KeyResult::Update;
                        }
                    }
                    if let Some(mapped) = apply_diacritic(cur, Diacritic::Horn) {
                        self.buffer[idx] = mapped;
                        self.raw.push(ch);
                        return KeyResult::Update;
                    }
                }
                self.push_raw(ch);
                KeyResult::Append
            }
            '8' => {
                // breve on a
                if let Some(idx) = self.find_last_vowel_matching('a') {
                    let cur = self.buffer[idx];
                    let (base, _) = strip_tone(cur);
                    if base.to_ascii_lowercase() == 'ă' {
                        if let Some(plain) = remove_horn_breve(cur) {
                            self.buffer[idx] = plain;
                            self.buffer.push(ch);
                            self.raw.push(ch);
                            return KeyResult::Update;
                        }
                    }
                    if let Some(mapped) = apply_diacritic(cur, Diacritic::Breve) {
                        self.buffer[idx] = mapped;
                        self.raw.push(ch);
                        return KeyResult::Update;
                    }
                }
                self.push_raw(ch);
                KeyResult::Append
            }
            '9' => {
                // d → đ
                if let Some(last) = self.buffer.last().copied() {
                    if matches!(last, 'd' | 'D') {
                        if let Some(mapped) = apply_d_stroke(last) {
                            *self.buffer.last_mut().unwrap() = mapped;
                            self.raw.push(ch);
                            return KeyResult::Update;
                        }
                    } else if matches!(last, 'đ' | 'Đ') {
                        if let Some(plain) = undo_d_stroke(last) {
                            *self.buffer.last_mut().unwrap() = plain;
                            self.buffer.push(ch);
                            self.raw.push(ch);
                            return KeyResult::Update;
                        }
                    }
                }
                self.push_raw(ch);
                KeyResult::Append
            }
            c if c.is_alphabetic() || is_vowel(c) || is_d_family(c) => {
                self.push_raw(c);
                KeyResult::Append
            }
            _ => {
                if !self.buffer.is_empty() {
                    KeyResult::CommitAndPass
                } else {
                    KeyResult::CommitAndPass
                }
            }
        }
    }

    fn apply_tone_key(&mut self, tone: Tone, raw_key: char) -> KeyResult {
        // UniKey-style: uoi / uô → ươi before placing tone (nguoif → người).
        self.expand_uo_to_uow();

        let Some(idx) = tone_target_index(&self.buffer) else {
            self.push_raw(raw_key);
            return KeyResult::Append;
        };

        let cur = self.buffer[idx];
        let (base, existing) = strip_tone(cur);

        // Same tone again → undo (restore base, append key)
        if existing == tone && tone != Tone::None {
            self.buffer[idx] = base;
            self.buffer.push(raw_key);
            self.raw.push(raw_key);
            return KeyResult::Update;
        }

        // z / 0 removes tone
        if tone == Tone::None {
            if existing == Tone::None {
                self.push_raw(raw_key);
                return KeyResult::Append;
            }
            self.buffer[idx] = base;
            self.raw.push(raw_key);
            return KeyResult::Update;
        }

        if let Some(mapped) = apply_tone(base, tone) {
            // preserve case of `cur`
            let mapped = if cur.is_uppercase() || is_upper_viet(cur) {
                crate_upper(mapped)
            } else {
                mapped
            };
            self.buffer[idx] = mapped;
            self.raw.push(raw_key);
            KeyResult::Update
        } else {
            self.push_raw(raw_key);
            KeyResult::Append
        }
    }

    fn push_raw(&mut self, ch: char) {
        self.buffer.push(ch);
        self.raw.push(ch);
    }

    fn has_any_tone(&self) -> bool {
        self.buffer.iter().any(|&c| strip_tone(c).1 != Tone::None)
    }

    fn find_last_vowel_matching(&self, ascii_vowel: char) -> Option<usize> {
        self.buffer.iter().enumerate().rev().find_map(|(i, &c)| {
            let (base, _) = strip_tone(c);
            let b = base.to_ascii_lowercase();
            let plain = match b {
                'â' | 'ă' => 'a',
                'ê' => 'e',
                'ô' | 'ơ' => 'o',
                'ư' => 'u',
                x => x,
            };
            if plain == ascii_vowel || b == ascii_vowel {
                Some(i)
            } else {
                None
            }
        })
    }

    /// Like `find_last_vowel_matching`, but only if that vowel is the final character.
    fn last_char_vowel_matching(&self, ascii_vowel: char) -> Option<usize> {
        let idx = self.buffer.len().checked_sub(1)?;
        let c = self.buffer[idx];
        let (base, _) = strip_tone(c);
        let b = base.to_ascii_lowercase();
        let plain = match b {
            'â' | 'ă' => 'a',
            'ê' => 'e',
            'ô' | 'ơ' => 'o',
            'ư' => 'u',
            x => x,
        };
        if plain == ascii_vowel || b == ascii_vowel {
            Some(idx)
        } else {
            None
        }
    }

    fn find_horn_or_breve_target(&self) -> Option<usize> {
        // Prefer last a/o/u (including already marked for undo)
        self.buffer.iter().enumerate().rev().find_map(|(i, &c)| {
            let (base, _) = strip_tone(c);
            let b = base.to_ascii_lowercase();
            if matches!(b, 'a' | 'ă' | 'o' | 'ô' | 'ơ' | 'u' | 'ư') {
                Some(i)
            } else {
                None
            }
        })
    }

    fn find_vni_circumflex_target(&self) -> Option<usize> {
        self.buffer.iter().enumerate().rev().find_map(|(i, &c)| {
            let (base, _) = strip_tone(c);
            let b = base.to_ascii_lowercase();
            if matches!(b, 'a' | 'â' | 'e' | 'ê' | 'o' | 'ô') {
                Some(i)
            } else {
                None
            }
        })
    }

    fn find_vni_horn_target(&self) -> Option<usize> {
        self.buffer.iter().enumerate().rev().find_map(|(i, &c)| {
            let (base, _) = strip_tone(c);
            let b = base.to_ascii_lowercase();
            if matches!(b, 'o' | 'ô' | 'ơ' | 'u' | 'ư') {
                Some(i)
            } else {
                None
            }
        })
    }

    /// Convert nucleus `uo` / `uô` → `ươ` when followed by `i/c/ng/t/p/m/n` etc.
    /// Matches common UniKey behaviour for words like `người`, `tưởng`, `cuốn`→ still uô.
    /// We only auto-expand the classic `uoi` → `ươi` and `uon/uong/uoc/uot` patterns
    /// that Vietnamese orthography writes with ươ.
    fn expand_uo_to_uow(&mut self) {
        let chars = &self.buffer;
        if chars.len() < 2 {
            return;
        }
        // Find last 'u' then 'o'/'ô' sequence
        let mut u_idx = None;
        let mut o_idx = None;
        for (i, &c) in chars.iter().enumerate() {
            let (base, _) = strip_tone(c);
            let b = base.to_ascii_lowercase();
            if b == 'u' || b == 'ư' {
                u_idx = Some(i);
                o_idx = None;
            } else if matches!(b, 'o' | 'ô' | 'ơ') && u_idx.is_some() && o_idx.is_none() {
                o_idx = Some(i);
            }
        }
        let (Some(ui), Some(oi)) = (u_idx, o_idx) else {
            return;
        };
        if oi != ui + 1 {
            return;
        }
        let (ub, _) = strip_tone(chars[ui]);
        let (ob, _) = strip_tone(chars[oi]);
        if ub.to_ascii_lowercase() == 'ư' && ob.to_ascii_lowercase() == 'ơ' {
            return; // already expanded
        }
        if ub.to_ascii_lowercase() != 'u' {
            return;
        }
        if !matches!(ob.to_ascii_lowercase(), 'o' | 'ô') {
            return;
        }

        // Look at rest after o: i, c, t, p, m, n, ng, nh → ươ; else keep (e.g. uôn rare)
        let rest: String = chars[oi + 1..]
            .iter()
            .map(|c| strip_tone(*c).0.to_ascii_lowercase())
            .collect();
        let should = rest.is_empty()
            || rest.starts_with('i')
            || rest.starts_with('c')
            || rest.starts_with('t')
            || rest.starts_with('p')
            || rest.starts_with('m')
            || rest.starts_with("ng")
            || rest.starts_with('n')
            || rest.starts_with("nh");
        // `uo` + vowel a/e stays as uo (qua, …) — rest empty only if tone on bare uo; skip empty bare
        let should = if rest.is_empty() {
            false
        } else {
            should
        };
        // Always expand uoi
        let should = should || rest.starts_with('i');

        if !should {
            return;
        }

        let u_ch = chars[ui];
        let o_ch = chars[oi];
        let u_tone = strip_tone(u_ch).1;
        let o_tone = strip_tone(o_ch).1;
        let new_u = if u_ch.is_uppercase() || is_upper_viet(u_ch) {
            'Ư'
        } else {
            'ư'
        };
        let new_o = if o_ch.is_uppercase() || is_upper_viet(o_ch) {
            'Ơ'
        } else {
            'ơ'
        };
        self.buffer[ui] = apply_tone(new_u, u_tone).unwrap_or(new_u);
        self.buffer[oi] = apply_tone(new_o, o_tone).unwrap_or(new_o);
    }
}

fn is_circumflex_base(b: char) -> bool {
    matches!(b, 'â' | 'ê' | 'ô')
}

fn remove_circumflex(ch: char) -> Option<char> {
    let (base, tone) = strip_tone(ch);
    let plain = match base.to_ascii_lowercase() {
        'â' => 'a',
        'ê' => 'e',
        'ô' => 'o',
        _ => return None,
    };
    let plain = if base.is_uppercase() || is_upper_viet(base) {
        plain.to_ascii_uppercase()
    } else {
        plain
    };
    apply_tone(plain, tone)
}

fn remove_horn_breve(ch: char) -> Option<char> {
    let (base, tone) = strip_tone(ch);
    let plain = match base.to_ascii_lowercase() {
        'ă' => 'a',
        'ơ' => 'o',
        'ư' => 'u',
        _ => return None,
    };
    let plain = if is_upper_viet(base) || base.is_uppercase() {
        plain.to_ascii_uppercase()
    } else {
        plain
    };
    apply_tone(plain, tone)
}

fn is_word_separator(ch: char) -> bool {
    matches!(
        ch,
        '.' | ','
            | ';'
            | ':'
            | '!'
            | '?'
            | '"'
            | '\''
            | '('
            | ')'
            | '<'
            | '>'
            | '/'
            | '\\'
            | '|'
            | '@'
            | '#'
            | '$'
            | '%'
            | '^'
            | '&'
            | '*'
            | '-'
            | '+'
            | '='
            | '_'
            | '`'
            | '~'
    )
}

fn is_upper_viet(c: char) -> bool {
    // Do NOT use char ranges like 'Á'..='Ỵ' — they also cover lowercase Vietnamese.
    c.is_uppercase()
}

fn crate_upper(c: char) -> char {
    match c {
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
        c => c.to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn telex(s: &str) -> String {
        Engine::transform(InputMethod::Telex, s)
    }

    fn vni(s: &str) -> String {
        Engine::transform(InputMethod::Vni, s)
    }

    #[test]
    fn telex_basic_words() {
        assert_eq!(telex("Vieejt"), "Việt");
        assert_eq!(telex("xin chaof"), "xin chào");
        assert_eq!(telex("ddung"), "đung");
        assert_eq!(telex("ddungs"), "đúng");
        assert_eq!(telex("nguoif"), "người");
        assert_eq!(telex("Tuwf"), "Từ");
        assert_eq!(telex("Tuwr"), "Tử");
        assert_eq!(telex("hoaf"), "hoà");
    }

    #[test]
    fn telex_tones_and_marks() {
        assert_eq!(telex("as"), "á");
        assert_eq!(telex("af"), "à");
        assert_eq!(telex("ar"), "ả");
        assert_eq!(telex("ax"), "ã");
        assert_eq!(telex("aj"), "ạ");
        assert_eq!(telex("aa"), "â");
        assert_eq!(telex("aw"), "ă");
        assert_eq!(telex("ee"), "ê");
        assert_eq!(telex("oo"), "ô");
        assert_eq!(telex("ow"), "ơ");
        assert_eq!(telex("uw"), "ư");
        assert_eq!(telex("dd"), "đ");
    }

    #[test]
    fn telex_undo() {
        assert_eq!(telex("ass"), "as");
        assert_eq!(telex("aaa"), "aa");
        assert_eq!(telex("ddd"), "dd");
    }

    #[test]
    fn vni_basic() {
        assert_eq!(vni("Viet65"), "Việt");
        assert_eq!(vni("xin chao2"), "xin chào");
        assert_eq!(vni("d9ung1"), "đúng");
        assert_eq!(vni("a6"), "â");
        assert_eq!(vni("a8"), "ă");
        assert_eq!(vni("o7"), "ơ");
        assert_eq!(vni("u7"), "ư");
    }

    #[test]
    fn sentence() {
        assert_eq!(
            telex("Toi laf nguoif Vieejt Nam"),
            "Toi là người Việt Nam"
        );
    }

    #[test]
    fn english_names_not_mangled() {
        assert_eq!(telex("CodeKey"), "CodeKey");
        assert_eq!(telex("Facebook"), "Facebook");
        // Real VN still works in the same sentence
        assert_eq!(
            telex("dungf CodeKey owr Vieejt Nam"),
            "dùng CodeKey ở Việt Nam"
        );
    }

    #[test]
    fn veet_still_circumflex() {
        assert_eq!(telex("veet"), "vêt");
        assert_eq!(telex("Vieejt"), "Việt");
    }

    #[test]
    fn tone_on_ua_ia_diphthongs() {
        // của — dấu hỏi trên u, không phải a
        assert_eq!(telex("cuar"), "của");
        assert_eq!(telex("cuas"), "cúa");
        assert_eq!(telex("cuaf"), "cùa");
        assert_eq!(telex("muas"), "múa");
        // quả — sau qu, dấu trên a
        assert_eq!(telex("quar"), "quả");
        assert_eq!(telex("quas"), "quá");
        // tía — dấu trên i
        assert_eq!(telex("tias"), "tía");
        // giá — sau gi, dấu trên a
        assert_eq!(telex("gias"), "giá");
        // hoà — dấu trên a
        assert_eq!(telex("hoaf"), "hoà");
        // thủy
        assert_eq!(telex("thuyr"), "thuỷ");
    }
}
