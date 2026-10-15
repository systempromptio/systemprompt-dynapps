# Playwright PDF Guide — Technical Reference

## page.pdf() Options

```javascript
await page.pdf({
    path: 'output.pdf',              // output file
    format: 'A4',                     // or 'Letter', 'Legal'
    printBackground: true,            // CRITICAL: renders background colors/images
    preferCSSPageSize: false,         // true = use @page size from CSS instead
    margin: {                         // override @page margins
        top: '25mm',
        bottom: '25mm',
        left: '25mm',
        right: '25mm',
    },
    displayHeaderFooter: false,       // Chromium header/footer (usually not wanted)
    headerTemplate: '',               // HTML for header (if displayHeaderFooter: true)
    footerTemplate: '',               // HTML for footer
    scale: 1,                         // 0.1 to 2.0
    landscape: false,
    pageRanges: '',                   // e.g., '1-5' for specific pages
    tagged: true,                     // accessibility tags (PDF/UA)
    outline: true,                    // generate bookmarks from headings
});
```

### Key option interactions

- `printBackground: true` is **mandatory** — without it, all background colors and images are stripped
- `preferCSSPageSize: true` makes `@page { size: ... }` in CSS take precedence over the `format` option
- `margin` in page.pdf() OVERRIDES `@page { margin: ... }` in CSS. Use one or the other, not both
- `displayHeaderFooter: true` enables Chromium's built-in header/footer. These are rendered in a separate isolated context (no access to page CSS). Usually better to use CSS `position: fixed` for headers/footers instead
- `outline: true` generates PDF bookmarks from `<h1>`-`<h6>` elements — good for navigation in long documents

---

## @page CSS Rules

```css
/* Basic page setup */
@page {
    size: A4;                         /* A4, A3, Letter, Legal, or custom: 210mm 297mm */
    margin: 25mm;                     /* all sides */
}

/* Different margins for first page (cover) */
@page :first {
    margin: 0;                        /* full-bleed cover */
}

/* Left/right page margins (for bound documents) */
@page :left {
    margin-left: 30mm;               /* extra gutter on left pages */
}
@page :right {
    margin-right: 30mm;
}

/* Named pages for different sections */
.cover { page: cover; }
@page cover {
    margin: 0;
    background: #1C265D;
}

.appendix { page: appendix; }
@page appendix {
    margin: 15mm;
}
```

---

## Page Break Control

```css
/* Force break before element */
.page-break, .chapter { page-break-before: always; }

/* Avoid break before (e.g., first chapter) */
h1:first-of-type { page-break-before: avoid; }

/* Keep element together (no split across pages) */
table, figure, .card, .no-break { page-break-inside: avoid; }
tr { page-break-inside: avoid; }

/* Avoid orphans/widows */
p { orphans: 3; widows: 3; }

/* Avoid break after heading (keep with following content) */
h1, h2, h3 { page-break-after: avoid; }
```

### Common patterns

```css
/* Section: always starts on new page, never splits */
.section {
    page-break-before: always;
    page-break-inside: avoid;
}

/* Table: avoid split, but allow if too tall */
table {
    page-break-inside: avoid;
}
/* Individual rows: never split */
tr {
    page-break-inside: avoid;
}

/* Image with caption: keep together */
figure {
    page-break-inside: avoid;
}
```

---

## Fixed Headers & Footers (CSS approach)

Chromium's `displayHeaderFooter` is limited (no page CSS access). Use CSS instead:

```html
<div class="print-header">
    <img src="logo.svg" alt="Logo" style="height: 10mm;">
    <span>Document Title</span>
</div>

<div class="print-footer">
    <span>Indaws Business Solutions S.L.</span>
    <span class="page-number"></span>
</div>

<div class="content">
    <!-- your content here -->
</div>
```

```css
@media print {
    .print-header {
        position: fixed;
        top: 0; left: 25mm; right: 25mm;
        height: 15mm;
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-bottom: 0.5pt solid #8AC2DB;
    }
    .print-footer {
        position: fixed;
        bottom: 0; left: 25mm; right: 25mm;
        height: 15mm;
        display: flex;
        align-items: center;
        justify-content: space-between;
        font-size: 8pt;
        border-top: 0.5pt solid #8AC2DB;
    }
    .content {
        margin-top: 20mm;  /* space for header */
        margin-bottom: 20mm;  /* space for footer */
    }
}
```

**Limitation**: CSS `position: fixed` headers/footers repeat on every page but cannot show page numbers. For page numbers, use Chromium's `displayHeaderFooter` with `footerTemplate`:

