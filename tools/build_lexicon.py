"""bitg lug'atlarini quradi: crates/bitg-core/src/data/{uz,en,sh_split}.tsv

Manbalar (repoga kiritilmaydi, yuklab olinadi; faqat CC0 / CC BY / ruxsat beruvchi litsenziyalar):
  * Leipzig Corpora Collection, o'zbekcha (CC BY 4.0): https://wortschatz.uni-leipzig.de/en/download/Uzbek
      uzb_community_2017 (faqat kirill gaplar), uzb_wikipedia_2021_100K, uzb_wikipedia_2016_100K,
      uzb_wikipedia_2014_100K, uzb_news_2020_10K, uzb_newscrawl_2011_100K — "*-sentences.txt" fayllari
  * Mozilla Common Voice, uz sentence-collector.txt (CC0):
      https://github.com/common-voice/common-voice/tree/main/server/data/uz
  * Tatoeba, uzb_sentences.tsv (CC BY 2.0 FR): https://downloads.tatoeba.org/exports/per_language/uzb/
  * SCOWL 2020.12.07 (ruxsat beruvchi litsenziya, Kevin Atkinson): http://wordlist.aspell.net/
  Vikipediya XML dump (CC BY-SA) ataylab ishlatilmaydi: ShareAlike sharti binar litsenziyasiga mos emas.

Ishlatish (README "Lug'at" bo'limidagi to'liq buyruq):
  python -I tools/build_lexicon.py --scowl <scowl-dir>       --sentences-cyrillic-only <uzb_community_2017-sentences.txt>       --sentences <boshqa *-sentences.txt ...> <sentence-collector.txt> <uzb_sentences.tsv>       --exclude-sentences <eval_exclude_sentences.txt> --exclude-eval eval/uz.tsv eval/uz_noapos.tsv

Baholash to'plami (eval/*.tsv, tools/build_eval.py) gaplari lug'atga kirmaydi (o'qitish/test aralashmasin):
--exclude-sentences — kirill gaplarning aynan matni, --exclude-eval — eval qatorining kutilgan ustuni orqali.
community_2017 uchun "*-words.txt" emas, to'liq "*-sentences.txt" o'qiladi: words.txt sanog'idan eval gaplarini
to'liq ayirib bo'lmaydi.

Faqat yig'ilgan statistika (so'z + og'irlik) saqlanadi — matnlarning o'zi emas.
"""

import argparse
import glob
import math
import os
import re
import sys

APOS = "'`´ʹʻʼʽ‘’‛′＇"
APOS_RE = re.compile("[" + re.escape(APOS) + "]")
LATIN_WORD = re.compile(r"^[a-z']+$")
OUT_DIR = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "crates", "bitg-core", "src", "data")

CYR_VOWELS = set("аеёиоуўэюяы")
CYR = {
    "а": "a", "б": "b", "в": "v", "г": "g", "д": "d", "ж": "j", "з": "z",
    "и": "i", "й": "y", "к": "k", "л": "l", "м": "m", "н": "n", "о": "o",
    "п": "p", "р": "r", "с": "s", "т": "t", "у": "u", "ф": "f", "х": "x",
    "ч": "ch", "ш": "sh", "щ": "shch", "ы": "i", "э": "e", "ё": "yo",
    "ю": "yu", "я": "ya", "ў": "o'", "қ": "q", "ғ": "g'", "ҳ": "h",
    "ь": "", "ъ": "'",
}
CYR_E = "е"
CYR_TS = "ц"
CYR_HARD_SOFT = "ъь"
SPLIT_PAIRS = ("сҳ", "чҳ")  # s+h, ch+h kirillda


def cyr_to_old_latin(w):
    """Kirill so'zni eski lotin kalitiga (kichik harf, tutuq '). None — o'zbekcha emas."""
    out = []
    for i, c in enumerate(w):
        prev = w[i - 1] if i > 0 else ""
        if c == CYR_E:
            out.append("ye" if (i == 0 or prev in CYR_VOWELS or prev in CYR_HARD_SOFT) else "e")
        elif c == CYR_TS:
            out.append("ts" if prev in CYR_VOWELS else "s")
        elif c in CYR:
            out.append(CYR[c])
        else:
            return None
    return "".join(out)


def fold(w):
    return APOS_RE.sub("'", w.lower())


def is_cyr(s):
    return any("Ѐ" <= ch <= "ӿ" for ch in s)


