# Dynapps Brand System — Full Reference

## Color System

### Primary Palette (brand book canonical values)

| Name | HEX | RGB | CMYK |
|---|---|---|---|
| **Midnight Denim** | `#010114` | 1, 1, 20 | 7, 7, 0, 92 |
| **Unicorn Violet** | `#7B00DC` | 123, 0, 220 | 38, 86, 0, 14 |
| **Ferrarri Carmine** | `#F13C42` | 241, 60, 66 | 0, 71, 69, 5 |
| **Orange Oefans** | `#F65836` | 246, 88, 54 | 0, 62, 75, 4 |
| **Sunset Tangerine** | `#FF911F` | 255, 145, 31 | 0, 43, 88, 0 |

### Extended Palette (use for all digital/screen work)

| Name | HEX | RGB |
|---|---|---|
| **Sunset Tangerine** | `#FF7E23` | 255, 126, 36 |
| **Orange Oefans** | `#FF7759` | 255, 119, 89 |
| **Ferrarri Carmine** | `#FF575D` | 255, 87, 93 |
| **Unicorn Violet** | `#951CF3` | 149, 28, 243 |
| **Midnight Denim** | `#050543` | 5, 5, 67 |

### CSS Custom Properties

```css
:root {
  /* Extended palette — default for all digital work */
  --midnight:  #050543;
  --violet:    #951CF3;
  --carmine:   #FF575D;
  --orange:    #FF7759;
  --tangerine: #FF7E23;

  /* Depth variants */
  --deep:      #030330;
  --surface:   #070757;

  /* Light section */
  --snow:      #F5F5FA;
  --off-white: #F6F5F8;
  --white:     #FFFFFF;

  /* Text on dark backgrounds */
  --tw:   rgba(255,255,255,.92);   /* Primary text */
  --td:   rgba(255,255,255,.54);   /* Secondary */
  --tdd:  rgba(255,255,255,.28);   /* Muted */
  --lm:   #7272A8;                 /* Labels / meta */

  /* Grey scale (light sections) */
  --grey-100: #E8E7EC;
  --grey-200: #CDCBD4;
  --grey-400: #9290A6;
  --grey-600: #65637A;
}
```

## Gradient System

Three-color gradient at **45 degrees**: Unicorn Violet → Ferrarri Carmine → Orange Oefans.

```css
/* Primary (shapes, backgrounds, logo symbol) */
--grad: linear-gradient(45deg, var(--violet) 0%, var(--carmine) 52%, var(--orange) 100%);

/* Horizontal (borders, lines, dividers) */
--grad-h: linear-gradient(90deg, var(--violet) 0%, var(--carmine) 52%, var(--orange) 100%);
```

The 52% midpoint gives carmine slightly more presence. Keep margin around gradient elements.

**Use gradient on:** logo symbol (aperture icon), accent lines (nav underline, footer top), background shapes at very low opacity, icon backgrounds.
**Never:** on the wordmark, as full-page background, on body text.

## Typography

**Font**: Readex Pro (Google Webfont). Files bundled in `assets/fonts/`.

| Element | Weight | Size Guidance |
|---|---|---|
| Display / hero titles | Bold (700) | Large, with accent word highlighted |
| Section titles | SemiBold (600) | Clear hierarchy |
| Body text | Regular (400) | Comfortable reading size |
| Labels / nav / meta | Medium (500) | Small, uppercase, letter-spaced |
| Decorative numbers | Light (300) | Section numbers, large display |
| Subtle accents | ExtraLight (200) | Rare, decorative only |

**Fallback**: `'Readex Pro', system-ui, sans-serif`

**Loading for web**:
```html
<link rel="preconnect" href="https://fonts.googleapis.com">
<link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
<link href="https://fonts.googleapis.com/css2?family=Readex+Pro:wght@200;300;400;500;600;700&display=swap" rel="stylesheet">
```

### Headline Keyword Highlighting
In headlines, highlight the **key business action word** using an accent color:
- On dark backgrounds: use Tangerine (`var(--tangerine)`) or Carmine (`var(--carmine)`)
- On light backgrounds: use Violet (`var(--violet)`) or Carmine

In HTML: `<span class="accent">word</span>` with `.accent { color: var(--carmine); }`

## Logo System

### Quick-Select

