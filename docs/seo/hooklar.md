# Hooklar kutubxonasi

Hook — birinchi 1–3 soniyada to'xtatadigan gap, rasm yoki vosita. Qoidalar:
- faqat haqiqat: to'qima raqam, soxta sharh, «100 000 foydalanuvchi» kabi tekshirilmagan gap yo'q;
- bitta hook — bitta va'da; va'da sahifaning birinchi ekranida bajarilsin (aks holda odam qaytib ketadi va bu
  qidiruvda yomon signal);
- indekslanadigan sarlavhalar eski imloda, Telegram/OG/video ichida yangi alifbo ham bo'ladi (brend ovozi).

## 1. Asosiy brend hooki
- **«Eskicha yozing. Yangicha chiqadi.»** (bosh sahifa H1 — saqlanadi)
- Yangi alifboda: «Eskiça yozing. Yangiça çiqadi.» — OG rasm va Telegram uchun.
- Qo'shimcha: «Qayta o'rganish shart emas.» · «Tugma qidirmang.» · «Inglizchangiz buzilmaydi.»

## 2. Sarlavha formulalari (title / H1 / Telegram)

| Formula | Misol |
|---|---|
| Savol + darhol javob | «Pasportni almashtirish kerakmi? Yo'q — mana nega» |
| Raqam + vaqt | «Ş, Ç, Ğ, Ö ni 5 soniyada yozish» |
| Sana / countdown | «22-aprelgacha [N] kun. Kompyuteringiz tayyormi?» |
| Oldin → keyin | «Toshkent → Toşkent: xaritadagi nomlar qanday o'zgaradi» |
| Shaxsiy | «Ismingiz yangi alifboda qanday yoziladi?» |
| Kutilmagan | «Ng yo'qoldi — lekin «tong» o'zgarmaydi» · «Apostrof yo'qoladi deb o'ylaganmisiz? Yarmi qoladi» |
| Muammo → yechim | «100 betlik hujjat? Qo'lda tuzatmang» |
| Taqqoslash | «Klaviatura, konvertor yoki avto-o'girish: qaysi biri sizga kerak» |
| Sinov | «10 savol. Ko'pchilik 7-savolda adashadi» |

## 3. Vosita hooklari (link magnet) — ustuvorlik tartibida

| # | Vosita | Nega ishlaydi | Ulashish mexanikasi | Hajm |
|---|---|---|---|---|
| 1 | **Telegram bot (inline)** — `@bot matn` istalgan chatda | odamlar Telegram'da yozadi; har ishlatishda bot nomi chatda ko'rinadi | o'zi tarqaladi | bitg-core CLI/WASM ustida kichik servis |
| 2 | **Onlayn konvertor** | eng ko'p qidiriladigan vosita turi, qayta kelinadi | «Natijani ulashish» tugmasi → `t.me/share` | bor (WASM) — alohida sahifaga chiqarish |
| 3 | **Ismingiz yangi alifboda** | shaxsiy, qiziq, 5 soniya | natija kartasi PNG (1200×630) + «Telegram'da ulashish» | kichik (canvas yoki server OG) |
| 4 | **Countdown + embed** | yangiliklar saytlari va blogerlar o'z sahifasiga qo'yadi | `<iframe>`/skript kodi + «bitg.uz» havolasi | kichik |
| 5 | **Yangi alifbo testi** | raqobat, ulashiladigan natija | natija kartasi | o'rta |
| 6 | **Bepul PDF: o'qituvchi/ota-ona qo'llanmasi** | maktab guruhlarida tarqaladi | PDF ichida bitg.uz | kichik |
| 7 | **Klaviatura stikeri (chop etish uchun PDF)** | tugmaga yopishtiriladigan ö ğ ş ç | ofis, maktab | kichik |

## 4. Telegram post shablonlari

**Ishga tushish posti**
```
Eskiça yozing — yangiça çiqadi. ⌨️

2027-yil 22-aprelda yangi alifbo kuchga kiradi: Oʻ→Ö, Gʻ→Ğ, Sh→Ş, Ch→Ç.
bitg siz odatdagidek yozganingizda so'zni o'zi o'giradi. Inglizcha so'z va havolalarga tegmaydi.

✅ Bepul · ochiq kod · internetsiz
👉 bitg.uz
```

**Foydali post (har hafta)**
```
«Toshkent» yangi alifboda qanday yoziladi? 🤔
A) Toshkent  B) Toşkent  C) Toşkent'
Javob — izohlarda. Boshqa joy nomlari: bitg.uz/shaharlar
```

**Countdown (har oyning 22-sanasi)**
```
Yangi alifbogacha [N] kun qoldi. ⏳
Bugun bitta narsa qiling: kompyuteringizga ö ğ ş ç ni qo'shing → bitg.uz/klaviatura
```

## 5. Qisqa video (YouTube Shorts / Reels / TikTok) — birinchi 2 soniya

| Video | 0–2 s hook | Ko'rsatiladi |
|---|---|---|
| «Ş Ç Ğ Ö kompyuterda» | «Yangi harflarni qidirmang» (ekranda klaviatura) | Telegram'da «choy ichamiz» yoziladi → «çoy içamiz» |
| «Inglizcha buzilmaydi» | «Bu dastur «chat» va «choy»ni ajratadimi?» | aralash gap |
| «Backspace sehrli» | «Xato o'girdimi? ⌫» | so'z qaytadi |
| «Ismingiz yangi alifboda» | «Shohruh endi Şohruh» | ism kartasi |
| «Telefonda» | «Ilova kerak emas» | harfni bosib turish |

## 6. OG rasm qoidalari (har sahifaga alohida)
1200×630, katta yozuv: sahifa savoli yoki oldin → keyin («Toshkent → Toşkent»), pastda kichik `bitg.uz` va logo.
Shrift — Inter (latin-ext). Qora va oq mavzu emas, bitta aksent rang `#2f62c0`.
