use super::*;

/// Text left on screen once every word has ended.
fn run(cfg: Config, input: &str) -> String {
    let mut c = Composer::new(cfg);
    let mut out = String::new();
    for ch in input.chars() {
        if c.push(ch) == Push::Boundary {
            out.push_str(&c.finalize());
            out.push(ch);
            c.clear();
        }
    }
    out.push_str(&c.finalize());
    out
}

/// What the word shows while it is still being typed.
fn live(input: &str) -> String {
    let mut c = Composer::new(Config::default());
    for ch in input.chars() {
        assert_eq!(c.push(ch), Push::Accepted);
    }
    c.display().to_string()
}

fn telex(s: &str) -> String {
    run(Config::default(), s)
}

fn vni(s: &str) -> String {
    run(Config { method: Method::Vni, ..Config::default() }, s)
}

fn check(f: fn(&str) -> String, cases: &[(&str, &str)]) {
    let bad: Vec<_> = cases
        .iter()
        .filter_map(|(i, want)| {
            let got = f(i);
            (got != *want).then(|| format!("{i:?}: want {want:?}, got {got:?}"))
        })
        .collect();
    assert!(bad.is_empty(), "\n{}", bad.join("\n"));
}

#[test]
fn telex_basic_words() {
    check(
        telex,
        &[
            ("vieetj", "việt"),
            ("Vieetj", "Việt"),
            ("VIEETJ", "VIỆT"),
            ("nguoiwf", "người"),
            ("dduwowcj", "được"),
            ("dduwowngf", "đường"),
            ("thuowng", "thương"),
            ("huowng", "hương"),
            ("tieengs", "tiếng"),
            ("Vieetj Nam", "Việt Nam"),
            ("xin chaof cacs banj", "xin chào các bạn"),
            ("toois", "tối"),
            ("nhaan", "nhân"),
            ("ddi", "đi"),
            ("DDi", "Đi"),
            ("Dd", "Đ"),
            ("muwa", "mưa"),
            ("cuwf", "cừ"),
            ("quoocs", "quốc"),
            ("gias", "giá"),
            ("giaos", "giáo"),
            ("gif", "gì"),
            ("quas", "quá"),
            ("quyf", "quỳ"),
            ("khuyar", "khuỷa"),
            ("nghieeng", "nghiêng"),
            ("hoawcj", "hoặc"),
            ("loaij", "loại"),
            ("ngoaif", "ngoài"),
        ],
    );
}

#[test]
fn tone_placement_modern() {
    check(
        telex,
        &[
            ("hoaf", "hòa"),
            ("hoes", "hóe"),
            ("thuyr", "thủy"),
            ("toans", "toán"),
            ("khoer", "khỏe"),
            ("laays", "lấy"),
            ("muas", "múa"),
            ("mias", "mía"),
            ("tuis", "túi"),
            ("daus", "dáu"),
            ("xoayj", "xoạy"),
            ("luaanj", "luận"),
            ("buoons", "buốn"),
            ("ruowuj", "rượu"),
            ("yeeus", "yếu"),
            ("thuowr", "thuở"),
        ],
    );
}

#[test]
fn tone_placement_old_style() {
    let old = Config { tone_style: ToneStyle::Old, ..Config::default() };
    assert_eq!(run(old.clone(), "hoaf"), "hoà");
    assert_eq!(run(old.clone(), "thuyr"), "thuỷ");
    assert_eq!(run(old, "toans"), "toán");
}

#[test]
fn free_marking_reaches_back() {
    check(
        telex,
        &[
            ("tieengs", "tiếng"),
            ("vieetj", "việt"),
            ("viete", "viêt"),
            ("nguoiwf", "người"),
            ("tuoiws", "tưới"),
            ("hoasn", "hoán"),
        ],
    );
    let strict = Config { free_marking: false, ..Config::default() };
    assert_eq!(run(strict.clone(), "viete"), "viete");
    assert_eq!(run(strict, "vieet"), "viêt");
}

#[test]
fn undo_by_repeating_key() {
    for (typed, shown) in [
        ("aaa", "aa"),
        ("ass", "as"),
        ("aww", "aw"),
        ("ddd", "dd"),
        ("ww", "w"),
        ("uww", "uw"),
        ("oww", "ow"),
        ("vieets", "viết"),
        ("vieetss", "vieets"),
        ("asss", "ass"),
        ("aaaa", "aaa"),
    ] {
        assert_eq!(live(typed), shown, "{typed}");
    }
}

