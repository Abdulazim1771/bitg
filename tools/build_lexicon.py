"""bitg lug'atlarini quradi: crates/bitg-core/src/data/{uz,en,sh_split}.tsv

Manbalar (repoga kiritilmaydi, yuklab olinadi):
  * Leipzig Corpora Collection, o'zbekcha (CC BY 4.0): https://wortschatz.uni-leipzig.de/en/download/Uzbek
      uzb_wikipedia_2021_100K, uzb_community_2017 — "*-words.txt" fayllari (id<TAB>so'z<TAB>chastota)
  * SCOWL 2020.12.07 (ruxsat beruvchi litsenziya, Kevin Atkinson): http://wordlist.aspell.net/

Ishlatish:
  python -I tools/build_lexicon.py --scowl <scowl-dir> --leipzig <wiki-words.txt> \
      --leipzig-cyrillic-only <community-words.txt>

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


def read_leipzig(paths, cyr_only):
    uz = {}
    sh_split = {}
    joined = {}
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
                    key = cyr_to_old_latin(lw)
                    if key is None:
                        continue
                    if any(pair in lw for pair in SPLIT_PAIRS):
                        # Isʼhoq: kirillda s va h alohida — eski lotinda tutuqsiz "ishoq" ham, "is'hoq" ham yoziladi.
                        sh_split[key] = sh_split.get(key, 0) + f
                    elif "ш" in lw or "ч" in lw:
                        joined[key] = joined.get(key, 0) + f
                    uz[key] = uz.get(key, 0) + f
                    continue
                if only_cyr:
                    continue  # bu korpusning lotin qismi aralash (rus/ingliz) — faqat kirill olinadi
                key = fold(word)
                if not LATIN_WORD.match(key) or not re.search("[a-z]", key):
                    continue
                if key.startswith("'") or "''" in key:
                    continue
                uz[key] = uz.get(key, 0) + f
    # Kirill imlo xatolari (сҳу = шу) shovqin: ajratilgan shakl ш/ч li shakldan kamida 3 marta ko'p bo'lsin.
    sh_split = {k: f for k, f in sh_split.items() if f >= 3 * joined.get(k, 0)}
    return uz, sh_split


def uz_weight(f):
    return round(min(4.5, max(1.0, math.log10(f) + 1.0)), 2)


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


def risky(k):
    return "sh" in k or "ch" in k or "'" in k


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--scowl", required=True)
    ap.add_argument("--leipzig", nargs="+", required=True)
    ap.add_argument("--leipzig-cyrillic-only", nargs="*", default=[])
    ap.add_argument("--min-freq", type=int, default=3)
    ap.add_argument("--context-top", type=int, default=20000)
    a = ap.parse_args()

    uz, sh_split = read_leipzig(a.leipzig, a.leipzig_cyrillic_only)
    en = read_scowl(a.scowl)

    items = [(k, f) for k, f in uz.items() if f >= a.min_freq]
    keep = {k: f for k, f in items if risky(k)}
    for k, f in sorted(items, key=lambda kv: -kv[1])[: a.context_top]:
        keep[k] = f
    uz_out = sorted((k, uz_weight(f)) for k, f in keep.items())
    en_out = sorted(en.items())
    split_out = sorted((k, f) for k, f in sh_split.items() if f >= 2)

    os.makedirs(OUT_DIR, exist_ok=True)
    src = "Leipzig Corpora Collection (CC BY 4.0): " + ", ".join(
        os.path.basename(p) for p in a.leipzig + a.leipzig_cyrillic_only)
    write_tsv(os.path.join(OUT_DIR, "uz.tsv"),
              ["o'zbekcha so'zlar (eski lotin kaliti) va og'irligi 1..4.5 = log10(chastota)+1", src,
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
