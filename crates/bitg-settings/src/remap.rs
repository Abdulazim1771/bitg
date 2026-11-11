//! Klaviatura tartibi (ixtiyoriy): tray ilovasidagi past darajali klaviatura hook'i. Faqat bitg klaviaturasi
//! tanlangan oynada ishlaydi: tartibdagi tugma bosilsa, asl tugma yutiladi va belgi Unicode sifatida yoziladi.
//!
//! - O'ng Alt — AltGr: u ham yutiladi, shuning uchun menyu ochilmaydi va dastur Alt bosilganini ko'rmaydi.
//! - Ctrl, chap Alt yoki Win bosilgan bo'lsa tegilmaydi: Ctrl+C, Alt+Tab inglizcha joyida qoladi.
//! - Administrator huquqidagi oynaga yozib bo'lmaydi (UIPI) — u yerda tugmalar odatdagidek.
//! - Hook faqat rejim yoqilganda o'rnatiladi; bosilgan tugmalar hech qayerga yozilmaydi.

use std::sync::Mutex;
use std::time::Duration;

use bitg_tsf::config::{self, Config};
use bitg_tsf::keymap::{Keymap, REMAP_TAG};
use windows::Win32::Foundation::{CloseHandle, HANDLE, LPARAM, LRESULT, WPARAM};
use windows::Win32::Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    GetCurrentThreadId, OpenProcess, OpenProcessToken, PROCESS_NAME_WIN32,
    PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, SendInput, VIRTUAL_KEY, VK_CAPITAL, VK_CONTROL, VK_LMENU,
    VK_LWIN, VK_RMENU, VK_RWIN, VK_SHIFT,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetForegroundWindow, GetMessageW, GetWindowThreadProcessId,
    KBDLLHOOKSTRUCT, LLKHF_EXTENDED, LLKHF_INJECTED, MSG, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WH_KEYBOARD_LL, WM_KEYDOWN, WM_QUIT, WM_SYSKEYDOWN,
};
use windows::core::PWSTR;

/// bitg profili tanlangan oqimning HKL'i: past so'z — til (uz-Latn, 0x0443).
const LANG_UZ_LATN: usize = 0x0443;
const SLOTS: usize = 0x60;

struct Hook {
    map: Keymap,
    disabled: Vec<String>,
    /// O'ng Alt bizda ushlanib turibdi (dastur uni ko'rmagan).
    altgr: bool,
    /// Yutilgan tugmalar: ularning qo'yib yuborilishi ham yutiladi.
    eaten: [bool; SLOTS],
    /// Oxirgi oyna jarayoni va unda ishlash mumkinmi.
    proc_cache: Option<(u32, bool)>,
}

static HOOK: Mutex<Option<Hook>> = Mutex::new(None);
/// Hook oqimi (0 — o'rnatilmagan).
static THREAD: Mutex<u32> = Mutex::new(0);

