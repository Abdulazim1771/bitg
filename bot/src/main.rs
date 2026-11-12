//! @bitguzbot: matnni bitg yadrosi bilan yangi alifboga (yoki /eski bilan eski lotinga) o'giradi.
//! Long polling (getUpdates) — serverda kiruvchi port kerak emas.
//! Muhit: BOT_TOKEN (majburiy), SITE_URL (standart https://bitg.uz), API_URL (faqat sinov).
//! Token URL ichida bo'ladi, shuning uchun xato matnida URL hech qachon chiqarilmaydi.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;
use std::{env, process, thread};

use bitg_core::{OldStyle, Options, convert, to_old};
use serde_json::{Value, json};

/// Telegram xabar chegarasi 4096; UTF-16 hisobidagi farq uchun zaxira.
const CHUNK: usize = 4000;

struct Bot {
    agent: ureq::Agent,
    base: String,
    site: String,
    user: String,
    opts: Options,
    seen_msg: u64,
    seen_inline: u64,
}

impl Bot {
    fn call(&self, method: &str, body: Value) -> Result<Value, String> {
        let url = format!("{}/{method}", self.base);
        let resp = match self.agent.post(&url).send_json(body) {
            Ok(r) => r,
            Err(ureq::Error::Status(code, r)) => {
                let d = r
                    .into_json::<Value>()
                    .ok()
                    .and_then(|v| v["description"].as_str().map(str::to_owned))
                    .unwrap_or_default();
                return Err(format!("{method}: HTTP {code} {d}"));
            }
            Err(e) => return Err(format!("{method}: tarmoq xatosi ({:?})", e.kind())),
        };
        let v: Value = resp
            .into_json()
            .map_err(|_| format!("{method}: javob JSON emas"))?;
        if v["ok"].as_bool() == Some(true) {
            Ok(v["result"].clone())
        } else {
            Err(format!("{method}: {}", v["description"]))
        }
    }

    fn log_err(r: Result<Value, String>) {
        if let Err(e) = r {
            eprintln!("xato: {e}");
        }
    }

    fn new(&self, s: &str) -> String {
        convert(s, &self.opts)
    }

    fn help(&self) -> String {
        format!(
            "Salom! Men bitg botiman.\n\n\
             ✍️ Matn yuboring — yangi alifboda (ö ğ ş ç) qaytaraman.\n\
             ↩️ /eski matn — yangi alifbodan eski lotinga.\n\
             💬 Istalgan chatda: @{u} matn — natijani tanlang.\n\n\
             Kirill matnni ham yuborsangiz bo‘ladi. Inglizcha so‘z, havola va kodga tegmayman.\n\n\
             Kompyuterda yozayotganingizda o‘zi o‘girsin desangiz — Windows uchun bitg: {s}/yuklab-olish/",
            u = self.user,
            s = self.site.trim_end_matches('/')
        )
    }

    fn keyboard(&self) -> Value {
        let dl = format!("{}/yuklab-olish/", self.site.trim_end_matches('/'));
        json!({ "inline_keyboard": [
            [{ "text": "💻 Windows uchun bitg", "url": dl }],
            [{ "text": "💬 Boshqa chatda ishlatish", "switch_inline_query": "" }]
        ]})
    }

    fn send(&self, chat: i64, reply_to: Option<i64>, text: &str, markup: bool) {
        let parts = split(text);
        let n = parts.len();
        for (i, part) in parts.into_iter().enumerate() {
            let mut body = json!({ "chat_id": chat, "text": part, "link_preview_options": { "is_disabled": true } });
            if let (0, Some(id)) = (i, reply_to) {
                body["reply_parameters"] = json!({ "message_id": id, "allow_sending_without_reply": true });
            }
            if markup && i + 1 == n {
                body["reply_markup"] = self.keyboard();
            }
            Self::log_err(self.call("sendMessage", body));
        }
    }

