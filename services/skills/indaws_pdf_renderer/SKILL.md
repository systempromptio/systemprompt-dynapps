---
name: indaws-pdf-renderer
description: >
  Shared PDF generation engine using Playwright (Chromium headless). Use this skill when ANY skill
  needs to convert HTML to PDF — reports, proposals, documentation, invoices, presentations.
  This is the ONLY approved method for PDF generation in the Dynapps Spain AI ecosystem.
  Triggers: "generate PDF", "HTML to PDF", "render PDF", "export as PDF", "crear PDF",
  "generar informe PDF". Also use when another skill (dynapps-brand,
  odoo-migration-pitch, indaws-core-documenter) needs PDF output.
version: 1.0.3
effort: low
model: sonnet
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep]
---

# PDF Renderer v1.0

Single source of truth for PDF generation across all skills. **Playwright (Chromium headless) is the only engine.**

## Why Playwright, not WeasyPrint

| Aspect | Playwright | WeasyPrint |
|--------|-----------|------------|
| CSS fidelity | Full — what you see in Chrome is what you get | CSS 2.1 + partial CSS3. No grid, no variables, no 100vh |
| Workarounds needed | Almost none | Many (limited CSS requires defensive coding) |
| Dependencies | `npx playwright install chromium` (already installed) | pip + cairo + pango + gdk-pixbuf (problematic on macOS) |
| Glassmorphism/bento | Renders natively | Must simplify layouts |
| JS dynamic content | Yes (charts, animations) | No |
| Speed | ~3-5s per page | ~1-2s per page |

The 2-3 seconds of extra time do not justify maintaining two engines with incompatible CSS.

## How to generate a PDF

### Step 1: Create the HTML

Start from the Dynapps brand report template:
- `dynapps-brand/assets/templates/migration-report-template.html`

Replace placeholders (`{{DOCUMENT_TITLE}}`, `{{CONTENT}}`, `{{YEAR}}`), add your content.

### Step 2: Add print CSS

Add this inside `<style>` or a `<link>` stylesheet:

```css
@media print {
    /* Force background colors in print */
    body {
        -webkit-print-color-adjust: exact;
        print-color-adjust: exact;
    }

    /* Page size and margins */
    @page {
        size: A4;
        margin: 25mm;
    }

    /* First page: no margin (full-bleed cover) */
    @page :first {
        margin: 0;
    }

    /* Page breaks */
    h1 { page-break-before: always; }
    h1:first-of-type { page-break-before: avoid; }
    .no-break { page-break-inside: avoid; }
    .page-break { page-break-before: always; }

    /* Keep tables together */
    table, figure, .card { page-break-inside: avoid; }
    tr { page-break-inside: avoid; }

    /* Fixed header/footer */
    .print-header {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        height: 15mm;
    }
    .print-footer {
        position: fixed;
        bottom: 0;
        left: 0;
        right: 0;
        height: 15mm;
    }

    /* Hide non-print elements */
    .no-print, nav, .sidebar { display: none !important; }
}
```

### Step 3: Render with Playwright

Use the bundled script:

```bash
node /path/to/pdf-renderer/scripts/html-to-pdf.js input.html output.pdf
```

For inline usage in other skills, see `references/playwright-pdf-guide.md`.

### Step 4: Verify

Check: background colors render, page breaks correct, tables intact, fonts embedded, images loaded (absolute paths or data URIs).

## When brand skills call this skill

The brand skill (`dynapps-brand`) owns the **visual design** (colors, typography, cover layout). This skill owns the **technical rendering** (Playwright setup, page.pdf() options, CSS print rules, gotchas).

## GOTCHAS
1. **Playwright must be installed with browsers** — `npx playwright install chromium` is required on first use.
2. **CSS @page size must match expected output** — A4 is default but legal/letter sizes need explicit declaration.
3. **Web fonts need network access or local install** — Playwright in headless mode may not load Google Fonts. Embed fonts.
4. **Print CSS differs from screen CSS** — `display: flex` behaves differently in print. Test with print preview.
5. **PDF metadata (title, author) requires Playwright options** — set via `page.pdf({displayHeaderFooter: true})`.

## Delegates to
None — this is a leaf skill.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| dynapps-brand | common | PDF report rendering |
| migration-estimator | migration-lab | Render the migration-estimate PDF (+ page-break SSOT) |

For shared ecosystem patterns, see `common:references/shared/`.

## Reference

For detailed print CSS rules, page.pdf() options, and common gotchas, see `references/playwright-pdf-guide.md`.
