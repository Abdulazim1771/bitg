//! # bitg-core
//!
//! O'zbek matnini 2026-yilgi lotin alifbosiga (O'RQ-1184: ö ğ ş ç, tutuq ʼ) o'girish yadrosi.
//! Tarmoqqa chiqmaydi, fayl yozmaydi, holat saqlamaydi — IME, tray ilova, CLI, mobil klaviatura va web
//! (WASM) bir xil natija beradi.
//!
//! ```
//! let out = bitg_core::convert("O'zbekiston shaharlari: Toshkent", &bitg_core::Options::default());
//! assert_eq!(out, "Özbekiston şaharlari: Toşkent");
//! ```

pub mod chars;
pub mod cyrillic;
pub mod ffi;
pub mod lang;
pub mod latin;
mod lexicon;
pub mod reverse;
pub mod token;
#[cfg(target_arch = "wasm32")]
pub mod wasm;

pub use reverse::OldStyle;

use chars::{is_apostrophe, normalize};

/// O'girish sozlamalari.
#[derive(Clone, Debug)]
pub struct Options {
    /// Chiqishdagi tutuq belgisi (standart U+02BC).
    pub tutuq: char,
    /// Inglizcha so'z va gaplarni himoya qilish (check, "How are you shah").
    pub english_guard: bool,
    /// Kirill harfli so'zlarni ham yangi lotinga o'girish.
    pub cyrillic: bool,
    /// Hech qachon o'girilmaydigan so'zlar (kichik harfda, tutuq `'` ko'rinishida — `chars::fold_key`).
    pub exceptions: Vec<String>,
    /// Tushib qolgan apostrofni lug'atdan tiklash: "ozbek" → "özbek", "togri" → "töğri".
    pub restore_apostrophes: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            tutuq: chars::DEFAULT_TUTUQ,
            english_guard: true,
            cyrillic: true,
            exceptions: Vec::new(),
            restore_apostrophes: true,
        }
    }
}

impl Options {
    fn is_exception(&self, core: &str) -> bool {
        !self.exceptions.is_empty() && {
            let k = chars::fold_key(core);
            self.exceptions.contains(&k)
        }
    }
}

/// So'z uchun qaror: o'girilsinmi va nega (CLI va IME log'i uchun).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reason {
    Converted,
    Unchanged,
    Protected,
    Exception,
    English,
}

/// Bitta tokenning (bo'shliqsiz bo'lak) tahlili.
#[derive(Clone, Debug)]
struct Tok {
    start: usize,
    end: usize,
    evidence: f32,
    sentence_end: bool,
    /// Vergul, nuqtali vergul, ikki nuqta bilan tugaydi — gap bo'lagi chegarasi ("..., keyin choy").
    clause_end: bool,
}

const SENTENCE_END: &[char] = &['.', '!', '?', '…'];
const CLAUSE_END: &[char] = &[',', ';', ':'];

