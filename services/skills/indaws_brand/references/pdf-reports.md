# PDF Report Generation Reference

> **Color source of truth**: All color values (HEX, RGB, CSS vars) live in
> [`assets/brand-tokens.json`](../assets/brand-tokens.json). This document
> references colors by token name only. Do NOT hardcode hex values in generated
> code — read them from the tokens file or use the CSS custom properties.

## Table of Contents
1. [Page Layout](#page-layout)
2. [Cover Page](#cover-page)
3. [Interior Pages](#interior-pages)
4. [Tables & Charts](#tables--charts)
5. [Print Color Notes](#print-color-notes)
6. [Generation Methods](#generation-methods)

---

## Page Layout

### Standard A4 Document
- Page size: 210 x 297mm (A4)
- Margins: Top 25mm, Bottom 25mm, Left 25mm, Right 25mm
- Header zone: 15mm from top edge
- Footer zone: 15mm from bottom edge
- Content area: 160 x 247mm

### Header (All Pages)
- Logo: top-left, 24mm width minimum
- Use `logotipo-indaws-color-blue-gradient_blue.svg` on white pages
- Use `logotipo-indaws-color-white-gradient_blue.svg` on dark pages
- Optional: thin Blue Lilac line (0.5pt) below header

### Footer (All Pages)
- Left: "Indaws" or small logo (isotipo at 6mm minimum)
- Center: document title in Light Sky, 8pt Montserrat Regular
- Right: page number in Blue Space, 8pt Montserrat Regular
- Optional: thin Blue Lilac line (0.5pt) above footer

## Cover Page

### Layout
- Full-page Blue Space background
- Logo: centered horizontally, positioned at 30% from top
- Use `logotipo-indaws-color-white-gradient_blue.svg`
- Logo width: 60% of page width

### Title Block (below logo, centered)
- Document title: Montserrat Bold, 28pt, White, UPPERCASE
- Subtitle / client name: Montserrat SemiBold, 18pt, Light Sky
- Date: Montserrat Regular, 12pt, Light Sky
- Spacing: 15mm between title elements

### Optional Elements
- Warm Yellow accent line (2pt, 40mm width) between title and subtitle
- Version number: bottom-right corner, 8pt, Light Sky

## Interior Pages

### Section Headers
- H1: Montserrat Bold, 24pt, Blue Lilac, UPPERCASE. Page break before H1 recommended.
- H2: Montserrat Bold, 18pt, Blue Lilac, UPPERCASE
- H3: Montserrat SemiBold, 14pt, Blue Space
- Body: Montserrat Regular, 11pt, Blue Space, line-height 1.6

### Paragraph Style
- First paragraph after heading: no indent
- Subsequent paragraphs: 5mm indent or 6pt space-before (choose one consistently)
- Justified alignment for body text
- Left alignment for headings

### Callout Boxes
- Background: Light Sky at 20% opacity
- Left border: 3pt Blue Lilac
- Padding: 10mm
- Text: Montserrat Regular, 10pt, Blue Space

### Key Figures / Highlights
- Background: Blue Lilac
- Text: White, Montserrat Bold, 14pt
- Padding: 8mm
- Use for KPIs, key metrics, or important data points

## Tables & Charts

### Table Styling
- Header row: Blue Space background, White text, Montserrat Bold 10pt, UPPERCASE
- Body rows: alternating White / Light Sky at 15% opacity
- Borders: 0.5pt Light Sky
- Cell padding: 4mm vertical, 6mm horizontal
- Text: Montserrat Regular, 10pt, Blue Space

### Chart Color Sequence
When multiple data series are needed, use colors in this order:
1. Blue Lilac
2. Blue Space
3. Warm Yellow
4. Light Sky

### Chart Typography
- Chart title: Montserrat SemiBold, 12pt, Blue Space
- Axis labels: Montserrat Regular, 9pt, Blue Space
- Data labels: Montserrat Regular, 8pt, Blue Space
- Legend: Montserrat Regular, 9pt

## Print Color Notes

For professional printing, provide these Pantone references:

| Color | Pantone | CMYK (approximate) |
|---|---|---|
| Warm Yellow | 123 C | C:0 M:20 Y:85 K:0 |
| Blue Lilac | 2725 C | C:65 M:60 Y:0 K:0 |
| Blue Space | 289 C | C:100 M:85 Y:35 K:45 |

Use Pantone spot colors when budget allows. Otherwise, CMYK process is acceptable.

## Technical Document Components

For documents with code, APIs, or technical content, `base.html` includes these additional components:

### Code Blocks (executable code)

```html
<pre class="code-block"><code>def my_method(self):
    return self.env['res.partner'].search([])</code></pre>
```

Style: Blue Space background, Warm Yellow left border, Courier New 7pt. Use for Python, SQL, bash, XML.

### Diagram Blocks (ASCII art, flows)

```html
<pre class="diagram-block"><code>┌──────────┐     ┌──────────┐
│  Step 1   │────▶│  Step 2   │
└──────────┘     └──────────┘</code></pre>
```

Style: Blue Lilac at 5% opacity background, Blue Lilac left border. Use for ASCII diagrams, flow charts, architecture drawings.

### Metadata Grid

```html
<div class="metadata">
    <div class="item"><strong>Cliente</strong>Autosur</div>
    <div class="item"><strong>Versión</strong>18.0 CE/EE</div>
</div>
```

Style: 2-column grid, labels in Blue Lilac uppercase 7.5pt, values below. Use for document metadata instead of inline bold text.

### Long Code Blocks

Code blocks longer than 25 lines should be split into separate `<pre>` elements. Each one gets `page-break-inside: avoid` automatically. Never let a single `<pre>` span more than one page.

## Generation Method

For PDF rendering instructions, print CSS rules, and page layout gotchas, see `pdf-renderer/SKILL.md` and `pdf-renderer/references/`.

**Page-break SSOT**: multi-page tables, repeating `tfoot` totals, footer pinning, orphan
headings and dark-bar specificity bugs are documented once in
`pdf-renderer/references/playwright-pdf-guide.md` ("Multi-page report patterns 9–14"). Follow
that shared guide; keep this file to the Indaws brand layer only.

**Brand-specific notes:**
- Start from `assets/templates/base.html`
- Use the self-hosted Montserrat from `assets/fonts/Montserrat-VariableFont_wght.ttf` with absolute path in `@font-face`.
