use std::ops::Range;

use crate::syllable::{is_plausible, onset_len, screen_marks, split, tone_index};
use crate::table::{is_vowel_base, render_into};
use crate::{Config, Letter, Mark, Method, Tone};

/// What a raw key ended up doing; one entry per raw key.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Role {
    /// Became letter number `n` of the word.
    Letter(usize),
    /// Acted as a modifier (tone, or mark on letter `n`).
    Mod(Option<usize>),
    /// Cancelled by an undo; contributes nothing, not even to the plain text. Holds the index
    /// of the raw key that did the cancelling, which Backspace needs: deleting a later letter
    /// must leave this key dead (`eeee` minus one is `ee`, not `ê`).
    Dead(usize),
}

pub(crate) struct Composed {
    pub display: String,
    pub plain: String,
    pub roles: Vec<Role>,
    pub plausible: bool,
    /// Letter carrying the tone on screen, if any.
    pub tone_at: Option<usize>,
}

#[derive(Clone, Copy)]
enum Action {
    Tone(Tone),
    ClearTone,
    /// Circumflex; `Some(c)` pins the target vowel (Telex `aa`, `ee`, `oo`), `None` picks any.
    Circ(Option<char>),
    /// Horn and/or breve (Telex `w`, VNI 7 and 8).
    Hat {
        horn: bool,
        breve: bool,
        standalone: bool,
    },
    Stroke,
}

fn action_for(method: Method, c: char) -> Option<Action> {
    Some(match (method, c) {
        (Method::Telex, 's') | (Method::Vni, '1') => Action::Tone(Tone::Acute),
        (Method::Telex, 'f') | (Method::Vni, '2') => Action::Tone(Tone::Grave),
        (Method::Telex, 'r') | (Method::Vni, '3') => Action::Tone(Tone::Hook),
        (Method::Telex, 'x') | (Method::Vni, '4') => Action::Tone(Tone::Tilde),
        (Method::Telex, 'j') | (Method::Vni, '5') => Action::Tone(Tone::Dot),
        (Method::Telex, 'z') | (Method::Vni, '0') => Action::ClearTone,
        (Method::Telex, 'a' | 'e' | 'o') => Action::Circ(Some(c)),
        (Method::Vni, '6') => Action::Circ(None),
        (Method::Telex, 'w') => Action::Hat { horn: true, breve: true, standalone: true },
        (Method::Vni, '7') => Action::Hat { horn: true, breve: false, standalone: false },
        (Method::Vni, '8') => Action::Hat { horn: false, breve: true, standalone: false },
        (Method::Telex, 'd') | (Method::Vni, '9') => Action::Stroke,
        _ => return None,
    })
}

struct State<'a> {
    cfg: &'a Config,
    letters: Vec<Letter>,
    tone: Tone,
    tone_src: Option<usize>,
    /// After an undo the rest of the word is typed literally.
    frozen: bool,
    roles: Vec<Role>,
}

pub(crate) fn compose(raw: &[char], cfg: &Config) -> Composed {
    let mut st = State {
        cfg,
        letters: Vec::new(),
        tone: Tone::None,
        tone_src: None,
        frozen: false,
        roles: Vec::with_capacity(raw.len()),
    };
    for (i, &c) in raw.iter().enumerate() {
        st.step(i, c);
    }
    st.finish(raw)
}

