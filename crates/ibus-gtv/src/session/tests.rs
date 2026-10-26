use super::*;
use gtv_core::{Config, Method};

const ALL_MODES: [Mode; 5] =
    [Mode::Surrounding, Mode::Backspace, Mode::Select, Mode::ForwardKeys, Mode::Preedit];

/// Application model: text before the cursor, an optional selection, an optional preedit.
#[derive(Default)]
struct App {
    text: Vec<char>,
    selected: usize,
    preedit: String,
    /// A selected inline-autocomplete suggestion after the caret (browser address bar): the
    /// first BackSpace or Delete removes it, a commit replaces it.
    ghost: bool,
}

impl App {
    fn drop_selection(&mut self) {
        for _ in 0..self.selected {
            self.text.pop();
        }
        self.selected = 0;
    }

    fn apply(&mut self, out: &Out) {
        match out {
            Out::Commit(t) => {
                self.drop_selection();
                self.ghost = false;
                self.preedit.clear();
                self.text.extend(t.chars());
            }
            Out::DeleteSurrounding(n) => {
                for _ in 0..*n {
                    self.text.pop();
                }
            }
            Out::Preedit(p) => self.preedit = p.clone().unwrap_or_default(),
            Out::Key(k) if k.state & RELEASE_MASK == 0 => self.native_key(k.keyval, k.state),
            Out::Key(_) | Out::Pause | Out::RequireSurroundingText => {}
        }
    }

    fn native_key(&mut self, keyval: u32, state: u32) {
        match keyval {
            KEY_DELETE => self.ghost = false,
            KEY_BACKSPACE if self.ghost => self.ghost = false,
            KEY_BACKSPACE if self.selected > 0 => self.drop_selection(),
            KEY_BACKSPACE => {
                self.text.pop();
            }
            KEY_LEFT if state & SHIFT_MASK != 0 => {
                self.selected = (self.selected + 1).min(self.text.len())
            }
            kv => {
                if let Some(c) = keyval_char(kv) {
                    self.drop_selection();
                    self.text.push(c);
                }
            }
        }
    }

    fn shown(&self) -> String {
        self.text.iter().collect::<String>() + &self.preedit
    }
}

struct Rig {
    app: App,
    session: Session,
}

impl Rig {
    fn new(mode: Mode, cfg: Config) -> Self {
        Rig { app: App::default(), session: Session::new(cfg, mode) }
    }

    fn press(&mut self, keyval: u32, state: u32) {
        let (handled, outs) = self.session.process_key(KeyEvent { keyval, keycode: 0, state });
        for o in &outs {
            self.app.apply(o);
        }
        if !handled {
            self.app.native_key(keyval, state);
        }
        // The release must be swallowed exactly when the press was.
        let (rel, _) =
            self.session.process_key(KeyEvent { keyval, keycode: 0, state: state | RELEASE_MASK });
        assert_eq!(rel, handled, "release of {keyval:#x} must mirror the press");
    }

    fn type_str(&mut self, s: &str) -> &mut Self {
        for c in s.chars() {
            self.press(c as u32, if c.is_uppercase() { SHIFT_MASK } else { 0 });
        }
        self
    }

    fn shown(&self) -> String {
        self.app.shown()
    }
}

fn telex(mode: Mode, input: &str) -> String {
    let mut rig = Rig::new(mode, Config::default());
    rig.type_str(input);
    // Preedit still pending at the end is committed by the frontend on focus-out.
    for o in rig.session.reset() {
        rig.app.apply(&o);
    }
    rig.shown()
}

fn every_mode(input: &str, want: &str) {
    for mode in ALL_MODES {
        assert_eq!(telex(mode, input), want, "{mode:?} typing {input:?}");
    }
}

#[test]
fn sentences_come_out_right_in_every_mode() {
    every_mode("Vieetj Nam", "Việt Nam");
    every_mode("xin chaof cacs banj, ddi hocj.", "xin chào các bạn, đi học.");
    every_mode("Nguoiwf Vieetj ddang hocj laapj trinhf", "Người Việt đang học lập trình");
    every_mode("DDUWOWCJ", "ĐƯỢC");
    every_mode("hoaf thuyr khoer", "hòa thủy khỏe");
}

#[test]
fn english_survives_in_every_mode() {
    every_mode("class off error press address", "class of eror press address");
    every_mode("www.example.com", "ww.example.com");
    every_mode(
        "sudo apt update && git push origin main",
        "sudo apt update && git push origin main",
    );
    every_mode("cargo build --release", "cargo build --release");
}

#[test]
fn words_with_wrong_tone_for_final_are_restored() {
    every_mode("text catf", "text catf");
    every_mode("windows", "windows");
}

#[test]
fn backspace_mid_word_keeps_composition() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config::default());
        rig.type_str("vieetj");
        rig.press(KEY_BACKSPACE, 0);
        rig.type_str("t");
        for o in rig.session.reset() {
            rig.app.apply(&o);
        }
        assert_eq!(rig.shown(), "việt", "{mode:?}");
    }
}

#[test]
fn backspace_to_empty_then_new_word() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config::default());
        rig.type_str("aas");
        rig.press(KEY_BACKSPACE, 0);
        rig.type_str("ban");
        for o in rig.session.reset() {
            rig.app.apply(&o);
        }
        assert_eq!(rig.shown(), "ban", "{mode:?}");
    }
}

#[test]
fn enter_and_tab_end_the_word() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config::default());
        rig.type_str("toans");
        rig.press(KEY_RETURN, 0);
        assert_eq!(rig.shown(), "toán", "{mode:?}");
    }
}

