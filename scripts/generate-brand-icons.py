#!/usr/bin/env python3
"""Generate Bastion icon assets from one SVG source."""

from __future__ import annotations

import io
from pathlib import Path

from PIL import Image

try:
    from resvg import render as resvg_render
    from resvg import usvg
except ImportError as exc:  # pragma: no cover - explicit dependency hint for manual usage
    raise SystemExit("Missing dependencies: install `resvg` and `pillow` first.") from exc


ROOT = Path(__file__).resolve().parents[1]
BRANDING_DIR = ROOT / "assets" / "branding"
PUBLIC_DIR = ROOT / "ui" / "public"
IDENTITY_TRANSFORM = (1, 0, 0, 0, 1, 0)

SVG_TEMPLATE = """<svg xmlns="http://www.w3.org/2000/svg" viewBox="96 100 320 320" width="{size}" height="{size}">
  <defs>
    <linearGradient id="gradBlue" x1="0%" y1="0%" x2="100%" y2="100%">
      <stop offset="0%" stop-color="#2563EB" />
      <stop offset="100%" stop-color="#06B6D4" />
    </linearGradient>
    <filter id="shadowBlue" x="-20%" y="-20%" width="140%" height="140%">
      <feDropShadow dx="0" dy="8" stdDeviation="12" flood-color="#2563EB" flood-opacity="0.3" />
    </filter>
  </defs>
  <g filter="url(#shadowBlue)">
    <path d="M128 140 C128 128.9 137 120 148 120 H188 V150 A10 10 0 0 0 208 150 V120 H246 V150 A10 10 0 0 0 266 150 V120 H304 V150 A10 10 0 0 0 324 150 V120 H364 C375 120 384 128.9 384 140 V240 C384 330 300 380 256 400 C212 380 128 330 128 240 Z" fill="url(#gradBlue)" />
    <path d="M256 190 A 45 45 0 1 1 211 235 A 45 45 0 0 1 256 190 Z" fill="none" stroke="#FFFFFF" stroke-width="16" stroke-linecap="round" stroke-dasharray="180 80"/>
    <circle cx="256" cy="235" r="14" fill="#FFFFFF" />
  </g>
</svg>
"""


def build_svg(size: int = 512) -> str:
    return SVG_TEMPLATE.format(size=size)


def render_icon(size: int) -> Image.Image:
    db = usvg.FontDatabase.default()
    db.load_system_fonts()
    opts = usvg.Options.default()
    tree = usvg.Tree.from_str(build_svg(size), opts, db)
    png_bytes = bytes(resvg_render(tree, IDENTITY_TRANSFORM))
    return Image.open(io.BytesIO(png_bytes)).convert("RGBA")


def write_svg(path: Path) -> None:
    path.write_text(build_svg(512), encoding="utf-8")


def main() -> None:
    BRANDING_DIR.mkdir(parents=True, exist_ok=True)
    PUBLIC_DIR.mkdir(parents=True, exist_ok=True)

    master = render_icon(1024)
    master.save(BRANDING_DIR / "icon-master-1024.png")
    write_svg(BRANDING_DIR / "icon-master.svg")

    for size in (512, 256, 192, 180, 64, 48, 32, 16):
        render_icon(size).save(BRANDING_DIR / f"icon-{size}.png")

    render_icon(64).save(
        PUBLIC_DIR / "favicon.ico",
        sizes=[(16, 16), (32, 32), (48, 48), (64, 64)],
    )
    render_icon(32).save(PUBLIC_DIR / "favicon-32x32.png")
    render_icon(16).save(PUBLIC_DIR / "favicon-16x16.png")

    render_icon(256).save(
        BRANDING_DIR / "bastion.ico",
        sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)],
    )


if __name__ == "__main__":
    main()
