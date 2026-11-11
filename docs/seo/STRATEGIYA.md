# bitg.uz — SEO / AEO / GEO strategiyasi

Holat: 2026-11-12. Sayt hozir `bitg.dtwin.uz` da (bitta sahifa, `web/`), `bitg.uz` 24 soatda ochiladi.
Maqsad: «yangi alifbo» mavzusidagi **har turdagi** qidiruvda (Google, Yandex, Bing, ChatGPT, Perplexity, Gemini,
Google AI Overviews, Yandex Alisa) bitg birinchi sahifada va AI javobida manba sifatida chiqsin.

Shu papkadagi fayllar:

| Fayl | Nima |
|---|---|
| `STRATEGIYA.md` | shu hujjat: tamoyillar, sahifa arxitekturasi, ssenariylar, taqvim, KPI |
| `SAYT-AGENTI-UCHUN.md` | saytni qurayotgan agentga qisqa topshiriq (texnik talablar) |
| `ISHGA-TUSHIRISH.md` | bitg.dtwin.uz → bitg.uz ko'chish kuni qadamlar (runbook) |
| `sahifalar.json` | ~50 sahifa brifi: slug, title, description, H1, hook, kalit so'zlar, bo'limlar, schema, ichki havolalar |
| `kalit-sozlar.csv` | so'rovlar → klaster → maqsad URL |
| `hooklar.md` | sarlavha, Telegram post, video, OG rasm uchun hooklar kutubxonasi |
| `kontent/*.md` | eng muhim sahifalarning tayyor matni |
| `public/` | `robots.txt`, `llms.txt`, `sitemap.xml`, JSON-LD namunalar, Caddy redirect |

---

## 1. Eng muhim 7 xulosa (ustuvorlik tartibida)

### 1.1 ⛔ Sayt matni faqat yangi alifboda — qidiruvda bu ko'rinmaslik demak
Hozirgi `web/index.html` HTML'i yangi harflarda: «Yuklab **oliş**», «Windows **uçun**», «**Oçiq** kod». Odamlar esa
eski imloda qidiradi: «yuklab olish», «uchun», «klaviatura yuklab olish». Qidiruv tizimlari diakritikani qisman
yig'adi (`ş → s`, `ç → c`), lekin `ç ≠ ch`, `ş ≠ sh`: `uçun` → `ucun`, `uchun` emas. Turkcha IR tadqiqotida
harf shakllari farq qilganda topish sifati sezilarli tushishi ko'rsatilgan.

**Qaror:** indekslanadigan HTML matni (title, H1, description, asosiy matn) **eski lotin imloda**. Yangi alifbo —
misollar, namunalar va «Yangi» tugmasi (JS, alohida URL emas). 2027-apreldan keyin Search Console'da yangi
harfli so'rovlar ulushi oshsa, title'larga yangi shaklni ham qo'shamiz. Brend ovozi (yangi alifbo) — hero
animatsiya, rasm, OG, Telegram postlarda qoladi.

### 1.2 Bitta sahifa = bitta so'rov oilasi. Kamida 45 sahifa kerak
Bitta landing faqat «bitg» va 2–3 so'rov uchun chiqadi. Qidiruv niyatlari 7 xil (dastur, qonun, harflar,
klaviatura, hayotiy savollar, ruscha, inglizcha) — har biri alohida sahifa, alohida title. To'liq ro'yxat:
`sahifalar.json`, qisqasi — 3-bo'lim.

