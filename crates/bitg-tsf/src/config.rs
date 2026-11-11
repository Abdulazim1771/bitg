//! Sozlamalar: `%APPDATA%\bitg\settings.ini` (kalit=qiymat) va `%APPDATA%\bitg\exceptions.txt` (har qatorda so'z).
//! IME faqat o'qiydi; yozuvchi — tray/sozlamalar ilovasi. Fayl yo'q yoki buzuq bo'lsa — standart qiymatlar.
//!
//! ```ini
//! enabled=1
//! english_guard=1
//! cyrillic=1
//! tutuq=ʼ
//! undo_backspace=1
//! fallback=1
//! disabled_apps=termius.exe, windowsterminal.exe
//! ```
//!
//! `fallback` — matnni TSF'ga bermaydigan dasturlar (Telegram Desktop): bosilgan harflar kuzatiladi va so'z
//! Backspace + qayta yozish bilan almashtiriladi. `disabled_apps` — bitg umuman ishlamaydigan dasturlar
//! (standart: terminallar, chunki u yerda parol va buyruqlar yoziladi). Bo'sh qiymat = hamma joyda.

use bitg_core::Options;
use bitg_core::chars::fold_key;

#[derive(Debug, Clone)]
pub struct Config {
    pub enabled: bool,
    pub undo_backspace: bool,
    pub fallback: bool,
    /// Kichik harflarda: "termius.exe".
    pub disabled_apps: Vec<String>,
    pub opts: Options,
}

/// Terminallar: buyruq va parol yoziladi — standart holatda tegilmaydi.
pub const DEFAULT_DISABLED_APPS: &[&str] = &[
    "termius.exe",
    "windowsterminal.exe",
    "openconsole.exe",
    "conhost.exe",
    "putty.exe",
    "kitty.exe",
    "mintty.exe",
    "alacritty.exe",
    "wezterm-gui.exe",
    "mobaxterm.exe",
];

impl Config {
    pub fn app_disabled(&self, exe: &str) -> bool {
        let exe = exe.to_lowercase();
        self.disabled_apps.contains(&exe)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enabled: true,
            undo_backspace: true,
            fallback: true,
            disabled_apps: DEFAULT_DISABLED_APPS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            opts: Options::default(),
        }
    }
}

fn flag(v: &str) -> Option<bool> {
    match v.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" | "ha" => Some(true),
        "0" | "false" | "no" | "off" | "yoq" | "yo'q" => Some(false),
        _ => None,
    }
}

/// `settings.ini` va `exceptions.txt` matnidan. Noma'lum kalitlar e'tiborsiz.
pub fn parse(settings: &str, exceptions: &str) -> Config {
    let mut c = Config::default();
    for line in settings.lines() {
        let line = line.trim().trim_start_matches('\u{FEFF}');
        if line.is_empty()
            || line.starts_with('#')
            || line.starts_with(';')
            || line.starts_with('[')
        {
            continue;
        }
        let Some((k, v)) = line.split_once('=') else {
            continue;
        };
        let k = k.trim().to_ascii_lowercase();
        match k.as_str() {
            "enabled" => c.enabled = flag(v).unwrap_or(c.enabled),
            "english_guard" => c.opts.english_guard = flag(v).unwrap_or(c.opts.english_guard),
            "cyrillic" => c.opts.cyrillic = flag(v).unwrap_or(c.opts.cyrillic),
            "restore_apostrophes" => {
                c.opts.restore_apostrophes = flag(v).unwrap_or(c.opts.restore_apostrophes)
            }
            "undo_backspace" => c.undo_backspace = flag(v).unwrap_or(c.undo_backspace),
            "fallback" => c.fallback = flag(v).unwrap_or(c.fallback),
            "disabled_apps" => {
                c.disabled_apps = v
                    .split([',', ';'])
                    .map(|a| a.trim().to_lowercase())
                    .filter(|a| !a.is_empty())
                    .collect();
            }
            "tutuq" => {
                let mut it = v.trim().chars();
                if let (Some(ch), None) = (it.next(), it.next()) {
                    c.opts.tutuq = ch;
                }
            }
            _ => {}
        }
    }
    for w in exceptions.lines() {
        let w = w.trim().trim_start_matches('\u{FEFF}');
        if w.is_empty() || w.starts_with('#') {
            continue;
        }
        let k = fold_key(w);
        if !c.opts.exceptions.contains(&k) {
            c.opts.exceptions.push(k);
        }
    }
    c
}

/// Diskdan o'qish (Windows: %APPDATA%\bitg). Xato — standart.
pub fn load() -> Config {
    let Some(base) = std::env::var_os("APPDATA") else {
        return Config::default();
    };
    let dir = std::path::PathBuf::from(base).join("bitg");
    let s = std::fs::read_to_string(dir.join("settings.ini")).unwrap_or_default();
    let e = std::fs::read_to_string(dir.join("exceptions.txt")).unwrap_or_default();
    parse(&s, &e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_when_empty() {
        let c = parse("", "");
        assert!(c.enabled && c.undo_backspace && c.opts.english_guard && c.opts.cyrillic);
        assert_eq!(c.opts.tutuq, '\u{02BC}');
    }

    #[test]
    fn reads_values_and_ignores_garbage() {
        let c = parse(
            "\u{FEFF}[bitg]\n# izoh\nenabled=0\nenglish_guard = off\ntutuq='\nundo_backspace=yo'q\nfoo=bar\nbroken\n",
            "Shoh\n# izoh\n  O‘zbekcha \nshoh\n",
        );
        assert!(!c.enabled && !c.opts.english_guard && !c.undo_backspace);
        assert!(c.opts.cyrillic);
        assert_eq!(c.opts.tutuq, '\'');
        assert_eq!(
            c.opts.exceptions,
            vec!["shoh".to_string(), "o'zbekcha".to_string()]
        );
    }

    #[test]
    fn disabled_apps_default_and_override() {
        let c = parse("", "");
        assert!(c.app_disabled("Termius.exe") && !c.app_disabled("Telegram.exe"));
        let c = parse(
            "disabled_apps=
fallback=0",
            "",
        );
        assert!(!c.app_disabled("Termius.exe") && !c.fallback);
        let c = parse("disabled_apps = Foo.exe; bar.exe", "");
        assert!(c.app_disabled("foo.EXE") && c.app_disabled("bar.exe"));
    }

    #[test]
    fn bad_tutuq_keeps_default() {
        assert_eq!(parse("tutuq=ab", "").opts.tutuq, '\u{02BC}');
    }
}
