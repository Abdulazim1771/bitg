//! TSF xizmati: probel/Enter/Tab bosilganda kursor oldidagi so'zni o'qib, yangi alifboga o'giradi.
//! Tugmalar hech qachon "yeyilmaydi" (OnKeyDown doim FALSE) — dastur o'z ishini odatdagidek qiladi.

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::rc::Rc;

use bitg_core::{Options, Session};
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_BACK, VK_CONTROL, VK_DELETE, VK_DOWN, VK_END, VK_ESCAPE, VK_HOME, VK_LEFT,
    VK_MENU, VK_NEXT, VK_PRIOR, VK_RETURN, VK_RIGHT, VK_SPACE, VK_TAB, VK_UP,
};
use windows::Win32::UI::TextServices::{
    ITfContext, ITfEditSession, ITfEditSession_Impl, ITfKeyEventSink, ITfKeyEventSink_Impl,
    ITfKeystrokeMgr, ITfRange, ITfTextInputProcessor, ITfTextInputProcessor_Impl,
    ITfTextInputProcessorEx, ITfTextInputProcessorEx_Impl, ITfThreadMgr, TF_AE_END, TF_ANCHOR_END,
    TF_DEFAULT_SELECTION, TF_E_SYNCHRONOUS, TF_ES_ASYNCDONTCARE, TF_ES_READWRITE, TF_ES_SYNC,
    TF_SELECTION, TF_SELECTIONSTYLE,
};
use windows_core::{BOOL, GUID, IUnknownImpl, Interface, Ref, Result, implement};

use crate::log;
use crate::plan;

/// Kursor oldidan o'qiladigan eng ko'p UTF-16 birlik.
const LOOKBACK: i32 = 64;

#[derive(Default)]
struct State {
    thread_mgr: Option<ITfThreadMgr>,
    tid: u32,
    session: Session,
    opts: Options,
}

#[implement(ITfTextInputProcessorEx, ITfTextInputProcessor, ITfKeyEventSink)]
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

fn pressed(vk: u16) -> bool {
    let s = unsafe { GetKeyState(vk as i32) };
    s < 0
}

impl ITfTextInputProcessor_Impl for TextService_Impl {
    fn Activate(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32) -> Result<()> {
        self.ActivateEx(ptim, tid, 0)
    }

    fn Deactivate(&self) -> Result<()> {
        let mut st = self.state.borrow_mut();
        if let Some(tm) = st.thread_mgr.take() {
            if let Ok(ks) = tm.cast::<ITfKeystrokeMgr>() {
                let _ = unsafe { ks.UnadviseKeyEventSink(st.tid) };
            }
        }
        st.session.reset();
        log::write("deactivate");
        Ok(())
    }
}

impl ITfTextInputProcessorEx_Impl for TextService_Impl {
    fn ActivateEx(&self, ptim: Ref<'_, ITfThreadMgr>, tid: u32, _flags: u32) -> Result<()> {
        let tm = ptim.ok()?.clone();
        let sink: ITfKeyEventSink = self.to_object().to_interface();
        let ks: ITfKeystrokeMgr = tm.cast()?;
        unsafe { ks.AdviseKeyEventSink(tid, &sink, true)? };
        {
            let mut st = self.state.borrow_mut();
            st.thread_mgr = Some(tm);
            st.tid = tid;
            st.session.reset();
        }
        // Lug'atni oldindan yuklash: birinchi so'zda kechikish bo'lmasin.
        std::thread::spawn(bitg_core::lang::warm_up);
        log::write("activate");
        Ok(())
    }
}

impl ITfKeyEventSink_Impl for TextService_Impl {
    fn OnSetFocus(&self, _foreground: BOOL) -> Result<()> {
        self.state.borrow_mut().session.reset();
        Ok(())
    }

    fn OnTestKeyDown(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnTestKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnKeyDown(&self, pic: Ref<'_, ITfContext>, w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        let r = catch_unwind(AssertUnwindSafe(|| {
            self.key_down(pic.ok().ok(), w.0 as u16)
        }));
        if r.is_err() {
            log::write("panic in key_down");
        }
        Ok(false.into())
    }

    fn OnKeyUp(&self, _pic: Ref<'_, ITfContext>, _w: WPARAM, _l: LPARAM) -> Result<BOOL> {
        Ok(false.into())
    }

    fn OnPreservedKey(&self, _pic: Ref<'_, ITfContext>, _guid: *const GUID) -> Result<BOOL> {
        Ok(false.into())
    }
}

impl TextService_Impl {
    fn key_down(&self, ctx: Option<&ITfContext>, vk: u16) {
        if pressed(VK_CONTROL.0) || pressed(VK_MENU.0) {
            return;
        }
        let k = windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY(vk);
        match k {
            VK_SPACE | VK_RETURN | VK_TAB => {
                if let Some(ctx) = ctx {
                    self.convert_before_caret(ctx);
                }
                if k == VK_RETURN {
                    self.state.borrow_mut().session.reset();
                }
            }
            VK_LEFT | VK_RIGHT | VK_UP | VK_DOWN | VK_HOME | VK_END | VK_PRIOR | VK_NEXT
            | VK_BACK | VK_DELETE | VK_ESCAPE => self.state.borrow_mut().session.reset(),
            _ => {}
        }
    }

