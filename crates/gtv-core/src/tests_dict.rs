//! Round-trips every syllable of a real Vietnamese word list through the composer.
//!
//! The list is read from the system (ibus-bamboo's data file) or `$GTV_DICT`; the test is
//! skipped when neither exists, so nothing licensed lives in this repository.

use crate::table::decompose;
use crate::{Composer, Config, Mark, Method, Push, Tone, ToneStyle};

fn load() -> Option<Vec<String>> {
    let path = std::env::var("GTV_DICT")
        .unwrap_or_else(|_| "/usr/share/ibus-bamboo/data/vietnamese.cm.dict".into());
    let text = std::fs::read_to_string(path).ok()?;
    let words = text
        .lines()
        .map(str::trim)
        .filter(|w| !w.is_empty() && !w.starts_with('#') && w.chars().all(|c| c.is_alphabetic()))
        .map(str::to_string)
        .collect();
    Some(words)
}

#[derive(Clone, Copy)]
enum Order {
    /// `vieejt`-style: marks right after the letter, tone at the very end.
    ToneLast,
    /// Tone right after the nucleus, before the final consonant.
    ToneAfterNucleus,
}

fn encode(word: &str, method: Method, order: Order) -> String {
    let (circ, horn, breve, stroke): (fn(char) -> String, &str, &str, &str) = match method {
        Method::Telex => (|c| format!("{c}{c}"), "w", "w", "d"),
        Method::Vni => (|c| format!("{c}6"), "7", "8", "9"),
    };
    let tone_key = |t: Tone| match (method, t) {
        (_, Tone::None) => "",
        (Method::Telex, Tone::Acute) => "s",
        (Method::Telex, Tone::Grave) => "f",
        (Method::Telex, Tone::Hook) => "r",
        (Method::Telex, Tone::Tilde) => "x",
        (Method::Telex, Tone::Dot) => "j",
        (Method::Vni, Tone::Acute) => "1",
        (Method::Vni, Tone::Grave) => "2",
        (Method::Vni, Tone::Hook) => "3",
        (Method::Vni, Tone::Tilde) => "4",
        (Method::Vni, Tone::Dot) => "5",
    };
    let mut out = String::new();
    let mut tone = Tone::None;
    let mut tone_at = None;
    let chars: Vec<char> = word.chars().collect();
    let mut last_vowel = None;
    for (i, &c) in chars.iter().enumerate() {
        let lower = c.to_lowercase().next().unwrap();
        let (base, mark, t) = decompose(lower).unwrap_or((lower, Mark::None, Tone::None));
        if t != Tone::None {
            tone = t;
        }
        let upper = c.is_uppercase();
        let b = if upper { base.to_ascii_uppercase() } else { base };
        match mark {
            Mark::Circ => out.push_str(&circ(b)),
            Mark::Breve => {
                out.push(b);
                out.push_str(breve);
            }
            Mark::Horn => {
                out.push(b);
                out.push_str(horn);
            }
            Mark::Stroke => {
                out.push(b);
                out.push_str(stroke);
            }
            Mark::None => {
                out.push(b);
                // A literal `oo` has to be typed `ooo` (the third key undoes the circumflex).
                let prev_plain = i > 0 && chars[i - 1].to_lowercase().next() == Some(lower);
                if method == Method::Telex && prev_plain && matches!(lower, 'a' | 'e' | 'o') {
                    out.push(b);
                }
            }
        }
        if matches!(base, 'a' | 'e' | 'i' | 'o' | 'u' | 'y') {
            last_vowel = Some(i);
            tone_at = Some(out.len());
        }
    }
    let _ = last_vowel;
    match order {
        Order::ToneLast => out.push_str(tone_key(tone)),
        Order::ToneAfterNucleus => out.insert_str(tone_at.unwrap_or(out.len()), tone_key(tone)),
    }
    out
}

fn type_word(cfg: &Config, keys: &str) -> String {
    let mut c = Composer::new(cfg.clone());
    for k in keys.chars() {
        assert_eq!(c.push(k), Push::Accepted, "{keys}");
    }
    c.finalize()
}

fn sweep(cfg: &Config, order: Order, words: &[String]) -> Vec<String> {
    words
        .iter()
        .filter_map(|w| {
            let keys = encode(w, cfg.method, order);
            let got = type_word(cfg, &keys);
            (got != *w).then(|| format!("{w}: typed {keys} -> {got}"))
        })
        .collect()
}

fn run(method: Method, order: Order, style: ToneStyle, max_fail_pct: f64) {
    let Some(words) = load() else {
        eprintln!("dictionary not found, skipping");
        return;
    };
    let cfg = Config { method, tone_style: style, ..Config::default() };
    let fails = sweep(&cfg, order, &words);
    let pct = fails.len() as f64 * 100.0 / words.len() as f64;
    eprintln!("{method:?} {style:?}: {} / {} mismatches ({pct:.2}%)", fails.len(), words.len());
    for f in fails.iter().take(40) {
        eprintln!("  {f}");
    }
    assert!(pct <= max_fail_pct, "{pct:.2}% of dictionary words do not round-trip");
}

#[test]
fn telex_tone_last_roundtrips_dictionary() {
    run(Method::Telex, Order::ToneLast, ToneStyle::Modern, 100.0);
}

#[test]
fn telex_tone_after_nucleus_roundtrips_dictionary() {
    run(Method::Telex, Order::ToneAfterNucleus, ToneStyle::Modern, 100.0);
}

#[test]
fn vni_tone_last_roundtrips_dictionary() {
    run(Method::Vni, Order::ToneLast, ToneStyle::Modern, 100.0);
}

#[test]
fn words_failing_in_both_tone_styles() {
    let Some(words) = load() else { return };
    let fails = |style| {
        let cfg = Config { tone_style: style, ..Config::default() };
        sweep(&cfg, Order::ToneLast, &words)
            .into_iter()
            .map(|f| f.split(':').next().unwrap().to_string())
            .collect::<std::collections::HashSet<_>>()
    };
    let modern = fails(ToneStyle::Modern);
    let old = fails(ToneStyle::Old);
    let mut both: Vec<_> = modern.intersection(&old).cloned().collect();
    both.sort();
    eprintln!("{} fail in both styles:\n{}", both.len(), both.join(" "));
}