/// Sozlamalarga qarab hook'ni o'rnatadi, yangilaydi yoki olib tashlaydi.
pub fn apply(cfg: &Config) {
    let map = cfg.resolve_keymap(&config::read_keymap());
    let on = cfg.enabled && cfg.keymap && !map.is_empty();
    {
        let mut h = HOOK.lock().unwrap_or_else(|e| e.into_inner());
        let (altgr, eaten) = h
            .as_ref()
            .map_or((false, [false; SLOTS]), |o| (o.altgr, o.eaten));
        *h = on.then(|| Hook {
            map,
            disabled: cfg.disabled_apps.clone(),
            altgr,
            eaten,
            proc_cache: None,
        });
    }
    let mut t = THREAD.lock().unwrap_or_else(|e| e.into_inner());
    if on && *t == 0 {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || run(tx));
        *t = rx.recv_timeout(Duration::from_secs(2)).unwrap_or(0);
    } else if !on && *t != 0 {
        unsafe {
            let _ = PostThreadMessageW(*t, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        *t = 0;
    }
}

/// Ishga tushganda va fayllar o'zgarganda (qo'lda tahrir ham) — har 2 soniyada tekshiradi.
pub fn watch() {
    apply(&config::load());
    std::thread::spawn(|| {
        let mut last = config::stamp();
        loop {
            std::thread::sleep(Duration::from_secs(2));
            let now = config::stamp();
            if now != last {
                last = now;
                apply(&config::load());
            }
        }
    });
}

fn run(ready: std::sync::mpsc::Sender<u32>) {
    unsafe {
        let module = GetModuleHandleW(None).ok();
        let Ok(hook) = SetWindowsHookExW(WH_KEYBOARD_LL, Some(proc), module.map(Into::into), 0)
        else {
            let _ = ready.send(0);
            return;
        };
        let _ = ready.send(GetCurrentThreadId());
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        let _ = UnhookWindowsHookEx(hook);
    }
}

fn down(vk: VIRTUAL_KEY) -> bool {
    unsafe { GetAsyncKeyState(vk.0 as i32) < 0 }
}

/// Fokusdagi oynada bitg klaviaturasi tanlanganmi va shu dasturda ishlash mumkinmi.
fn active_here(h: &mut Hook) -> bool {
    unsafe {
        let wnd = GetForegroundWindow();
        let mut pid = 0u32;
        let tid = GetWindowThreadProcessId(wnd, Some(&mut pid));
        if tid == 0 || (GetKeyboardLayout(tid).0 as usize) & 0xFFFF != LANG_UZ_LATN {
            return false;
        }
        if let Some((p, ok)) = h.proc_cache
            && p == pid
        {
            return ok;
        }
        let ok = match process_info(pid) {
            Some((exe, elevated)) => !elevated && !h.disabled.contains(&exe),
            None => false,
        };
        h.proc_cache = Some((pid, ok));
        ok
    }
}

/// Jarayon fayli nomi (kichik harfda) va administrator huquqidami.
unsafe fn process_info(pid: u32) -> Option<(String, bool)> {
    unsafe {
        let p = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 520];
        let mut n = buf.len() as u32;
        let name =
            QueryFullProcessImageNameW(p, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut n)
                .ok()
                .map(|_| {
                    let full = String::from_utf16_lossy(&buf[..n as usize]);
                    full.rsplit('\\').next().unwrap_or("").to_lowercase()
                });
        let mut tok = HANDLE::default();
        let elevated = if OpenProcessToken(p, TOKEN_QUERY, &mut tok).is_ok() {
            let mut e = TOKEN_ELEVATION::default();
            let mut len = 0u32;
            let r = GetTokenInformation(
                tok,
                TokenElevation,
                Some(&mut e as *mut _ as *mut _),
                std::mem::size_of::<TOKEN_ELEVATION>() as u32,
                &mut len,
            );
            let _ = CloseHandle(tok);
            r.is_err() || e.TokenIsElevated != 0
        } else {
            true // tokenni ochib bo'lmadi (administrator yoki himoyalangan jarayon) — tegmaymiz
        };
        let _ = CloseHandle(p);
        Some((name?, elevated))
    }
}

fn type_char(c: char) {
    let mut buf = [0u16; 2];
    let units = c.encode_utf16(&mut buf);
    let mut inputs = Vec::with_capacity(4);
    for (i, &u) in units.iter().enumerate() {
        let extra = (REMAP_TAG << 32) | if i == 0 { c as u64 } else { 0 };
        for flags in [KEYEVENTF_UNICODE, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP] {
            inputs.push(INPUT {
                r#type: INPUT_KEYBOARD,
                Anonymous: INPUT_0 {
                    ki: KEYBDINPUT {
                        wVk: VIRTUAL_KEY(0),
                        wScan: u,
                        dwFlags: flags,
                        time: 0,
                        dwExtraInfo: extra as usize,
                    },
                },
            });
        }
    }
    unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) };
}

/// Qaytaradi: tugma yutilsinmi (o'rniga belgi yozilgan).
fn decide(k: &KBDLLHOOKSTRUCT, is_down: bool) -> bool {
    if k.flags.0 & LLKHF_INJECTED.0 != 0 {
        return false;
    }
    let mut g = HOOK.lock().unwrap_or_else(|e| e.into_inner());
    let Some(h) = g.as_mut() else { return false };
    if k.vkCode == VK_RMENU.0 as u32 {
        if !is_down {
            return std::mem::take(&mut h.altgr);
        }
        if h.altgr || active_here(h) {
            h.altgr = true;
            return true;
        }
        return false;
    }
    let scan = k.scanCode as usize;
    if k.flags.0 & LLKHF_EXTENDED.0 != 0 || scan >= SLOTS {
        return false;
    }
    if !is_down {
        return std::mem::take(&mut h.eaten[scan]);
    }
    if down(VK_CONTROL) || down(VK_LMENU) || down(VK_LWIN) || down(VK_RWIN) {
        return false;
    }
    let shift = down(VK_SHIFT);
    let caps = unsafe { GetKeyState(VK_CAPITAL.0 as i32) } & 1 != 0;
    let Some(c) = h.map.lookup(scan as u8, shift, caps, h.altgr) else {
        return false;
    };
    if !active_here(h) {
        return false;
    }
    h.eaten[scan] = true;
    drop(g);
    type_char(c);
    true
}

unsafe extern "system" fn proc(code: i32, w: WPARAM, l: LPARAM) -> LRESULT {
    if code >= 0 {
        let k = unsafe { &*(l.0 as *const KBDLLHOOKSTRUCT) };
        let is_down = matches!(w.0 as u32, WM_KEYDOWN | WM_SYSKEYDOWN);
        if std::panic::catch_unwind(|| decide(k, is_down)).unwrap_or(false) {
            return LRESULT(1);
        }
    }
    unsafe { CallNextHookEx(None, code, w, l) }
}