```javascript
await page.pdf({
    displayHeaderFooter: true,
    headerTemplate: '<span></span>',  // empty header
    footerTemplate: `
        <div style="width: 100%; font-size: 8pt; font-family: Montserrat, sans-serif;
                    padding: 0 25mm; display: flex; justify-content: flex-end;">
            <span class="pageNumber"></span> / <span class="totalPages"></span>
        </div>
    `,
    margin: { top: '25mm', bottom: '25mm', left: '25mm', right: '25mm' },
});
```

Available template variables: `<span class="date">`, `<span class="title">`, `<span class="url">`, `<span class="pageNumber">`, `<span class="totalPages">`.

---

## Fonts

### Embedding custom fonts

For fonts to render in PDF, they must be loaded BEFORE `page.pdf()` runs:

```javascript
// Option 1: Self-hosted font in HTML (recommended)
// In your HTML <style>:
@font-face {
    font-family: 'Montserrat';
    src: url('file:///absolute/path/to/Montserrat-VariableFont_wght.ttf') format('truetype');
    font-display: swap;
}

// Option 2: Google Fonts (requires network)
// In your HTML <head>:
<link href="https://fonts.googleapis.com/css2?family=Montserrat:wght@300;400;600;700&display=swap" rel="stylesheet">
```

**Always use absolute paths** for local fonts. Relative paths break when Playwright navigates to `file://` URLs.

---

## Images

### Reliable image loading

```javascript
// Wait for all images to load before rendering PDF
await page.goto('file://' + filePath, { waitUntil: 'networkidle' });
// OR for local files with slow image loading:
await page.waitForLoadState('load');
await page.waitForTimeout(500);  // safety buffer for font rendering
```

### Image formats
- **SVG**: best for logos, icons (infinite resolution)
- **PNG**: best for screenshots, diagrams (lossless)
- **JPEG**: acceptable for photos (lossy but smaller)
- **Data URIs**: most reliable — no file path issues: `<img src="data:image/png;base64,...">`

For logos/brand assets, always convert to **data URI** or use **absolute file paths**.

---

## Gotchas

### 1. Background colors missing
**Symptom**: Cover page is white, all backgrounds gone.
**Fix**: `printBackground: true` in page.pdf() options. ALWAYS set this.

### 2. Margins double-applied
**Symptom**: Content has too much whitespace.
**Fix**: Use EITHER `@page { margin }` in CSS OR `margin` in page.pdf(). Not both. If using `@page`, set `preferCSSPageSize: true`.

### 3. Font not rendering
**Symptom**: Text falls back to serif/system font.
**Fix**: Use absolute path in `@font-face` src. Wait for `networkidle` before pdf(). For Google Fonts, ensure network access.

### 4. Images broken
**Symptom**: Broken image icons in PDF.
**Fix**: Use absolute `file://` paths or data URIs. Relative paths resolve against the page URL, which may not be what you expect.

### 5. Table split across pages
**Symptom**: Table header on one page, rows on next.
**Fix**: `table { page-break-inside: avoid; }` and `tr { page-break-inside: avoid; }`. For very long tables, add `thead { display: table-header-group; }` to repeat headers.

### 6. Cover page not full-bleed
**Symptom**: Cover has white margins.
**Fix**: Use `@page :first { margin: 0; }` and make the cover div `width: 100vw; height: 100vh; padding: 25mm;` (handles its own internal padding).

### 7. CSS Grid/Flexbox broken
**Symptom**: Layout looks different from browser.
**Fix**: This is a WeasyPrint issue, NOT Playwright. Playwright renders full CSS. If you see this, check you're not mixing print/screen media queries incorrectly.

### 8. Slow rendering
**Symptom**: PDF takes 10+ seconds.
**Fix**: Heavy pages with many images. Preload images, use `waitUntil: 'load'` instead of `'networkidle'` if no external resources needed. Reduce image sizes.

---

## Multi-page report patterns (production gotchas 9–13)

These apply to **any** branded multi-page report (estimates, audits, proposals). They are
brand-agnostic — `dynapps-brand`, `dynapps-brand` and any document skill should follow them.

### 9. Repeating `<tfoot>` — total appears mid-table
**Symptom**: a table total ("Total …", "Subtotal …") shows at the bottom of *every* page the
table spans, so it appears prematurely mid-table.
**Cause**: `<tfoot>` defaults to `display: table-footer-group`, which repeats per page.
**Fix**: make it render once, at the real end:
```css
tfoot { display: table-row-group; }   /* total appears once; thead still repeats */
```
Keep `thead { display: table-header-group; }` so the header keeps repeating.

