//! TSF xizmati. Tugmalarni HECH QACHON ushlab qolmaydi (OnTestKeyDown doim FALSE — tugma dasturga boradi).
//!
//! 1) Asosiy yo'l: fokusdagi matnning o'zgarishini kuzatadi (ITfTextEditSink). So'zdan keyin probel/Enter/Tab
//!    yozilgani ko'rinsa (kursor oldidagi matn = oldingi matn + bo'shliq), so'zni asinxron tahrirda almashtiradi.
//! 2) Zaxira yo'l (dastur matnni TSF'ga bermasa — Telegram Desktop): bosilgan harflardan so'z yig'iladi
//!    (`typed`), probeldan keyin Backspace + Unicode kiritish (SendInput) bilan almashtiriladi.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use std::time::{Duration, Instant};

use bitg_core::Session;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, GetKeyState, GetKeyboardLayout, GetKeyboardState, INPUT, INPUT_0,
    INPUT_KEYBOARD, KEYBD_EVENT_FLAGS, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, SendInput,
    ToUnicodeEx, VIRTUAL_KEY, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME,
    VK_LBUTTON, VK_LEFT, VK_LWIN, VK_MENU, VK_NEXT, VK_PACKET, VK_PRIOR, VK_RBUTTON, VK_RETURN,
    VK_RIGHT, VK_RWIN, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::TextServices::{
    ITfContext, ITfDocumentMgr, ITfEditRecord, ITfEditSession, ITfEditSession_Impl,
    ITfKeyEventSink, ITfKeyEventSink_Impl, ITfKeystrokeMgr, ITfRange, ITfSource, ITfTextEditSink,
    ITfTextEditSink_Impl, ITfTextInputProcessor, ITfTextInputProcessor_Impl,
    ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl, ITfThreadMgr, ITfThreadMgrEventSink,
    ITfThreadMgrEventSink_Impl, TF_AE_END, TF_ANCHOR_END, TF_DEFAULT_SELECTION, TF_ES_ASYNC,
    TF_ES_READWRITE, TF_SELECTION, TF_SELECTIONSTYLE,
};
use windows_core::{BOOL, GUID, IUnknown, IUnknownImpl, Interface, Ref, Result, implement};

use crate::config::{self, Config};
use crate::log;
use crate::plan;
use crate::typed::Typed;

/// Kursor oldidan o'qiladigan eng ko'p UTF-16 birlik.
const LOOKBACK: i32 = 64;
const NO_COOKIE: u32 = u32::MAX;

struct State {
    thread_mgr: Option<ITfThreadMgr>,
    tid: u32,
    tm_cookie: u32,
    /// Matn o'zgarishi kuzatilayotgan kontekst.
    watched: Option<(ITfContext, u32)>,
    /// Oxirgi ko'rilgan "kursor oldidagi matn".
    last: Vec<u16>,
    /// Bu kontekstda matn o'qilmadi (Qt/IMM32 kabi dasturlar) — log bir marta.
    blind_logged: bool,
    session: Session,
    cfg: Config,
    /// Sozlama fayllari holati va oxirgi tekshiruv (oyna o'zgartirsa — dasturni qayta ochmasdan qabul qilinadi).
    stamp: config::Stamp,
    checked: Option<Instant>,
    /// Shu dasturda ishlaydimi (enabled va disabled_apps bo'yicha).
    active: bool,
    /// Kontekst matnni bermayapti — zaxira yo'l ishlaydi.
    blind: bool,
    key_sink: bool,
    typed: Typed,
    /// O'zimiz yuborgan tugmalar (Backspace / VK_PACKET) — ularni kuzatmaymiz.
    injecting: u32,
    inject_until: Option<Instant>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            thread_mgr: None,
            tid: 0,
            tm_cookie: NO_COOKIE,
            watched: None,
            last: Vec::new(),
            blind_logged: false,
            session: Session::new(),
            cfg: Config::default(),
            stamp: Default::default(),
            checked: None,
            active: false,
            blind: true,
            key_sink: false,
            typed: Typed::default(),
            injecting: 0,
            inject_until: None,
        }
    }
}

#[implement(
    ITfTextInputProcessorEx,
    ITfTextInputProcessor,
    ITfThreadMgrEventSink,
    ITfTextEditSink,
    ITfKeyEventSink
)]
pub struct TextService {
    state: Rc<RefCell<State>>,
}

impl TextService {
    pub fn new() -> Self {
        Self {
            state: Rc::new(RefCell::new(State::default())),
        }
    }
}