class Counts:
    def __init__(self):
        self.uz = {}
        self.sh_split = {}
        self.joined = {}
        self.cyr = {}  # faqat kirill matndan: apostrof noaniqligi yo'q (бўлган / болган)
        self.words_lat = set()  # *-words.txt dagi lotin so'zlar (atoqli ot filtri ularga tegmaydi)
        self.sent_lat = set()  # gaplardan olingan lotin so'zlar
        self.lat_lower = set()  # ... va ulardan kichik harf bilan ham uchraganlari

    def add_cyr(self, lw, f):
        key = cyr_to_old_latin(lw)
        if key is None:
            return
        if any(pair in lw for pair in SPLIT_PAIRS):
            # Isʼhoq: kirillda s va h alohida — eski lotinda tutuqsiz "ishoq" ham, "is'hoq" ham yoziladi.
            self.sh_split[key] = self.sh_split.get(key, 0) + f
        elif "ш" in lw or "ч" in lw:
            self.joined[key] = self.joined.get(key, 0) + f
        self.uz[key] = self.uz.get(key, 0) + f
        self.cyr[key] = self.cyr.get(key, 0) + f

    def add_latin(self, word, f):
        key = fold(word)
        if not LATIN_WORD.match(key) or not re.search("[a-z]", key):
            return
        if key.startswith("'") or "''" in key:
            return
        self.uz[key] = self.uz.get(key, 0) + f


# Gap darajasidagi filtr (--sentences): o'zbek kirilining o'ziga xos harflari bor, boshqa tillarniki yo'q.
UZ_CYR_MARK = re.compile("[ўқғҳ]")
FOREIGN_CYR = re.compile("[ӣӯҷәөүұіїєґњљђћџһңҗӊӑӗӧӱ]")
CYR_TOKEN = re.compile("[а-яёўқғҳ]+(?:ъ[а-яёўқғҳ]+)*")
LAT_TOKEN = re.compile("[A-Za-z]+(?:'[A-Za-z]+)*'?")
# Lotin gap o'zbekcha ekaniga belgi: o'/g' yoki "q" (u siz). Inglizcha/ruscha lotin gaplar o'tmaydi.
UZ_LAT_MARK = re.compile(r"[og]'[a-z]|q(?!u)|\b(va|bilan|uchun|ham|bu|bir)\b")


def sentence_text(line):
    # Leipzig: id<TAB>gap; Tatoeba: id<TAB>til<TAB>gap; Common Voice: gap
    return line.rstrip("\n").split("\t")[-1].strip()


