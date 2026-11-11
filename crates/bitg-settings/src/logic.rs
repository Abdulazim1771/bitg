//! Oyna bilan Rust o'rtasidagi JSON xabarlar. Platformaga bog'liq emas — testlanadi.
//!
//! Oyna yuboradi: `{"cmd":"load"}`, `{"cmd":"save","settings":{..},"exceptions":[..]}`,
//! `{"cmd":"preview","id":N,"text":"..","settings":{..},"exceptions":[..]}`, `{"cmd":"open_folder"}`.

use bitg_core::Reason;
use bitg_tsf::config::{self, Config, DEFAULT_DISABLED_APPS};
use serde_json::{Value, json};

pub fn settings_json(c: &Config) -> Value {
    json!({
        "enabled": c.enabled,
        "english_guard": c.opts.english_guard,
        "cyrillic": c.opts.cyrillic,
        "restore_apostrophes": c.opts.restore_apostrophes,
        "tutuq": c.opts.tutuq.to_string(),
        "fallback": c.fallback,
        "undo_backspace": c.undo_backspace,
        "disabled_apps": c.disabled_apps,
    })
}

/// Oynadagi qiymatlardan `Config` (yo'q yoki noto'g'ri maydon — standart qiymat).
pub fn config_from(settings: &Value, exceptions: &[String]) -> Config {
    let mut c = Config::default();
    let b = |k: &str, d: bool| settings.get(k).and_then(Value::as_bool).unwrap_or(d);
    c.enabled = b("enabled", c.enabled);
    c.opts.english_guard = b("english_guard", c.opts.english_guard);
    c.opts.cyrillic = b("cyrillic", c.opts.cyrillic);
    c.opts.restore_apostrophes = b("restore_apostrophes", c.opts.restore_apostrophes);
    c.fallback = b("fallback", c.fallback);
    c.undo_backspace = b("undo_backspace", c.undo_backspace);
    if let Some(t) = settings.get("tutuq").and_then(Value::as_str) {
        let mut it = t.chars();
        if let (Some(ch), None) = (it.next(), it.next()) {
            c.opts.tutuq = ch;
        }
    }
    if let Some(apps) = settings.get("disabled_apps").and_then(Value::as_array) {
        c.disabled_apps.clear();
        for a in apps.iter().filter_map(Value::as_str) {
            let a = a.trim().to_lowercase();
            if !a.is_empty() && !a.contains([',', ';', '\n']) && !c.disabled_apps.contains(&a) {
                c.disabled_apps.push(a);
            }
        }
    }
    c.opts.exceptions = config::parse("", &exceptions.join("\n")).opts.exceptions;
    c
}

/// Istisnolar fayli: izohsiz, bo'sh bo'lmagan qatorlar, takrorsiz (yozilishi saqlanadi).
pub fn exception_words(raw: &str) -> Vec<String> {
    let mut seen = Vec::new();
    let mut out = Vec::new();
    for w in raw.lines() {
        let w = w.trim().trim_start_matches('\u{FEFF}');
        if w.is_empty() || w.starts_with('#') || w.chars().any(char::is_whitespace) {
            continue;
        }
        let k = bitg_core::chars::fold_key(w);
        if !seen.contains(&k) {
            seen.push(k);
            out.push(w.to_string());
        }
    }
    out
}

fn strings(v: Option<&Value>) -> Vec<String> {
    v.and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub fn load() -> Value {
    let (s, e) = config::read_raw();
    let c = config::parse(&s, &e);
    json!({
        "kind": "loaded",
        "settings": settings_json(&c),
        "exceptions": exception_words(&e),
        "default_apps": DEFAULT_DISABLED_APPS,
        "folder": config::dir().map(|d| d.display().to_string()),
    })
}

/// Ikkala faylni yozadi. Avval vaqtinchalik faylga, so'ng almashtirish — IME yarim yozilgan faylni o'qimaydi.
pub fn save(msg: &Value) -> Value {
    let words = exception_words(&strings(msg.get("exceptions")).join("\n"));
    let c = config_from(msg.get("settings").unwrap_or(&Value::Null), &words);
    let result = (|| -> std::io::Result<()> {
        let dir = config::dir().ok_or_else(|| std::io::Error::other("APPDATA topilmadi"))?;
        std::fs::create_dir_all(&dir)?;
        let mut ex = String::from("# O'girilmaydigan so'zlar, har qatorda bittadan.\n");
        for w in &words {
            ex.push_str(w);
            ex.push('\n');
        }
        write_atomic(&dir.join(config::EXCEPTIONS_FILE), &ex)?;
        write_atomic(&dir.join(config::SETTINGS_FILE), &config::to_ini(&c))
    })();
    match result {
        Ok(()) => json!({"kind": "saved"}),
        Err(e) => json!({"kind": "save_failed", "error": e.to_string()}),
    }
}

fn write_atomic(path: &std::path::Path, text: &str) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path)
}