| Background | Logo File |
|---|---|
| White / Light | `Dynapps_Logo_Horizontal_Color_Black.svg` |
| Midnight / Dark | `Dynapps_Logo_Horizontal_Color_White.svg` |
| Violet / Carmine / Orange / Gradient | White logo variant |

For vertical variants with "experts in **odoo**" baseline, use the corresponding `Vertical` files.

### Brand Name Capitalisation
- **In the logo/wordmark**: always lowercase **dynapps**
- **In written text**: always capitalised **Dynapps** — proper noun with capital D

### Rules
- Clear space: minimum width of the letter "d" on all sides
- Gradient applies only to the aperture symbol, never to wordmark
- Never: reorder symbol/wordmark, alter proportions, reduce opacity, apply gradient to full logo

## Brand Icons

Eight service-pillar icons in `assets/icons/`:
Boost · Build · Extra · Improve · Integrate · Manage · Personalise & Develop · Streamline

## Visual Design Patterns

### Background Mode Logic

Dynapps content defaults to **light backgrounds**. Dark backgrounds are reserved for high-impact moments.

**Presentations (PPTX) — Light-first sandwich:**

| Slide Type | Background | When |
|---|---|---|
| Title / cover slide | **Dark** (`--midnight`) | First slide only |
| Content slides | **Light** (`--snow` or `--white`) | All standard content |
| Accent / spotlight slide | **Dark** (`--deep`) | Max 1 per deck |
| Closing / thank-you slide | **Dark** (`--midnight`) | Final slide only |

On light slides: `Dynapps_Logo_Horizontal_Color_Black.svg`, text in `--midnight`, cards white with `--grey-100` border.
On dark slides: `Dynapps_Logo_Horizontal_Color_White.svg`, text in `--tw`, ambient glow optional.

**HTML documents — Dark hero, light body:**

| Section | Background |
|---|---|
| Navigation bar | Dark glass (`rgba(5,5,67,0.88)`) |
| Hero / opening | **Dark** (`--midnight`) |
| Body sections | **Light** (`--snow` / `--white`) |
| Feature / spotlight section | **Dark** (`--deep`) — max 1 |
| Footer | **Dark** (`--deep`) |

**Documents (DOCX / PDF) — Always light.** White/snow backgrounds exclusively.

### Section Background Reference

| Class | Background | Text | Usage |
|---|---|---|---|
| on-light | `--snow` (#F5F5FA) | Dark (#050543) | **Default** |
| on-white | `--white` (#FFFFFF) | Dark (#050543) | Clean variant |
| on-dark | `--midnight` (#050543) | White (--tw) | Title, closing, nav, hero |
| on-deep | `--deep` (#030330) | White (--tw) | Accent/spotlight (use sparingly) |

### UI Micro-Patterns

- **Glass nav bar**: `background: rgba(5,5,67,0.88); backdrop-filter: blur(20px);` with gradient line underneath
- **Gradient accent lines**: 1-3px, `--grad-h`, opacity 0.3-0.4
- **Pill shapes**: `border-radius: 100px` with subtle borders
- **Cards on light**: `background: #FFFFFF`, `border: 1px solid var(--grey-100)`, shadow, rounded 12-16px
- **Cards on dark**: `background: var(--surface)` or `rgba(255,255,255,.04)`, rounded 12-16px
- **Scroll reveal**: Fade up with `translateY(18-40px)`, `opacity: 0→1`, cubic-bezier easing
- **Ambient glow**: Radial gradients of brand colors at 0.06-0.10 opacity — dark slides only
- **Section numbering**: Light-weight (300) numbers like "01", "02" before section titles

## Visual Review Checklist

- [ ] Only brand colors used (extended palette for digital)
- [ ] Readex Pro font loaded and applied
- [ ] Correct logo variant for background color
- [ ] Logo clear space respected (width of "d")
- [ ] Gradient at 45 degrees (Violet → Carmine → Orange)
- [ ] Gradient only on symbol, never full logo
- [ ] Brand name: "Dynapps" in text (capital D), "dynapps" only in logo/wordmark
- [ ] Headline keyword highlighted in accent color
- [ ] Light backgrounds used by default; dark only for title/closing/accent
- [ ] Correct logo variant matches slide/section background (black on light, white on dark)
- [ ] Gradient accent lines subtle, not overwhelming