### 10. Number/text invisible on a dark total bar
**Symptom**: a dark "Total" row shows the label but the hours/amount cell is blank.
**Cause**: CSS specificity — a generic numeric rule like `.tbl td.h { color: #050543 }`
overrides `tfoot td { color:#fff }`, painting dark text on a dark bar (invisible).
**Fix**: re-assert the color with higher specificity:
```css
tfoot td.h { color: #fff; }
```

### 11. Numeric cells wrap onto two lines
**Symptom**: "21 h" / "44 h" breaks as "21" + "h" in a narrow numeric column.
**Fix**: `white-space: nowrap` on the hours/amount cells.

### 12. Footer floating mid-page on the last page
**Symptom**: the closing footer band sits halfway down the last page when content is short.
**Fix**: wrap the last section + footer in a full-height flex column and push the footer down:
```css
.lastpage        { min-height: 297mm; display: flex; flex-direction: column; page-break-before: always; }
.lastpage .footer{ margin-top: auto; }   /* pinned to the page bottom */
```
(For a footer that must appear on EVERY page, use the `position: fixed` approach above instead.)

### 13. Orphan section heading at a page bottom
**Symptom**: a section heading lands at the bottom of a page, its content on the next.
**Fix**: keep panels that fit on one page together, and never orphan headings:
```css
.panel.keep   { page-break-inside: avoid; }   /* on sections that fit one page */
h1,h2,h3,
.eyebrow      { page-break-after: avoid; }
```
**Do NOT** put `page-break-inside: avoid` on a long table panel — it must split row-by-row
(`tr { page-break-inside: avoid }` keeps individual rows intact).

### 14b. Inconsistent top/bottom margin across pages
**Symptom**: page 1 of content starts lower than continuation pages (the table/panel resumes
flush at the very top edge on page 2+), and panel fractures look cut.
**Cause**: vertical margins set as **block padding** (`.wrapper{padding-top:15mm}`) only apply at
the block's start, NOT on every page it spans.
**Fix**: put per-page margins on `@page`, and keep the cover full-bleed:
```css
@page       { size:A4; margin:14mm 0; }   /* consistent top/bottom on EVERY page */
@page:first { margin:0; }                  /* full-bleed cover */
```
Render so CSS @page wins: `page.pdf({ preferCSSPageSize:true, printBackground:true })` and
**do NOT pass a `margin` option** — a `margin` in `page.pdf()` (e.g. `--margin=0`) overrides
@page and reintroduces the inconsistency. A full-height last page (`min-height:269mm` = 297−2×14)
still pins its footer to the bottom of the printable area.

### 14. Section numbering consistency
If sections are numbered 02/03/04, the first is **01** — don't replace it with a word
("Summary") and skip the number. Keep `01 · 02 · 03 · 04`.

### 15. Table overflows its container (long unbreakable tokens) → columns look misaligned
**Symptom**: a table's right edge pushes past its panel/page margin (numeric column ends ~2–3 mm
from the paper edge instead of at the content margin), and different tables overflow by different
amounts so they look "misaligned" — even though header and body share the same columns.
**Cause**: `table-layout: auto` (the default) sizes columns to content, and `width: 100%` is a
**minimum, not a maximum**. A long token with no break opportunity — e.g. a technical identifier
`indaws_server_action_mass_edit_extend` (underscores do **not** create soft-wrap points), a URL, or
a long `code` span — sets a large min-content width, so the table grows wider than its container and
bleeds toward the page edge. Monospace `code`/module columns are the usual trigger.
**Fix**: pin the layout and let long tokens wrap.
```css
table      { table-layout: fixed; }              /* columns obey declared widths, can't overflow */
td.mod, td.func, .tdesc, td small { overflow-wrap: anywhere; word-break: break-word; }
td.h, th.h { white-space: nowrap; }              /* keep short numbers on one line (see #11) */
```
```html
<table><colgroup><col style="width:25%"><col style="width:21%"><col style="width:12%">
<col style="width:36%"><col style="width:6%"></colgroup>…</table>
```
Give **every** table an explicit `<colgroup>` (widths summing to 100%). With `table-layout:fixed`
the `<colgroup>` is authoritative, so thead/tbody/tfoot columns line up pixel-perfect and every
table ends at the same right margin. Verify by measuring, not by eye: extract word bboxes
(`page.get_text("words")` via PyMuPDF) and check the last column's right edge equals the panel
content margin (e.g. ~517 pt on A4 with 16 mm page + 9 mm panel padding), not ~588 pt (near the
595 pt paper edge).

---

## Complete Script

See `scripts/html-to-pdf.js` for the production-ready script with all options, error handling, and font loading.
