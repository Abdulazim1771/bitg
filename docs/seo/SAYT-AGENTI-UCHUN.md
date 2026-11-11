# Sayt agenti uchun: bitg.uz SEO talablari

Bu topshiriq `web/` ni qurayotgan agent uchun. To'liq asos: `docs/seo/STRATEGIYA.md`.
Sahifalar ro'yxati va har birining title/description/H1/bo'limlari: `docs/seo/sahifalar.json`.
Tayyor matnlar: `docs/seo/kontent/`. Statik fayllar: `docs/seo/public/`.

## Majburiy (bularsiz SEO ishlamaydi)

1. **Ko'p sahifa, statik HTML.** Har `slug` uchun o'z `index.html` (masalan `web/yangi-alifbo/index.html`).
   Matn JS'siz HTML ichida bo'lsin. Umumiy header/footer — build vaqtida shablon (qo'lda nusxalash emas).
2. **Indekslanadigan matn eski lotin imloda — `index.html` da 2026-11-12 da BAJARILDI.** HTML eski imloda,
   standart holat `data-abc` yo'q = eski; «Yangi» tugmasi `toNew` bilan o'giradi (`app.js` → `T()`), tanlov
   `localStorage['bitg-abc']='new'`. Apostrof: `o‘ g‘` = U+2018, tutuq = U+2019 (kun.uz/gazeta.uz bilan bir xil,
   odamlar `o'` yozib qidirganda mos keladi). `data-no-conv` ichidagi UI yorliqlari `data-conv` bilan qayta
   yoqiladi. Yangi sahifalar ham shu qoidada yoziladi: matn eski imloda, demo/misol `data-no-conv`.
3. **Base URL bitta joyda** (`SITE = "https://bitg.uz"`). canonical, og:url, sitemap, hreflang, JSON-LD shundan.
   Hozir `bitg.dtwin.uz` yozilgan joylar (robots.txt, sitemap.xml, og:url, og:image, canonical) — shu o'zgaruvchiga.
4. **Har sahifada:** `<title>`, `meta description`, `canonical`, `og:*` (sahifaga xos `og:image`),
   `hreflang` to'plami (uz / uz-Cyrl / ru / en / x-default — sahifaning tarjimasi bo'lsa), JSON-LD
   (`public/jsonld/` namunalar), breadcrumb (ichki sahifalarda), «Yangilangan: YYYY-MM-DD», pastda «Bog'liq»
   bloki (`links`).
5. **`<html lang>`**: `uz` (lotin), `uz-Cyrl`, `ru`, `en`.
6. **WASM faqat kerakli sahifada** (`/konvertor`, bosh sahifa sinov bo'limi) — lazy. Boshqa sahifalar
   yengil bo'lsin (LCP < 2.5 s mobil).
7. **Fayllar ildizda:** `robots.txt`, `sitemap.xml`, `llms.txt` (`public/` dan), IndexNow kalit fayli
   (`<kalit>.txt`, kalitni foydalanuvchi bilan yaratish), `favicon.ico`, `apple-touch-icon.png`.
8. **404 sahifa** (`404.html`): konvertor havolasi + asosiy bo'limlar.
9. **Sanalar config'dan:** `lawNo = "O'RQ-1184"`, `signedDate = 2026-10-21`, `lawDate = 2027-04-22`.
   Countdown statik matn + JS hisob (JS'siz ham sana ko'rinsin).
10. **Mahsulotda yo'q narsa sahifada «reja» deb yoziladi** (Mac, Linux, Android, iOS, Word fayl o'girish).
    Hech qayerda to'qima raqam, sharh yoki foydalanuvchi soni yo'q.

## 1-to'lqin (birinchi deploy — 15 sahifa)
`/`, `/yuklab-olish`, `/konvertor`, `/yangi-alifbo`, `/yangi-alifbo-qonuni`, `/yangi-harflarni-yozish`,
`/klaviatura`, `/klaviatura/windows`, `/savollar`, `/qiyoslash`, `/haqida`, `/maxfiylik`, `/ru/`,
`/ru/novyy-uzbekskiy-alfavit`, `/en/`. Matni tayyor bo'lganlari: `kontent/` ichida.

## Tekshiruv (deploy oldidan)
- [ ] `curl -s https://<host>/<slug>/ | grep -c '<h1'` = 1, title va description bor, JS'siz matn ko'rinadi
- [ ] canonical = `https://bitg.uz/<slug>/` (oxirgi `/` siyosati bir xil)
- [ ] sitemap'dagi har URL 200 qaytaradi
- [ ] Rich Results Test / Schema validator: JSON-LD xatosiz
- [ ] Telegram'da havola preview (har sahifa o'z rasmi bilan)
- [ ] Lighthouse mobil: SEO 100, Performance ≥ 90