### 1.3 Bizning ustunligimiz — boshqalarda yo'q narsa
Raqobatchilar (ogir.uz, kiritish.uz, tezyechim.uz, alifbochi.uz, alifbo.akbarovlabs.uz, webnum.uz,
transliterator.uz, parsing.uz, GitHub'dagi layoutlar) ikki turda: **klaviatura tartibi** yoki **onlayn
konvertor**. bitg — **yozayotganda istalgan dasturda avtomatik o'giradigan, inglizchani buzmaydigan, ⌫ bilan
qaytadigan** dastur + o'lchangan aniqlik (o'zbekcha 99.17%, inglizcha o'zgarmay qolishi 99.73%). Har sahifada shu
uch farq va raqamlar takrorlanadi — AI «qaysi biri yaxshi» savolida aynan raqam va farqni iqtibos qiladi.

### 1.4 Qidiruv cho'qqilari aniq — kontent undan oldin tayyor bo'lsin
| Sana | Hodisa | Kutilgan qidiruv |
|---|---|---|
| 2026-10-21 | qonun imzolandi | o'tdi (birinchi to'lqin) |
| ~2026-12 oxiri | hukumat amalga oshirish dasturi (qonun: 2 oy ichida) | «yangi alifbo dasturi», «qachon» |
| 2027-01…03 | darslik, klaviatura, tashkilotlar tayyorgarligi | «klaviatura», «konvertor», «word» |
| **2027-04-22** | **qonun kuchga kiradi** | **eng katta cho'qqi** |
| 2027-09 | 1-sinf yangi darsliklari | «bolalar», «maktab», «o'qituvchi» |

Google yangi sahifani 2–8 haftada barqarorlashtiradi → **asosiy 45 sahifa 2027-fevralgacha indeksda** bo'lishi shart.

### 1.5 Yandex — 16–24% (mobilda ko'proq), Bing — ChatGPT'ning ko'zi
O'zbekistonda Google ~75–78%, Yandex ~16–24% (StatCounter 2026). ChatGPT qidiruvi Bing indeksidan oladi.
Shuning uchun **uchala** webmaster panel (Google, Yandex, Bing) + IndexNow birinchi kuni ulanadi.

### 1.6 AI manbani qanday tanlaydi
Ahrefs ma'lumotiga ko'ra ChatGPT va AI Overviews iqtiboslari atigi ~11% ustma-ust tushadi — har platforma
alohida. AI Overviews iqtiboslarining katta qismi organik top-10 dan tashqaridagi sahifalardan. Ishlaydigan
naqshlar: har H2 ostida **1–2 gaplik to'g'ridan-to'g'ri javob** ("answer capsule"), birlamchi manbaga havola
(lex.uz, gazeta.uz), aniq raqam va sana, jadval, «yangilangan» sanasi, tashqi saytlarda tilga olinish
(Vikipediya, YouTube, GitHub, yangiliklar, Reddit/Habr). Real-time tizimlar (Perplexity, AI Overviews, Alisa)
kunlar–haftalarda yangilanadi, o'qitilgan modellar 6–12 oyda: GPTBot va boshqa o'qitish botlariga ruxsat **hozir**
berilishi kerak (sayt reklama maqsadida — modellar bitg'ni «bilishi» bizga foyda).

### 1.7 Telegram — O'zbekistonning asosiy kanali va havolalar manbai
Trafik va tashqi havolalarning katta qismi Telegram'dan keladi. Open Graph rasm/sarlavha har sahifada alohida
bo'ladi, va **Telegram bot** (inline rejim: istalgan chatda `@bot matn` → yangi alifbo) eng kuchli tarqatish
hook'i (5-bo'lim).

---

## 2. Texnik asos (birinchi kun)

