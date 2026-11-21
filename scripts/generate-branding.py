"""Regenerate committed icon exports (Python 3, librsvg, ImageMagick; macOS for ICNS)."""
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BRAND = ROOT / "assets/branding"


def run(*args):
    subprocess.run([str(arg) for arg in args], check=True)


def render(source, destination, size):
    run("rsvg-convert", "-w", size, "-h", size, BRAND / source, "-o", destination)


render("icon.svg", BRAND / "icon.png", 1024)
render("icon.svg", BRAND / "window-icon.png", 256)
render("logo.svg", BRAND / "logo.png", 1024)
run("rsvg-convert", "-w", 1740, BRAND / "banner.svg", "-o", BRAND / "banner.png")
run("magick", BRAND / "icon.png", "-define", "icon:auto-resize=256,128,64,48,32,16", BRAND / "icon.ico")
if shutil.which("iconutil"):
    with tempfile.TemporaryDirectory() as temporary:
        iconset = Path(temporary) / "Yaplay.iconset"
        iconset.mkdir()
        for size in (16, 32, 128, 256, 512):
            for scale in (1, 2):
                suffix = "@2x" if scale == 2 else ""
                render("icon.svg", iconset / f"icon_{size}x{size}{suffix}.png", size * scale)
        run("iconutil", "-c", "icns", iconset, "-o", BRAND / "icon.icns")
else:
    print("ICNS unchanged: regenerate on macOS with iconutil before releasing icon changes.")

render("icon.svg", BRAND / "apple-touch-icon.png", 180)
print("Branding exports generated in assets/branding.")
