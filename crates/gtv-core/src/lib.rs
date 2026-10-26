//! Vietnamese composition core, independent of any input method framework.
//!
//! A [`Composer`] holds the raw keystrokes of the word being typed and recomputes the whole
//! word after every key. Undo, free-style marking and auto-restore of non-Vietnamese words all
//! fall out of replaying from the raw keys instead of mutating on-screen text.

mod compose;
mod syllable;
mod table;

use compose::{compose, Role};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Telex,
    Vni,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToneStyle {
    /// hòa, khỏe, thủy — tone on the first vowel of oa/oe/uy (Unikey's default).
    Modern,
    /// hoà, khoẻ, thuỷ — tone on the second vowel of oa/oe/uy.
    Old,
}

#[derive(Clone, Debug)]
pub struct Config {
    pub method: Method,
    pub tone_style: ToneStyle,
    /// Allow a modifier key to reach back past trailing consonants (`tiengs`, `viete`).
    pub free_marking: bool,
    /// Fall back to the typed keys when the result is not a plausible Vietnamese syllable.
    pub spell_check: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            method: Method::Telex,
            tone_style: ToneStyle::Modern,
            free_marking: true,
            spell_check: true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Tone {
    None = 0,
    Acute,
    Grave,
    Hook,
    Tilde,
    Dot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mark {
    None,
    Circ,
    Breve,
    Horn,
    Stroke,
}

#[derive(Clone, Debug)]
pub(crate) struct Letter {
    /// Lowercase base character; non-letters (VNI digits that did not apply) are kept as is.
    pub base: char,
    pub mark: Mark,
    pub upper: bool,
    /// Born from a standalone Telex `w` (`ư`), so a second `w` removes the letter again.
    pub from_w: bool,
    /// Raw key index that set `mark`, so an undo can retire exactly that key.
    pub mark_src: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Push {
    /// The key belongs to the word; read the new text from [`Composer::display`].
    Accepted,
    /// The key ends the word and was not consumed. The caller must reset and pass it through.
    Boundary,
}

pub struct Composer {
    cfg: Config,
    raw: Vec<char>,
    roles: Vec<Role>,
    display: String,
    plain: String,
    plausible: bool,
    tone_at: Option<usize>,
}

impl Composer {
    pub fn new(cfg: Config) -> Self {
        Composer {
            cfg,
            raw: Vec::new(),
            roles: Vec::new(),
            display: String::new(),
            plain: String::new(),
            plausible: true,
            tone_at: None,
        }
    }

    pub fn config(&self) -> &Config {
        &self.cfg
    }

    pub fn set_config(&mut self, cfg: Config) {
        self.cfg = cfg;
        self.clear();
    }

    pub fn clear(&mut self) {
        self.raw.clear();
        self.roles.clear();
        self.display.clear();
        self.plain.clear();
        self.plausible = true;
        self.tone_at = None;
    }

    pub fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    /// Text the word should currently show on screen.
    pub fn display(&self) -> &str {
        &self.display
    }

    /// The word with every Vietnamese transformation reverted (what the user physically typed,
    /// minus keys cancelled by an undo).
    pub fn plain(&self) -> &str {
        &self.plain
    }

    /// Text the word must have once it ends: exactly what is already shown, so nothing changes
    /// at the word boundary. Non-Vietnamese words are shown as typed (`class`, `off`, `www`).
    pub fn finalize(&self) -> String {
        self.display.clone()
    }

    pub fn push(&mut self, c: char) -> Push {
        if !self.is_word_key(c) {
            return Push::Boundary;
        }
        self.raw.push(c);
        self.refresh();
        Push::Accepted
    }

    /// Mirrors a Backspace that the application applies itself: removes the last visible letter
    /// but keeps tone and marks of the remaining ones. Returns false when there was nothing.
    pub fn backspace(&mut self) -> bool {
        let Some(last) = self.letter_count().checked_sub(1) else {
            let had = !self.raw.is_empty();
            self.clear();
            return had;
        };
        let drop_tone = self.tone_at == Some(last);
        let last_key = self.roles.iter().position(|r| *r == Role::Letter(last));
        let mut keep = Vec::with_capacity(self.raw.len());
        for (i, &c) in self.raw.iter().enumerate() {
            let drop = match self.roles[i] {
                Role::Letter(l) => l == last,
                Role::Mod(Some(l)) => l == last,
                Role::Mod(None) => drop_tone,
                // A cancelled key stays cancelled unless the key that cancelled it is deleted.
                Role::Dead(by) => Some(by) == last_key,
            };
            if !drop {
                keep.push(c);
            }
        }
        self.raw = keep;
        self.refresh();
        true
    }

    fn letter_count(&self) -> usize {
        self.roles.iter().filter(|r| matches!(r, Role::Letter(_))).count()
    }

    fn refresh(&mut self) {
        let out = compose(&self.raw, &self.cfg);
        self.roles = out.roles;
        self.display = out.display;
        self.plain = out.plain;
        self.plausible = out.plausible;
        self.tone_at = out.tone_at;
    }

    fn is_word_key(&self, c: char) -> bool {
        if c.is_ascii_alphabetic() {
            return true;
        }
        if self.cfg.method != Method::Vni || !c.is_ascii_digit() {
            return false;
        }
        // A VNI digit belongs to the word only if it acts as a modifier (or cancels one).
        let dead_before = self.roles.iter().filter(|r| matches!(r, Role::Dead(_))).count();
        let mut probe = self.raw.clone();
        probe.push(c);
        let out = compose(&probe, &self.cfg);
        let dead_after = out.roles.iter().filter(|r| matches!(r, Role::Dead(_))).count();
        matches!(out.roles.last(), Some(Role::Mod(_))) || dead_after > dead_before
    }
}

/// Minimal edit turning `old` into `new` on screen: how many chars to delete from the end of
/// `old`, and the text to insert afterwards.
pub fn diff(old: &str, new: &str) -> (usize, String) {
    let common = old.chars().zip(new.chars()).take_while(|(a, b)| a == b).count();
    let delete = old.chars().count() - common;
    (delete, new.chars().skip(common).collect())
}

#[cfg(test)]
mod tests;
#[cfg(test)]
mod tests_dict;
