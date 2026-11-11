//! So'z o'zbekchami yoki inglizchami — dalillar yig'indisi (musbat = o'zbekcha, manfiy = inglizcha).
//!
//! Manbalar: o'zbek korpusi chastotasi (Leipzig Corpora, CC BY), inglizcha so'zlar (SCOWL, ruxsat beruvchi
//! litsenziya), o'zbek qo'shimchalari bo'yicha o'zak qidirish va harf belgilari. Lug'atlar
//! `tools/build_lexicon.py` bilan quriladi va binarga kiritiladi.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::chars::fold_key;

static UZ_TSV: &str = include_str!("data/uz.tsv");
static EN_TSV: &str = include_str!("data/en.tsv");

fn parse(tsv: &'static str) -> HashMap<&'static str, f32> {
    let mut m = HashMap::with_capacity(tsv.len() / 10);
    for line in tsv.lines() {
        if line.starts_with('#') {
            continue;
        }
        if let Some((w, s)) = line.split_once('\t') {
            if let Ok(v) = s.trim().parse::<f32>() {
                m.insert(w, v);
            }
        }
    }
    m
}

fn uz() -> &'static HashMap<&'static str, f32> {
    static M: OnceLock<HashMap<&'static str, f32>> = OnceLock::new();
    M.get_or_init(|| parse(UZ_TSV))
}

fn en() -> &'static HashMap<&'static str, f32> {
    static M: OnceLock<HashMap<&'static str, f32>> = OnceLock::new();
    M.get_or_init(|| parse(EN_TSV))
}

static SPLIT_TSV: &str = include_str!("data/sh_split.tsv");

fn split() -> &'static HashMap<&'static str, f32> {
    static M: OnceLock<HashMap<&'static str, f32>> = OnceLock::new();
    M.get_or_init(|| parse(SPLIT_TSV))
}

/// Kirillda s|h alohida bo'lgan so'z (Исҳоқ → Ishoq): yozuvdagi "sh" ş ga aylanmasligi kerak.
pub fn is_sh_split(word: &str) -> bool {
    split().contains_key(fold_key(word).as_str())
}

/// Lug'atlarni oldindan yuklash (IME ishga tushganda, birinchi so'z kutib qolmasin).
pub fn warm_up() {
    let _ = uz().len() + en().len() + split().len();
}

// Eski lotin yozuvidagi qo'shimchalar — uzunidan qisqasiga. Lug'atda yo'q so'zning o'zagini topish uchun.
const SUFFIXES: &[&str] = &[
    "lardagi",
    "laridan",
    "larning",
    "laridagi",
    "imizning",
    "ingizning",
    "ayotgan",
    "yotgan",
    "yapti",
    "moqda",
    "lardan",
    "larida",
    "larini",
    "larga",
    "larda",
    "lari",
    "ning",
    "ngiz",
    "imiz",
    "ingiz",
    "dagi",
    "gani",
    "ganlar",
    "gan",
    "kan",
    "qan",
    "lik",
    "liq",
    "chi",
    "chilik",
    "siz",
    "dan",
    "tan",
    "lar",
    "lab",
    "lash",
    "ni",
    "ga",
    "ka",
    "qa",
    "da",
    "ta",
    "di",
    "ib",
    "ish",
    "moq",
    "mas",
    "may",
    "sa",
    "sin",
    "dir",
    "man",
    "san",
    "miz",
    "dek",
    "day",
    "cha",
    "roq",
    "gina",
    "li",
    "mi",
    "ku",
    "im",
    "ing",
    "si",
    "i",
];

fn uz_weight(key: &str) -> Option<f32> {
    if let Some(&w) = uz().get(key) {
        return Some(w);
    }
    // O'zak + qo'shimcha: kitob|larimizdan. Ikki qatlamgacha.
    let mut best: Option<f32> = None;
    for s1 in SUFFIXES {
        if let Some(stem) = key.strip_suffix(s1) {
            if stem.chars().count() < 3 {
                continue;
            }
            if let Some(&w) = uz().get(stem) {
                best = Some(best.map_or(w * 0.8, |b: f32| b.max(w * 0.8)));
                continue;
            }
            for s2 in SUFFIXES {
                if let Some(stem2) = stem.strip_suffix(s2) {
                    if stem2.chars().count() >= 3 {
                        if let Some(&w) = uz().get(stem2) {
                            best = Some(best.map_or(w * 0.6, |b: f32| b.max(w * 0.6)));
                        }
                    }
                }
            }
        }
    }
    best
}

/// Harflarning o'zi aytadigan dalil (lug'atdan tashqari so'zlar uchun).
fn letter_cues(key: &str) -> f32 {
    let c: Vec<char> = key.chars().collect();
    let mut s = 0.0f32;
    for i in 0..c.len() {
        let a = c[i];
        let b = c.get(i + 1).copied();
        match a {
            'w' => s -= 3.0,
            'c' if b != Some('h') => s -= 3.0, // o'zbek lotinida c faqat ch ichida
            'q' if b != Some('u') => s += 2.5,
            'x' => s += 0.5,
            _ => {}
        }
        match (a, b) {
            ('t', Some('h')) | ('p', Some('h')) | ('c', Some('k')) => s -= 2.0,
            ('e', Some('e')) | ('o', Some('o')) => s -= 1.5,
            ('e', Some('a')) | ('o', Some('u')) | ('a', Some('w')) => s -= 1.0,
            ('o' | 'g', Some('\'')) => s += 3.0,
            _ => {}
        }
    }
    if key.ends_with("tion") || key.ends_with("ness") || key.ends_with("ment") {
        s -= 2.0;
    }
    s
}

/// Inglizcha qisqartma yoki egalik: don't, we're, you'll, they've, Obama's, CNN's, I'm, he'd.
/// O'zbek so'zi tutuq + s / re / ll / ve bilan tugamaydi. 'm, 'd — faqat o'zagi inglizcha bo'lsa (ta'm o'zbekcha).
fn is_english_contraction(key: &str) -> bool {
    let always = ["n't", "'re", "'ve", "'ll", "'s"].iter().any(|s| {
        key.strip_suffix(s)
            .is_some_and(|stem| stem.chars().count() >= 1)
    });
    always
        || ["'m", "'d"].iter().any(|s| {
            key.strip_suffix(s)
                .is_some_and(|stem| en().contains_key(stem) && uz_weight(stem).is_none())
        })
}

/// So'z bo'yicha dalil: >0 o'zbekcha, <0 inglizcha, 0 — bilmaymiz.
pub fn evidence(word: &str) -> f32 {
    let key = fold_key(word);
    if key.is_empty() {
        return 0.0;
    }
    if is_english_contraction(&key) {
        return -6.0;
    }
    let en_w = en().get(key.as_str()).copied();
    let uz_w = uz_weight(&key);
    // Apostrofli inglizcha so'z (o'clock, ma'am, rock'n'roll) — o'/tutuq bo'lsa ham o'zbekcha emas.
    if en_w.is_some() && uz_w.is_none() && key.contains('\'') {
        return -6.0;
    }
    match (uz_w, en_w) {
        (Some(u), Some(e)) => u - e,
        (Some(u), None) => u + letter_cues(&key).min(0.0) * 0.5,
        (None, Some(e)) => -e,
        (None, None) => letter_cues(&key),
    }
}

/// Lug'atda bormi (test va CLI uchun).
pub fn known(word: &str) -> (bool, bool) {
    let key = fold_key(word);
    (uz_weight(&key).is_some(), en().contains_key(key.as_str()))
}
