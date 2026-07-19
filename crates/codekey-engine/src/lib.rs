//! CodeKey composition engine — Telex / VNI → Vietnamese Unicode.
//!
//! Pure logic, no I/O. Used by CLI, IBus, Fcitx5 (via FFI), and tests.

mod charset;
mod engine;
mod method;
mod syllable;
mod tables;
mod tone;
mod validate;

pub use engine::{Engine, EngineOptions, KeyResult};
pub use method::InputMethod;
pub use tone::Tone;
pub use validate::{is_plausible_vietnamese, should_restore_english};

/// Crate version string.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
