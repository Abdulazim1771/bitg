//! Windows bilan bog'liq ishlar: bitta nusxa, Windows bilan ishga tushish, klaviatura o'rnatilganini tekshirish.

use windows::Win32::Foundation::{ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Registry::{
    HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, REG_SZ, RRF_RT_ANY, RegDeleteKeyValueW,
    RegGetValueW, RegSetKeyValueW,
};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, INFINITE, OpenEventW, SetEvent,
    WaitForSingleObject,
};
use windows::core::{HSTRING, PCWSTR};

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const RUN_VALUE: &str = "bitg";
const CLSID: &str = "{E37D4F70-F82A-435E-AF8B-82E264391062}";
const PROFILE: &str = "{BED48FC8-8690-4712-BBE0-1FD3284CE7B7}";
const MUTEX: &str = r"Local\bitg-app";
const SHOW_EVENT: &str = r"Local\bitg-app-show";

fn exists(root: HKEY, key: &str, value: Option<&str>) -> bool {
    let key = HSTRING::from(key);
    let value = value.map(HSTRING::from);
    let vp = value
        .as_ref()
        .map_or(PCWSTR::null(), |v| PCWSTR(v.as_ptr()));
    unsafe { RegGetValueW(root, &key, vp, RRF_RT_ANY, None, None, None) }.is_ok()
}

/// IME DLL ro'yxatdan o'tganmi (regsvr32).
pub fn ime_registered() -> bool {
    exists(
        HKEY_CLASSES_ROOT,
        &format!(r"CLSID\{CLSID}\InprocServer32"),
        None,
    )
}

/// bitg klaviaturasi foydalanuvchining tillar ro'yxatida bormi (O'zbek lotin → bitg).
pub fn keyboard_added() -> bool {
    exists(
        HKEY_CURRENT_USER,
        r"Control Panel\International\User Profile\uz-Latn",
        Some(&format!("0443:{CLSID}{PROFILE}")),
    )
}

pub fn autostart() -> bool {
    exists(HKEY_CURRENT_USER, RUN_KEY, Some(RUN_VALUE))
}

pub fn set_autostart(on: bool) -> windows::core::Result<()> {
    let key = HSTRING::from(RUN_KEY);
    let name = HSTRING::from(RUN_VALUE);
    if on {
        let exe = std::env::current_exe().map_err(|_| windows::core::Error::from_thread())?;
        let cmd = format!("\"{}\" --tray", exe.display());
        let wide: Vec<u16> = cmd.encode_utf16().chain(Some(0)).collect();
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                &key,
                &name,
                REG_SZ.0,
                Some(wide.as_ptr().cast()),
                (wide.len() * 2) as u32,
            )
        }
        .ok()
    } else {
        let r = unsafe { RegDeleteKeyValueW(HKEY_CURRENT_USER, &key, &name) };
        if r.is_ok() || !autostart() {
            Ok(())
        } else {
            r.ok()
        }
    }
}

/// Birinchi nusxami? Yo'q bo'lsa — birinchisiga "oynani ko'rsat" signalini yuboradi.
/// Mutex jarayon tugaguncha ochiq turadi (HANDLE ataylab yopilmaydi).
pub fn first_instance() -> bool {
    let created = unsafe { CreateMutexW(None, true, &HSTRING::from(MUTEX)) };
    let already = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    if created.is_ok() && !already {
        return true;
    }
    if let Ok(ev) = unsafe { OpenEventW(EVENT_MODIFY_STATE, false, &HSTRING::from(SHOW_EVENT)) } {
        let _ = unsafe { SetEvent(ev) };
    }
    false
}

/// Boshqa nusxa "ko'rsat" desa — `on_show` chaqiriladi (alohida oqimda kutadi).
pub fn listen_show(on_show: impl Fn() + Send + 'static) {
    let Ok(ev) = (unsafe { CreateEventW(None, false, false, &HSTRING::from(SHOW_EVENT)) }) else {
        return;
    };
    let ev = ev.0 as usize; // HANDLE Send emas — raqam sifatida o'tkazamiz
    std::thread::spawn(move || {
        let h = HANDLE(ev as *mut _);
        loop {
            if unsafe { WaitForSingleObject(h, INFINITE) }.0 != 0 {
                return;
            }
            on_show();
        }
    });
}
