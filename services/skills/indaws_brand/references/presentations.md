# Presentation Generation Reference

> **Color source of truth**: All color values (HEX, RGB, CSS vars) live in
> [`assets/brand-tokens.json`](../assets/brand-tokens.json). This document
> references colors by token name only. Do NOT hardcode hex values in generated
> code — read them from the tokens file or use the CSS custom properties.

## Table of Contents
1. [Slide Dimensions](#slide-dimensions)
2. [Slide Types](#slide-types)
3. [Typography for Slides](#typography-for-slides)
4. [Color Usage](#color-usage)
5. [Image & Chart Rules](#image--chart-rules)
6. [Transitions & Animation](#transitions--animation)

---

## Slide Dimensions

- Standard: 16:9 (33.87 x 19.05 cm / 1920 x 1080 px)
- Margins/safe zone: 5% inset from all edges
- Logo: always in one of the 4 corners, never larger than 10% of slide width

## Slide Types

### 1. Title Slide
- Background: Blue Space (solid) or Blue Gradient
- Logo: centered, 40% of slide width, `logotipo-indaws-color-white-gradient_blue.svg`
- Title: below logo, Montserrat Bold 44pt, White, UPPERCASE, centered
- Subtitle: Montserrat Regular 24pt, Light Sky, centered
- Optional: Warm Yellow accent line (2px) between title and subtitle

### 2. Section Divider
- Background: Blue Lilac (solid)
- Section title: Montserrat Bold 48pt, White, UPPERCASE, centered vertically
- Isotype watermark (optional): `isotipo-indaws-bluelilac.svg` at 50% opacity, large, positioned at right edge bleeding off-slide

### 3. Content Slide
- Background: White
- Logo: top-left corner, small (isotipo or full logo at minimum size)
- Title: Montserrat Bold 32pt, Blue Lilac, UPPERCASE, top-left aligned
- Body text: Montserrat Regular 18pt, Blue Space, left-aligned
- Use columns (2-column max) for dense content
- Bullet points: Blue Lilac square bullets, 18pt

### 4. Image + Text Slide
- Split layout: 50/50 or 60/40
- Image side: full-bleed, with brand color duotone filter if needed
- Text side: White or Blue Space background
- Logo: on text side, corner aligned with text

### 5. Data / Chart Slide
- Background: White
- Chart area: centered, max 80% of slide width
- Follow chart color sequence from visual-identity.md
- Data labels: Montserrat Regular 14pt
- Source note: bottom-left, Montserrat Regular 10pt, Light Sky

### 6. Quote Slide
- Background: Blue Space
- Large opening quotation mark: Warm Yellow, decorative
- Quote text: Montserrat Regular Italic 24pt, White, centered
- Attribution: Montserrat SemiBold 16pt, Light Sky, below quote

### 7. Closing Slide
- Background: Blue Space or Blue Gradient
- Logo: centered, same size as title slide
- Contact info: Montserrat Regular 18pt, White, centered below logo
- Website / email in Light Sky
- Optional: "Gracias" in Montserrat Bold 36pt, White, UPPERCASE above logo

## Typography for Slides

Sizes are larger than documents due to projection viewing distance.

| Element | Weight | Size | Transform | Color |
|---|---|---|---|---|
| Slide Title | Bold (700) | 32-48pt | UPPERCASE | Blue Lilac or White |
| Subtitle | Regular (400) | 20-24pt | Normal | Light Sky or Blue Space |
| Body | Regular (400) | 16-20pt | Normal | Blue Space or White |
| Caption | Regular (400) | 12-14pt | Normal | Light Sky |
| Data Label | Regular (400) | 14-16pt | Normal | Blue Space |

## Color Usage

### Background Assignment
| Slide Type | Primary Background | Alternative |
|---|---|---|
| Title | Blue Space | Blue Gradient |
| Section Divider | Blue Lilac | Blue Gradient |
| Content | White | Light Sky (10% opacity) |
| Data/Chart | White | - |
| Quote | Blue Space | Blue Lilac |
| Closing | Blue Space | Blue Gradient |

### Accent Color
- Warm Yellow for: CTA buttons, highlight bars, key numbers, decorative elements
- Never use Warm Yellow as full-slide background

## Image & Chart Rules

- All photos: apply brand duotone filter or use B&W
- Exception: product screenshots or UI demos may use natural color
- Charts follow the color sequence: Blue Lilac > Blue Space > Warm Yellow > Light Sky
- Icons: use simple line icons in Blue Lilac or White (matching background)
- No clip art, no stock illustration styles

## Navigation Controls (HTML presentations)

HTML slide decks must include keyboard navigation (arrows + space) and visible buttons. The buttons must be visible on both dark and light slide backgrounds:

```css
.nav button {
  background: var(--blue-space);
  border: 1px solid var(--blue-lilac);
  color: white;
  padding: 0.5rem 1rem;
  border-radius: 6px;
  font-family: 'Montserrat', sans-serif;
  font-size: 0.85rem;
}
.nav button:hover { background: var(--blue-lilac); }
```

The slide counter uses a Blue Space pill so it reads on any background:
```css
.slide-counter {
  color: var(--white);
  background: var(--blue-space);
  padding: 0.2rem 0.6rem;
  border-radius: 4px;
  opacity: 0.6;
}
```

Never use `rgba(255,255,255,...)` for navigation elements — they disappear on white content slides.

### Slide counter initialization

The counter element must NOT have a hardcoded total (e.g., `1 / 15`). Initialize it dynamically on load so it always reflects the actual number of slides:

```html
<div class="slide-counter" id="counter"></div>

<script>
const slides = document.querySelectorAll('.slide');
let current = 0;
document.getElementById('counter').textContent = `1 / ${slides.length}`;

function show(n) {
  slides[current].classList.remove('active');
  current = (n + slides.length) % slides.length;
  slides[current].classList.add('active');
  document.getElementById('counter').textContent = `${current + 1} / ${slides.length}`;
}
</script>
```

This prevents the counter from showing a stale total when slides are added or removed during development.

## Transitions & Animation

- Slide transitions: **None** or **Fade** (0.3s). No flashy effects.
- Content animation: **Appear** only (no fly-in, bounce, spin)
- Charts: can use **Wipe** from left for revealing data
- Keep animation minimal. Content clarity takes priority over visual effects.
