# Yaplay artwork

The original overlay mascot uses two rounded windows and a smiling face. The palette is peach `#FFB49B` on charcoal `#141418`. The banner and wordmark use **Geist Pixel Square**, converted to vector outlines so consumers do not need to install a font. Geist is distributed under the [SIL Open Font License](Geist-LICENSE.txt).

`logo.svg`, `icon.svg`, `wordmark.svg`, and `banner.svg` are the source artwork. PNG, ICO, and ICNS files are committed exports used by the website and desktop release builds. `window-icon.png` is embedded into native windows; `icon.ico` is embedded into the Windows executable; `icon.icns` is bundled into the macOS application.

After editing artwork, run `python3 scripts/generate-branding.py` from the repository root. This needs `rsvg-convert` and ImageMagick (`brew install librsvg imagemagick` on macOS). Generate ICNS on macOS with the built-in `iconutil`. Run `node site/scripts/prepare.mjs` to copy the current artwork into the website. The original design drafts remain in `output/branding/`; production code uses this directory.
