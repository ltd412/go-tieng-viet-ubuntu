//! Per-input-context typing state machine. Knows nothing about D-Bus: key events go in, a list
//! of [`Out`] actions comes out, and the frontend turns those into IBus signals.

use gtv_core::{diff, Composer, Config, Push};
use serde::Deserialize;

pub const SHIFT_MASK: u32 = 1 << 0;
const CONTROL_MASK: u32 = 1 << 2;
const MOD1_MASK: u32 = 1 << 3;
const MOD4_MASK: u32 = 1 << 6;
/// Set by IBus on key events the engine itself forwarded; they must not be processed again.
const FORWARDED_MASK: u32 = 1 << 25;
const SUPER_MASK: u32 = 1 << 26;
const HYPER_MASK: u32 = 1 << 27;
const META_MASK: u32 = 1 << 28;
pub const RELEASE_MASK: u32 = 1 << 30;
const SHORTCUT_MASK: u32 =
    CONTROL_MASK | MOD1_MASK | MOD4_MASK | SUPER_MASK | HYPER_MASK | META_MASK;

const KEY_BACKSPACE: u32 = 0xff08;
const KEY_TAB: u32 = 0xff09;
const KEY_RETURN: u32 = 0xff0d;
const KEY_ESCAPE: u32 = 0xff1b;
const KEY_LEFT: u32 = 0xff51;
const KEY_DELETE: u32 = 0xffff;
const KEY_KP_ENTER: u32 = 0xff8d;
const KEYCODE_BACKSPACE: u32 = 14;
const KEYCODE_LEFT: u32 = 105;
const KEYCODE_DELETE: u32 = 111;
const KEYCODE_TAB: u32 = 15;

/// How a rewrite of the word already on screen reaches the application.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    /// DeleteSurroundingText + CommitText. Cleanest, needs client support.
    Surrounding,
    /// Forwarded BackSpace key events, then CommitText.
    Backspace,
    /// Forwarded Shift+Left to select the old text, then CommitText replaces the selection.
    /// Immune to browser address-bar autocomplete eating the backspace.
    Select,
    /// Backspaces and the replacement text all travel as forwarded key events, so the whole
    /// edit is one ordered stream.
    ForwardKeys,
    /// Classic underlined preedit; always safe, never invisible.
    Preedit,
    /// Vietnamese typing off for this app: every key goes through untouched.
    Off,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KeyEvent {
    pub keyval: u32,
    pub keycode: u32,
    pub state: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    Commit(String),
    DeleteSurrounding(usize),
    Key(KeyEvent),
    /// Let forwarded keys reach the application before the text that follows them. Some
    /// toolkits (Chrome) process forwarded keys later than a commit sent right after.
    Pause,
    /// `None` hides the preedit.
    Preedit(Option<String>),
    /// Frontend-only: ask the client to start sending surrounding text.
    RequireSurroundingText,
}

pub struct Session {
    composer: Composer,
    /// What the word currently looks like in the application (not used by `Preedit`).
    screen: String,
    mode: Mode,
    swallow_release: Option<u32>,
    /// The caret sits in a browser address bar, where the text after it may be a selected inline
    /// autocomplete suggestion that the first BackSpace would eat instead of the typed letter.
    clear_suggestion: bool,
}

impl Session {
    pub fn new(cfg: Config, mode: Mode) -> Self {
        Session {
            composer: Composer::new(cfg),
            screen: String::new(),
            mode,
            swallow_release: None,
            clear_suggestion: false,
        }
    }

