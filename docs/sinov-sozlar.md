# bitg — qo'lda sinov to'plami

Har yangi versiyadan keyin shu ro'yxatni **Telegram**, **Chrome** (masalan, qidiruv qatori) va **Bloknot**da yozib
chiqing. «Kutilgan» ustuni `bitg` CLI chiqishidan olingan (yadro bitta, IME ham shuni ishlatadi). Yozish paytida
so'z **probel bosilganda** o'giriladi — oxirgi so'zdan keyin ham probel bosing.

Natija farq qilsa: qaysi dastur, qaysi qator — va `%LOCALAPPDATA%\bitg\tsf.log` oxirgi 30 qatori (unda matn yo'q).

## 1. Asosiy harflar

| Yozing | Kutilgan |
|---|---|
| o'zbek o'quvchi g'alaba bog'cha | özbek öquvçi ğalaba boğça |
| shahar shirin choy chiroyli qishloq | şahar şirin çoy çiroyli qişloq |
| o'shanda ko'chada g'ishtli uy | öşanda köçada ğiştli uy |
| ma'no e'tibor ta'lim san'at | maʼno eʼtibor taʼlim sanʼat |
| mash'al ash'or | maşʼal aşʼor |

## 2. Apostrofsiz yozilgan (lug'atdan tiklanadi)

Har qatorni alohida yozing (har biridan keyin Enter): bir qatordagi so'zlar bir-biriga ta'sir qiladi (2.2 ga qarang).

| Yozing | Kutilgan |
|---|---|
| ozbek togri qoshimcha korinish ogir | özbek töğri qöşimça köriniş oğir |
| shunday shisha shoshma ishlash | şunday şişa şoşma işlaş |
| qoshimcha qoshiq qoshni qoshish | qöşimça qöşiq qöşni qöşiş |
| ozbek ozbekcha ozbekiston ozim ozingiz | özbek özbekça özbekiston özim özingiz |
| togri togrisi togrilash | töğri töğrisi töğrilaş |
| korinish korsatish kordim kormadim korib | köriniş körsatiş kördim körmadim körib |
| bolib boladi bolgan bolsa bolmaydi | bölib böladi bölgan bölsa bölmaydi |
| ogil ogirlik goshtni gisht | öğil öğirlik göştni ğişt |
| sozlar sorash orta ortasida ornini orgatish | sözlar söraş örta örtasida örnini örgatiş |
| yol yoldosh yolovchi kop kopchilik kopincha | yöl yöldoş yölovçi köp köpçilik köpinça |
| yoqoldi yoqotish chogi chogroq bogcha | yöqoldi yöqotiş çoği çoğroq boğça |
| Bu ish yaxshi boladi | Bu iş yaxşi böladi (avval `boladi` qolardi) |

### 2.1. Chiziqchali so'z (bo'lak-bo'lak tiklanadi)

| Yozing | Kutilgan |
|---|---|
| Davlat qollab-quvvatlash dasturi togridan-togri ishlaydi | Davlat qöllab-quvvatlaş dasturi töğridan-töğri işlaydi |
| Ozmi-ko'pmi pul bor | Ozmi-köpmi pul bor (bir bo'lagida o' yozilgan — qolganiga tegilmaydi) |

### 2.2. Ikki ma'noli so'zlar: oz / o'z, yoq / yo'q, otgan / o'tgan, qol / qo'l, tort / to'rt

Yolg'iz yoki apostrof qo'yadigan odam yozsa — **tegilmaydi**. Matnda o'/g' umuman yo'q va boshqa so'z tiklangan
bo'lsa (muallif apostrof qo'ymayapti) — chastota hal qiladi.

| Yozing | Kutilgan |
|---|---|
| Uyda qol, oz qoldi, ozod bor | o'zgarmaydi (boshqa so'z tiklanmadi) |
| O'zbekistonda oz qoldi | Özbekistonda oz qoldi (o' yozilgan — `oz` "kam" ma'nosida) |
| Biz ozbek tilida oz fikrimizni aytamiz | Biz özbek tilida öz fikrimizni aytamiz |
| Ozbekistonda ish yoq, otgan yili kop odam ketdi | Özbekistonda iş yöq, ötgan yili köp odam ketdi |
| Togri, hech narsa yoq. | Töğri, heç narsa yöq. |
| Chatda yolg'iz `oz ` yozing | `oz` (birinchi so'z — taxmin yo'q) |
| `ozbek tilida oz ` (so'zma-so'z, probel bilan) | `özbek tilida öz ` |

## 3. Joy va kishi nomlari (o'zbek imlosida — o'giriladi)

