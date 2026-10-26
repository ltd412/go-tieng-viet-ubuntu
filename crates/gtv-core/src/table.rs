use crate::{Mark, Tone};

/// Precomposed (NFC) forms of a vowel for the six tones, indexed by `Tone as usize`.
fn vowel_row(base: char, mark: Mark) -> Option<[char; 6]> {
    Some(match (base, mark) {
        ('a', Mark::None) => ['a', 'á', 'à', 'ả', 'ã', 'ạ'],
        ('a', Mark::Breve) => ['ă', 'ắ', 'ằ', 'ẳ', 'ẵ', 'ặ'],
        ('a', Mark::Circ) => ['â', 'ấ', 'ầ', 'ẩ', 'ẫ', 'ậ'],
        ('e', Mark::None) => ['e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ'],
        ('e', Mark::Circ) => ['ê', 'ế', 'ề', 'ể', 'ễ', 'ệ'],
        ('i', Mark::None) => ['i', 'í', 'ì', 'ỉ', 'ĩ', 'ị'],
        ('o', Mark::None) => ['o', 'ó', 'ò', 'ỏ', 'õ', 'ọ'],
        ('o', Mark::Circ) => ['ô', 'ố', 'ồ', 'ổ', 'ỗ', 'ộ'],
        ('o', Mark::Horn) => ['ơ', 'ớ', 'ờ', 'ở', 'ỡ', 'ợ'],
        ('u', Mark::None) => ['u', 'ú', 'ù', 'ủ', 'ũ', 'ụ'],
        ('u', Mark::Horn) => ['ư', 'ứ', 'ừ', 'ử', 'ữ', 'ự'],
        ('y', Mark::None) => ['y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ'],
        _ => return None,
    })
}

/// Inverse of rendering, for tests and diagnostics: `ế` -> (`e`, circumflex, acute).
#[cfg(test)]
pub(crate) fn decompose(c: char) -> Option<(char, Mark, Tone)> {
    const TONES: [Tone; 6] =
        [Tone::None, Tone::Acute, Tone::Grave, Tone::Hook, Tone::Tilde, Tone::Dot];
    for base in ['a', 'e', 'i', 'o', 'u', 'y'] {
        for mark in [Mark::None, Mark::Circ, Mark::Breve, Mark::Horn] {
            if let Some(row) = vowel_row(base, mark) {
                if let Some(t) = row.iter().position(|&x| x == c) {
                    return Some((base, mark, TONES[t]));
                }
            }
        }
    }
    (c == 'đ').then_some(('d', Mark::Stroke, Tone::None))
}

pub(crate) fn is_vowel_base(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Lowercase letter as it appears on screen, without tone.
pub(crate) fn letter_char(base: char, mark: Mark) -> char {
    match (base, mark) {
        ('d', Mark::Stroke) => 'đ',
        _ => vowel_row(base, mark).map_or(base, |r| r[0]),
    }
}

pub(crate) fn render_into(out: &mut String, base: char, mark: Mark, tone: Tone, upper: bool) {
    let c = match vowel_row(base, mark) {
        Some(row) => row[tone as usize],
        None => letter_char(base, mark),
    };
    if upper {
        out.extend(c.to_uppercase());
    } else {
        out.push(c);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_are_distinct_and_nfc_sized() {
        for (b, m) in [
            ('a', Mark::None),
            ('a', Mark::Breve),
            ('a', Mark::Circ),
            ('e', Mark::None),
            ('e', Mark::Circ),
            ('i', Mark::None),
            ('o', Mark::None),
            ('o', Mark::Circ),
            ('o', Mark::Horn),
            ('u', Mark::None),
            ('u', Mark::Horn),
            ('y', Mark::None),
        ] {
            let row = vowel_row(b, m).unwrap();
            for (i, c) in row.iter().enumerate() {
                assert!(!row[..i].contains(c), "duplicate in row {b}{m:?}");
            }
        }
    }

    #[test]
    fn upper_and_stroke() {
        let mut s = String::new();
        render_into(&mut s, 'd', Mark::Stroke, Tone::None, true);
        render_into(&mut s, 'o', Mark::Horn, Tone::Dot, true);
        assert_eq!(s, "ĐỢ");
    }
}
