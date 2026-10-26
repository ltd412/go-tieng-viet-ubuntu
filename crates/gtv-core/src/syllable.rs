use std::ops::Range;

use crate::table::{is_vowel_base, letter_char};
use crate::{Letter, Mark, Tone, ToneStyle};

const ONSETS: &[&str] = &[
    "b", "c", "d", "đ", "g", "h", "k", "l", "m", "n", "p", "q", "r", "s", "t", "v", "x", "ch",
    "gh", "gi", "kh", "kr", "ng", "nh", "ph", "qu", "th", "tr", "ngh",
];

/// Every nucleus that occurs in Vietnamese, lenient on purpose: wrongly rejecting a real word
/// is worse than wrongly accepting an English one.
const NUCLEI: &[&str] = &[
    "a", "ă", "â", "e", "ê", "i", "o", "ô", "ơ", "u", "ư", "y", //
    "ai", "ao", "au", "ay", "âu", "ây", "eo", "êu", "ia", "iê", "iu", "oa", "oă", "oe", "oi", "oo",
    "ôi", "ơi", "ua", "uâ", "uê", "ui", "uô", "uy", "ưa", "ưi", "ưu", "ươ", "uơ", "yê", //
    "iêu", "yêu", "oai", "oao", "oay", "oeo", "uai", "uao", "uay", "uây", "uôi", "uya", "uyê",
    "uyu", "ươi", "ươu", "uêu",
];

const CODAS: &[&str] = &["", "c", "ch", "k", "m", "n", "ng", "nh", "p", "t"];

pub(crate) struct Parts {
    pub onset: usize,
    pub nucleus: Range<usize>,
    pub coda: Range<usize>,
}

fn vowel(letters: &[Letter], i: usize) -> bool {
    letters.get(i).is_some_and(|l| is_vowel_base(l.base))
}

/// Length of the initial consonant cluster. `qu` and `gi` count as onsets only when a vowel
/// follows, otherwise the `u`/`i` is the nucleus (`gì`, `qu` alone).
pub(crate) fn onset_len(letters: &[Letter]) -> usize {
    let b = |i: usize| letters.get(i).map_or('\0', |l| l.base);
    if matches!((b(0), b(1)), ('q', 'u') | ('g', 'i')) && vowel(letters, 2) {
        return 2;
    }
    if (b(0), b(1), b(2)) == ('n', 'g', 'h') {
        return 3;
    }
    let pair = [b(0), b(1)];
    if ["ng", "nh", "ch", "gh", "kh", "kr", "ph", "th", "tr"].iter().any(|p| p.chars().eq(pair)) {
        return 2;
    }
    match letters.first() {
        Some(l) if !is_vowel_base(l.base) => 1,
        _ => 0,
    }
}

pub(crate) fn split(letters: &[Letter]) -> Parts {
    let onset = onset_len(letters);
    let mut end = onset;
    while vowel(letters, end) {
        end += 1;
    }
    Parts { onset, nucleus: onset..end, coda: end..letters.len() }
}

fn text(letters: &[Letter], marks: &[Mark], range: Range<usize>) -> String {
    range.map(|i| letter_char(letters[i].base, marks[i])).collect()
}

/// `ươ` without a final consonant is written `uơ` (thuở, huơ); `marks` is the on-screen form.
pub(crate) fn screen_marks(letters: &[Letter], parts: &Parts) -> Vec<Mark> {
    let mut marks: Vec<Mark> = letters.iter().map(|l| l.mark).collect();
    let n = &parts.nucleus;
    if n.len() >= 2 && parts.coda.is_empty() {
        let (u, o) = (n.end - 2, n.end - 1);
        let horned = |i: usize, base: char| letters[i].base == base && marks[i] == Mark::Horn;
        if horned(u, 'u') && horned(o, 'o') {
            marks[u] = Mark::None;
        }
    }
    marks
}

/// Whether `letters` could be the start of (or all of) a Vietnamese syllable.
pub(crate) fn is_plausible(letters: &[Letter], marks: &[Mark], parts: &Parts, tone: Tone) -> bool {
    if letters.is_empty() {
        return true;
    }
    let onset = text(letters, marks, 0..parts.onset);
    if !onset.is_empty() && !ONSETS.contains(&onset.as_str()) {
        return false;
    }
    if parts.nucleus.is_empty() {
        return parts.coda.is_empty();
    }
    if onset == "q" || !NUCLEI.contains(&text(letters, marks, parts.nucleus.clone()).as_str()) {
        return false;
    }
    let coda = text(letters, marks, parts.coda.clone());
    if !CODAS.contains(&coda.as_str()) {
        return false;
    }
    // Final k exists only in Central Highlands names after ă (Đắk Lắk, Đắk Nông).
    if coda == "k" && text(letters, marks, parts.nucleus.clone()) != "ă" {
        return false;
    }
    // Stop finals only carry sắc or nặng.
    !(matches!(coda.as_str(), "c" | "ch" | "k" | "p" | "t")
        && matches!(tone, Tone::Grave | Tone::Hook | Tone::Tilde))
}

/// Index of the letter that carries the tone mark.
pub(crate) fn tone_index(
    letters: &[Letter],
    marks: &[Mark],
    parts: &Parts,
    style: ToneStyle,
) -> Option<usize> {
    let n = parts.nucleus.clone();
    match n.len() {
        0 => return None,
        1 => return Some(n.start),
        _ => {}
    }
    if let Some(i) = n.clone().rev().find(|&i| marks[i] != Mark::None) {
        return Some(i);
    }
    if n.len() >= 3 {
        return Some(n.start + 1);
    }
    let pair = text(letters, marks, n.clone());
    let second = !parts.coda.is_empty()
        || (style == ToneStyle::Old && matches!(pair.as_str(), "oa" | "oe" | "uy"));
    Some(if second { n.start + 1 } else { n.start })
}