    fn on_message(&mut self, m: &Value) {
        let (Some(chat), Some(text)) = (m["chat"]["id"].as_i64(), m["text"].as_str()) else {
            return;
        };
        self.seen_msg += 1;
        let private = m["chat"]["type"].as_str() == Some("private");
        // Guruhda javob qaysi xabarga ekanini ko'rsatamiz; shaxsiy chatda shart emas.
        let reply_to = if private { None } else { m["message_id"].as_i64() };

        if let Some(cmdline) = text.strip_prefix('/') {
            let end = cmdline.find(char::is_whitespace).unwrap_or(cmdline.len());
            let (head, rest) = cmdline.split_at(end);
            let mut it = head.splitn(2, '@');
            let cmd = it.next().unwrap_or("").to_ascii_lowercase();
            if let Some(to) = it.next() {
                if !to.eq_ignore_ascii_case(&self.user) {
                    return; // boshqa botga yozilgan buyruq
                }
            }
            let rest = rest.trim();
            let source = if rest.is_empty() {
                m["reply_to_message"]["text"]
                    .as_str()
                    .or_else(|| m["reply_to_message"]["caption"].as_str())
            } else {
                Some(rest)
            };
            match cmd.as_str() {
                "start" | "yordam" | "help" => self.send(chat, reply_to, &self.help(), true),
                "eski" | "yangi" => match source {
                    Some(src) if cmd == "eski" && !has_new(src) => self.send(
                        chat,
                        reply_to,
                        "Matnda yangi alifbo harflari (ö ğ ş ç) yo‘q — u allaqachon eski lotinda.",
                        false,
                    ),
                    Some(src) => {
                        let out = if cmd == "eski" {
                            to_old(src, OldStyle::Official)
                        } else {
                            self.new(src)
                        };
                        self.reply_converted(chat, reply_to, src, &out, private);
                    }
                    None => self.send(
                        chat,
                        reply_to,
                        &format!("/{cmd} dan keyin matn yozing yoki biror xabarga javob qilib /{cmd} yuboring."),
                        false,
                    ),
                },
                _ if private => self.send(chat, reply_to, &self.help(), true),
                _ => {}
            }
            return;
        }

        if private {
            let out = self.new(text);
            self.reply_converted(chat, None, text, &out, true);
        }
    }

    fn reply_converted(&self, chat: i64, reply_to: Option<i64>, src: &str, out: &str, markup: bool) {
        if out == src {
            let hint = if has_new(src) {
                "O‘giriladigan so‘z topilmadi — matn allaqachon yangi alifboda. Eski lotinga: /eski"
            } else {
                "O‘giriladigan so‘z topilmadi: matnda o‘, g‘, sh, ch yoki kirill harflari yo‘q."
            };
            self.send(chat, reply_to, hint, false);
        } else {
            self.send(chat, reply_to, out, markup);
        }
    }

    fn on_inline(&mut self, q: &Value) {
        let Some(id) = q["id"].as_str() else { return };
        self.seen_inline += 1;
        let text = q["query"].as_str().unwrap_or("").trim();
        let mut results = Vec::new();
        if !text.is_empty() {
            let new = self.new(text);
            let title = if new == text { "Yangi alifbo (o‘zgarish yo‘q)" } else { "Yangi alifbo: ö ğ ş ç" };
            results.push(article("n", title, &new));
            // Eski lotinga faqat matnda yangi harflar bo'lsa: eski matnga to_old "sʼh" kabi ajratgich qo'yadi.
            if has_new(text) {
                results.push(article("o", "Eski lotin: o‘ g‘ sh ch", &to_old(text, OldStyle::Official)));
            }
        }
        let body = json!({
            "inline_query_id": id,
            "results": results,
            "cache_time": 300,
            "button": { "text": "Kompyuterda ham: bitg", "start_parameter": "inline" }
        });
        Self::log_err(self.call("answerInlineQuery", body));
    }

    fn setup(&self) {
        let cmds = json!({ "commands": [
            { "command": "start", "description": "Boshlash" },
            { "command": "eski", "description": "Yangi alifbodan eski lotinga" },
            { "command": "yangi", "description": "Eski lotin yoki kirilldan yangi alifboga" },
            { "command": "yordam", "description": "Qanday ishlatish" }
        ]});
        Self::log_err(self.call("setMyCommands", cmds));
        Self::log_err(self.call("setMyShortDescription", json!({
            "short_description": "Matnni yangi o‘zbek alifbosiga (ö ğ ş ç) o‘giraman. Kompyuter uchun: bitg.uz"
        })));
        Self::log_err(self.call("setMyDescription", json!({
            "description": "Matn yuboring — yangi o‘zbek alifbosida (ö ğ ş ç) qaytaraman. Kirill ham bo‘ladi. /eski — eski lotinga. Istalgan chatda: @bitguzbot matn.\n\nWindows uchun bepul dastur: bitg.uz"
        })));
    }
}