#[test]
fn ctrl_shortcut_passes_through_and_ends_word() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config::default());
        rig.type_str("toan");
        let (handled, outs) = rig.session.process_key(KeyEvent {
            keyval: 'a' as u32,
            keycode: 0,
            state: CONTROL_MASK,
        });
        assert!(!handled, "{mode:?}");
        for o in &outs {
            rig.app.apply(o);
        }
        rig.type_str("s");
        // `s` starts a fresh word: no tone applied to the old one.
        for o in rig.session.reset() {
            rig.app.apply(&o);
        }
        assert_eq!(rig.shown(), "toans", "{mode:?}");
    }
}

#[test]
fn arrow_key_drops_the_word_without_rewriting() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config::default());
        rig.type_str("vieet");
        rig.press(0xff51, 0);
        rig.type_str("s");
        for o in rig.session.reset() {
            rig.app.apply(&o);
        }
        // The arrow moved the cursor in a real app; the model only checks no tone was applied.
        assert!(rig.shown().ends_with('s'), "{mode:?}: {}", rig.shown());
    }
}

#[test]
fn printable_keys_are_always_committed_never_passed_through() {
    // A passed-through key is queued by the toolkit and applied after signals sent for the next
    // key, which reorders text. Everything printable must travel the ordered signal channel.
    for mode in [Mode::Surrounding, Mode::Backspace, Mode::Select] {
        let mut session = Session::new(Config::default(), mode);
        for c in "hello, x".chars() {
            let (handled, out) =
                session.process_key(KeyEvent { keyval: c as u32, keycode: 0, state: 0 });
            assert!(handled, "{mode:?} {c}");
            assert_eq!(out, vec![Out::Commit(c.to_string())], "{mode:?} {c}");
        }
    }
    let mut session = Session::new(Config::default(), Mode::ForwardKeys);
    let (handled, out) = session.process_key(KeyEvent { keyval: 'h' as u32, keycode: 0, state: 0 });
    assert!(handled && out.len() == 2 && out.iter().all(|o| matches!(o, Out::Key(_))));
}

#[test]
fn rewrites_are_minimal() {
    let mut s = Session::new(Config::default(), Mode::Surrounding);
    for c in "toan".chars() {
        s.process_key(KeyEvent { keyval: c as u32, keycode: 0, state: 0 });
    }
    let (handled, out) = s.process_key(KeyEvent { keyval: 's' as u32, keycode: 0, state: 0 });
    assert!(handled);
    assert_eq!(out, vec![Out::DeleteSurrounding(2), Out::Commit("án".into())]);
}

#[test]
fn backspace_mode_forwards_backspaces_then_commits() {
    let mut s = Session::new(Config::default(), Mode::Backspace);
    for c in "toan".chars() {
        s.process_key(KeyEvent { keyval: c as u32, keycode: 0, state: 0 });
    }
    let (handled, out) = s.process_key(KeyEvent { keyval: 's' as u32, keycode: 0, state: 0 });
    assert!(handled);
    let keys = out.iter().filter(|o| matches!(o, Out::Key(_))).count();
    assert_eq!(keys, 4, "two backspaces, press+release each");
    assert!(
        out.contains(&Out::Pause),
        "forwarded keys must be given time to land before the commit"
    );
    assert_eq!(out.last(), Some(&Out::Commit("án".into())));
}

#[test]
fn vni_works_through_the_session() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config { method: Method::Vni, ..Config::default() });
        rig.type_str("Vie6t5 Nam 2024 d9i");
        for o in rig.session.reset() {
            rig.app.apply(&o);
        }
        assert_eq!(rig.shown(), "Việt Nam 2024 đi", "{mode:?}");
    }
}

#[test]
fn keypad_digits_are_digits() {
    let mut rig = Rig::new(Mode::Surrounding, Config { method: Method::Vni, ..Config::default() });
    rig.type_str("a");
    rig.press(0xffb6, 0); // KP_6
    assert_eq!(rig.shown(), "â");
}

#[test]
fn set_mode_flushes_preedit() {
    let mut s = Session::new(Config::default(), Mode::Preedit);
    s.process_key(KeyEvent { keyval: 'a' as u32, keycode: 0, state: 0 });
    let out = s.set_mode(Mode::Surrounding);
    assert_eq!(out, vec![Out::Commit("a".into()), Out::Preedit(None)]);
}

#[test]
fn address_bar_suggestion_does_not_eat_the_typed_letter() {
    for (clear, want) in [(true, "vì"), (false, "viì")] {
        let mut rig = Rig::new(Mode::Backspace, Config::default());
        rig.session.set_clear_suggestion(clear);
        rig.type_str("vi");
        rig.app.ghost = true; // Chrome completed "viet.example.com" after the caret
        rig.type_str("f");
        assert_eq!(rig.shown(), want, "clear_suggestion={clear}");
    }
}

#[test]
fn off_mode_passes_every_key_through() {
    let mut rig = Rig::new(Mode::Off, Config::default());
    rig.type_str("vieetj ddi");
    assert_eq!(rig.shown(), "vieetj ddi");
}

#[test]
fn backspace_after_undone_mark_keeps_what_is_left() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config::default());
        rig.type_str("eee");
        rig.press(KEY_BACKSPACE, 0);
        rig.type_str(" ");
        assert_eq!(rig.shown(), "e ", "{mode:?}");
    }
}

#[test]
fn four_e_then_backspace_then_space_keeps_two() {
    for mode in ALL_MODES {
        let mut rig = Rig::new(mode, Config::default());
        rig.type_str("eeee");
        assert_eq!(rig.shown(), "eee", "{mode:?}");
        rig.press(KEY_BACKSPACE, 0);
        rig.type_str(" ");
        assert_eq!(rig.shown(), "ee ", "{mode:?}");
    }
}
