"""O'rnatuvchi (WiX) rasmlari: dialog 493x312 (chapda 164px brend paneli), banner 493x58, litsenziya RTF.

    python -I tools/make_installer_art.py   # installer/assets/*

Rasmlar repoda saqlanadi — CI bu skriptni ishlatmaydi. Shrift: Windows'dagi Bahnschrift.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "installer" / "assets"
ICON = ROOT / "crates" / "bitg-settings" / "assets" / "bitg.png"
FONT = Path("C:/Windows/Fonts/bahnschrift.ttf")

LAPIS = (42, 75, 176)
LAPIS_DEEP = (27, 50, 128)
FIRUZA = (12, 132, 117)
PAPER = (247, 248, 245)
SCALE = 4  # katta chizib, keyin kichraytiramiz


def font(px: float, weight: int = 600) -> ImageFont.FreeTypeFont:
    f = ImageFont.truetype(str(FONT), int(px * SCALE))
    try:
        f.set_variation_by_axes([weight, 100])
    except Exception:
        pass
    return f


def dialog() -> Image.Image:
    w, h, panel = 493 * SCALE, 312 * SCALE, 164 * SCALE
    im = Image.new("RGB", (w, h), PAPER)
    d = ImageDraw.Draw(im)
    # Panel: yuqoridan pastga lojuvard gradient.
    for y in range(h):
        t = y / h
        c = tuple(int(LAPIS[i] * (1 - t) + LAPIS_DEEP[i] * t) for i in range(3))
        d.line([(0, y), (panel, y)], fill=c)
    # Fonda yangi harflar — xira, katta.
    ghost = font(74, 700)
    for ch, x, y in (("Ö", 16, 26), ("Ğ", 88, 26), ("Ş", 16, 104), ("Ç", 88, 104)):
        d.text((x * SCALE, y * SCALE), ch, font=ghost, fill=(52, 86, 186))
    # Ikonka va so'z belgisi.
    icon = Image.open(ICON).convert("RGBA").resize((56 * SCALE, 56 * SCALE), Image.LANCZOS)
    im.paste(icon, (22 * SCALE, 196 * SCALE), icon)
    d.text((22 * SCALE, 258 * SCALE), "bitg", font=font(26, 700), fill="white")
    d.text((23 * SCALE, 290 * SCALE), "2026 alifbosi", font=font(9.5, 400), fill=(196, 210, 245))
    # Panel cheti: firuza chiziq.
    d.rectangle((panel - 3 * SCALE, 0, panel, h), fill=FIRUZA)
    return im.resize((493, 312), Image.LANCZOS)


def banner() -> Image.Image:
    w, h = 493 * SCALE, 58 * SCALE
    im = Image.new("RGB", (w, h), PAPER)
    d = ImageDraw.Draw(im)
    icon = Image.open(ICON).convert("RGBA").resize((40 * SCALE, 40 * SCALE), Image.LANCZOS)
    im.paste(icon, (w - 50 * SCALE, 9 * SCALE), icon)
    d.rectangle((0, h - 2 * SCALE, w, h), fill=FIRUZA)
    return im.resize((493, 58), Image.LANCZOS)


def license_rtf() -> str:
    text = (ROOT / "LICENSE-MIT").read_text(encoding="utf-8").strip()
    body = text.replace("\\", "\\\\").replace("{", "\\{").replace("}", "\\}")
    paras = "\\par\n".join(p.replace("\n", " ") for p in body.split("\n\n"))
    return (
        "{\\rtf1\\ansi\\deff0{\\fonttbl{\\f0 Segoe UI;}}\\fs18\n"
        "{\\b bitg} \\endash  MIT yoki Apache-2.0 (tanlovingiz bo'yicha). Quyida MIT matni; "
        "Apache-2.0: https://github.com/dtwinuz/bitg/blob/main/LICENSE-APACHE\\par\\par\n"
        f"{paras}\\par\n}}\n"
    )


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    dialog().save(OUT / "dialog.bmp")
    banner().save(OUT / "banner.bmp")
    (OUT / "license.rtf").write_text(license_rtf(), encoding="ascii")
    print("ok:", ", ".join(p.name for p in sorted(OUT.iterdir())))


if __name__ == "__main__":
    main()