| Element | Talab |
|---|---|
| Statik HTML | Har sahifa o'z URL'ida tayyor HTML (SSG). Matn JS'siz ko'rinsin — AI botlarning ko'pi JS ishlatmaydi |
| `<title>` | ≤ 60 belgi, kalit so'z boshida, oxirida `· bitg` |
| `meta description` | 140–160 belgi: javob + harakat |
| `canonical` | har sahifada absolyut `https://bitg.uz/...`; oxiridagi `/` siyosati hamma joyda bir xil |
| `hreflang` | `uz` (lotin), `uz-Cyrl` (`/kr/`), `ru` (`/ru/`), `en` (`/en/`), `x-default` → `/` |
| Open Graph | har sahifaga alohida `og:title`, `og:description`, `og:image` (1200×630, sahifa mavzusi yozilgan) |
| JSON-LD | `Organization` + `WebSite` (hamma sahifada), `SoftwareApplication` (/, /yuklab-olish, klaviatura sahifalari), `Article` (qo'llanmalar), `FAQPage` (savollar), `BreadcrumbList` (ichki sahifalar) |
| Sana | har maqolada ko'rinadigan «Yangilangan: YYYY-MM-DD» + `dateModified` |
| Muallif | `/haqida` sahifasi, maqolalarda muallif (E-E-A-T) |
| robots.txt | hammaga ochiq, AI botlar ochiq (`public/robots.txt`) |
| sitemap.xml | faqat 200 qaytaradigan canonical URL, haqiqiy `lastmod` |
| llms.txt | ixtiyoriy, zarar qilmaydi (`public/llms.txt`) |
| IndexNow | kalit fayl ildizda; deploy'dan keyin o'zgargan URL'lar Bing+Yandex'ga yuboriladi |
| Tezlik | WASM (1.4 MB) faqat `/konvertor` va sinov bo'limida, lazy; boshqa sahifalarda yuklanmasin |
| 404 | foydali 404: konvertor + asosiy havolalar |
| www | `www.bitg.uz` → `bitg.uz` 301 |
| Ichki havola | har sahifa pastida «Bog'liq»: 3–5 havola (`sahifalar.json` → `links`) + breadcrumb |

**FAQPage / HowTo haqida:** Google 2023-yildan FAQ rich result'ni asosan davlat/sog'liq saytlariga cheklagan,
HowTo rich result'ni o'chirgan. `FAQPage`ni baribir qo'yamiz (Bing, Yandex va AI tizimlari o'qiydi), lekin
Google'da alohida ko'rinish kutmaymiz.

**Apostrof:** HTML'da `o‘ g‘` — U+2018, tutuq — U+2019 (kun.uz, gazeta.uz kabi). U+02BB (rasmiy ʻ) Unicode'da
*harf* sanaladi, shuning uchun qidiruv tizimi `oʻzbek`ni bitta token deb, `o'zbek` so'rovi bilan moslamasligi
mumkin; U+2018 tinish belgisi, `'` bilan bir xil ishlaydi. 4 haftadan keyin Search Console so'rovlarida tekshiramiz.

