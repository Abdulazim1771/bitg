//! TSF xizmati. Tugmalarga UMUMAN tegmaydi (key sink yo'q — tugma hech qachon ushlanib qolmaydi).
//! Fokusdagi matnning o'zgarishini kuzatadi (ITfTextEditSink): foydalanuvchi so'zdan keyin probel/Enter/Tab
//! yozgani ko'rinsa (kursor oldidagi matn = oldingi matn + bo'shliq), o'sha so'zni asinxron tahrirda almashtiradi.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use bitg_core::Session;
use windows::Win32::UI::TextServices::{
    ITfContext, ITfDocumentMgr, ITfEditRecord, ITfEditSession, ITfEditSession_Impl, ITfRange,
    ITfSource, ITfTextEditSink, ITfTextEditSink_Impl, ITfTextInputProcessor,
    ITfTextInputProcessor_Impl, ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl,
    ITfThreadMgr, ITfThreadMgrEventSink, ITfThreadMgrEventSink_Impl, TF_AE_END, TF_ANCHOR_END,
    TF_DEFAULT_SELECTION, TF_ES_ASYNC, TF_ES_READWRITE, TF_SELECTION, TF_SELECTIONSTYLE,
};
use windows_core::{GUID, IUnknown, IUnknownImpl, Interface, Ref, Result, implement};

use crate::config::{self, Config};
use crate::log;
use crate::plan;

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
        }
    }
}

#[implement(
    ITfTextInputProcessorEx,
    ITfTextInputProcessor,
    ITfThreadMgrEventSink,
    ITfTextEditSink
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
            }
            log::write("deactivate");
            Ok(())
        })
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        let tm = ptim.ok()?.clone();
        let cfg = config::load();
        let enabled = cfg.enabled;
        {
            let mut st = self.state.borrow_mut();
            st.thread_mgr = Some(tm.clone());
            st.tid = tid;
            st.cfg = cfg;
        }
        if !enabled {
            log::write("activate (disabled in settings)");
            return Ok(());
        }
        let me: IUnknown = self.to_object().to_interface();
        let src: ITfSource = tm.cast()?;
        let cookie = unsafe { src.AdviseSink(&ITfThreadMgrEventSink::IID, &me)? };
        self.state.borrow_mut().tm_cookie = cookie;
        // Hozir fokusdagi hujjat.
        if let Ok(dim) = unsafe { tm.GetFocus() } {
            self.watch(Some(&dim));
        }
        std::thread::spawn(bitg_core::lang::warm_up);
        log::write("activate");
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
        if let Some((ctx, cookie)) = old {
            if let Ok(src) = ctx.cast::<ITfSource>() {
                let _ = unsafe { src.UnadviseSink(cookie) };
            }
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
        if t.trailing != 1 {
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