| Yozing | Kutilgan |
|---|---|
| Toshkent Samarqand Farg'ona Qo'qon Chirchiq | Toşkent Samarqand Farğona Qöqon Çirçiq |
| Shohruh Shahnoza Chori Mohichehra | Şohruh Şahnoza Çori Mohiçehra |
| Ishoq Is'hoq dishormonal | Ishoq Ishoq dishormonal |
| Is'hoq mas'ul as'hob mus'haf | Ishoq masʼul ashob mushaf (s'h — ş emas, s+h) |
| Ishoq Ishoqova Ishoqqa nusha nushasi nushalari | o'zgarmaydi (kirillda Исҳоқ, нусҳа: s va h alohida) |
| Pushkin Chexov Shekspir | Puşkin Çexov Şekspir |
| Shohruh Shahnoza Sherzod Charos Chori | Şohruh Şahnoza Şerzod Çaros Çori |
| Toshkent Chirchiq Shahrisabz Qarshi Farg'ona Qo'qon | Toşkent Çirçiq Şahrisabz Qarşi Farğona Qöqon |

## 4. Inglizcha so'z va brendlar (o'zgarmaydi)

| Yozing | Kutilgan |
|---|---|
| check chat shop shopping english | o'zgarmaydi |
| chrome shell photoshop Chevrolet Toshiba | o'zgarmaydi |
| Chromeda Shellga Photoshopni Chevroletni | o'zgarmaydi |
| Facebook WhatsApp YouTube Chrome Telegram | o'zgarmaydi |
| Chevrolet Nexia Cobalt Shell Toshiba Microsoft Intel | o'zgarmaydi |
| WhatsAppdan YouTubeda Telegramda | o'zgarmaydi |
| check chat shop shopping show cash fresh English | o'zgarmaydi |
| I'm going to the shop | o'zgarmaydi |
| Don't touch the cash, she said | o'zgarmaydi |
| Rock'n'roll at nine o'clock | o'zgarmaydi (o' bor, lekin inglizcha) |
| salom chrome, keyin shell | o'zgarmaydi (avval `salom çrome` bo'lardi — tuzatildi) |
| Please check the shop, keyin choy ichamiz | Please check the shop, keyin çoy içamiz |

## 5. Havola, pochta, kod, qisqartma

| Yozing | Kutilgan |
|---|---|
| https://bitg.dtwin.uz info@dtwin.uz | o'zgarmaydi |
| git push origin main | o'zgarmaydi |
| MCHJ AQSH TSH SHHT | MÇJ AQŞ TSH ŞHT |
| C-uchli MR-shartli | C-uçli MR-şartli |
| O'ZBEKISTON SHAHAR CHOY | ÖZBEKISTON ŞAHAR ÇOY |

## 6. Tirnoq va kirill

| Yozing | Kutilgan |
|---|---|
| Kecha "choy" ichdik | Keça "çoy" içdik |
| 'Yaxshi tong' dedi | 'Yaxşi tong' dedi (tirnoq tutuqqa aylanmaydi) |
| Ассалому алайкум, ўзбекча ёзинг | Assalomu alaykum, özbekça yozing |

## 7. Harakatlar (ayniqsa Telegram)

| Qiling | Kutilgan |
|---|---|
| Chatni oching, darhol `shunday ` yozing (birinchi so'z) | `şunday ` |
| `choy ` → o'girilgach **Backspace** | `choy` qaytadi; Istisnolarga **tushmaydi** (sozlama standart o'chiq) |
| Sozlamalar → «Qaytarilgan so'zni istisnoga yozish» ni yoqing, oldingi qadamni takrorlang | `choy` Istisnolar sahifasida ko'rinadi; qidirib o'chirsa bo'ladi |
| `choyx` yozing, Backspace bilan `x` ni o'chiring, probel | `çoy ` |
| So'z o'rtasini sichqoncha bilan bosib harf qo'shing, probel | so'z **o'zgarmaydi** (kursor qayerdaligi noma'lum — ataylab) |
| To'xtamasdan tez yozing: `shu bilan choy ichamiz ` | `şu bilan çoy içamiz `, harflar aralashmasin, qotmasin |
| EN tartibiga o'tib xuddi shuni yozing | hech narsa o'girilmaydi; tezlik UZ bilan bir xil bo'lsin |

## 8. Aralash gaplar

| Yozing | Kutilgan |
|---|---|
| Kecha Chevroletda Toshkentga bordim va choy ichdim | Keça Chevroletda Toşkentga bordim va çoy içdim |
| Telegramda shunday xabar keldi: ozbekcha yozing | Telegramda şunday xabar keldi: özbekça yozing |
| iPhone 15 Pro sotib oldim, juda chiroyli | iPhone 15 Pro sotib oldim, juda çiroyli |
| Bugun meeting bor, keyin choy ichamiz | Bugun meeting bor, keyin çoy içamiz |
| o'zbek o'g'il g'oz bo'lim to'g'ri ko'p | özbek öğil ğoz bölim töğri köp |
| g'isht g'alla g'urur tog' bog' sog'liq | ğişt ğalla ğurur toğ boğ soğliq |
| mash'al ash'or ta'sir sur'at ma'lumot e'lon | maşʼal aşʼor taʼsir surʼat maʼlumot eʼlon |

## 9. Ma'lum cheklovlar (xato deb yozmang)

| Yozing | Chiqadi | Nega |
|---|---|---|
| Menga chat yozing | Menga chat yozing | `chat` o'zbek korpusida deyarli yo'q, inglizchada keng tarqalgan |
| kitobni oqib chiqdim | kitobni oqib çiqdim | `oqib` (oqmoq) ham haqiqiy so'z — o'qib deb taxmin qilinmaydi |
| Men sozimni aytdim | Men sozimni aytdim | `so'zimni` korpusda kam uchraydi, `soz` esa haqiqiy so'z |