**Telegram:** kanal [t.me/bitguz](https://t.me/bitguz), bot [@bitguzbot](https://t.me/bitguzbot) — saytning footer'ida,
`Organization.sameAs` va `llms.txt` da.

---

## 3. Sahifa arxitekturasi (to'liq — `sahifalar.json`)

```
/                                   bosh sahifa — «Eskicha yozing, yangicha chiqadi»
/yuklab-olish                       Windows MSI, versiya, SHA-256, VirusTotal, tizim talablari
/konvertor                          onlayn konvertor (WASM, internetsiz): eski↔yangi, kirill→yangi
/yangi-alifbo                       PILLAR: Yangi o'zbek alifbosi 2026 — 28 harf, nima o'zgardi
/yangi-alifbo-qonuni                O'RQ-1184: sanalar, nimani o'zgartiradi va nimani o'zgartirmaydi
/yangi-alifbo-qachon                countdown: 2027-yil 22-aprelgacha N kun
/harflar                            yangi harflar umumiy
  /harflar/o                        Oʻ → Ö
  /harflar/g                        Gʻ → Ğ
  /harflar/sh                       Sh → Ş
  /harflar/ch                       Ch → Ç
  /harflar/ng                       Ng nima bo'ldi
  /harflar/tutuq-belgisi            tutuq belgisi ʼ qoladi
/yangi-harflarni-yozish             ş ç ğ ö qanday yoziladi (kompyuter + telefon, hammasi bir joyda)
/klaviatura                         hub: qaysi qurilma
  /klaviatura/windows
  /klaviatura/mac
  /klaviatura/linux                 (bitg: reja; hozir IBus layout yo'li)
  /klaviatura/android
  /klaviatura/iphone
  /klaviatura/tartiblar             Tilmoch / Turkcha Q / AltGr / maxsus — taqqoslash
/kirilldan-yangi-alifboga
/eski-va-yangi-alifbo-farqi
/imlo                               yangi alifboda yozish (rasmiy imlo qoidalari chiqquncha — ehtiyotkor matn)
/savollar                           FAQ, 30 savol
/pasport-va-hujjatlar               pasport, diplom, familiya
/maktab                             darsliklar 2027–2031, ota-ona, o'qituvchi
/yangi-alifbo-testi                 10 savollik test (ulashiladigan natija) — HOOK
/ismlar                             ismingiz yangi alifboda (qidiruv + ulashish kartasi) — HOOK
  /ismlar/{ism}                     top-100 ism (3-to'lqin, sifat nazorati bilan)
/shaharlar                          viloyat, shahar, tuman nomlari yangi alifboda
/word-va-excel                      hujjatni o'girish (bitg: reja; hozir — konvertor orqali)
/tashkilotlar                       kompaniya/idora uchun o'tish: GPO/Intune jim o'rnatish
/dasturchilar                       CLI, Rust, WASM, C ABI, ochiq kod
/qiyoslash                          bitg vs klaviatura tartiblari vs onlayn konvertorlar (halol jadval)
/telefonda                          telefonda hozir nima qilish (bosib turish, Tilmoch)
/yangiliklar                        xronologiya: qonun → dastur → kuchga kirish
/haqida                             loyiha, muallif, aloqa
/maxfiylik                          matn saqlanmaydi, tarmoq yo'q
/kr/  /kr/yangi-alifbo  /kr/klaviatura  /kr/savollar               (o'zbek kirill)
/ru/  /ru/novyy-uzbekskiy-alfavit  /ru/raskladka  /ru/konverter  /ru/zakon  /ru/voprosy
/en/  /en/new-uzbek-alphabet  /en/keyboard  /en/converter
```

**To'lqinlar** (`sahifalar.json` → `wave`):
- **1-to'lqin (ishga tushish haftasi, 15 ta):** `/`, `/yuklab-olish`, `/konvertor`, `/yangi-alifbo`,
  `/yangi-alifbo-qonuni`, `/yangi-harflarni-yozish`, `/klaviatura`, `/klaviatura/windows`, `/savollar`,
  `/qiyoslash`, `/haqida`, `/maxfiylik`, `/ru/`, `/ru/novyy-uzbekskiy-alfavit`, `/en/`.
- **2-to'lqin (dekabr):** harflar (7), qolgan klaviatura (5), kirill, farq, pasport, maktab, qachon, telefonda,
  tashkilotlar, dasturchilar, `/kr/*`, qolgan `/ru/*`, `/en/*`.
- **3-to'lqin (yanvar–mart):** test, ismlar, shaharlar, imlo, word-va-excel, yangiliklar, `/ismlar/{ism}`.

**Dasturiy (programmatic) sahifalar — ehtiyot:** Google'ning «scaled content abuse» siyosati bir xil shablonli
yupqa sahifalarni jazolaydi. `/ismlar/{ism}` faqat har birida haqiqiy qo'shimcha bo'lsa: eski / yangi / kirill /
pasportdagi lotin shakli, ma'nosi va kelib chiqishi (manba bilan), o'xshash ismlar. Boshlanishi 100 ta;
Search Console'da ko'rinish bo'lsa — kengaytiriladi. Har so'z uchun sahifa (`/soz/choy`) **qilinmaydi**.

---

## 4. Har sahifa shabloni (AEO/GEO uchun)

```
H1 = asosiy so'rov (eski imloda)
[answer capsule] 1–2 gap: savolga to'g'ridan-to'g'ri javob + sana/raqam
[jadval yoki qadamlar] — ko'z bilan skanerlanadigan
H2 = keyingi savol (odamlar shunday so'raydi) → 1–2 gaplik javob → tafsilot
...
[bitg bloki] — muammoni bitg qanday hal qiladi + CTA (Yuklab olish / Konvertor)
[Manbalar] — lex.uz, gazeta.uz, kun.uz... havolalar
[Yangilangan: sana] [Muallif]
[Bog'liq sahifalar] 3–5 havola
```
Qoidalar: to'qima raqam yo'q; har fakt manbali; bitta sahifa — bitta niyat; bir xil matn ko'p sahifaga
ko'chirilmaydi (har sahifa o'z burchagidan).

