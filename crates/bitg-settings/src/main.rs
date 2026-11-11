//! bitg sozlamalari oynasi. Sozlamalar darhol `%APPDATA%\bitg` ga yoziladi; IME ularni bir soniya ichida,
//! dasturlarni qayta ochmasdan qabul qiladi. Oldindan ko'rish IME bilan bir xil yadroni ishlatadi.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

#[cfg_attr(not(windows), allow(dead_code))]
mod logic;

#[cfg(not(windows))]
fn main() {
    eprintln!("bitg-settings hozircha faqat Windows uchun.");
}

#[cfg(windows)]
fn main() {
    if let Err(e) = app::run() {
        eprintln!("bitg-settings: {e}");
        std::process::exit(1);
    }
}

#[cfg(windows)]
mod app {
    use serde_json::Value;
    use tao::dpi::LogicalSize;
    use tao::event::{Event, WindowEvent};
    use tao::event_loop::{ControlFlow, EventLoopBuilder};
    use tao::window::WindowBuilder;
    use wry::{WebContext, WebViewBuilder};

    use crate::logic;

    const PAGE: &str = include_str!("ui.html");

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let event_loop = EventLoopBuilder::<String>::with_user_event().build();
        let proxy = event_loop.create_proxy();
        let window = WindowBuilder::new()
            .with_title("bitg sozlamalari")
            .with_inner_size(LogicalSize::new(980.0, 680.0))
            .with_min_inner_size(LogicalSize::new(420.0, 520.0))
            .build(&event_loop)?;

        // WebView2 ma'lumotlari: Program Files'ga yozib bo'lmaydi — foydalanuvchi papkasiga.
        let data = std::env::var_os("LOCALAPPDATA")
            .map(|d| std::path::PathBuf::from(d).join("bitg").join("webview"));
        let mut ctx = WebContext::new(data);
        let webview = WebViewBuilder::new_with_web_context(&mut ctx)
            .with_html(PAGE)
            .with_devtools(cfg!(debug_assertions))
            .with_ipc_handler(move |req| {
                let reply = handle(req.body());
                if let Some(r) = reply {
                    let _ = proxy.send_event(r.to_string());
                }
            })
            .build(&window)?;

        event_loop.run(move |event, _, flow| {
            *flow = ControlFlow::Wait;
            match event {
                Event::UserEvent(json) => {
                    // JSON — JavaScript ifodasi sifatida ham to'g'ri.
                    let _ = webview.evaluate_script(&format!("window.bitg && bitg.recv({json})"));
                }
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => *flow = ControlFlow::Exit,
                _ => {}
            }
        });
    }

    fn handle(body: &str) -> Option<Value> {
        let msg: Value = serde_json::from_str(body).ok()?;
        match msg.get("cmd")?.as_str()? {
            "load" => Some(logic::load()),
            "save" => Some(logic::save(&msg)),
            "preview" => Some(logic::preview(&msg)),
            "open_folder" => {
                if let Some(dir) = bitg_tsf::config::dir() {
                    let _ = std::fs::create_dir_all(&dir);
                    let _ = std::process::Command::new("explorer").arg(dir).spawn();
                }
                None
            }
            _ => None,
        }
    }
}