#[test]
fn ended_non_vietnamese_words_restore_typed_keys() {
    check(
        telex,
        &[
            ("class", "class"),
            ("off", "of"),
            ("error", "eror"),
            ("press", "press"),
            ("address", "address"),
            ("www.example.com", "ww.example.com"),
            ("pass word", "pas word"),
            ("ass", "as"),
            ("aaa", "aa"),
            ("ddd", "dd"),
            ("vieetss", "vieets"),
        ],
    );
}

#[test]
fn remove_tone_with_z() {
    check(telex, &[("vieetjz", "viêt"), ("az", "az"), ("tez", "tez")]);
}

#[test]
fn standalone_w_is_u_horn() {
    check(telex, &[("w", "ư"), ("W", "Ư"), ("nw", "nư"), ("thw", "thư"), ("wf", "ừ")]);
}

#[test]
fn english_words_are_left_alone() {
    check(
        telex,
        &[
            ("windows", "windows"),
            ("text", "text"),
            ("user", "user"),
            ("install", "install"),
            ("sudo apt update", "sudo apt update"),
            ("show", "show"),
            ("add", "add"),
            ("address", "address"),
            ("function", "function"),
            ("class", "class"),
            ("error", "eror"),
            ("export", "export"),
            ("first", "first"),
            ("press", "press"),
            ("software", "software"),
            ("git push origin main", "git push origin main"),
            ("cargo build --release", "cargo build --release"),
            ("README.md", "README.md"),
            ("http://localhost:8080", "http://localhost:8080"),
        ],
    );
}

#[test]
fn stop_finals_reject_wrong_tones() {
    check(telex, &[("catf", "catf"), ("cats", "cát"), ("tacj", "tạc"), ("tapr", "tapr")]);
}

#[test]
fn boundaries_reset_the_word() {
    check(
        telex,
        &[
            ("chaof, banj.", "chào, bạn."),
            ("aas aas", "ấ ấ"),
            ("ddi hocj", "đi học"),
            ("(vieetj)", "(việt)"),
            ("a1b2", "a1b2"),
        ],
    );
}

#[test]
fn uppercase_variants() {
    check(telex, &[("NGUOIWF", "NGƯỜI"), ("DDUWOWCJ", "ĐƯỢC"), ("Nguoiwf", "Người")]);
}

#[test]
fn vni_basic_words() {
    check(
        vni,
        &[
            ("viet65", "việt"),
            ("Viet65", "Việt"),
            ("d9uoc75", "được"),
            ("ngu7o7i2", "người"),
            ("tie6ng1", "tiếng"),
            ("d9i", "đi"),
            ("hoa2", "hòa"),
            ("a6", "â"),
            ("a8", "ă"),
            ("o7", "ơ"),
            ("u7", "ư"),
            ("tuoi7", "tươi"),
            ("a66", "a6"),
            ("a11", "a1"),
            ("vie6n2", "viền"),
            ("viet650", "viêt"),
            ("viet652", "viet652"),
        ],
    );
}

#[test]
fn vni_digits_outside_words_pass_through() {
    check(vni, &[("2024", "2024"), ("x2", "x2"), ("abc 123", "abc 123"), ("100d9", "100đ")]);
}

#[test]
fn backspace_keeps_remaining_composition() {
    let mut c = Composer::new(Config::default());
    for ch in "vieetj".chars() {
        c.push(ch);
    }
    assert_eq!(c.display(), "việt");
    c.backspace();
    assert_eq!(c.display(), "việ");
    c.backspace();
    assert_eq!(c.display(), "vi", "tone goes away with the vowel that carried it");
    c.backspace();
    c.backspace();
    assert!(c.is_empty());
    assert!(!c.backspace());
}

#[test]
fn backspace_after_undo_does_not_resurrect_mark() {
    let mut c = Composer::new(Config::default());
    for ch in "aaa".chars() {
        c.push(ch);
    }
    assert_eq!(c.display(), "aa");
    c.backspace();
    assert_eq!(c.display(), "a");
}

