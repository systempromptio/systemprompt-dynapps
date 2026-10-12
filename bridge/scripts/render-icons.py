#!/usr/bin/env python3
"""Regenerate the Dynapps Bridge raster icons from the master `assets/icon.svg`.

Produces, idempotently, from the Dynapps mark master:
  - window-icon-1024.png  (1024x1024, GUI window + macOS .icns source)
  - tray-icon.png         (44x44, the Dynapps mark, for the system tray)
  - app-icon.ico          (multi-resolution 16/32/48/256, embedded into the .exe)

Requires Pillow. The master is an SVG wrapping one embedded 512px PNG, so the
raster is decoded straight from it; cairosvg is used only when it is installed
and the master has no embedded image:
    python3 bridge/scripts/render-icons.py
"""

import base64
import io
import re
from pathlib import Path

from PIL import Image

ASSETS = Path(__file__).resolve().parent.parent / "assets"
MASTER = ASSETS / "icon.svg"


EMBEDDED_PNG = re.compile(rb"data:image/png;base64,([A-Za-z0-9+/=]+)")


def render(svg_bytes: bytes, size: int) -> Image.Image:
    embedded = EMBEDDED_PNG.search(svg_bytes)
    if embedded:
        image = Image.open(io.BytesIO(base64.b64decode(embedded.group(1))))
        return image.convert("RGBA").resize((size, size), Image.LANCZOS)
    import cairosvg

    png = cairosvg.svg2png(bytestring=svg_bytes, output_width=size, output_height=size)
    return Image.open(io.BytesIO(png)).convert("RGBA")


def main() -> None:
    master = MASTER.read_bytes()

    render(master, 1024).save(ASSETS / "window-icon-1024.png")
    # The gradient mark reads on both the dark macOS menu bar and a light
    # Windows tray, so the tray icon is the same master at 44px.
    render(master, 44).save(ASSETS / "tray-icon.png")

    # Render at full resolution and let the ICO writer emit every frame. The base
    # image must be the largest size — Pillow's ICO writer silently drops any
    # requested size larger than the source image.
    ico_sizes = [16, 32, 48, 256]
    render(master, max(ico_sizes)).save(
        ASSETS / "app-icon.ico",
        format="ICO",
        sizes=[(s, s) for s in ico_sizes],
    )

    print("Regenerated:", ", ".join(
        f.name for f in (
            ASSETS / "window-icon-1024.png",
            ASSETS / "tray-icon.png",
            ASSETS / "app-icon.ico",
        )
    ))


if __name__ == "__main__":
    main()