    /// No word is being composed, so the mode can change without losing text.
    pub fn is_idle(&self) -> bool {
        self.composer.is_empty()
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Marks a key whose press the frontend consumed after the fact (it forwarded the key
    /// itself), so its release is swallowed too.
    pub fn consume_release_of(&mut self, keyval: u32) {
        self.swallow_release = Some(keyval);
    }

    pub fn set_clear_suggestion(&mut self, on: bool) {
        self.clear_suggestion = on;
    }

    /// Changes how edits are delivered; call at focus-in, never mid-word.
    pub fn set_mode(&mut self, mode: Mode) -> Vec<Out> {
        let out = self.reset();
        self.mode = mode;
        out
    }

    /// Drops the word in progress (focus change, mouse click). A preedit is committed so no
    /// typed text is lost; other modes already have the text in the application.
    pub fn reset(&mut self) -> Vec<Out> {
        let mut out = Vec::new();
        self.end_word(&mut out, false);
        self.swallow_release = None;
        out
    }

    /// Returns whether the key was consumed, and what to send to the application.
    pub fn process_key(&mut self, ev: KeyEvent) -> (bool, Vec<Out>) {
        let mut out = Vec::new();
        let handled = self.key(ev, &mut out);
        if handled && ev.state & RELEASE_MASK == 0 {
            self.swallow_release = Some(ev.keyval);
        }
        (handled, out)
    }

    fn key(&mut self, ev: KeyEvent, out: &mut Vec<Out>) -> bool {
        if ev.state & RELEASE_MASK != 0 {
            return self.swallow_release.take_if(|k| *k == ev.keyval).is_some();
        }
        if self.mode == Mode::Off || ev.state & FORWARDED_MASK != 0 || is_modifier(ev.keyval) {
            return false;
        }
        if ev.state & SHORTCUT_MASK != 0 {
            self.end_word(out, true);
            return false;
        }
        match ev.keyval {
            KEY_BACKSPACE => return self.backspace(out),
            KEY_TAB if self.clear_suggestion => {
                // Address bar: a Tab that went through the input method no longer accepts Chrome's
                // inline suggestion; the same key forwarded by the engine does.
                self.end_word(out, true);
                push_key(out, KEY_TAB, KEYCODE_TAB, 0);
                return true;
            }
            KEY_RETURN | KEY_KP_ENTER | KEY_TAB | KEY_ESCAPE => {
                self.end_word(out, true);
                return false;
            }
            _ => {}
        }
        let Some(c) = keyval_char(ev.keyval) else {
            // Arrows, Home, Delete, F-keys...: the cursor may leave the word.
            self.end_word(out, false);
            return false;
        };
        match self.composer.push(c) {
            Push::Boundary => {
                self.end_word(out, true);
                // Not passed through: a native key event is queued by the toolkit and would be
                // applied after signals we send for the next key.
                self.send_text(&c.to_string(), out);
                true
            }
            Push::Accepted => self.update(c, out),
        }
    }

    fn update(&mut self, typed: char, out: &mut Vec<Out>) -> bool {
        let new = self.composer.display().to_string();
        log::trace!("typed {typed:?} screen={:?} display={new:?}", self.screen);
        if self.mode == Mode::Preedit {
            out.push(Out::Preedit(Some(new)));
            return true;
        }
        let mut appended = String::with_capacity(self.screen.len() + typed.len_utf8());
        appended.push_str(&self.screen);
        appended.push(typed);
        if new == appended {
            self.send_text(&typed.to_string(), out);
        } else if new != self.screen {
            let (delete, insert) = diff(&self.screen, &new);
            self.rewrite(delete, &insert, out);
        }
        self.screen = new;
        true
    }

    fn backspace(&mut self, out: &mut Vec<Out>) -> bool {
        if self.composer.is_empty() {
            return false;
        }
        self.composer.backspace();
        if self.mode == Mode::Preedit {
            let shown = (!self.composer.is_empty()).then(|| self.composer.display().to_string());
            out.push(Out::Preedit(shown));
            return true;
        }
        self.screen.pop();
        if self.composer.is_empty() {
            self.screen.clear();
        }
        if self.mode == Mode::Surrounding {
            out.push(Out::DeleteSurrounding(1));
        } else {
            if self.clear_suggestion {
                push_key(out, KEY_DELETE, KEYCODE_DELETE, 0);
            }
            push_key(out, KEY_BACKSPACE, KEYCODE_BACKSPACE, 0);
        }
        true
    }

    fn end_word(&mut self, out: &mut Vec<Out>, rewrite: bool) {
        if !self.composer.is_empty() {
            let fin = self.composer.finalize();
            if self.mode == Mode::Preedit {
                out.push(Out::Commit(fin));
                out.push(Out::Preedit(None));
            } else if rewrite && fin != self.screen {
                let (delete, insert) = diff(&self.screen, &fin);
                self.rewrite(delete, &insert, out);
            }
        }
        self.composer.clear();
        self.screen.clear();
    }

    fn rewrite(&mut self, delete: usize, insert: &str, out: &mut Vec<Out>) {
        match self.mode {
            Mode::Surrounding => {
                if delete > 0 {
                    out.push(Out::DeleteSurrounding(delete));
                }
                self.send_text(insert, out);
            }
            Mode::Backspace => {
                if delete > 0 && self.clear_suggestion {
                    // Delete removes a selected suggestion and is a no-op at the end of the text.
                    push_key(out, KEY_DELETE, KEYCODE_DELETE, 0);
                }
                for _ in 0..delete {
                    push_key(out, KEY_BACKSPACE, KEYCODE_BACKSPACE, 0);
                }
                if delete > 0 {
                    out.push(Out::Pause);
                }
                self.send_text(insert, out);
            }
            Mode::Select => {
                for _ in 0..delete {
                    push_key(out, KEY_LEFT, KEYCODE_LEFT, SHIFT_MASK);
                }
                if delete > 0 && insert.is_empty() {
                    push_key(out, KEY_BACKSPACE, KEYCODE_BACKSPACE, 0);
                }
                if delete > 0 {
                    out.push(Out::Pause);
                }
                self.send_text(insert, out);
            }
            Mode::ForwardKeys => {
                for _ in 0..delete {
                    push_key(out, KEY_BACKSPACE, KEYCODE_BACKSPACE, 0);
                }
                self.send_text(insert, out);
            }
            Mode::Preedit | Mode::Off => unreachable!("{:?} never rewrites", self.mode),
        }
    }

    fn send_text(&self, text: &str, out: &mut Vec<Out>) {
        if text.is_empty() {
            return;
        }
        if self.mode == Mode::ForwardKeys {
            for c in text.chars() {
                push_key(out, char_keyval(c), 0, if c.is_uppercase() { SHIFT_MASK } else { 0 });
            }
        } else {
            out.push(Out::Commit(text.to_string()));
        }
    }
}

fn push_key(out: &mut Vec<Out>, keyval: u32, keycode: u32, state: u32) {
    out.push(Out::Key(KeyEvent { keyval, keycode, state }));
    out.push(Out::Key(KeyEvent { keyval, keycode, state: state | RELEASE_MASK }));
}

/// Shift, Control, Alt, Super, Caps/Num Lock, ISO level shifts: never part of a word.
pub fn is_modifier(keyval: u32) -> bool {
    matches!(keyval, 0xffe1..=0xffee | 0xfe01..=0xfe0f | 0xff7f)
}

fn keyval_char(keyval: u32) -> Option<char> {
    match keyval {
        0x20..=0x7e | 0xa0..=0xff => char::from_u32(keyval),
        0x0100_0000..=0x0110_ffff => char::from_u32(keyval - 0x0100_0000),
        0xffb0..=0xffb9 => char::from_u32(u32::from(b'0') + keyval - 0xffb0),
        0xffaa => Some('*'),
        0xffab => Some('+'),
        0xffad => Some('-'),
        0xffae => Some('.'),
        0xffaf => Some('/'),
        _ => None,
    }
}

fn char_keyval(c: char) -> u32 {
    match c as u32 {
        cp @ (0x20..=0x7e | 0xa0..=0xff) => cp,
        cp => 0x0100_0000 + cp,
    }
}

#[cfg(test)]
mod tests;