#[test]
fn plain_reverts_transformations() {
    let mut c = Composer::new(Config::default());
    for ch in "vieetj".chars() {
        c.push(ch);
    }
    assert_eq!(c.plain(), "vieetj");
    let mut c = Composer::new(Config::default());
    for ch in "ass".chars() {
        c.push(ch);
    }
    assert_eq!(c.plain(), "as");
}

#[test]
fn diff_is_minimal() {
    assert_eq!(diff("viet", "viét"), (2, "ét".into()));
    assert_eq!(diff("hoa", "hóa"), (2, "óa".into()));
    assert_eq!(diff("tẽ", "text"), (1, "ext".into()));
    assert_eq!(diff("a", "a"), (0, String::new()));
    assert_eq!(diff("việt", "việ"), (1, String::new()));
}

#[test]
fn spell_check_off_always_applies() {
    let cfg = Config { spell_check: false, ..Config::default() };
    assert_eq!(run(cfg, "text"), "tẽt");
}

#[test]
fn o_after_horned_u_makes_the_pair_without_a_second_w() {
    assert_eq!(live("dduwocj"), "được");
    assert_eq!(live("duwoc"), "dươc");
    // The habitual second `w` confirms the pair instead of undoing it.
    assert_eq!(live("thuwowng"), "thương");
    assert_eq!(live("nguwowif"), "người");
    assert_eq!(live("thuwongf"), "thường");
}

#[test]
fn backspace_after_a_toned_word_removes_one_visible_letter() {
    let mut c = Composer::new(Config::default());
    for ch in "tieengs".chars() {
        c.push(ch);
    }
    assert_eq!(c.display(), "tiếng");
    c.backspace();
    assert_eq!(c.display(), "tiến");
    c.backspace();
    assert_eq!(c.display(), "tiế");
}

#[test]
fn tone_typed_after_circumflex_keeps_its_place() {
    assert_eq!(live("loox"), "lỗ");
    assert_eq!(live("looxi"), "lỗi");
    assert_eq!(live("looix"), "lỗi");
}

#[test]
fn third_mark_key_undoes_and_the_word_ends_as_shown() {
    for (typed, want) in [("eee", "ee"), ("aaa", "aa"), ("ddd", "dd"), ("ooo", "oo")] {
        assert_eq!(run(Config::default(), typed), want, "{typed}");
        assert_eq!(live(typed), want, "live {typed}");
    }
    // Like Unikey and macOS: a doubled tone key undoes the tone and types the letter once.
    assert_eq!(run(Config::default(), "off error class"), "of eror class");
    assert_eq!(live("conff"), "conf");
}

#[test]
fn backspace_after_undone_mark_does_not_apply_it_again() {
    let after = |typed: &str, deletes: usize| {
        let mut c = Composer::new(Config::default());
        for ch in typed.chars() {
            c.push(ch);
        }
        for _ in 0..deletes {
            c.backspace();
        }
        c.display().to_string()
    };
    assert_eq!(after("eeee", 0), "eee");
    assert_eq!(after("eeee", 1), "ee");
    assert_eq!(after("eeee", 2), "e");
    assert_eq!(after("eee", 1), "e");
    assert_eq!(after("aaaa", 1), "aa");
    assert_eq!(after("conffil", 2), "conf");
    assert_eq!(after("offf", 1), "of");
}

#[test]
fn circumflex_reaches_the_matching_vowel_behind_a_glide() {
    assert_eq!(live("baay"), "bây");
    assert_eq!(live("baya"), "bây");
    assert_eq!(live("caua"), "câu");
    // Not a syllable with the mark, so the typed keys stay.
    assert_eq!(live("khoeo"), "khoeo");
    assert_eq!(live("hoaa"), "hoaa");
}

#[test]
fn stroke_reaches_back_over_the_vowels() {
    assert_eq!(live("dod"), "đo");
    assert_eq!(live("dad"), "đa");
    assert_eq!(live("dodo"), "đô");
    assert_eq!(live("dduwocj"), "được");
    // Not a syllable with the stroke, so the typed keys stay.
    assert_eq!(live("dead"), "dead");
    // Doubling the key again undoes it, like every other mark.
    assert_eq!(live("dodd"), "dod");
    let strict = Config { free_marking: false, ..Config::default() };
    assert_eq!(run(strict, "dod"), "dod");
}
