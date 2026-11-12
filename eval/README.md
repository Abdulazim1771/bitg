# bitg baholash to'plami (benchmark)

bitg sifati shu uch faylda o'lchanadi. Har kim o'z konvertorini xuddi shu gaplarda solishtira olsin deb
ochiq e'lon qilingan.

| Fayl | Qatorlar | Nima | Manba |
|---|---|---|---|
| `uz.tsv` | 3000 | o'zbekcha gap: kirish = eski lotin (apostrof turlari aralash: `'` `ʻ` `’` `` ` ``), kutilgan = yangi lotin | Leipzig `uzb_community_2017` (kirill gaplar) |
| `uz_noapos.tsv` | 3000 (boshqa gaplar) | xuddi shunday, lekin o'/g' dagi apostrof tushib qolgan ("ozbek", "togri") | Leipzig `uzb_community_2017` |
| `en.tsv` | 3000 | inglizcha gap (sh / ch / apostrof bor): o'zgarmasligi kerak | Leipzig `eng_news_2020_10K` (1523) va `eng-com_web-public_2018_10K` (1477) |

Format: `til<TAB>kirish<TAB>kutilgan` (`en` qatorlarida kutilgan bo'sh — kirishning o'zi). `#` bilan
boshlangan qator — izoh.

```
cargo build --release -p bitg-cli
target/release/bitg eval eval/uz.tsv
target/release/bitg eval eval/uz_noapos.tsv
target/release/bitg eval eval/en.tsv
```

## Qanday qurilgan

[`tools/build_eval.py`](../tools/build_eval.py), `--seed 2026 --limit 3000`. O'zbekcha qatorda kirish ham,
kutilgan javob ham **kirill gapdan mustaqil** transliteratsiya qilinadi (kirill → eski lotin, kirill → yangi
lotin), shuning uchun konvertor eski lotindan yangi lotinni o'zi topishi kerak (`Is'hoq → Ishoq`,
`O'zA → ÖzA`, `Sh.Sharipov → Ş.Şaripov`). Faqat 20 dan ortiq kirill harfli, lotin harfsiz gaplar olingan.
Inglizcha gaplar — sh, ch yoki apostrof bor gaplar.

Ma'lum shovqin: `uzb_community_2017` — foydalanuvchi izohlari, kirill matnda imlo xatolari bor (`булиб`,
`колди`, `Киргизистон`). Kutilgan javob xatoni saqlaydi, bitg esa ba'zan uni to'g'rilaydi — bunday "xato"
haqiqiy xato emas.

## O'qitishdan ajratilgan

Bu gaplar lug'at qurishda **ishlatilmaydi**: [`tools/build_lexicon.py`](../tools/build_lexicon.py)
`--exclude-eval eval/uz.tsv eval/uz_noapos.tsv` bilan har bir manba gapini eval qatorining kutilgan
ustuniga solishtiradi va mos kelganini tashlab yuboradi (community_2017 da 6002 ta, newscrawl_2011 da 767 ta —
takror gaplar bilan). Shu sababli community_2017 uchun `*-words.txt` sanog'i emas, to'liq `*-sentences.txt`
o'qiladi: words.txt dan eval gaplarini aniq ayirib bo'lmaydi.

## Litsenziya

Gaplar [Leipzig Corpora Collection](https://wortschatz.uni-leipzig.de/en/download) dan olingan; yuklab
olinadigan korpuslar **CC BY 4.0** litsenziyasida tarqatiladi
([Terms of Usage](https://wortschatz.uni-leipzig.de/en/usage)). Iqtibos:

> D. Goldhahn, T. Eckart, U. Quasthoff: Building Large Monolingual Dictionaries at the Leipzig Corpora
> Collection: From 100 to 200 Languages. In: Proceedings of the 8th International Language Resources and
> Evaluation (LREC'12), 2012.

O'zgartirish: gaplar tasodifiy tanlangan, kirilldan eski va yangi lotinga transliteratsiya qilingan,
apostrof tushirilgan (`uz_noapos.tsv`). Bu fayllar ham CC BY 4.0 shartida tarqatiladi (repo kodining
MIT / Apache-2.0 litsenziyasi ularga tegishli emas).
