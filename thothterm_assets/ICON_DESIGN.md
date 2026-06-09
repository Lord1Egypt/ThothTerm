# 𓆣 ThothTerm — Icon Design Guide

## Design Concept

The ThothTerm icon is pure ancient Egypt. Dark lapis lazuli background (Egypt's sacred blue), gold hieroglyphic elements, and a terminal cursor that proves this is a tool for modern commands.

## Elements Used

| Symbol | Meaning | Position |
|--------|---------|---------|
| 𓆣 Scarab (Khepri) | Transformation, creation, new beginnings — you start a new process, you transform systems | Center — dominant element |
| ☥ Ankh | Life, power, eternity — your terminal session lives on | Top center, turquoise |
| 𓂀 Eye of Ra (Wadjet) | Protection, all-seeing — the terminal sees all output | Bottom center |
| Gold wings | Power and divinity in Egyptian art | Scarab wings left/right |
| Terminal cursor `▌` | You're in a terminal | Bottom right, turquoise glow |

## Color Palette

| Color | Hex | Meaning |
|-------|-----|---------|
| Lapis lazuli dark | `#0a0f2e` | Background — sacred Egyptian blue |
| Midnight blue | `#1a2a5e` | Gradient center |
| Gold | `#ffd700` | Primary elements — divine Egyptian gold |
| Dark gold | `#d4a800` | Shading |
| Turquoise | `#40e0d0` | Ankh + cursor — Egyptian faience color |

## File: `thothterm.svg`

The SVG is the master source. Convert to other formats:

```bash
# Install Inkscape (best quality)
sudo apt install inkscape

# Generate PNG sizes
inkscape --export-png=assets/thothterm_16.png   --export-width=16   assets/thothterm.svg
inkscape --export-png=assets/thothterm_32.png   --export-width=32   assets/thothterm.svg
inkscape --export-png=assets/thothterm_64.png   --export-width=64   assets/thothterm.svg
inkscape --export-png=assets/thothterm_128.png  --export-width=128  assets/thothterm.svg
inkscape --export-png=assets/thothterm_256.png  --export-width=256  assets/thothterm.svg
inkscape --export-png=assets/thothterm_512.png  --export-width=512  assets/thothterm.svg

# Windows ICO (multi-size)
convert \
  assets/thothterm_16.png  \
  assets/thothterm_32.png  \
  assets/thothterm_64.png  \
  assets/thothterm_128.png \
  assets/thothterm_256.png \
  assets/thothterm.ico

# macOS ICNS
mkdir -p thothterm.iconset
cp assets/thothterm_16.png  thothterm.iconset/icon_16x16.png
cp assets/thothterm_32.png  thothterm.iconset/icon_16x16@2x.png
cp assets/thothterm_32.png  thothterm.iconset/icon_32x32.png
cp assets/thothterm_64.png  thothterm.iconset/icon_32x32@2x.png
cp assets/thothterm_128.png thothterm.iconset/icon_128x128.png
cp assets/thothterm_256.png thothterm.iconset/icon_128x128@2x.png
cp assets/thothterm_256.png thothterm.iconset/icon_256x256.png
cp assets/thothterm_512.png thothterm.iconset/icon_256x256@2x.png
cp assets/thothterm_512.png thothterm.iconset/icon_512x512.png
iconutil -c icns thothterm.iconset -o assets/thothterm.icns
```

## Preview (ASCII)

```
    ╔══════════════════════════════╗
    ║  ██████████████████████████ ║
    ║  █   DARK LAPIS LAZULI    █ ║
    ║  █                        █ ║
    ║  █         ☥ (teal)       █ ║
    ║  █                        █ ║
    ║  █    ✦  𓆣 SCARAB  ✦     █ ║
    ║  █   (gold, wings wide)   █ ║
    ║  █                        █ ║
    ║  █      𓂀 (gold)         █ ║
    ║  █                   ▌    █ ║
    ║  ██████████████████████████ ║
    ╚══════════════════════════════╝
```

## Notes for Designer (if hiring one)

- Keep the dark blue background — it feels like the night sky over Egypt
- The scarab should feel powerful, not cute
- The ankh and Eye of Ra should feel like they're watching over the terminal
- The terminal cursor `▌` in turquoise is what ties it to technology
- Gold lines should look like they're carved into lapis lazuli
- Reference: Tutankhamun's golden scarab pectoral jewelry
