//! Yozish paytidagi (IME) o'girish: so'zlar birma-bir, kontekst faqat oldingi so'zlar.

use bitg_core::{Options, Session};

fn typed(words: &[&str]) -> Vec<String> {
    let opts = Options::default();
    let mut s = Session::new();
    words
        .iter()
        .map(|w| {
            s.complete_word(w, &opts)
                .map(|e| e.insert)
                .unwrap_or_else(|| w.to_string())
        })
        .collect()
}

#[test]
fn missing_apostrophe_is_restored_while_typing() {
    assert_eq!(typed(&["qoshimcha"]), ["qöşimça"]);
    assert_eq!(typed(&["shunday", "qoshimcha"]), ["şunday", "qöşimça"]);
    assert_eq!(typed(&["şunday", "qoshimcha"]), ["şunday", "qöşimça"]);
    assert_eq!(typed(&["ozbekcha", "togri"]), ["özbekça", "töğri"]);
}

#[test]
fn uzbek_word_after_english_word_still_converts_while_typing() {
    assert_eq!(typed(&["check", "shapaloq"]), ["check", "şapaloq"]);
    assert_eq!(typed(&["ok", "choynak"]), ["ok", "çoynak"]);
    // Inglizcha gap davom etsa — o'zgarmaydi
    assert_eq!(typed(&["check", "the", "shop"])[2], "shop");
}

#[test]
fn english_words_and_brands_stay_while_typing() {
    for w in [
        "chrome",
        "Chrome",
        "shell",
        "Photoshop",
        "Photoshopda",
        "Chevrolet",
        "Chevroletni",
        "Toshiba",
    ] {
        assert_eq!(typed(&[w]), [w], "{w} alone");
        assert_eq!(typed(&["men", w, "ochdim"])[1], w, "{w} in a sentence");
    }
}
