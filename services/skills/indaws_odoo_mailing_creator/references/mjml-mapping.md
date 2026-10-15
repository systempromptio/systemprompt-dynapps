# HTML → MJML mapping & Outlook patches

How to turn a source HTML design into MJML faithfully, and the few manual patches MJML does not do. Brand-agnostic: extract the source's own colors/fonts; never impose a house style.

## Table of contents
- [1. Document skeleton](#1-document-skeleton)
- [2. Block → component mapping](#2-block--component-mapping)
- [3. Fonts](#3-fonts)
- [4. Gradients, shadows, radius, dark mode](#4-gradients-shadows-radius-dark-mode)
- [5. Outlook patches (mj-raw)](#5-outlook-patches-mj-raw)
- [6. Gotchas](#6-gotchas)

---

## 1. Document skeleton
```xml
<mjml>
  <mj-head>
    <mj-font name="<FAMILY>" href="<google-fonts-url>" />        <!-- if the source uses a webfont -->
    <mj-attributes>
      <mj-all font-family="<FAMILY>, Arial, Helvetica, sans-serif" />  <!-- Arial first fallback for Outlook -->
    </mj-attributes>
    <mj-style>/* @media, dark mode, box-shadow, radius for modern clients */</mj-style>
    <mj-raw><meta name="color-scheme" content="light dark"><meta name="supported-color-schemes" content="light dark"></mj-raw>
  </mj-head>
  <mj-body background-color="<PAGE_BG>">
    <mj-section ...><mj-column>...</mj-column></mj-section>
  </mj-body>
</mjml>
```
Read the source for its real `PAGE_BG`, brand colors, and font; plug them in. Width defaults to 600px (email standard).

## 2. Block → component mapping
| Source block | MJML | Notes |
|---|---|---|
| Full-width band / hero with bg image | `mj-section` (bg color/url) or `mj-hero` | These are the ONLY two with Outlook bg support (MJML emits VML). |
| Columns side by side | adjacent `mj-column` | Stack automatically on mobile. |
| **Equal-height cards in a row** | `mj-table` with sibling `<td>` cells in one `<tr>` (NOT `mj-column`/`mj-group`/`min-height` — those don't equalize). Gap = a spacer `<td>` whose width makes the row sum to exactly 100%. Stack on mobile via `@media{td.cardcell{display:block;width:100%}}`. | See `odoo-rendering-gotchas.md` §3,§9,§10. Never use `mj-raw` with a full `<table>` — it breaks the body/bg wrapper (§4); use the native `mj-table`. |
| Paragraph / heading | `mj-text` | Accepts inline HTML; no MJML children. |
| Button / CTA | `mj-button` (`border-radius`, `background-color`, `href`) | Bulletproof (tables + `<a>`). Outlook radius needs VML (see §5). |
| Image | `mj-image` (`src` absolute https, `alt`, `width`) | — |
| Inline `<svg>` icon | PNG (default) OR emoji (fallback) | Emails don't render inline SVG. Default: `svg_to_png.cjs` + Odoo ir.attachment upload. Fallback to emoji if rsvg/upload unavailable (see `svg-assets.md`). |
| Divider | `mj-divider` | — |
| Vertical space | `mj-spacer` | — |
| Data table | `mj-table` | Plain HTML only. |
| Social icons | `mj-social` | — |
| Anything MJML can't express | `mj-raw` | Raw HTML escape hatch. |

## 3. Fonts
- Webfont from Google: `mj-font` (MJML auto-gates the `<link>`+`@import` in `[if !mso]`, so Outlook never tries to load it and falls back).
- Self-hosted `@font-face`: put it in `mj-style` (head). Apple Mail/iOS use it; Gmail/Outlook ignore it (expected).
- Always set a safe first fallback (`Arial`/`Helvetica`) in `mj-all` so Outlook does not drop to Times.

## 4. Gradients, shadows, radius, dark mode
Pattern: solid as the MJML attribute (universal fallback, auto-inlined) + the fancy bit in `mj-style` (non-inline `@media`/`!important`) for modern clients.
```xml
<mj-section background-color="#635bff" css-class="hero">...</mj-section>
```
```xml
<mj-style>
  .hero div { background-image: linear-gradient(135deg,#635bff,#0a2540) !important; }  /* modern only */
  .card { box-shadow: 0 4px 16px rgba(10,37,64,.18); }                                  /* Outlook ignores */
  @media (prefers-color-scheme: dark){ .dark-bg{background-color:#121212!important} .dark-txt{color:#f1f1f1!important} }
</mj-style>
```
Use neutral dark `#121212`/`#f1f1f1`, not pure black/white (avoids harsh Apple Mail inversion).

## 5. Outlook patches (mj-raw)
MJML does NOT do these — add them only if the source design requires them:
- **Rounded button in Outlook (Word):** MJML emits `border-radius` as CSS, which Word ignores (square). For a real rounded CTA, add a `v:roundrect` inside `mj-raw` gated `[if mso]`, with the normal `mj-button` gated `[if !mso]`. `arcsize` is the radius in **%**. Many B2B teams accept a square CTA in classic Outlook and only round the primary CTA.
- **Real gradient in Outlook:** `v:fill type="gradient"` inside `mj-raw` `[if mso]`. Modern clients get the CSS gradient (§4); the mso block is the Outlook fallback.
- **Dark mode meta:** `meta name="color-scheme"` via `mj-raw` (MJML has no component for it).

Reliable mso pattern inside `mj-raw`: keep each conditional block **balanced and self-contained**, or use the inverted `<!--[if !mso]><!-->` form. A stray `<![endif]-->` can swallow the next tag.

## 6. Gotchas
- `css-class` lands on the **outermost** tag of the component, NOT the inner text `<div>`. To style text, prefer MJML attributes (`font-size`, `color`) or a child selector (`.cls div{...}`). Always inspect the compiled HTML.
- `mj-text`/`mj-button`/`mj-table`/`mj-raw` accept no MJML children — HTML/text only.
- **Literal `<` inside text breaks the minifier.** A real `<` in `mj-text`/`mj-table` content (e.g. `< 2 %`) is read as a tag open and html-minifier-terser can mangle/drop what follows. Write it as the entity `&lt;` (e.g. `&lt; 2 %`). It bit us in mailing 05. (MJML/upstream also support wrapping a tricky run in `<!-- htmlmin:ignore -->…<!-- htmlmin:ignore -->`, but the entity is simpler and predictable here.)
- **`mj-group` vs `mj-table` for columns** — both keep columns side by side; pick by need:
  - Need columns of **equal height** (cards that must match) → `mj-table` sibling `<td>` cells (gotcha `odoo-rendering-gotchas.md` §3; `mj-group`/`min-height` do NOT equalize).
  - Only need **anti-stacking** on mobile (keep 2 columns side by side, heights can differ) → `mj-group` is the lighter, native tool. Use it when equal height is irrelevant.
- Placeholders: plain UPPERCASE words pass through MJML and juice untouched. Avoid bare `{{ }}` for *fill-later* placeholders. For real per-recipient personalization use Odoo's render syntax — see `odoo-personalization.md` (body=`<t t-out>`, preview/subject=`{{ }}`).
- Pin `mjml@4.18.0` (v5 is async and breaks the sync pipeline).