/// Sozlama fayllari ko'pi bilan sekundiga bir marta tekshiriladi.
const CONFIG_CHECK: Duration = Duration::from_secs(1);

impl State {
    fn apply(&mut self, cfg: Config, stamp: config::Stamp) {
        self.active = cfg.enabled && !cfg.app_disabled(log::process());
        self.cfg = cfg;
        self.stamp = stamp;
        self.checked = Some(Instant::now());
    }

    /// Fayllar o'zgargan bo'lsa qayta o'qiydi. Qaytaradi: shu dasturda ishlash kerakmi.
    fn refresh(&mut self) -> bool {
        if self.checked.is_none_or(|t| t.elapsed() >= CONFIG_CHECK) {
            self.checked = Some(Instant::now());
            let now = config::stamp();
            if now != self.stamp {
                self.apply(config::load(), now);
                log::write(if self.active {
                    "settings reloaded: active"
                } else {
                    "settings reloaded: off for this app"
                });
            }
        }
        self.active
    }
}

fn guard<T: Default>(what: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => {
            log::write(&format!("{what} failed hr={:#x}", e.code().0));
            Ok(T::default())
        }
        Err(_) => {
            log::write(&format!("panic in {what}"));
            Ok(T::default())
        }
    }
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        guard("deactivate", || {
            self.unwatch();
            let mut st = self.state.borrow_mut();
            if let Some(tm) = st.thread_mgr.take() {
                if st.tm_cookie != NO_COOKIE {
                    if let Ok(src) = tm.cast::<ITfSource>() {
                        let _ = unsafe { src.UnadviseSink(st.tm_cookie) };
                    }
                    st.tm_cookie = NO_COOKIE;
                }
                if st.key_sink {
                    if let Ok(ks) = tm.cast::<ITfKeystrokeMgr>() {
                        let _ = unsafe { ks.UnadviseKeyEventSink(st.tid) };
                    }
                    st.key_sink = false;
                }
            }
            log::write("deactivate");
            Ok(())
        })
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        let tm = ptim.ok()?.clone();
        let stamp = config::stamp();
        let cfg = config::load();
        let active = {
            let mut st = self.state.borrow_mut();
            st.thread_mgr = Some(tm.clone());
            st.tid = tid;
            st.apply(cfg, stamp);
            st.active
        };
        // Kuzatuvchilar har doim ulanadi: sozlama oynada yoqilsa, dasturni qayta ochish shart emas.
        // O'chiq holatda ular faqat kuzatadi, hech narsa yozmaydi (har chaqiruvda `refresh`).
        let me: IUnknown = self.to_object().to_interface();
        let src: ITfSource = tm.cast()?;
        let cookie = unsafe { src.AdviseSink(&ITfThreadMgrEventSink::IID, &me)? };
        self.state.borrow_mut().tm_cookie = cookie;
        let sink: ITfKeyEventSink = self.to_object().to_interface();
        let ks: ITfKeystrokeMgr = tm.cast()?;
        match unsafe { ks.AdviseKeyEventSink(tid, &sink, true) } {
            Ok(()) => self.state.borrow_mut().key_sink = true,
            Err(e) => log::write(&format!("advise key sink failed hr={:#x}", e.code().0)),
        }
        // Hozir fokusdagi hujjat.
        if let Ok(dim) = unsafe { tm.GetFocus() } {
            self.watch(Some(&dim));
        }
        std::thread::spawn(bitg_core::lang::warm_up);
        log::write(if active {
            "activate"
        } else {
            "activate: off for this app (settings)"
        });
        Ok(())
    }
}

impl ITfThreadMgrEventSink_Impl for TextService_Impl {
    fn OnInitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnUninitDocumentMgr(&self, _pdim: Ref<'_, ITfDocumentMgr>) -> Result<()> {
        Ok(())
    }

    fn OnSetFocus(
        &self,
        focus: Ref<'_, ITfDocumentMgr>,
        _prev: Ref<'_, ITfDocumentMgr>,
    ) -> Result<()> {
        guard("focus", || {
            self.watch(focus.ok().ok());
            Ok(())
        })
    }

    fn OnPushContext(&self, pic: Ref<'_, ITfContext>) -> Result<()> {
        guard("push context", || {
            if let Ok(ctx) = pic.ok() {
                let dim = unsafe { ctx.GetDocumentMgr()? };
                self.watch(Some(&dim));
            }
            Ok(())
        })
    }

