"""Builds assets/icon.ico from assets/icon.png.

The Windows executable, installer and shortcuts all use the .ico, while the
runtime reads the .png. Keeping the generator here means an icon change is one
command instead of a binary edit.

    python scripts/make-icon.py
"""

from pathlib import Path

from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "assets" / "icon.png"
TARGET = ROOT / "assets" / "icon.ico"
SIZES = [16, 24, 32, 48, 64, 128, 256]

image = Image.open(SOURCE).convert("RGBA")
largest = max(SIZES)
if image.size != (largest, largest):
    image = image.resize((largest, largest), Image.LANCZOS)
image.save(TARGET, format="ICO", sizes=[(size, size) for size in SIZES])
print(f"{TARGET} ({TARGET.stat().st_size / 1024:.1f} KB)")