fn tokens(text: &str) -> Vec<Tok> {
    let mut v = Vec::new();
    let mut start: Option<usize> = None;
    for (i, c) in text.char_indices() {
        // Tire va ko'p nuqta ham so'z chegarasi: "doesn't—and", "hitter…he's".
        if c.is_whitespace() || matches!(c, '—' | '–' | '…') {
            if let Some(s) = start.take() {
                v.push(Tok {
                    start: s,
                    end: i,
                    evidence: 0.0,
                    sentence_end: false,
                    clause_end: false,
                });
            }
            if (c == '\n' || c == '…')
                && let Some(last) = v.last_mut()
            {
                last.sentence_end = true;
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        v.push(Tok {
            start: s,
            end: text.len(),
            evidence: 0.0,
            sentence_end: false,
            clause_end: false,
        });
    }
    for t in &mut v {
        let tok = &text[t.start..t.end];
        let trimmed = tok.trim_end_matches(|c: char| {
            matches!(c, '"' | '»' | '”' | ')' | ']') || is_apostrophe(c)
        });
        if trimmed.ends_with(SENTENCE_END) {
            t.sentence_end = true;
        }
        if trimmed.ends_with(CLAUSE_END) {
            t.clause_end = true;
        }
    }
    v
}

/// Kontekst: gapdagi boshqa so'zlarning dalili (har biri ±3), yaqinroq so'z kuchliroq (1/masofa),
/// gap bo'lagi chegarasidan (vergul) o'tgan so'z 3 baravar kuchsiz: "Please check, keyin choy ichamiz".
fn context(toks: &[Tok], range: std::ops::Range<usize>, skip: usize) -> f32 {
    let slice = &toks[range];
    let mut sum = 0.0f32;
    let mut weights = 0.0f32;
    for (j, t) in slice.iter().enumerate() {
        if j == skip || t.evidence == 0.0 {
            continue;
        }
        let dist = j.abs_diff(skip) as f32;
        let (lo, hi) = if j < skip { (j, skip) } else { (skip, j) };
        let crosses = slice[lo..hi].iter().any(|x| x.clause_end);
        let w = (1.0 / dist) * if crosses { 0.3 } else { 1.0 };
        sum += w * t.evidence.clamp(-3.0, 3.0);
        weights += w;
    }
    if weights == 0.0 { 0.0 } else { sum / weights }
}

/// Kontekst qanchalik kuchli ta'sir qiladi. Ikki tilda ham bor so'z (chat, shop) shunga qarab hal bo'ladi.
const CONTEXT_WEIGHT: f32 = 1.5;

/// sh/ch bor, lekin o'/g'/tutuq yo'q so'z inglizcha bo'lishi mumkin — dalilga qaraymiz.
fn english_blocks(core: &str, own: f32, ctx: f32) -> bool {
    if !latin::digraph_only(core) {
        // o'/g' yoki tutuq — o'zbekcha belgi; faqat aniq inglizcha so'z (o'clock) yoki qisqartma to'xtatadi.
        return own <= -5.0;
    }
    // Keng tarqalgan inglizcha so'z, o'zbek lug'atida yo'q (check, shop, chat with) — o'zbekcha gap ichida ham.
    if own <= STRONG_ENGLISH && !lang::known(core).0 {
        return true;
    }
    own + CONTEXT_WEIGHT * ctx < 0.0
}

/// SCOWL 10/20 darajasidagi (eng ko'p uchraydigan) inglizcha so'zning dalili.
const STRONG_ENGLISH: f32 = -2.0;
/// Kontekst shundan inglizcharoq bo'lsa apostrof tiklanmaydi ("Gogol ozone ...").
const RESTORE_CTX_MIN: f32 = -1.0;
/// Inglizcha lug'atdagi so'zni (song → so'ng) tiklash uchun gap shunchalik aniq o'zbekcha bo'lsin.
const RESTORE_EN_CTX: f32 = 1.5;

/// Bitta tokenni o'giradi (kontekst bilan). `quote_open` — oldingi so'zlarda ochilgan tirnoq ('Yaxshi tong').
fn convert_token(
    tok: &str,
    opts: &Options,
    own: f32,
    ctx: f32,
    quote_open: &mut bool,
) -> (String, Reason) {
    if opts.cyrillic && cyrillic::has_cyrillic(tok) {
        let out = cyrillic::to_new_latin(tok, opts.tutuq);
        let reason = if out == tok {
            Reason::Unchanged
        } else {
            Reason::Converted
        };
        return (out, reason);
    }
    if token::is_protected(tok) {
        return (tok.to_string(), Reason::Protected);
    }
    let parts = token::split_edges(tok);
    let lead = parts.lead;
    let mut core = parts.core;
    let mut trail = parts.trail;

    // Tirnoq: boshida apostrof — ochiladi; ochiq tirnoqli so'z apostrof bilan tugasa — bu yopuvchi tirnoq.
    let opens = lead.chars().any(is_apostrophe);
    if (opens || *quote_open) && core.chars().last().is_some_and(is_apostrophe) {
        let cut = core
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(core.len());
        let full_trail_start = parts.lead.len() + cut;
        trail = &tok[full_trail_start..];
        core = &core[..cut];
        *quote_open = false;
    } else if opens {
        *quote_open = true;
    }
    if trail.chars().any(is_apostrophe) {
        *quote_open = false;
    }

    // Chiziqchali so'z qism-qism: C-uchli → C-uçli, MR-shartli → MR-şartli, CH-III o'zgarmaydi.
    if core.contains('-')
        && !core.starts_with('-')
        && !core.ends_with('-')
        && core
            .split('-')
            .any(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_uppercase()))
    {
        return convert_compound(lead, core, trail, opts, ctx);
    }
    // Unlisiz katta harfli qisqartma (TSH, PTCH, CH) va o'zbekchada yo'q harfli so'z (Pöschl, Müller).
    if is_acronym(core) || has_foreign_letters(core) {
        return (tok.to_string(), Reason::Unchanged);
    }
    // Brend nomi (Photoshop, Chevroletda) — savdo belgisi asl yozuvida qoladi.
    if opts.english_guard && lang::is_brand(core) {
        return (tok.to_string(), Reason::Protected);
    }

    // Tushib qolgan apostrof (ozbek → o'zbek). Inglizcha gap ichida emas; istisno so'zlarga tegilmaydi.
    let restored;
    if opts.restore_apostrophes
        && ctx > RESTORE_CTX_MIN
        && !opts.is_exception(core)
        && let Some(r) = lang::restore_apostrophes(core, ctx >= RESTORE_EN_CTX)
    {
        restored = r;
        core = &restored;
    }

    // Harfning o'zi: "O' harfi", "G' yoki O'" — inglizcha gapda bo'lmasa o'giriladi (tirnoq yuqorida ajratilgan).
    if lead.is_empty() && ctx > RESTORE_CTX_MIN && is_lone_letter(core) && !opts.is_exception(core)
    {
        let new_core = latin::to_new(core, opts.tutuq);
        if new_core != core {
            return (format!("{lead}{new_core}{trail}"), Reason::Converted);
        }
    }
    // Boshqa 2 belgili apostrofli bo'lak (rock 'n' roll, d') — so'z emas, tirnoq yoki qisqartma.
    if core.is_empty()
        || core.chars().count() <= 2 && core.chars().any(is_apostrophe)
        || !latin::has_convertible(core)
    {
        return (tok.to_string(), Reason::Unchanged);
    }
    if opts.is_exception(core) {
        return (tok.to_string(), Reason::Exception);
    }
    if opts.english_guard
        && (english_blocks(core, own, ctx) || english_with_suffix(core) || german_sch(core))
    {
        return (tok.to_string(), Reason::English);
    }
    let new_core = if lang::is_sh_split(core) || is_dis_h(core) {
        latin::to_new_split(core, opts.tutuq)
    } else {
        latin::to_new(core, opts.tutuq)
    };
    if new_core == core {
        return (tok.to_string(), Reason::Unchanged);
    }
    let mut out = String::with_capacity(tok.len());
    out.push_str(lead);
    out.push_str(&new_core);
    out.push_str(trail);
    (out, Reason::Converted)
}

/// Chiziqcha bilan ajralgan bo'laklar alohida qaror oladi; natija — eng kuchli sabab.
fn convert_compound(
    lead: &str,
    core: &str,
    trail: &str,
    opts: &Options,
    ctx: f32,
) -> (String, Reason) {
    let mut out = String::with_capacity(core.len() + lead.len() + trail.len() + 4);
    out.push_str(lead);
    let mut reason = Reason::Unchanged;
    for (i, part) in core.split('-').enumerate() {
        if i > 0 {
            out.push('-');
        }
        let own = if opts.english_guard {
            lang::evidence(part)
        } else {
            0.0
        };
        // Qisqartmali birikma (C-uchli) atrofidagi lotin harflar gapni "inglizcha" qilib ko'rsatadi — e'tiborsiz.
        let (conv, r) = convert_token(part, opts, own, ctx.max(0.0), &mut false);
        out.push_str(&conv);
        reason = match (reason, r) {
            (_, Reason::Converted) | (Reason::Converted, _) => Reason::Converted,
            (Reason::Unchanged, r) => r,
            (r, _) => r,
        };
    }
    out.push_str(trail);
    (out, reason)
}

/// Lotincha qisqartma: unlisiz, SH/CH bilan tugaydi (TSH, PTCH) yoki faqat "CH"/"SH".
/// O'zbekcha qisqartmalar (MCHJ, KXSHT, SHHT, AQSH) o'giriladi.
fn is_acronym(core: &str) -> bool {
    core.chars().count() >= 2
        && core.chars().all(|c| c.is_ascii_uppercase())
        && !core
            .chars()
            .any(|c| matches!(c, 'A' | 'E' | 'I' | 'O' | 'U'))
        && (core.len() == 2 || core.ends_with("SH") || core.ends_with("CH"))
}

/// O'zbek alifbosida yo'q harf (Müller, Señor) yoki ö bilan nemischa "sch" (Pöschl, Tröltsch).
/// ö ning o'zi yangi alifbo harfi: "köcha" (ö tugmadan yozilgan) o'giriladi.
fn has_foreign_letters(core: &str) -> bool {
    // ë yo'q: eski o'zbek lotin matnlarida "yo" o'rnida keladi (ëzuvchi, ësh).
    const FOREIGN: &str = "äüßéèêáàâïîñøåæœúùûíìóòôãõýÄÜÉÈÊÁÀÂÏÎÑØÅÆŒÚÙÛÍÌÓÒÔÃÕÝ";
    if core.chars().any(|c| FOREIGN.contains(c)) {
        return true;
    }
    core.contains(['ö', 'Ö']) && core.to_lowercase().contains("sch")
}

/// Nemischa "sch" (Fischer, Schmidt). O'zbekchada s+ch faqat qo'shimchada (asoschi, bokschi) — ular lug'atda.
fn german_sch(core: &str) -> bool {
    let l = core.to_lowercase();
    l.contains("sch") && (l.starts_with("sch") || !lang::known(core).0)
}

/// dis + gormon (dishormonal, dishidroz, disharmoniya): s va h alohida.
fn is_dis_h(core: &str) -> bool {
    let l = core.to_lowercase();
    ["dishorm", "dishidr", "disharm"]
        .iter()
        .any(|p| l.starts_with(p))
}

/// O'zbek qo'shimchasi qo'shilgan inglizcha nom: Chromeda, Shellga, dashboardlar.
const UZ_SUFFIXES: &[&str] = &[
    "larning", "larni", "larga", "larda", "lardan", "dagi", "ning", "dan", "lar", "dir", "da",
    "ga", "ka", "ni",
];

fn english_with_suffix(core: &str) -> bool {
    // Lug'atdagi kichik harfli so'z (matchda) — o'zbekcha; bosh harfli nom (Shellga) — tekshiriladi.
    if lang::known(core).0 && !core.starts_with(|c: char| c.is_uppercase()) {
        return false;
    }
    let lower = core.to_lowercase();
    UZ_SUFFIXES.iter().any(|suf| {
        lower.strip_suffix(suf).is_some_and(|stem| {
            stem.chars().count() >= 4
                && latin::has_convertible(stem)
                && lang::evidence(stem) <= -2.0
        })
    })
}

/// `o'`, `G'` va h.k.: yolg'iz o/g harfi + apostrof.
fn is_lone_letter(core: &str) -> bool {
    let mut c = core.chars();
    matches!(
        (c.next(), c.next(), c.next()),
        (Some('o' | 'O' | 'g' | 'G'), Some(a), None) if is_apostrophe(a)
    )
}

/// Matnni yangi alifboga o'giradi. Bo'shliqlar va qator tuzilishi aynan saqlanadi.
pub fn convert(text: &str, opts: &Options) -> String {
    convert_explained(text, opts).0
}

/// O'girish va har bir token bo'yicha sabab (`bitg explain` uchun).
pub fn convert_explained(text: &str, opts: &Options) -> (String, Vec<(String, Reason, f32)>) {
    let text = normalize(text);
    let text = text.as_ref();
    let mut toks = tokens(text);
    for t in &mut toks {
        let core = token::split_edges(&text[t.start..t.end]).core;
        t.evidence = if opts.english_guard {
            lang::evidence(core)
        } else {
            0.0
        };
    }
    let mut out = String::with_capacity(text.len() + 16);
    let mut report = Vec::with_capacity(toks.len());
    let mut last = 0;
    let mut sent_start = 0;
    let mut quote_open = false;
    for i in 0..toks.len() {
        if i > 0 && toks[i - 1].sentence_end {
            sent_start = i;
        }
        let mut sent_end = i;
        while sent_end + 1 < toks.len() && !toks[sent_end].sentence_end {
            sent_end += 1;
        }
        let t = &toks[i];
        out.push_str(&text[last..t.start]);
        let ws = &text[last..t.start];
        if ws.contains('\n') {
            quote_open = false; // tirnoq qatordan qatorga o'tmaydi
        }
        let ctx = context(&toks, sent_start..sent_end + 1, i - sent_start);
        let tok = &text[t.start..t.end];
        let (conv, reason) = convert_token(tok, opts, t.evidence, ctx, &mut quote_open);
        report.push((tok.to_string(), reason, t.evidence));
        out.push_str(&conv);
        last = t.end;
    }
    out.push_str(&text[last..]);
    (out, report)
}

/// Yangi alifbodan eski lotinga.
pub fn to_old(text: &str, style: OldStyle) -> String {
    reverse::to_old(text, style)
}

/// Yozish paytida (IME, tray): so'zlar birma-bir keladi, kontekst — faqat shu gapdagi oldingi so'zlar.
#[derive(Debug, Default, Clone)]
pub struct Session {
    recent: Vec<Tok>,
    quote_open: bool,
}

/// Yozish paytidagi tahrir: so'z oxiridan `delete` ta belgini o'chirib, `insert` yozilsin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    pub delete: usize,
    pub insert: String,
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    /// Kursor boshqa joyga o'tdi, Enter, boshqa maydon — kontekst unutiladi.
    pub fn reset(&mut self) {
        self.recent.clear();
        self.quote_open = false;
    }

    /// So'z tugadi (probel, tinish belgisi, Enter). O'zgarish kerak bo'lsa — tahrir.
    pub fn complete_word(&mut self, word: &str, opts: &Options) -> Option<Edit> {
        let word_n = normalize(word);
        let word_n = word_n.as_ref();
        let core = token::split_edges(word_n).core;
        let own = if opts.english_guard {
            lang::evidence(core)
        } else {
            0.0
        };
        let trimmed = word_n
            .trim_end_matches(|c: char| matches!(c, '"' | '»' | '”' | ')') || is_apostrophe(c));
        let me = Tok {
            start: 0,
            end: 0,
            evidence: own,
            sentence_end: trimmed.ends_with(SENTENCE_END),
            clause_end: trimmed.ends_with(CLAUSE_END),
        };
        let mut window = self.recent.clone();
        window.push(me.clone());
        let ctx = context(&window, 0..window.len(), window.len() - 1);
        let (conv, _) = convert_token(word_n, opts, own, ctx, &mut self.quote_open);
        if me.sentence_end {
            self.recent.clear();
        } else {
            self.recent.push(me);
            if self.recent.len() > 8 {
                self.recent.remove(0);
            }
        }
        if conv == word {
            return None;
        }
        Some(Edit {
            delete: word.chars().count(),
            insert: conv,
        })
    }
}