    fn OnPopContext(&self, _pic: Ref<'_, ITfContext>) -> Result<()> {
        Ok(())
    }
}

impl ITfTextEditSink_Impl for TextService_Impl {
    fn OnEndEdit(
        &self,
        pic: Ref<'_, ITfContext>,
        ec: u32,
        _rec: Ref<'_, ITfEditRecord>,
    ) -> Result<()> {
        guard("end edit", || {
            let Ok(ctx) = pic.ok() else { return Ok(()) };
            self.on_text_changed(ctx, ec)
        })
    }
}

impl TextService_Impl {
    fn unwatch(&self) {
        let old = self.state.borrow_mut().watched.take();
        if let Some((ctx, cookie)) = old
            && let Ok(src) = ctx.cast::<ITfSource>()
        {
            let _ = unsafe { src.UnadviseSink(cookie) };
        }
    }

    /// Fokusdagi hujjatning yuqori kontekstida matn o'zgarishini kuzatishni boshlaydi.
    fn watch(&self, dim: Option<&ITfDocumentMgr>) {
        self.unwatch();
        {
            let mut st = self.state.borrow_mut();
            st.last.clear();
            st.session.reset();
            st.blind_logged = false;
            st.blind = true;
            st.typed.boundary();
        }
        let Some(dim) = dim else { return };
        let Ok(ctx) = (unsafe { dim.GetTop() }) else {
            return;
        };
        let me: IUnknown = self.to_object().to_interface();
        let Ok(src) = ctx.cast::<ITfSource>() else {
            return;
        };
        match unsafe { src.AdviseSink(&ITfTextEditSink::IID, &me) } {
            Ok(cookie) => self.state.borrow_mut().watched = Some((ctx, cookie)),
            Err(e) => log::write(&format!("advise edit sink failed hr={:#x}", e.code().0)),
        }
    }

    fn on_text_changed(&self, ctx: &ITfContext, ec: u32) -> Result<()> {
        let Some((_, before, reached_start)) = read_before_caret(ctx, ec)? else {
            return Ok(());
        };
        let prev = std::mem::replace(&mut self.state.borrow_mut().last, before.clone());
        if !before.is_empty() {
            self.state.borrow_mut().blind = false;
        }
        if before.is_empty() {
            let mut st = self.state.borrow_mut();
            if !st.blind_logged {
                st.blind_logged = true;
                log::write("no text before caret in this context (app does not expose text)");
            }
            return Ok(());
        }
        // Faqat bitta bo'shliq yozilganda: yangi = eski + bo'shliq (bufer 64 da kesilganini ham hisobga olib).
        let Some(&last) = before.last() else {
            return Ok(());
        };
        if !matches!(last, 0x20 | 0x09 | 0x0A | 0x0D | 0xA0) {
            return Ok(());
        }
        let head = &before[..before.len() - 1];
        // Qat'iy: avvalgi matn aynan shu (yoki 64 dan uzun bo'lib kesilgan) — joylashtirish (paste) emas.
        let typed_space = prev.ends_with(head);
        if !typed_space || prev.is_empty() {
            return Ok(());
        }
        let Some(t) = plan::word_before_caret(&before, reached_start) else {
            return Ok(());
        };
        if t.trailing != 1 || !self.state.borrow_mut().refresh() {
            return Ok(());
        }
        let edit = {
            let mut st = self.state.borrow_mut();
            let st = &mut *st;
            st.session.complete_word(&t.word, &st.cfg.opts)
        };
        if last == 0x0A || last == 0x0D {
            self.state.borrow_mut().session.reset();
        }
        let Some(edit) = edit else {
            log::write("word unchanged");
            return Ok(());
        };
        let es: ITfEditSession = Replace {
            ctx: ctx.clone(),
            word: t.word,
            insert: edit.insert,
        }
        .into();
        let tid = self.state.borrow().tid;
        if let Err(e) = unsafe { ctx.RequestEditSession(tid, &es, TF_ES_ASYNC | TF_ES_READWRITE) } {
            log::write(&format!("request edit failed hr={:#x}", e.code().0));
        }
        Ok(())
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _foreground: BOOL) -> Result<()> {
        self.state.borrow_mut().typed.boundary();
        Ok(())
    }