    fn convert_before_caret(&self, ctx: &ITfContext) {
        let tid = self.state.borrow().tid;
        let es: ITfEditSession = EditSession {
            ctx: ctx.clone(),
            state: self.state.clone(),
        }
        .into();
        unsafe {
            match ctx.RequestEditSession(tid, &es, TF_ES_SYNC | TF_ES_READWRITE) {
                Ok(hr) if hr == TF_E_SYNCHRONOUS => {
                    // Dastur sinxron qulfni bermadi — probel yozilgandan keyin ishlaymiz.
                    if let Err(e) =
                        ctx.RequestEditSession(tid, &es, TF_ES_ASYNCDONTCARE | TF_ES_READWRITE)
                    {
                        log::write(&format!("async request failed hr={:#x}", e.code().0));
                    }
                }
                Ok(_) => {}
                Err(e) => log::write(&format!("sync request failed hr={:#x}", e.code().0)),
            }
        }
    }
}

#[implement(ITfEditSession)]
struct EditSession {
    ctx: ITfContext,
    state: Rc<RefCell<State>>,
}

impl ITfEditSession_Impl for EditSession_Impl {
    fn DoEditSession(&self, ec: u32) -> Result<()> {
        match catch_unwind(AssertUnwindSafe(|| self.run(ec))) {
            Ok(Ok(outcome)) => {
                if let Some(o) = outcome {
                    log::write(o);
                }
                Ok(())
            }
            Ok(Err(e)) => {
                log::write(&format!("edit failed hr={:#x}", e.code().0));
                Err(e)
            }
            Err(_) => {
                log::write("panic in edit session");
                Ok(())
            }
        }
    }
}

impl EditSession_Impl {
    /// Natija log uchun: None = hech narsa qilinmadi.
    fn run(&self, ec: u32) -> Result<Option<&'static str>> {
        let caret = match self.caret(ec)? {
            Some(r) => r,
            None => return Ok(None),
        };

        // Kursor oldidagi matn.
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
        let reached_start = moved > -LOOKBACK;

        let Some(t) = plan::word_before_caret(&buf, reached_start) else {
            return Ok(None);
        };
        let edit = {
            let mut st = self.state.borrow_mut();
            let st = &mut *st;
            st.session.complete_word(&t.word, &st.opts)
        };
        let Some(edit) = edit else {
            return Ok(Some("word unchanged"));
        };

        // Almashtiriladigan diapazon: [kursor - trailing - word_len, kursor - trailing).
        let target = unsafe { caret.Clone()? };
        let back = (t.trailing + t.word_len) as i32;
        let mut m = 0i32;
        unsafe {
            target.ShiftStart(ec, -back, &mut m, std::ptr::null())?;
            if m != -back {
                return Ok(Some("range shift mismatch"));
            }
            target.ShiftEnd(ec, -(t.trailing as i32), &mut m, std::ptr::null())?;
        }
        let text: Vec<u16> = edit.insert.encode_utf16().collect();
        unsafe { target.SetText(ec, 0, &text)? };

        // Kursorni so'zdan (va bo'shliqdan) keyinga qo'yish.
        let after = unsafe { target.Clone()? };
        unsafe {
            after.Collapse(ec, TF_ANCHOR_END)?;
            if t.trailing > 0 {
                after.ShiftEnd(ec, t.trailing as i32, &mut m, std::ptr::null())?;
                after.Collapse(ec, TF_ANCHOR_END)?;
            }
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
        Ok(Some(if t.trailing == 0 {
            "converted sync"
        } else {
            "converted async"
        }))
    }

    /// Bo'sh tanlov (kursor) bo'lsa — uning diapazoni.
    fn caret(&self, ec: u32) -> Result<Option<ITfRange>> {
        let mut sel = [TF_SELECTION::default()];
        let mut fetched = 0u32;
        unsafe {
            self.ctx
                .GetSelection(ec, TF_DEFAULT_SELECTION, &mut sel, &mut fetched)?
        };
        let [s] = sel;
        let range = std::mem::ManuallyDrop::into_inner(s.range);
        if fetched != 1 {
            return Ok(None);
        }
        let Some(range) = range else { return Ok(None) };
        if unsafe { range.IsEmpty(ec)? }.as_bool() {
            Ok(Some(range))
        } else {
            Ok(None) // matn belgilangan — tegmaymiz
        }
    }
}

#[allow(dead_code)]
fn _iids() -> [GUID; 2] {
    [ITfTextInputProcessorEx::IID, ITfTextInputProcessor::IID]
}
