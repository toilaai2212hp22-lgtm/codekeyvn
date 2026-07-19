//! C FFI for the CodeKey Vietnamese composition engine.
//!
//! Used by the Fcitx5 C++ addon (`fcitx5-addon/`).

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

use codekey_engine::{Engine, InputMethod, KeyResult};

/// Opaque engine handle for C.
pub struct CodeKeyEngine {
    inner: Engine,
}

fn method_from_c(m: c_int) -> InputMethod {
    match m {
        1 => InputMethod::Vni,
        _ => InputMethod::Telex,
    }
}

/// Create engine. `method`: 0=Telex, 1=VNI.
#[no_mangle]
pub extern "C" fn codekey_engine_new(method: c_int) -> *mut CodeKeyEngine {
    Box::into_raw(Box::new(CodeKeyEngine {
        inner: Engine::new(method_from_c(method)),
    }))
}

/// Free engine.
#[no_mangle]
pub unsafe extern "C" fn codekey_engine_free(eng: *mut CodeKeyEngine) {
    if !eng.is_null() {
        drop(Box::from_raw(eng));
    }
}

/// Free a string returned by this library.
#[no_mangle]
pub unsafe extern "C" fn codekey_string_free(s: *mut c_char) {
    if !s.is_null() {
        drop(CString::from_raw(s));
    }
}

/// Feed one Unicode code point. Returns KeyResult code:
/// 0=Update, 1=Append, 2=CommitAndPass, 3=Commit, 4=Backspace, 5=Ignored
#[no_mangle]
pub unsafe extern "C" fn codekey_engine_feed(eng: *mut CodeKeyEngine, ch: u32) -> c_int {
    if eng.is_null() {
        return 5;
    }
    let Some(c) = char::from_u32(ch) else {
        return 5;
    };
    match (*eng).inner.feed(c) {
        KeyResult::Update => 0,
        KeyResult::Append => 1,
        KeyResult::CommitAndPass => 2,
        KeyResult::Commit => 3,
        KeyResult::Backspace => 4,
        KeyResult::Ignored => 5,
    }
}

#[no_mangle]
pub unsafe extern "C" fn codekey_engine_backspace(eng: *mut CodeKeyEngine) -> c_int {
    if eng.is_null() {
        return 2;
    }
    match (*eng).inner.backspace() {
        KeyResult::Backspace => 4,
        KeyResult::CommitAndPass => 2,
        KeyResult::Update => 0,
        KeyResult::Append => 1,
        KeyResult::Commit => 3,
        KeyResult::Ignored => 5,
    }
}

/// Current preedit (caller must `codekey_string_free`).
#[no_mangle]
pub unsafe extern "C" fn codekey_engine_preedit(eng: *const CodeKeyEngine) -> *mut c_char {
    if eng.is_null() {
        return ptr::null_mut();
    }
    str_to_c(&(*eng).inner.preedit())
}

/// Commit and clear; returns committed text (free with `codekey_string_free`).
#[no_mangle]
pub unsafe extern "C" fn codekey_engine_commit(eng: *mut CodeKeyEngine) -> *mut c_char {
    if eng.is_null() {
        return ptr::null_mut();
    }
    str_to_c(&(*eng).inner.commit_text())
}

#[no_mangle]
pub unsafe extern "C" fn codekey_engine_reset(eng: *mut CodeKeyEngine) {
    if !eng.is_null() {
        (*eng).inner.reset();
    }
}

#[no_mangle]
pub unsafe extern "C" fn codekey_engine_set_enabled(eng: *mut CodeKeyEngine, enabled: c_int) {
    if !eng.is_null() {
        (*eng).inner.set_enabled(enabled != 0);
    }
}

#[no_mangle]
pub unsafe extern "C" fn codekey_engine_is_enabled(eng: *const CodeKeyEngine) -> c_int {
    if eng.is_null() {
        return 0;
    }
    (*eng).inner.is_enabled() as c_int
}

/// Offline transform. Free result with `codekey_string_free`.
#[no_mangle]
pub unsafe extern "C" fn codekey_transform(method: c_int, input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return ptr::null_mut();
    }
    let s = match CStr::from_ptr(input).to_str() {
        Ok(s) => s,
        Err(_) => return ptr::null_mut(),
    };
    let out = Engine::transform(method_from_c(method), s);
    str_to_c(&out)
}

fn str_to_c(s: &str) -> *mut c_char {
    match CString::new(s) {
        Ok(c) => c.into_raw(),
        Err(_) => ptr::null_mut(),
    }
}