    /// Faqat kuzatadi. Doim FALSE: TRUE qaytarilsa TSF tugmani dasturga bermay qo'yadi (qotib qolish).
    fn OnTestKeyDown(&self, pic: Ref<'_, ITfContext>, w: WPARAM, l: LPARAM) -> Result<BOOL> {
        let _ = guard("key", || {
            if pic.is_null() {
                // Matn maydoni emas (Explorer ro'yxati, o'yin): Backspace bu yerda boshqa ish qiladi — tegmaymiz.
                self.state.borrow_mut().typed.lost();
            } else {
                self.observe_key(w.0 as u16, l.0);
            }
            Ok(())
        });
        Ok(false.into())
    }

    fn OnTestKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnKeyDown(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _guid: *const GUID) -> Result<BOOL> {
        Ok(false.into())
    }
}

fn held(vk: VIRTUAL_KEY) -> bool {
    let s = unsafe { GetKeyState(vk.0 as i32) };
    s < 0
}

/// Tugma qaysi belgini beradi (joriy tartib, Shift/CapsLock bilan). Klaviatura holatini o'zgartirmaydi.
fn key_char(vk: u16, lparam: isize) -> Option<char> {
    let mut state = [0u8; 256];
    unsafe { GetKeyboardState(&mut state).ok()? };
    let scan = ((lparam >> 16) & 0xFF) as u32;
    let mut buf = [0u16; 4];
    // 0x4: dead-key holatini buzmaslik (Windows 10 1607+)
    let n = unsafe {
        ToUnicodeEx(
            vk as u32,
            scan,
            &state,
            &mut buf,
            0x4,
            Some(GetKeyboardLayout(0)),
        )
    };
    if n != 1 {
        return None;
    }
    char::from_u32(buf[0] as u32).filter(|c| !c.is_control())
}

fn key_input(vk: VIRTUAL_KEY, scan: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0xB17B,
            },
        },
    }
}

impl TextService_Impl {
    fn observe_key(&self, vk: u16, lparam: isize) {
        let mut st = self.state.borrow_mut();
        let k = VIRTUAL_KEY(vk);
        // O'zimiz yuborgan tugmalar.
        if st.injecting > 0 {
            if st.inject_until.is_some_and(|t| Instant::now() < t) {
                if k == VK_BACK || k == VK_PACKET {
                    st.injecting -= 1;
                    return;
                }
            } else {
                st.injecting = 0;
            }
        }
        // Tugmalar orasida sichqoncha bosilgan bo'lsa — kursor boshqa joyda bo'lishi mumkin.
        let clicked = unsafe {
            (GetAsyncKeyState(VK_LBUTTON.0 as i32) & 1) != 0
                || (GetAsyncKeyState(VK_RBUTTON.0 as i32) & 1) != 0
        };
        if clicked {
            st.typed.lost();
        }
        if held(VK_CONTROL) || held(VK_MENU) || held(VK_LWIN) || held(VK_RWIN) {
            st.typed.lost(); // Ctrl+V, Ctrl+Z, Alt+Tab ...
            return;
        }
        match k {
            VK_SPACE => {
                let word = st.typed.finish();
                if !st.blind {
                    return; // matn o'qiladigan kontekst — asosiy yo'l ishlaydi
                }
                let Some(word) = word else { return };
                if !st.refresh() || !st.cfg.fallback {
                    return;
                }
                let st = &mut *st;
                let Some(edit) = st.session.complete_word(&word, &st.cfg.opts) else {
                    log::write("fallback: word unchanged");
                    return;
                };
                drop_and_retype(st, word.chars().count(), &edit.insert);
            }
            VK_RETURN | VK_TAB => {
                st.typed.boundary();
                st.session.reset();
            }
            VK_BACK => st.typed.backspace(),
            VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT
            | VK_DELETE | VK_ESCAPE | VK_PACKET => {
                st.typed.lost();
                st.session.reset();
            }
            _ => {
                // Shift, CapsLock, F1... belgi bermaydi — e'tiborsiz.
                if let Some(c) = key_char(vk, lparam) {
                    st.typed.push(c);
                }
            }
        }
    }
}

