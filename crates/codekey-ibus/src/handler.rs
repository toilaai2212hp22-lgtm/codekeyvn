//! Vietnamese composition handler for one IBus input context.

use codekey_engine::{Engine, InputMethod, KeyResult};

use crate::keysyms::{self, BACKSPACE, RELEASE_MASK};
use crate::types::IBusText;

/// Outgoing signals / actions for the IBus engine interface.
#[derive(Debug, Clone)]
pub enum Action {
    CommitText(String),
    UpdatePreedit {
        text: String,
        cursor: u32,
        visible: bool,
    },
    HidePreedit,
    ForwardKey {
        keyval: u32,
        keycode: u32,
        state: u32,
    },
}

pub struct CodeKeyHandler {
    engine: Engine,
}

impl CodeKeyHandler {
    pub fn new(method: InputMethod) -> Self {
        Self {
            engine: Engine::new(method),
        }
    }

    pub fn from_engine_name(name: &str) -> Self {
        let method = if name.to_ascii_lowercase().contains("vni") {
            InputMethod::Vni
        } else {
            InputMethod::Telex
        };
        Self::new(method)
    }

    pub fn process_key_event(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> (bool, Vec<Action>) {
        // Ignore key releases and pure modifiers.
        if state & RELEASE_MASK != 0 || keysyms::is_modifier(keyval) {
            return (false, Vec::new());
        }

        // Ctrl/Alt/Super combos → let the app handle (except we may commit first).
        if state & (keysyms::CONTROL_MASK | keysyms::MOD1_MASK | keysyms::MOD4_MASK) != 0 {
            if !self.engine.is_empty() {
                let mut actions = self.commit_preedit();
                actions.push(Action::ForwardKey {
                    keyval,
                    keycode,
                    state,
                });
                return (true, actions);
            }
            return (false, Vec::new());
        }

        if keyval == BACKSPACE {
            return match self.engine.backspace() {
                KeyResult::Backspace => (true, self.preedit_actions()),
                KeyResult::CommitAndPass => (false, Vec::new()),
                _ => (false, Vec::new()),
            };
        }

        let Some(ch) = keysyms::keyval_to_char(keyval) else {
            if !self.engine.is_empty() {
                let mut actions = self.commit_preedit();
                actions.push(Action::ForwardKey {
                    keyval,
                    keycode,
                    state,
                });
                return (true, actions);
            }
            return (false, Vec::new());
        };

        match self.engine.feed(ch) {
            KeyResult::Update | KeyResult::Append => (true, self.preedit_actions()),
            KeyResult::Backspace => (true, self.preedit_actions()),
            KeyResult::Commit => {
                let mut actions = self.commit_preedit();
                // Commit the separator itself
                actions.push(Action::CommitText(ch.to_string()));
                (true, actions)
            }
            KeyResult::CommitAndPass => {
                if self.engine.is_empty() {
                    // nothing composing — do not consume
                    (false, Vec::new())
                } else {
                    let mut actions = self.commit_preedit();
                    actions.push(Action::ForwardKey {
                        keyval,
                        keycode,
                        state,
                    });
                    (true, actions)
                }
            }
            KeyResult::Ignored => (false, Vec::new()),
        }
    }

    pub fn reset(&mut self) -> Vec<Action> {
        if self.engine.is_empty() {
            return Vec::new();
        }
        self.engine.reset();
        vec![Action::HidePreedit]
    }

    pub fn focus_out(&mut self) -> Vec<Action> {
        self.commit_preedit()
    }

    fn preedit_actions(&self) -> Vec<Action> {
        let text = self.engine.preedit();
        if text.is_empty() {
            vec![Action::HidePreedit]
        } else {
            let cursor = text.chars().count() as u32;
            vec![Action::UpdatePreedit {
                text,
                cursor,
                visible: true,
            }]
        }
    }

    fn commit_preedit(&mut self) -> Vec<Action> {
        let text = self.engine.commit_text();
        if text.is_empty() {
            vec![Action::HidePreedit]
        } else {
            vec![Action::CommitText(text), Action::HidePreedit]
        }
    }
}

/// Helper used by engine interface to build IBusText.
pub fn preedit_text(s: &str) -> IBusText {
    IBusText::underlined(s)
}

pub fn commit_text(s: &str) -> IBusText {
    IBusText::plain(s)
}