fn reason_name(r: Reason) -> &'static str {
    match r {
        Reason::Converted => "converted",
        Reason::Unchanged => "unchanged",
        Reason::Protected => "protected",
        Reason::Exception => "exception",
        Reason::English => "english",
    }
}

/// Matn bo'laklari (bo'shliq bo'yicha): `[{"s": kirish, "d": chiqish, "r": sabab}]`. Bo'shliqlar ham bo'lak
/// (`r` = null). `convert` bo'shliqlarni aynan saqlaydi, shuning uchun kirish va chiqish bo'laklari mos keladi.
pub fn preview(msg: &Value) -> Value {
    let text = msg.get("text").and_then(Value::as_str).unwrap_or("");
    let words = exception_words(&strings(msg.get("exceptions")).join("\n"));
    let c = config_from(msg.get("settings").unwrap_or(&Value::Null), &words);
    let src = bitg_core::chars::normalize(text);
    let (out, report) = bitg_core::convert_explained(&src, &c.opts);
    let a = chunks(&src);
    let b = chunks(&out);
    let mut segs = Vec::with_capacity(a.len());
    if a.len() != b.len() {
        // Kutilmagan holat: bo'laklab ko'rsatib bo'lmaydi, butun matn bitta bo'lak.
        segs.push(json!({"s": src, "d": out, "r": null}));
    } else {
        let mut rep = report.iter().peekable();
        for (s, d) in a.iter().zip(&b) {
            if s.trim().is_empty() {
                segs.push(json!({"s": s, "d": d, "r": null}));
                continue;
            }
            // Bo'lak ichidagi tokenlar (tire bilan bo'lingan bo'lishi mumkin): eng muhim sababni olamiz.
            let mut pos = 0;
            let mut best: Option<Reason> = None;
            while let Some((tok, r, _)) = rep.peek() {
                let Some(i) = s[pos..].find(tok.as_str()) else {
                    break;
                };
                pos += i + tok.len();
                best = Some(match (best, *r) {
                    (Some(Reason::Converted), _) | (_, Reason::Converted) => Reason::Converted,
                    (Some(Reason::Unchanged) | None, r) => r,
                    (Some(b), _) => b,
                });
                rep.next();
            }
            segs.push(json!({"s": s, "d": d, "r": best.map(reason_name)}));
        }
    }
    json!({
        "kind": "preview",
        "id": msg.get("id").cloned().unwrap_or(Value::Null),
        "out": out,
        "segments": segs,
    })
}

/// Bo'shliq va bo'shliq bo'lmagan qismlar navbatma-navbat.
fn chunks(s: &str) -> Vec<&str> {
    let mut v = Vec::new();
    let mut start = 0;
    let mut ws: Option<bool> = None;
    for (i, c) in s.char_indices() {
        let w = c.is_whitespace();
        if ws.is_some_and(|p| p != w) {
            v.push(&s[start..i]);
            start = i;
        }
        ws = Some(w);
    }
    if start < s.len() {
        v.push(&s[start..]);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_marks_converted_and_english() {
        let v = preview(&json!({"id": 3, "text": "Please check, keyin  choy", "settings": {}}));
        assert_eq!(v["id"], 3);
        assert_eq!(v["out"], "Please check, keyin  çoy");
        let segs = v["segments"].as_array().unwrap();
        let r = |s: &str| {
            segs.iter()
                .find(|x| x["s"] == s)
                .map(|x| x["r"].clone())
                .unwrap()
        };
        assert_eq!(r("choy"), "converted");
        assert_eq!(r("check,"), "english");
        assert_eq!(r("  "), Value::Null);
        let joined: String = segs.iter().map(|x| x["d"].as_str().unwrap()).collect();
        assert_eq!(joined, v["out"]);
    }

    #[test]
    fn preview_uses_unsaved_settings() {
        let off =
            preview(&json!({"text": "ozbek choy", "settings": {"restore_apostrophes": false}}));
        assert_eq!(off["out"], "ozbek çoy");
        let ex = preview(&json!({"text": "choy", "settings": {}, "exceptions": ["Choy"]}));
        assert_eq!(ex["out"], "choy");
        assert_eq!(ex["segments"][0]["r"], "exception");
    }

    #[test]
    fn config_from_ignores_bad_values() {
        let c = config_from(
            &json!({"enabled": "ha", "tutuq": "ab", "disabled_apps": ["Foo.EXE", " ", "a,b", 5]}),
            &[],
        );
        assert!(c.enabled);
        assert_eq!(c.opts.tutuq, '\u{02BC}');
        assert_eq!(c.disabled_apps, vec!["foo.exe"]);
    }

    #[test]
    fn exception_words_cleans_list() {
        assert_eq!(
            exception_words("# izoh\nShoh\n\nshoh\n  O‘zbekcha \nikki soz\n"),
            vec!["Shoh", "O‘zbekcha"]
        );
    }
}