---

## 5. Hook strategiyasi (to'liq kutubxona — `hooklar.md`)

Hook — odamni to'xtatib, bosishga undaydigan birinchi gap, rasm yoki vosita. Uch qatlam:

1. **Sarlavha hooklari** (title, H1, OG, Telegram): savol, raqam, foyda, sana.
   «Ş, Ç, Ğ, Ö ni kompyuterda 5 soniyada yozing», «22-aprelgacha 161 kun: kompyuteringiz tayyormi?»
2. **Vosita hooklari (link magnet)** — odamlar o'zi ulashadi, saytlar havola qo'yadi:
   - **Ismingiz yangi alifboda** — ism yoziladi → chiroyli karta (OG rasm) → Telegram'ga ulashish.
   - **Yangi alifbo testi** — 10 savol, natija kartasi.
   - **Countdown** `/yangi-alifbo-qachon` — yangiliklar saytlari havola qo'yadi.
   - **Onlayn konvertor** — eng ko'p qaytib keladigan sahifa.
   - **Telegram bot** — inline rejim: istalgan chatda `@bot matn` → yangi alifbo. Har ishlatishda bot nomi chatda
     ko'rinadi = bepul reklama. bitg-core (CLI/WASM) ustida qisqa ish.
3. **Ijtimoiy isbot hooklari** — faqat haqiqiy raqamlar: yuklab olishlar soni, 99.17% aniqlik (README'dagi
   o'lchov), ochiq kod.

---

## 6. Tashqi signallar (authority) — AI ishonadigan joylar

| Kanal | Nima qilish | Nega |
|---|---|---|
| GitHub `dtwinuz/bitg` | **public** bo'lsin; README birinchi qatorida bitg.uz; topics: `uzbek`, `alphabet`, `keyboard`, `transliteration`, `ime` | AI va dasturchilar uchun asosiy manba |
| Microsoft Store + winget | listing (tavsif = brend jumlasi) | Bing/Copilot signali, ishonch |
| Telegram | `@bitg_uz` kanal + bot; o'zbek IT/ta'lim kanallariga post | asosiy trafik |
| YouTube | 3 qisqa video: «Ş Ç Ğ Ö kompyuterda», «Telefonda», «Matnni bir bosishda o'girish» | «qanday» so'rovlarida video blok; AI manbasi |
| Yangiliklar | kun.uz, daryo.uz, gazeta.uz, spot.uz, podrobno.uz — press-reliz | bitta maqola o'nlab oddiy havoladan qimmat |
| uz.wikipedia | «Oʻzbek alifbosi» maqolasiga mustaqil manba orqali (yangilik maqolasi chiqqach) | Vikipediya qoidalari doirasida, o'zimiz reklama yozmaymiz |
| Ta'lim | o'qituvchilar guruhlari, maktablar uchun bepul PDF qo'llanma | ishonchli havolalar |
| Kataloglar | AlternativeTo, Product Hunt, o'zbek dastur yuklash saytlari | qo'shimcha havola va brend so'rovi |
| Reddit / Habr | `r/uzbekistan`, Habr'da «Rust'da o'zbek transliteratori» texnik maqola | AI'lar Reddit va Habr'ni ko'p iqtibos qiladi |

**Brend birligi:** hamma joyda bir xil nom va bitta jumla:
«**bitg** — yozayotganingizda oʻzbek matnini 2026-yilgi yangi alifboga (ö ğ ş ç) avtomatik oʻgiradigan bepul,
ochiq kodli dastur.» `Organization.sameAs` ga barcha profil havolalari.

---

## 7. Ssenariylar (nima bo'lsa — nima qilamiz)

| # | Ssenariy | Belgi | Javob |
|---|---|---|---|
| S1 | Hukumat rasmiy klaviatura/konvertor chiqaradi | dastur e'loni, lex.uz | 24 soatda `/yangiliklar` + `/qiyoslash`: «rasmiy tartib + bitg avto-o'girish birga ishlaydi»; rasmiy layout bo'lsa — bitg'ga shu tartib qo'shiladi va sahifada «rasmiy tartib bitg'da bor» |
| S2 | Microsoft Windows'ga yangi o'zbek layout qo'shadi | Windows update | `/klaviatura/windows`: o'rnatilgan layout yo'li + «eskicha yozganingizni o'giradigani — bitg». Farq: avto-o'girish, inglizcha himoya |
| S3 | Raqobatchi «yangi alifbo klaviatura»da yuqorida | GSC pozitsiya | sahifani chuqurlashtirish (skrinshot, video, jadval), tashqi havola kampaniyasi, `/qiyoslash`da halol taqqoslash |
| S4 | Qonun muddati o'zgaradi | rasmiy xabar | `/yangi-alifbo-qachon` va barcha sanali sahifalar bitta sozlamadan (`lawDate`) yangilanadi; yangilik posti |
| S5 | 22-aprel trafik portlashi | GSC/analytics | statik sayt + CDN; MSI GitHub Releases'da ham; konvertor to'liq brauzerda — server yuki yo'q |
| S6 | AI noto'g'ri javob beradi (eski ma'lumot, boshqa dastur) | oylik AI testi | answer capsule'ni aniqlashtirish, sana qo'shish, IndexNow bilan qayta indekslash, tashqi manbada to'g'ri faktni chiqarish |
| S7 | Alifbo tanqidi / siyosiy bahs | ijtimoiy tarmoq | betaraf: «biz o'tishni osonlashtiramiz»; faktlar, fikr emas; eskiga qaytarish (`to_old`) ham borligini ko'rsatish — «ikki tomonga» |
| S8 | Rasmiy imlo qoidalari chiqadi | lex.uz | `/imlo` qayta yoziladi, bitg-core qoidalari tekshiriladi, changelog |
| S9 | Defender/SmartScreen ogohlantirishi (imzosiz MSI) | foydalanuvchi shikoyati | `/yuklab-olish`da «Batafsil → Baribir ishga tushirish» rasmli yo'riqnoma, VirusTotal havolasi, SHA-256, ochiq kod |
| S10 | Kimdir bitg nomi/sayti nusxasini qiladi | brend qidiruvi | `Organization` schema, GitHub, Store — rasmiy manba aniq |

---

## 8. Taqvim (2026-11-12 dan)

| Muddat | Ish |
|---|---|
| **T0 (bugun, 24 soat)** | shu papka; sayt agenti multi-page'ga o'tadi; 1-to'lqin matnlari (`kontent/`); OG rasmlar; robots/llms/sitemap tayyor |
| **T+1 (bitg.uz ochildi)** | `ISHGA-TUSHIRISH.md`: deploy, 301, GSC (Domain property, DNS TXT), Yandex Webmaster, Bing (GSC'dan import), IndexNow, sitemap |
| 1-hafta | 15 sahifa jonli; Telegram kanal + bot; GitHub public + README; winget PR |
| 2–4-hafta | 2-to'lqin (~30 sahifa); YouTube 3 video; Habr/Reddit texnik post |
| Dekabr oxiri | hukumat dasturi chiqqan kuni → `/yangiliklar` + barcha sahifa sanalari; press-reliz №1 |
| Yanvar–mart | 3-to'lqin (test, ismlar, shaharlar); tashkilotlarga B2B xat; o'qituvchilar uchun PDF |
| Aprel 1–21 | press-reliz №2 («22-aprelga tayyormisiz»), countdown; hamma sahifada yangilangan sana |
| **22-aprel** | kuchga kirish kuni: yangilik, Telegram; title'larga yangi harfli shaklni qo'shish GSC bo'yicha hal qilinadi |
| Sentabr | maktab mavsumi: `/maktab`, ota-onalar uchun |

---

## 9. KPI va monitoring

| Ko'rsatkich | Maqsad (2027-04-22 gacha) | Qayerdan |
|---|---|---|
| Indekslangan sahifalar | ≥ 45 | GSC → Pages, Yandex Webmaster |
| «yangi alifbo klaviatura» pozitsiya | top-3 (Google, Yandex) | GSC → Performance |
| «yangi alifbo» klasteri ko'rsatishlar | har oy o'sish | GSC |
| «bitg» brend so'rovlari | o'sish | GSC |
| Yuklab olishlar | birinchi oy — bazaviy qiymat | GitHub Releases API + sayt |
| AI'da tilga olinish | 20 savoldan ≥ 10 tasida bitg | oylik qo'lda test |

**Oylik AI testi** (ChatGPT, Perplexity, Gemini, Copilot, Yandex Alisa, Google AI Overview — har biriga):
1. Yangi o'zbek alifbosi uchun kompyuter dasturi bormi?
2. Ş, Ç, Ğ, Ö ni Windows'da qanday yozaman?
3. Matnni yangi o'zbek alifbosiga avtomatik o'giradigan dastur?
4. Yangi o'zbek alifbosi qachon kuchga kiradi?
5. Yangi alifbo klaviatura yuklab olish
6. Как печатать новые узбекские буквы на клавиатуре?
7. Best keyboard for the new Uzbek alphabet
8. bitg nima?
9. Kirill matnni yangi lotin alifbosiga o'girish
10. Yangi alifbo uchun eng yaxshi konvertor qaysi?

(+10 ta mavzuga qarab.) Natija jadvalga: tilga olindimi, havola bormi, faktlar to'g'rimi.

---

## Manbalar
- Qonun va sanalar: [gazeta.uz 2026-10-21](https://www.gazeta.uz/oz/2026/10/21/alphabet/), [kun.uz](https://kun.uz/news/2026/10/21/ozbek-lotin-alifbosini-takomillashtirishga-doir-qonun-prezident-tomonidan-imzolandi), [infoedu.uz — 22-aprel](https://infoedu.uz/yangi-alifbo-qonuni-2027-yil-22-aprelda-kuchga-kiradi), [daryo — darsliklar 2031](https://daryo.uz/en/2026/09/10/uzbekistan-to-fully-switch-school-textbooks-to-updated-alphabet-by-2031/), [anhor — hujjatlar](https://anhor.uz/uzl/news/rasman-alifbo-ozgaradi-amaldagi-hujjatlar-va-pullar-oz-kuchida-qoladi)
- Qidiruv ulushi: [StatCounter Uzbekistan](https://gs.statcounter.com/search-engine-market-share/all/uzbekistan)
- Diakritika va qidiruv: [TÜBİTAK — Effects of diacritics on Turkish IR](https://aj.tubitak.gov.tr/elektrik/issues/elk-12-20-5/elk-20-5-9-1010-819.pdf)
- GEO: [Erlin — GEO trends 2026](https://www.erlin.ai/blog/generative-engine-optimization-trends), [Yandex Neuro](https://yandex.com/company/news/16-04-2024)
- Domen ko'chishi: [Google Change of Address](https://support.google.com/webmasters/answer/9370220)
- OpenAI botlari: https://platform.openai.com/docs/bots
