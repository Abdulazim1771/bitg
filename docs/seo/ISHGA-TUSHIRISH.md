# Ishga tushirish kuni: bitg.dtwin.uz → bitg.uz

## Hozir (bitg.uz ochilguncha, 24 soat)
- Sayt `bitg.dtwin.uz` da ishlayveradi. Uni **Search Console'ga qo'shmang va sitemap yubormang** — indeksga
  kirmagan sayt ko'chishda muammo bermaydi.
- Kod ichida base URL bitta o'zgaruvchi (`SITE`) — ko'chish = bitta qiymat.
- Tayyorlab qo'ying: IndexNow kaliti (32 belgili hex), OG rasmlar, `public/` fayllari.

## T+0: domen ochilganda (tartib bilan)

| # | Qadam | Tekshiruv |
|---|---|---|
| 1 | DNS: `bitg.uz` va `www.bitg.uz` → dtwin edge (A/AAAA yoki CNAME) | `nslookup bitg.uz` |
| 2 | Caddy: `bitg.uz` sayt bloki + avtomatik TLS; `www.bitg.uz` → `bitg.uz` 301; `bitg.dtwin.uz` → `https://bitg.uz{uri}` 301 (`public/caddy-redirect.txt`). ⛔ dtwin edge'da qo'lda tahrir/`docker cp` yo'q — faqat dtwin repo deploy orqali | `curl -I http://bitg.uz` → 308/301 https; `curl -I https://bitg.dtwin.uz/konvertor/` → 301 `https://bitg.uz/konvertor/` |
| 3 | `SITE = https://bitg.uz` bilan build + deploy | `curl -s https://bitg.uz/ \| grep canonical` |
| 4 | robots.txt, sitemap.xml, llms.txt, IndexNow kalit fayli ildizda | `curl https://bitg.uz/robots.txt` |
| 5 | **Google Search Console** → Domain property `bitg.uz` → DNS TXT yozuvi | Verified |
| 6 | GSC → Sitemaps → `https://bitg.uz/sitemap.xml` | Success |
| 7 | GSC → URL Inspection → 1-to'lqin 15 URL → «Request indexing» | Kuniga cheklov bor — eng muhimidan boshlang |
| 8 | **Bing Webmaster Tools** → «Import from Google Search Console» | sitemap avtomatik |
| 9 | **Yandex Webmaster** → sayt qo'shish (meta yoki DNS) → sitemap → «Pereobxod stranic» | Verified |
| 10 | IndexNow: barcha URL'larni yuborish (`api.indexnow.org` — Bing va Yandex ikkalasiga boradi) | HTTP 200/202 |
| 11 | GitHub README, Telegram kanal, winget/Store tavsifi → `bitg.uz` havolasi | — |
| 12 | Telegram'da har sahifa preview'ini tekshirish (`@WebpageBot` bilan keshni yangilash) | rasm va sarlavha to'g'ri |

**Change of Address (GSC):** faqat `bitg.dtwin.uz` oldindan GSC'da tasdiqlangan va indekslangan bo'lsa kerak.
Hozir indeksda deyarli yo'q, shuning uchun 301 yetarli; agar GSC'da `bitg.dtwin.uz` sahifalari ko'rinsa —
`dtwin.uz` domain property orqali Change of Address yuboriladi.

## IndexNow so'rovi namunasi
```bash
curl -s -X POST https://api.indexnow.org/indexnow -H 'Content-Type: application/json; charset=utf-8' -d '{
  "host": "bitg.uz",
  "key": "<KALIT>",
  "keyLocation": "https://bitg.uz/<KALIT>.txt",
  "urlList": ["https://bitg.uz/", "https://bitg.uz/konvertor/", "https://bitg.uz/yangi-alifbo/"]
}'
```
Har deploy'dan keyin o'zgargan URL'lar shu yo'l bilan yuborilsin (CI qadami).

## T+7 kun
- GSC → Pages: indekslangan / indekslanmagan sabablari
- GSC → Performance: birinchi so'rovlar (qaysi imloda qidirilmoqda — `o'` / `ö` / apostrofsiz)
- Yandex Webmaster → «Poiskovye zaprosy»
- AI testi (`STRATEGIYA.md` 9-bo'lim) — bazaviy qiymat