fn has_new(s: &str) -> bool {
    s.chars().any(|c| "öğşçÖĞŞÇ".contains(c))
}

/// Xabarni CHUNK belgidan oshmaydigan qismlarga bo'ladi; iloji bo'lsa qator yoki bo'sh joyda.
fn split(text: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut rest: &str = text;
    while rest.chars().count() > CHUNK {
        let hard = rest.char_indices().nth(CHUNK).map_or(rest.len(), |(i, _)| i);
        let window = &rest[..hard];
        let cut = window
            .rfind('\n')
            .or_else(|| window.rfind(' '))
            .filter(|&i| i > hard / 2)
            .map_or(hard, |i| i + 1);
        parts.push(rest[..cut].to_owned());
        rest = &rest[cut..];
    }
    if !rest.is_empty() || parts.is_empty() {
        parts.push(rest.to_owned());
    }
    parts
}

fn article(id: &str, title: &str, text: &str) -> Value {
    let desc: String = text.chars().take(120).collect();
    let body: String = text.chars().take(CHUNK).collect();
    json!({
        "type": "article",
        "id": id,
        "title": title,
        "description": desc,
        "input_message_content": { "message_text": body, "link_preview_options": { "is_disabled": true } }
    })
}

fn main() {
    let token = env::var("BOT_TOKEN").unwrap_or_default();
    if token.trim().is_empty() {
        eprintln!("BOT_TOKEN berilmagan");
        process::exit(2);
    }
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(10))
        .timeout_read(Duration::from_secs(70))
        .timeout_write(Duration::from_secs(20))
        .build();
    let mut bot = Bot {
        agent,
        // API_URL faqat sinov uchun (soxta Telegram server).
        base: format!(
            "{}/bot{}",
            env::var("API_URL").unwrap_or_else(|_| "https://api.telegram.org".into()),
            token.trim()
        ),
        site: env::var("SITE_URL").unwrap_or_else(|_| "https://bitg.uz".into()),
        user: "bitguzbot".into(),
        opts: Options::default(),
        seen_msg: 0,
        seen_inline: 0,
    };

    match bot.call("getMe", json!({})) {
        Ok(me) => {
            if let Some(u) = me["username"].as_str() {
                bot.user = u.to_owned();
            }
            println!("ishga tushdi: @{} ({})", bot.user, bot.site);
        }
        Err(e) => {
            eprintln!("token tekshiruvi o'tmadi: {e}");
            process::exit(1);
        }
    }
    bot.setup();

    let mut offset: i64 = 0;
    let mut logged = 0u64;
    loop {
        let poll = json!({ "offset": offset, "timeout": 50, "allowed_updates": ["message", "inline_query"] });
        match bot.call("getUpdates", poll) {
            Ok(Value::Array(updates)) => {
                for u in updates {
                    if let Some(id) = u["update_id"].as_i64() {
                        offset = offset.max(id + 1);
                    }
                    // Bitta xabardagi xato butun botni to'xtatmasin (offset allaqachon oldinga surilgan).
                    let r = catch_unwind(AssertUnwindSafe(|| {
                        if u["message"].is_object() {
                            bot.on_message(&u["message"]);
                        } else if u["inline_query"].is_object() {
                            bot.on_inline(&u["inline_query"]);
                        }
                    }));
                    if r.is_err() {
                        eprintln!("xato: update {} ishlovida panic", u["update_id"]);
                    }
                }
                let total = bot.seen_msg + bot.seen_inline;
                if total >= logged + 50 {
                    logged = total;
                    println!("statistika: {} xabar, {} inline", bot.seen_msg, bot.seen_inline);
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("xato: {e}");
                thread::sleep(Duration::from_secs(5));
            }
        }
    }
}
