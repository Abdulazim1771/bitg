"""bitg ikonkalari: lojuvard kvadrat ichida "ö" (nuqtalari firuza). O'chiq holat — kulrang.

    python -I tools/make_icons.py   # crates/bitg-settings/assets/bitg.ico, bitg-off.ico

Shrift: Windows'dagi Bahnschrift. Natija repoda saqlanadi — CI bu skriptni ishlatmaydi.
"""

from pathlib import Path

from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "crates" / "bitg-settings" / "assets"
FONT = Path("C:/Windows/Fonts/bahnschrift.ttf")
SIZES = [16, 20, 24, 32, 40, 48, 64, 128, 256]


def draw(size: int, bg: str, dots: str) -> Image.Image:
    s = 1024  # katta chizib, keyin kichraytiramiz (silliq chetlar)
    im = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    pad = 0 if size <= 24 else 40
    d.rounded_rectangle((pad, pad, s - pad, s - pad), radius=230, fill=bg)
    font = ImageFont.truetype(str(FONT), 900)
    try:
        font.set_variation_by_axes([600, 100])  # qalinlik, kenglik
    except Exception:
        pass
    # "o" va nuqtalarni alohida: nuqtalar boshqa rangda.
    box = d.textbbox((0, 0), "o", font=font)
    w, h = box[2] - box[0], box[3] - box[1]
    x = (s - w) / 2 - box[0]
    y = s * 0.62 - h / 2 - box[1]
    d.text((x, y), "o", font=font, fill="white")
    r = s * 0.078
    cy = y + box[1] - s * 0.115
    for cx in (s / 2 - s * 0.125, s / 2 + s * 0.125):
        d.ellipse((cx - r, cy - r, cx + r, cy + r), fill=dots)
    return im.resize((size, size), Image.LANCZOS)


def save(name: str, bg: str, dots: str) -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    big = draw(256, bg, dots)
    imgs = [draw(n, bg, dots) for n in SIZES]
    big.save(OUT / name, sizes=[(n, n) for n in SIZES], append_images=imgs)
    big.save(OUT / name.replace(".ico", ".png"))


if __name__ == "__main__":
    save("bitg.ico", "#2a4bb0", "#6ee7d3")
    save("bitg-off.ico", "#5d6773", "#c9ced6")
    print("ok", OUT)