/// Probel navbatda bizdan oldin turibdi: u yozilgach so'z + probelni o'chirib, yangisini yozamiz.
fn drop_and_retype(st: &mut State, word_chars: usize, insert: &str) {
    let mut inputs = Vec::new();
    for _ in 0..=word_chars {
        inputs.push(key_input(VK_BACK, 0, KEYBD_EVENT_FLAGS(0)));
        inputs.push(key_input(VK_BACK, 0, KEYEVENTF_KEYUP));
    }
    let units: Vec<u16> = insert.encode_utf16().chain(std::iter::once(0x20)).collect();
    for &u in &units {
        inputs.push(key_input(VIRTUAL_KEY(0), u, KEYEVENTF_UNICODE));
        inputs.push(key_input(
            VIRTUAL_KEY(0),
            u,
            KEYEVENTF_UNICODE | KEYEVENTF_KEYUP,
        ));
    }
    st.injecting = (word_chars + 1 + units.len()) as u32;
    st.inject_until = Some(Instant::now() + Duration::from_millis(1500));
    let sent = unsafe { SendInput(&inputs, std::mem::size_of::<INPUT>() as i32) } as usize;
    if sent == inputs.len() {
        log::write("fallback: converted");
    } else {
        st.injecting = 0;
        log::write(&format!("fallback: SendInput sent {sent}/{}", inputs.len()));
    }
}

/// Kursor (bo'sh tanlov) va undan oldingi matn. Tanlov bo'sh bo'lmasa — None.
fn read_before_caret(ctx: &ITfContext, ec: u32) -> Result<Option<(ITfRange, Vec<u16>, bool)>> {
    let mut sel = [TF_SELECTION::default()];
    let mut fetched = 0u32;
    unsafe { ctx.GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)? };
    let [s] = sel;
    let range = std::mem::ManuallyDrop::into_inner(s.range);
    if fetched != 1 {
        return Ok(None);
    }
    let Some(caret) = range else { return Ok(None) };
    if !unsafe { caret.IsEmpty(ec)? }.as_bool() {
        return Ok(None);
    }
    let before = unsafe { caret.Clone()? };
    let mut moved = 0i32;
    unsafe { before.ShiftStart(ec, -LOOKBACK, &mut moved, std::ptr::null())? };
    let want = (-moved) as usize;
    let mut buf = vec![0u16; want];
    let mut got = 0u32;
    if want > 0 {
        unsafe { before.GetText(ec, 0, &mut buf, &mut got)? };
    }
    buf.truncate(got as usize);
    Ok(Some((caret, buf, moved > -LOOKBACK)))
}

/// Asinxron tahrir: matn hali ham "so'z + bitta bo'shliq" bo'lsa — so'zni almashtiradi.
#[implement(ITfEditSession)]
struct Replace {
    ctx: ITfContext,
    word: String,
    insert: String,
}

impl ITfEditSession_Impl for Replace_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        guard("edit session", || {
            let r = self.run(ec)?;
            log::write(r);
            Ok(())
        })
    }
}

impl Replace_Impl {
    fn run(&self, ec: u32) -> Result<&'static str> {
        let Some((caret, before, reached)) = read_before_caret(&self.ctx, ec)? else {
            return Ok("skip: selection changed");
        };
        let Some(t) = plan::word_before_caret(&before, reached) else {
            return Ok("skip: word gone");
        };
        if t.word != self.word || t.trailing != 1 {
            return Ok("skip: text changed meanwhile");
        }
        let target = unsafe { caret.Clone()? };
        let back = (t.trailing + t.word_len) as i32;
        let mut m = 0i32;
        unsafe {
            target.ShiftStart(ec, -back, &mut m, std::ptr::null())?;
            if m != -back {
                return Ok("skip: range shift mismatch");
            }
            target.ShiftEnd(ec, -(t.trailing as i32), &mut m, std::ptr::null())?;
        }
        let text: Vec<u16> = self.insert.encode_utf16().collect();
        unsafe { target.SetText(ec, 0, &text)? };

        // Kursor: so'z va bo'shliqdan keyin.
        let after = unsafe { target.Clone()? };
        unsafe {
            after.Collapse(ec, TF_ANCHOR_END)?;
            after.ShiftEnd(ec, t.trailing as i32, &mut m, std::ptr::null())?;
            after.Collapse(ec, TF_ANCHOR_END)?;
        }
        let sel = TF_SELECTION {
            range: std::mem::ManuallyDrop::new(Some(after)),
            style: TF_SELECTIONSTYLE {
                ase: TF_AE_END,
                fInterimChar: false.into(),
            },
        };
        let r = unsafe { self.ctx.SetSelection(ec, std::slice::from_ref(&sel)) };
        drop(std::mem::ManuallyDrop::into_inner(sel.range));
        r?;
        Ok("converted")
    }
}

#[allow(dead_code)]
fn _iids() -> [GUID; 2] {
    [ITfTextInputProcessorEx::IID, ITfTextInputProcessor::IID]
}
