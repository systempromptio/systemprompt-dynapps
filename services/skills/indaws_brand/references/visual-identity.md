# Visual Identity Reference

**Canonical token values (hex, RGB, CSS vars, font weights) are in `assets/brand-tokens.json`.** This file provides usage context, combinations, and rules that go beyond raw values.

## Table of Contents
1. [Color Specifications](#color-specifications)
2. [Color Combinations](#color-combinations)
3. [Typography Details](#typography-details)
4. [Logo Usage Rules](#logo-usage-rules)
5. [Photography & Art Direction](#photography--art-direction)
6. [Gradient Rules](#gradient-rules)

---

## Color Specifications

### Primary Palette

| Name | HEX | RGB | Pantone | Usage |
|---|---|---|---|---|
| Warm Yellow | `#E5B92B` | 229, 185, 43 | 123 C | Accents, CTAs, highlights |
| Blue Lilac | `#6B68FA` | 107, 104, 250 | 2725 C | Digital, titles, innovation contexts |
| Blue Space | `#1C265D` | 28, 38, 93 | 289 C | Dark backgrounds, body text, formal/serious |
| Light Sky | `#8AC2DB` | 138, 194, 219 | - | Support, light backgrounds, secondary elements |

### Background-Foreground Pairing Rules

| Background | Text Color | Heading Color | CTA Color |
|---|---|---|---|
| White | Blue Space | Blue Lilac | Warm Yellow |
| Blue Space | White | White | Warm Yellow |
| Blue Lilac | White | White | Warm Yellow |
| Light Sky | Blue Space | Blue Space | Blue Lilac |
| Gradient (Blue) | White | White | Warm Yellow |

### Colors to Never Use
- Do not introduce colors outside the defined palette
- No grays: use Blue Space at reduced opacity for subtle backgrounds if needed
- No pure black (#000000): always use Blue Space for dark text

## Color Combinations

### Approved Pairings
- Blue Space + Warm Yellow (high contrast, formal + accent)
- Blue Lilac + White (digital, innovation)
- Blue Lilac + Warm Yellow (vibrant, creative contexts)
- Blue Space + Light Sky (subtle, professional)
- Blue Space + White (maximum contrast, body text)

### Avoid
- Warm Yellow as background (too dominant, only for accents and CTAs)
- Light Sky as text color (insufficient contrast)
- Blue Lilac text on Blue Space background (low contrast)

## Typography Details

### Font: Montserrat (Variable Weight)

Font files: `assets/fonts/Montserrat-VariableFont_wght.ttf` (normal) and `Montserrat-Italic-VariableFont_wght.ttf` (italic).

| Element | Weight | Style | Size Range | Transform | Color |
|---|---|---|---|---|---|
| H1 | 700 (Bold) | Normal | 2-3rem / 32-48px | UPPERCASE | Blue Lilac or White |
| H2 | 700 (Bold) | Normal | 1.5-2rem / 24-32px | UPPERCASE | Blue Lilac or White |
| H3 | 600 (SemiBold) | Normal | 1.25-1.5rem / 20-24px | Normal case | Blue Space or White |
| H4 | 600 (SemiBold) | Normal | 1-1.25rem / 16-20px | Normal case | Blue Space or White |
| Body | 400 (Regular) | Normal | 1rem / 16px | Normal case | Blue Space or White |
| Caption | 400 (Regular) | Normal | 0.85rem / 14px | Normal case | Blue Space or Light Sky |
| CTA Button | 700 (Bold) | Normal | 0.9rem / 14px | UPPERCASE | Blue Space on Yellow bg |

### Line Height & Spacing
- Body: line-height 1.6
- Headings: line-height 1.2
- Letter spacing for uppercase headings: 0.02-0.05em
- Paragraph spacing: 1rem bottom margin

### Fallback
System fallback: `Verdana, sans-serif`. Use only when Montserrat cannot load.

## Logo Usage Rules

### Available Variants

**Full Logos** (`assets/logos/`):
| File | Use When |
|---|---|
| `logotipo-indaws-color-blue-gradient_blue.svg` | Light backgrounds (White, Light Sky) |
| `logotipo-indaws-color-white-gradient_blue.svg` | Dark backgrounds (Blue Space, Blue Lilac, dark images) |
| `logotipo-indaws-color-white-gradient_yellow.svg` | Dark backgrounds, alternative warm variant |
| `logotipo-indaws-monocolor-black.svg` | B&W printing, monochrome contexts |
| `logotipo-indaws-monocolor-white.svg` | Dark backgrounds, monochrome contexts |

**Isotypes** (`assets/isotype/`):
| File | Use When |
|---|---|
| `isotipo-indaws-bluelilac.svg` | Avatars, icons on light backgrounds |
| `isotipo-indaws-gradient_blue.svg` | Digital contexts, app icons |
| `isotipo-indaws-gradient_yellow.svg` | Alternative warm variant |
| `isotipo-indaws-yellow.svg` | Accent contexts |

**Favicons** (`assets/favicons/`):
| File | Use When |
|---|---|
| `indaws-mosca-bg_bluelilac.png` | Default favicon, Blue Lilac brand context |
| `indaws-mosca-bg_bluespace.png` | Formal/serious contexts |
| `indaws-mosca-bg_white.png` | Light interfaces |

### Placement Rules
- Full logo: ONLY in 4 corners or absolute center
- If text is left-aligned, logo goes top-left or bottom-left
- Alignment must be coherent: left text = left logo

### Clear Space
Minimum clear space around the logo: width equivalent to the letter "N" in the logotype. No other element may invade this zone.

### Minimum Sizes
| Context | Full Logo | Isotype |
|---|---|---|
| Digital (screen) | 70px width | 20px width |
| Print | 24mm width | 6mm width |

### Prohibitions
- Do not stretch, squash, or rotate the full logo
- Do not apply drop shadows or effects
- Do not change the logo colors
- The isotype ("W") MAY be rotated 90 degrees or cropped/bled at edges for decorative use

## Photography & Art Direction

### Image Treatment
- Integrate images with brand palette: apply brand color filters, duotones, or flat color overlays
- Complex/noisy photos: convert to B&W to prevent competition with brand colors
- Exception: real event photos or news images may remain in natural color if clean (no overlaid graphics)

### Duotone Combinations
- Blue Space + Blue Lilac (primary duotone)
- Blue Space + Light Sky (subtle, professional)
- Blue Lilac + Warm Yellow (vibrant, creative)

### Composition Rules
- Logo placement on images: only 4 corners or absolute center
- Text on images: ensure sufficient contrast (use semi-transparent Blue Space overlay if needed)
- Avoid placing text over busy/detailed areas of photos

## Gradient Rules

### Direction
Gradients always flow from DARK to LIGHT. Never light-to-dark.

### Defined Gradients
| Name | From | To | CSS |
|---|---|---|---|
| Blue Gradient | `#6B68FA` (Blue Lilac) | `#8AC2DB` (Light Sky) | `linear-gradient(135deg, #6B68FA 0%, #8AC2DB 100%)` |
| Yellow Gradient | `#8AC2DB` (Light Sky) | `#E5B92B` (Warm Yellow) | `linear-gradient(135deg, #8AC2DB 0%, #E5B92B 100%)` |

### Usage
- Blue Gradient: primary digital contexts, innovation, development
- Yellow Gradient: creative/warm contexts, secondary option
- Use gradients for backgrounds, decorative elements, and isotype fills
- Do not use gradients for text