impl State<'_> {
    fn step(&mut self, i: usize, c: char) {
        // `ư` + `o` is only ever `ươ` (duwocj -> được): the second horn needs no `w`.
        if !self.frozen
            && self.cfg.method == Method::Telex
            && c.eq_ignore_ascii_case(&'o')
            && self.letters.last().is_some_and(|l| l.base == 'u' && l.mark == Mark::Horn)
        {
            self.roles.push(Role::Letter(self.letters.len()));
            self.letters.push(Letter {
                base: 'o',
                mark: Mark::Horn,
                upper: c.is_uppercase(),
                from_w: false,
                mark_src: None,
            });
            return;
        }
        if !self.frozen {
            if let Some(action) = action_for(self.cfg.method, c.to_ascii_lowercase()) {
                if self.apply(i, c, action) {
                    return;
                }
            }
        }
        self.literal(c);
    }

    fn literal(&mut self, c: char) {
        self.roles.push(Role::Letter(self.letters.len()));
        self.letters.push(Letter {
            base: c.to_lowercase().next().unwrap_or(c),
            mark: Mark::None,
            upper: c.is_uppercase(),
            from_w: false,
            mark_src: None,
        });
    }

    fn has_vowel(&self) -> bool {
        self.letters.iter().any(|l| is_vowel_base(l.base))
    }

    /// Last run of vowels, the only place a modifier may land.
    fn cluster(&self) -> Option<Range<usize>> {
        let n = self.letters.len();
        let floor = onset_len(&self.letters);
        let last = (floor..n).rev().find(|&i| is_vowel_base(self.letters[i].base))?;
        let mut start = last;
        while start > floor && is_vowel_base(self.letters[start - 1].base) {
            start -= 1;
        }
        if !self.cfg.free_marking && last + 1 != n {
            return None;
        }
        Some(start..last + 1)
    }

    fn plausible(&self) -> bool {
        if !self.cfg.spell_check {
            return true;
        }
        let parts = split(&self.letters);
        let marks = screen_marks(&self.letters, &parts);
        is_plausible(&self.letters, &marks, &parts, self.tone)
    }

    fn set_mark(&mut self, p: usize, mark: Mark, src: usize) {
        self.letters[p].mark = mark;
        self.letters[p].mark_src = Some(src);
    }

    fn undo_mark(&mut self, by: usize, p: usize) {
        if let Some(src) = self.letters[p].mark_src.take() {
            self.roles[src] = Role::Dead(by);
        }
        self.letters[p].mark = Mark::None;
    }

    /// Sets `mark` on letter `p`, or reverts it when already set (the same key typed twice).
    /// Returns false on revert so the key falls through as a literal letter.
    fn toggle(&mut self, i: usize, p: usize, mark: Mark) -> bool {
        if self.letters[p].mark == mark {
            if !self.plausible() {
                // The user never saw this mark applied, so the key is just a letter.
                return false;
            }
            self.undo_mark(i, p);
            self.frozen = true;
            return false;
        }
        self.set_mark(p, mark, i);
        self.roles.push(Role::Mod(Some(p)));
        true
    }

    fn apply(&mut self, i: usize, c: char, action: Action) -> bool {
        match action {
            Action::Tone(t) => {
                if !self.has_vowel() {
                    return false;
                }
                if self.tone == t {
                    if !self.plausible() {
                        return false;
                    }
                    self.tone = Tone::None;
                    if let Some(src) = self.tone_src.take() {
                        self.roles[src] = Role::Dead(i);
                    }
                    self.frozen = true;
                    return false;
                }
                self.tone = t;
                self.tone_src = Some(i);
                self.roles.push(Role::Mod(None));
                true
            }
            Action::ClearTone => {
                if self.tone == Tone::None {
                    return false;
                }
                self.tone = Tone::None;
                self.tone_src = None;
                self.roles.push(Role::Mod(None));
                true
            }
            Action::Circ(target) => {
                let Some(cl) = self.cluster() else { return false };
                let hit = |l: &Letter| matches!(l.base, 'a' | 'e' | 'o');
                // Telex reaches back only to the last vowel (`khoeo` must not become `khôe`);
                // VNI digits are unambiguous, so any a/e/o in the cluster will do.
                let pick = match target {
                    Some(t) => Some(cl.end - 1).filter(|&p| self.letters[p].base == t),
                    None => cl.rev().find(|&p| hit(&self.letters[p])),
                };
                match pick {
                    Some(p) => self.toggle(i, p, Mark::Circ),
                    None => false,
                }
            }
            Action::Hat { horn, breve, standalone } => {
                self.apply_hat(i, c, horn, breve, standalone)
            }
            Action::Stroke => {
                let p = match self.cfg.method {
                    Method::Telex => self.letters.len().checked_sub(1),
                    Method::Vni => (!self.letters.is_empty()).then_some(0),
                };
                match p.filter(|&p| self.letters[p].base == 'd') {
                    Some(p) => self.toggle(i, p, Mark::Stroke),
                    None => false,
                }
            }
        }
    }

    fn apply_hat(&mut self, i: usize, c: char, horn: bool, breve: bool, standalone: bool) -> bool {
        if standalone {
            // `ww` after a lone `w` -> `ư` gives back a plain `w`.
            if self.letters.last().is_some_and(|l| l.from_w) {
                let src = self.roles.iter().rposition(|r| matches!(r, Role::Letter(_))).unwrap();
                self.letters.pop();
                self.roles[src] = Role::Dead(i);
                self.frozen = true;
                return false;
            }
            if !self.has_vowel() {
                self.roles.push(Role::Letter(self.letters.len()));
                self.letters.push(Letter {
                    base: 'u',
                    mark: Mark::Horn,
                    upper: c.is_uppercase(),
                    from_w: true,
                    mark_src: None,
                });
                return true;
            }
        }
        let Some(cl) = self.cluster() else { return false };
        let idx: Vec<usize> = cl.collect();
        let base = |s: &Self, p: usize| s.letters[p].base;

        if horn {
            if let Some(&u) = idx.iter().find(|&&p| {
                base(self, p) == 'u' && idx.contains(&(p + 1)) && base(self, p + 1) == 'o'
            }) {
                let o = u + 1;
                if self.letters[u].mark == Mark::Horn && self.letters[o].mark == Mark::Horn {
                    if self.letters[o].mark_src.is_none() {
                        // `ơ` came from the `o` typed after `ư`; the habitual extra `w`
                        // (thuwowng) confirms it rather than undoing it.
                        self.roles.push(Role::Mod(Some(o)));
                        return true;
                    }
                    if !self.plausible() {
                        return false;
                    }
                    self.undo_mark(i, u);
                    self.undo_mark(i, o);
                    self.frozen = true;
                    return false;
                }
                self.set_mark(u, Mark::Horn, i);
                self.set_mark(o, Mark::Horn, i);
                self.roles.push(Role::Mod(Some(o)));
                return true;
            }
            if idx.len() == 2 && base(self, idx[0]) == 'u' && base(self, idx[1]) == 'a' {
                return self.toggle(i, idx[0], Mark::Horn);
            }
        }
        let target = if breve {
            idx.iter().rev().find(|&&p| base(self, p) == 'a').map(|&p| (p, Mark::Breve))
        } else {
            None
        };
        let target = target.or_else(|| {
            if horn {
                idx.iter().find(|&&p| matches!(base(self, p), 'o' | 'u')).map(|&p| (p, Mark::Horn))
            } else {
                None
            }
        });
        match target {
            Some((p, mark)) => self.toggle(i, p, mark),
            None => false,
        }
    }

    fn finish(self, raw: &[char]) -> Composed {
        let parts = split(&self.letters);
        let marks = screen_marks(&self.letters, &parts);
        let plain: String = raw
            .iter()
            .zip(&self.roles)
            .filter(|(_, r)| !matches!(r, Role::Dead(_)))
            .map(|(&c, _)| c)
            .collect();
        let plausible =
            !self.cfg.spell_check || is_plausible(&self.letters, &marks, &parts, self.tone);
        let tone_at = (self.tone != Tone::None)
            .then(|| tone_index(&self.letters, &marks, &parts, self.cfg.tone_style))
            .flatten();
        let display = if plausible {
            let mut out = String::with_capacity(raw.len() * 2);
            for (p, l) in self.letters.iter().enumerate() {
                let tone = if Some(p) == tone_at { self.tone } else { Tone::None };
                render_into(&mut out, l.base, marks[p], tone, l.upper);
            }
            out
        } else {
            plain.clone()
        };
        Composed { display, plain, roles: self.roles, plausible, tone_at }
    }
}