def file_sentences(paths):
    for p in paths:
        with open(p, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                yield sentence_text(line)


class Exclude:
    """Lug'atga kirmaydigan gaplar: aniq matn (--exclude-sentences) yoki eval to'plami (--exclude-eval).
    Eval qatorining kutilgan ustuni = build_eval.translit(kirill_gap, NEW), shuning uchun kirill gapni xuddi
    shunday o'girib solishtiramiz."""

    def __init__(self, path, eval_paths):
        self.raw = set()
        self.expected = set()
        self.translit = None
        if path:
            with open(path, encoding="utf-8") as fh:
                self.raw = {ln.strip() for ln in fh if ln.strip()}
        for p in eval_paths:
            with open(p, encoding="utf-8") as fh:
                for ln in fh:
                    parts = ln.rstrip("\n").split("\t")
                    if not ln.startswith("#") and len(parts) == 3 and parts[0].startswith("uz"):
                        self.expected.add(parts[2])
        if self.expected:
            import importlib.util
            sys.dont_write_bytecode = True  # tools/__pycache__ qolmasin
            here = os.path.join(os.path.dirname(os.path.abspath(__file__)), "build_eval.py")
            spec = importlib.util.spec_from_file_location("build_eval", here)
            be = importlib.util.module_from_spec(spec)
            spec.loader.exec_module(be)
            self.translit = lambda s: be.translit(s, be.NEW)

    def __bool__(self):
        return bool(self.raw or self.expected)

    def __contains__(self, s):
        if s in self.raw:
            return True
        return bool(self.expected) and is_cyr(s) and self.translit(s) in self.expected


def read_sentences(c, sentences, exclude, latin_too, seen):
    used = skipped = dup = 0
    for s in sentences:
        s = s.strip()
        if not s:
            continue
        if s in exclude:
            skipped += 1
            continue
        h = hash(s)
        if h in seen:
            dup += 1  # bir gap bir nechta korpusda (Vikipediya 2014/2016/2021) — bir marta sanaladi
            continue
        seen.add(h)
        low = s.lower()
        if is_cyr(low):
            if not UZ_CYR_MARK.search(low) or FOREIGN_CYR.search(low):
                continue  # ruscha, tojikcha, qozoqcha gap
            used += 1
            for t in CYR_TOKEN.findall(low):
                c.add_cyr(t, 1)
        elif latin_too:
            f = fold(low)
            if not UZ_LAT_MARK.search(f) or "w" in f:
                continue
            used += 1
            for t in LAT_TOKEN.findall(APOS_RE.sub("'", s)):
                if t.endswith("'") and not t.endswith(("o'", "g'")):
                    t = t[:-1]
                key = t.lower()
                if t == key:
                    c.lat_lower.add(key)
                c.sent_lat.add(key)
                c.add_latin(key, 1)
    print(f"sentences: used={used} excluded(eval)={skipped} duplicate={dup}", file=sys.stderr)


def drop_latin_names(c):
    """Gaplardan olingan, faqat bosh harf bilan uchragan lotin so'zlar — atoqli ot (Pradesh, Manish): til dalili
    emas, inglizcha matnda ham uchraydi. Kirill matnda ham bo'lsa (Toshkent — Тошкент) qoladi."""
    n = 0
    for k in c.sent_lat - c.lat_lower:
        if k in c.uz and k not in c.cyr and k not in c.words_lat:
            del c.uz[k]
            n += 1
    print(f"atoqli otlar (faqat bosh harf, lotin): {n} ta olib tashlandi", file=sys.stderr)


def subtract_excluded(c, words_path, exclude):
    """--exclude-sentences: words.txt dan baholash gaplaridagi so'zlarni ayirish (o'qitish/test aralashmasin)."""
    sent = re.sub(r"-words\.txt$", "-sentences.txt", words_path)
    if not exclude or sent == words_path or not os.path.exists(sent):
        return
    tmp = Counts()
    n = 0
    with open(sent, encoding="utf-8", errors="replace") as fh:
        for line in fh:
            s = sentence_text(line)
            if s not in exclude:
                continue
            n += 1
            low = s.lower()
            if is_cyr(low):
                for t in CYR_TOKEN.findall(low):
                    tmp.add_cyr(t, 1)
    for d, src in ((c.uz, tmp.uz), (c.cyr, tmp.cyr), (c.sh_split, tmp.sh_split), (c.joined, tmp.joined)):
        for k, f in src.items():
            if k in d:
                d[k] = max(0, d[k] - f)
    print(f"{os.path.basename(words_path)}: {n} eval gapi ayirildi", file=sys.stderr)


def read_leipzig(paths, cyr_only, c):
    for p in [(p, False) for p in paths] + [(p, True) for p in cyr_only]:
        p, only_cyr = p
        with open(p, encoding="utf-8", errors="replace") as fh:
            for line in fh:
                parts = line.rstrip("\n").split("\t")
                if len(parts) < 3:
                    continue
                # wikipedia: id, so'z, chastota; community: id, so'z, so'z, chastota
                word, freq = parts[1], parts[-1]
                try:
                    f = int(freq)
                except ValueError:
                    continue
                lw = word.lower()
                if is_cyr(lw):
                    c.add_cyr(lw, f)
                elif not only_cyr:  # cyr_only korpusning lotin qismi aralash (rus/ingliz) — faqat kirill olinadi
                    c.add_latin(word, f)
                    c.words_lat.add(fold(word))


def finish_split(c):
    # Kirill imlo xatolari (сҳу = шу) shovqin: ajratilgan shakl ш/ч li shakldan kamida 3 marta ko'p bo'lsin.
    return {k: f for k, f in c.sh_split.items() if f >= 3 * c.joined.get(k, 0)}


def uz_weight(f):
    # 6.0 gacha: apostrof tiklashda haqiqiy nisbat kerak (bo'lib ≫ bolib). Til dalili baribir 4.5 da kesiladi.
    return round(min(6.0, max(1.0, math.log10(f) + 1.0)), 2)


def read_scowl(root):
    levels = {10: 4.0, 20: 3.5, 35: 2.5, 40: 2.0, 50: 1.5}
    en = {}
    for f in glob.glob(os.path.join(root, "final", "*")):
        m = re.match(r"(english|american|british)-(words|upper|contractions)\.(\d+)$", os.path.basename(f))
        if not m:
            continue
        lvl = int(m.group(3))
        if lvl not in levels:
            continue
        with open(f, encoding="latin-1") as fh:
            for w in fh.read().split("\n"):
                w = w.strip()
                if not w or not w.isascii():
                    continue
                lw = w.lower()
                risky = "sh" in lw or "ch" in lw or "'" in lw
                if lvl > 20 and not risky:
                    continue  # kontekst uchun faqat eng ko'p uchraydigan so'zlar
                en[lw] = max(en.get(lw, 0), levels[lvl])
    return en


def write_tsv(path, header, items):
    with open(path, "w", encoding="utf-8", newline="\n") as o:
        for h in header:
            o.write("# " + h + "\n")
        for k, v in items:
            o.write(f"{k}\t{v}\n")


# Leipzig bo'lmagan gap manbalari (yo'l ichidagi kalit so'z -> uz.tsv sarlavhasidagi atributsiya).
SOURCE_NOTES = [
    ("commonvoice", "Mozilla Common Voice uz sentences (CC0)"),
    ("tatoeba", "Tatoeba (CC BY 2.0 FR), https://tatoeba.org"),
]


BARE_REF = 1.29
# lang.rs dagi `is_sh_split` o'zak qidiruvi bilan mos: eng ko'p uchraydigan qo'shimchalar, uzunidan qisqasiga.
SPLIT_SUFFIXES = ("laridan", "larning", "lari", "ning", "dagi", "dan", "lar", "ga", "da", "ni", "si", "i")


def risky(k):
    return "sh" in k or "ch" in k or "'" in k


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scowl", required=True)
    ap.add_argument("--leipzig", nargs="*", default=[], help="Leipzig *-words.txt (id, so'z, chastota)")
    ap.add_argument("--leipzig-cyrillic-only", nargs="*", default=[])
    ap.add_argument("--sentences", nargs="*", default=[],
                    help="gap fayllari (Leipzig *-sentences.txt, Tatoeba tsv, Common Voice txt): so'zlar sanaladi")
    ap.add_argument("--sentences-cyrillic-only", nargs="*", default=[],
                    help="gap fayllari, faqat o'zbek kirill gaplari olinadi")
    ap.add_argument("--exclude-sentences", default=None,
                    help="fayl: har qatorda bitta gap — lug'atga kirmaydi")
    ap.add_argument("--exclude-eval", nargs="*", default=[],
                    help="eval/out/*.tsv: kutilgan ustuni kirill gapdan olingan (build_eval.py) — o'sha kirill gaplar "
                         "lug'atga kirmaydi (o'qitish/test aralashmasin)")
    ap.add_argument("--min-freq", type=int, default=3)
    ap.add_argument("--context-top", type=int, default=40000)
    ap.add_argument("--en-homograph-min", type=int, default=4,
                    help="gap manbalaridagi inglizcha omograf (chat) kirill matnda kamida shuncha uchrasin")
    ap.add_argument("--weight-scale", type=float, default=1.0,
                    help="chastota og'irlikka aylanishidan oldin shunga ko'paytiriladi (korpus kattalashganda "
                         "lang.rs dagi mutlaq chegaralar o'z ma'nosini saqlashi uchun)")
    ap.add_argument("--bare-rare-min", type=float, default=100,
                    help="o'/g' li shakl kamida shuncha uchrasa-yu, apostrofsizi kirillda uchramasa (bo'ladi / boladi) — "
                         "apostrofsiz shakl 0 og'irlik bilan yoziladi; 0 = o'chiq")
    a = ap.parse_args()

    exclude = Exclude(a.exclude_sentences, a.exclude_eval)
    c = Counts()
    read_leipzig(a.leipzig, a.leipzig_cyrillic_only, c)
    for p in a.leipzig + a.leipzig_cyrillic_only:
        subtract_excluded(c, p, exclude)
    seen = set()
    read_sentences(c, file_sentences(a.sentences), exclude, True, seen)
    read_sentences(c, file_sentences(a.sentences_cyrillic_only), exclude, False, seen)
    drop_latin_names(c)
    uz, cyr, sh_split = c.uz, c.cyr, finish_split(c)
    en = read_scowl(a.scowl)

    items = [(k, f) for k, f in uz.items() if f >= a.min_freq]
    keep = {k: f for k, f in items if risky(k)}
    for k, f in sorted(items, key=lambda kv: -kv[1])[: a.context_top]:
        keep[k] = f
    # Apostrof tiklash uchun o'/g' siz shakl: haqiqiy so'zmi (qo'l / qol, bo'r / bor) — buni faqat kirill
    # matn aytadi. Lotin korpusidagi "bolgan", "kora" — shoshib yozilgan xato, lug'atga kirmaydi.
    # Nomzod (o'/g' li shakl) butun korpusdan sanaladi, apostrofsiz shakl — faqat kirilldan. Lotin manbalar
    # ko'paysa nomzod sun'iy kuchayadi (o'rta ≫ orta), shuning uchun kirill sanog'i umumiy/kirill nisbati bilan
    # tekislanadi. BARE_REF — dastlabki lug'at (wikipedia_2021 + community_2017) nisbati: o'shanda ko'paytma 1.
    ratio = sum(uz.values()) / max(1, sum(cyr.values()))
    bare_factor = max(1.0, ratio / BARE_REF)
    print(f"umumiy/kirill = {ratio:.3f}, kirill sanog'i ko'paytmasi = {bare_factor:.3f}", file=sys.stderr)
    # Gaplardan olingan inglizcha omograf (chat, shop, show): lotin matndagi uchrashi o'zbekchaga dalil emas
    # (inglizcha ibora, nom) — og'irligi faqat kirill matndan va u yerda ham aniq uchrashi kerak (чат 3 marta — kam, шун 4).
    for k in list(keep):
        if risky(k) and k in en and k in c.sent_lat and k not in c.words_lat:
            if cyr.get(k, 0) >= a.en_homograph_min:
                keep[k] = cyr[k] * bare_factor
            else:
                del keep[k]
    for k in list(keep):
        if "o'" in k or "g'" in k:
            bare = k.replace("o'", "o").replace("g'", "g")
            if cyr.get(bare, 0) >= a.min_freq:
                keep[bare] = cyr[bare] * bare_factor
            elif a.bare_rare_min and keep.get(k, 0) * a.weight_scale >= a.bare_rare_min and bare not in en:
                # Kirillda deyarli uchramaydi (болади): lug'atga 0 og'irlik bilan yoziladi ("so'z emas"), aks holda
                # lang.rs uni o'zak orqali (bola|di) kuchli so'z deb hisoblab, bo'ladi ni tiklamay qo'yadi.
                keep[bare] = 0
            else:
                keep.pop(bare, None)
    uz_out = sorted((k, uz_weight(f * a.weight_scale) if f else 0) for k, f in keep.items())
    en_out = sorted(en.items())
    split = {k: f for k, f in sh_split.items() if f >= 2}
    # Qo'shimchasiz o'zak ham (mushafi → mushaf, nushada → nusha): lang.rs is_sh_split o'zak + qo'shimchani tekshiradi.
    for k, f in list(split.items()):
        for suf in SPLIT_SUFFIXES:
            stem = k[: -len(suf)]
            if k.endswith(suf) and len(stem) >= 5:
                split.setdefault(stem, f)
                break
    split_out = sorted(split.items())

    os.makedirs(OUT_DIR, exist_ok=True)
    src = "Leipzig Corpora Collection (CC BY 4.0): " + ", ".join(
        os.path.basename(p) for p in a.leipzig + a.leipzig_cyrillic_only
        + [p for p in a.sentences + a.sentences_cyrillic_only if os.path.basename(p).startswith("uzb_")])
    sent_paths = [p.replace("\\", "/").lower() for p in a.sentences + a.sentences_cyrillic_only]
    extra = [note for k, note in SOURCE_NOTES if any(k in p for p in sent_paths)]
    src = "; ".join([src] + extra)
    write_tsv(os.path.join(OUT_DIR, "uz.tsv"),
              ["o'zbekcha so'zlar (eski lotin kaliti) va og'irligi 1..6 = log10(chastota)+1; "
               "0 = o'/g' siz shakl kirillda uchramaydi (boladi: faqat bo'ladi)", src,
               "tools/build_lexicon.py bilan qurilgan"], uz_out)
    write_tsv(os.path.join(OUT_DIR, "en.tsv"),
              ["inglizcha so'zlar va og'irligi (SCOWL darajasi: 10=4.0 ... 50=1.5)",
               "SCOWL 2020.12.07, Copyright 2000-2018 Kevin Atkinson va boshqalar (ruxsat beruvchi litsenziya)",
               "tools/build_lexicon.py bilan qurilgan"], en_out)
    write_tsv(os.path.join(OUT_DIR, "sh_split.tsv"),
              ["kirillda s|h yoki ch|h alohida bo'lgan so'zlar (Ishoq, Mushaf): yozuvdagi sh ş ga aylanmaydi", src],
              split_out)
    print(f"uz={len(uz_out)} en={len(en_out)} sh_split={len(split_out)}", file=sys.stderr)


if __name__ == "__main__":
    main()
