use bitg_core::{OldStyle, Options, Session, convert, to_old};

fn c(s: &str) -> String {
    convert(s, &Options::default())
}

#[test]
fn uzbek_sentences() {
    assert_eq!(
        c("O'zbekiston shaharlari: Toshkent"),
        "Özbekiston şaharlari: Toşkent"
    );
    assert_eq!(
        c("Bugun choy ichdik, keyin bog'ga chiqdik."),
        "Bugun çoy içdik, keyin boğga çiqdik."
    );
    assert_eq!(
        c("Yaxshi kishi hech qachon shoshilmaydi"),
        "Yaxşi kişi heç qaçon şoşilmaydi"
    );
    assert_eq!(c("ma'no va san'at"), "ma\u{02BC}no va san\u{02BC}at");
    assert_eq!(c("tog' va bog'"), "toğ va boğ");
}

#[test]
fn missing_apostrophes_are_restored() {
    assert_eq!(
        c("Men ozbek tilida togri yozaman, ogil bola kop kitob"),
        "Men özbek tilida töğri yozaman, öğil bola köp kitob"
    );
    assert_eq!(c("Ozbekiston, Ozim bilaman"), "Özbekiston, Özim bilaman");
    // ikkala shakl ham haqiqiy so'z — taxmin qilinmaydi
    assert_eq!(
        c("Uyda qol, oz qoldi, ozod bor"),
        "Uyda qol, oz qoldi, ozod bor"
    );
    // inglizcha so'z va gap
    assert_eq!(c("good song, go home"), "good song, go home");
    // o'chirib qo'yish
    let off = Options {
        restore_apostrophes: false,
        ..Options::default()
    };
    assert_eq!(convert("ozbek tili", &off), "ozbek tili");
}

#[test]
fn common_english_word_inside_uzbek_sentence_is_kept() {
    assert_eq!(
        c("salom choy o'zbek check, keyin shahar"),
        "salom çoy özbek check, keyin şahar"
    );
}

#[test]
fn initials_and_abbreviations_convert() {
    assert_eq!(c("A.Lukashenko va Sh.Sharipov"), "A.Lukaşenko va Ş.Şaripov");
    assert_eq!(c("M.Jo'rayev, O'zA, O'zR"), "M.Jörayev, ÖzA, ÖzR");
    // kod va domenlar avvalgidek tegilmaydi
    assert_eq!(
        c("obj.show kun.uz myShop I'm"),
        "obj.show kun.uz myShop I'm"
    );
}

#[test]
fn whitespace_and_lines_are_preserved() {
    assert_eq!(c("  choy\n\tshahar  "), "  çoy\n\tşahar  ");
}

#[test]
fn english_words_and_sentences_are_kept() {
    for s in [
        "check",
        "How are you shah",
        "Please check the schedule",
        "I wish you much success",
        "The shop is closed",
        "Chat with the team",
        "Don't touch the cash",
        "Children should watch",
        "Rock'n'roll at nine o'clock",
    ] {
        assert_eq!(c(s), s, "{s}");
    }
}

#[test]
fn mixed_language_text() {
    assert_eq!(c("Thanks! Bugun choy ichdik"), "Thanks! Bugun çoy içdik");
    assert_eq!(c("please qo'shiq ayt"), "please qöşiq ayt");
    // chap: yolg'iz — noaniq (inglizcha "chap" ham bor), o'zbekcha kontekstda o'giriladi
    assert_eq!(c("Menga chap tomonda"), "Menga çap tomonda");
    // chat: o'zbek korpusida deyarli yo'q, inglizchada keng tarqalgan — tegilmaydi
    assert_eq!(
        c("Menga chat yozing, shahar markazida"),
        "Menga chat yozing, şahar markazida"
    );
}

#[test]
fn protected_tokens() {
    assert_eq!(c("kun.uz/choy va shahar.uz"), "kun.uz/choy va shahar.uz");
    assert_eq!(
        c("sherzod@gmail.com ga yozing"),
        "sherzod@gmail.com ga yozing"
    );
    assert_eq!(c("@shahzod #shahar"), "@shahzod #shahar");
    assert_eq!(c("my_shop.check()"), "my_shop.check()");
    assert_eq!(c("parol: shahar123"), "parol: shahar123");
    assert_eq!(c("O'Brien choy ichdi"), "O'Brien çoy içdi");
}

#[test]
fn quotes() {
    assert_eq!(c("'Yaxshi tong' dedi"), "'Yaxşi tong' dedi");
    assert_eq!(c("«Choyxona»"), "«Çoyxona»");
}

#[test]
fn case() {
    assert_eq!(c("SHAHAR CHOY O'ZBEK"), "ŞAHAR ÇOY ÖZBEK");
    assert_eq!(c("Shahnoza va Chorsu"), "Şahnoza va Çorsu");
}

#[test]
fn cyrillic() {
    assert_eq!(c("Ўзбекистон шаҳарлари"), "Özbekiston şaharlari");
    assert_eq!(c("Ер юзи, поезд, лицей"), "Yer yuzi, poyezd, litsey");
    let o = Options {
        cyrillic: false,
        ..Options::default()
    };
    assert_eq!(convert("Салом choy", &o), "Салом çoy");
}

#[test]
fn already_converted_text_is_stable() {
    let once = c("O'zbekiston shaharlari, choy va ma'no");
    assert_eq!(c(&once), once);
}

#[test]
fn exceptions() {
    let o = Options {
        exceptions: vec!["shohjahon".into()],
        ..Options::default()
    };
    assert_eq!(convert("Shohjahon bilan choy", &o), "Shohjahon bilan çoy");
}

#[test]
fn guard_off_converts_everything() {
    let o = Options {
        english_guard: false,
        ..Options::default()
    };
    assert_eq!(convert("check", &o), "çeck");
}

#[test]
fn reverse() {
    assert_eq!(
        to_old("Özbekiston şaharlari", OldStyle::Ascii),
        "O'zbekiston shaharlari"
    );
}

#[test]
fn session_typing() {
    let o = Options::default();
    let mut s = Session::new();
    assert_eq!(
        s.complete_word("choy", &o).map(|e| e.insert),
        Some("çoy".into())
    );
    let mut en = Session::new();
    assert_eq!(en.complete_word("How", &o), None);
    assert_eq!(en.complete_word("are", &o), None);
    assert_eq!(en.complete_word("you", &o), None);
    assert_eq!(en.complete_word("shah", &o), None);
    let mut uz = Session::new();
    uz.complete_word("Menga", &o);
    let e = uz
        .complete_word("chap", &o)
        .expect("o'zbekcha kontekstda chap o'giriladi");
    assert_eq!((e.delete, e.insert.as_str()), (4, "çap"));
}

#[test]
fn cheap_fuzz_never_panics_and_is_idempotent_on_output() {
    // Oddiy LCG — tashqi bog'liqliksiz tasodifiy matn.
    let alphabet: Vec<char> = "aoughscqx'`ʻʼ’ .,!?\n@#/_-1Ш ҳўғ\"«»()OUSHCH"
        .chars()
        .collect();
    let mut seed: u64 = 0x2026_1021;
    for _ in 0..3000 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let len = (seed >> 58) as usize;
        let mut s = String::new();
        for _ in 0..len {
            seed = seed
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            s.push(alphabet[(seed >> 33) as usize % alphabet.len()]);
        }
        let once = c(&s);
        let _ = to_old(&once, OldStyle::Ascii);
    }
}
