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
        if let Some((w, s)) = line.split_once('\t')
            && let Ok(v) = s.trim().parse::<f32>()
        {
            m.insert(w, v);
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
                if let Some(stem2) = stem.strip_suffix(s2)
                    && stem2.chars().count() >= 3
                    && let Some(&w) = uz().get(stem2)
                {
                    best = Some(best.map_or(w * 0.6, |b: f32| b.max(w * 0.6)));
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
    // "i" o'zbekchada ham bor (kirill «и»), lekin I'm / I'd — faqat inglizcha.
    always
        || key == "i'm"
        || key == "i'd"
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
    let uz_w = uz_weight(&key).map(|w| w.min(UZ_EVIDENCE_CAP));
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

/// Til dalilida o'zbek og'irligining shifti (lug'atda 6.0 gacha — chastota nisbatlari uchun).
const UZ_EVIDENCE_CAP: f32 = 4.5;

/// Ikkala shakli ham haqiqiy va tez-tez uchraydigan so'zlar: apostrofsiz yozilgan bo'lsa — taxmin qilinmaydi.
/// (qol "to'xta" / qo'l, oz / o'z, yoq / yo'q, otdi / o'tdi, tort / to'rt, som / so'm ...)
const NEVER_RESTORE: &[&str] = &[
    "qol", "qoling", "oz", "yoq", "on", "ot", "otdi", "otadi", "otib", "otgan", "otish", "otar",
    "otishi", "otganini", "otuvchi", "tort", "som", "bor", "tola", "toy", "top", "toq", "qoya",
    "otin", "oqi", "sol", "oshda", "bolaman", "bolamiz", "oldirdi",
];

/// Shu og'irlikdan yuqori so'z o'zi to'g'ri — apostrof qo'shilmaydi. Pastrog'i ko'pincha kirill korpusdagi
/// xato yozuv (кора, оша, огир, гарб) — uni nomzod bilan solishtiramiz.
const RESTORE_KNOWN: f32 = 3.3;
/// Nomzod asl so'zdan shuncha kuchli bo'lsin (log10 shkala: ~30 baravar ko'p uchraydi).
const RESTORE_MARGIN: f32 = 1.5;
/// Apostrofsiz shakl ham lug'atda bor, nomzod o'zak orqali taxmin: ~45 baravar ko'p uchrashi kerak
/// (1.5 da "oshganlar → o'shganlar" kabi xatolar chiqdi).
const RESTORE_MARGIN_BOTH: f32 = 1.65;
/// Ikkala shakl ham korpusda o'zi uchraydi (ogir/og'ir, garb/g'arb): ~16 baravar ko'p bo'lsa kifoya.
/// Ikkalasi ham haqiqiy so'z bo'lgan juftlar (toy/to'y, top/to'p) — `NEVER_RESTORE` da.
const RESTORE_MARGIN_DIRECT: f32 = 1.2;
/// Inglizcha so'z (song, bog, bug) faqat o'zbekcha gap ichida va nomzod keng tarqalgan bo'lsa tiklanadi.
const RESTORE_EN_MIN: f32 = 3.0;
/// Asl shakl korpusda o'zi uchramaydi — faqat o'zak orqali taxmin (organ|ib, soz|lar), nomzod esa o'zi uchraydi
/// (o'rganib, so'zlar): ~6 baravar ko'p uchrashi kifoya.
const RESTORE_MARGIN_STEM: f32 = 0.8;
/// O'zak orqali taxmin qilingan asl shakl shundan kuchli bo'lsa — tegilmaydi.
const RESTORE_KNOWN_STEM: f32 = 3.2;
/// Nomzodning eng kichik og'irligi (siyrak so'zlarga taxmin qilmaymiz).
const RESTORE_MIN: f32 = 2.0;
/// o/g o'rinlari ko'pi bilan (2^6 = 64 nomzod).
const RESTORE_MAX_SLOTS: usize = 6;

/// Tushib qolgan apostrofni tiklash: "ozbek" → "o'zbek", "Togri" → "To'g'ri", "ogil" → "o'g'il".
/// Asl so'z o'zi ma'lum bo'lsa (ozod, bor) — None. Inglizcha so'z (song) faqat `uz_context` da (gap aniq
/// o'zbekcha: "Bundan song keldim"). Harf registri saqlanadi.
pub fn restore_apostrophes(word: &str, uz_context: bool) -> Option<String> {
    let key = fold_key(word);
    let kc: Vec<char> = key.chars().collect();
    let wc: Vec<char> = word.chars().collect();
    if kc.len() != wc.len() || kc.len() < 3 || NEVER_RESTORE.contains(&key.as_str()) {
        return None;
    }
    let english = en().contains_key(key.as_str());
    if english && !uz_context {
        return None;
    }
    let w0 = uz_weight(&key).unwrap_or(0.0);
    let direct0 = uz().contains_key(key.as_str());
    if w0
        >= if direct0 {
            RESTORE_KNOWN
        } else {
            RESTORE_KNOWN_STEM
        }
    {
        return None;
    }
    let slots: Vec<usize> = (0..kc.len())
        .filter(|&i| matches!(kc[i], 'o' | 'g') && kc.get(i + 1) != Some(&'\''))
        .collect();
    if slots.is_empty() || slots.len() > RESTORE_MAX_SLOTS {
        return None;
    }
    let mut best: Option<(f32, u32, u32, bool)> = None; // (og'irlik, -qo'shimchalar soni, mask, o'zi uchraydimi)
    let mut cand = String::with_capacity(key.len() + slots.len());
    for mask in 1u32..(1 << slots.len()) {
        cand.clear();
        let mut si = 0;
        for (i, &c) in kc.iter().enumerate() {
            cand.push(c);
            if si < slots.len() && slots[si] == i {
                if mask & (1 << si) != 0 {
                    cand.push('\'');
                }
                si += 1;
            }
        }
        if let Some(w) = uz_weight(&cand) {
            let fewer = u32::MAX - mask.count_ones();
            if best.is_none_or(|(bw, bf, _, _)| w > bw || (w == bw && fewer > bf)) {
                best = Some((w, fewer, mask, uz().contains_key(cand.as_str())));
            }
        }
    }
    let (w, _, mask, cand_direct) = best?;
    // Ikkala shakl ham haqiqiy so'z (qol/qo'l, otdi/o'tdi, yoq/yo'q) — juda katta farq bo'lsagina.
    let margin = if w0 <= 0.0 {
        RESTORE_MARGIN
    } else if !direct0 && cand_direct {
        RESTORE_MARGIN_STEM
    } else if direct0 && cand_direct {
        RESTORE_MARGIN_DIRECT
    } else {
        RESTORE_MARGIN_BOTH
    };
    let min = if english { RESTORE_EN_MIN } else { RESTORE_MIN };
    if w < min || w < w0 + margin {
        return None;
    }
    let mut out = String::with_capacity(word.len() + slots.len());
    let mut si = 0;
    for (i, &c) in wc.iter().enumerate() {
        out.push(c);
        if si < slots.len() && slots[si] == i {
            if mask & (1 << si) != 0 {
                out.push('\'');
            }
            si += 1;
        }
    }
    Some(out)
}

/// Lug'atda bormi (test va CLI uchun).
pub fn known(word: &str) -> (bool, bool) {
    let key = fold_key(word);
    (uz_weight(&key).is_some(), en().contains_key(key.as_str()))
}
