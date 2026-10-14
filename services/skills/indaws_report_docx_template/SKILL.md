---
name: indaws-report-docx-template
description: This skill should be used when the user asks to "create a DOCX template from an image", "generate a docx report template", "design a Word template for Odoo", or provides a screenshot/mockup of a document they want to convert into a docxtpl template for indaws_report_docx.
version: 2.2.2
effort: medium
model: sonnet
---

# DOCX Template Generator

Generate `docxtpl` templates for `indaws_report_docx` from a visual mockup.

## Workflow

### Phase 0: Setup

**0a. Source folder** -- Ask: "En que carpeta estan los documentos de referencia?" Scan for `*.{pdf,png,jpg,jpeg,webp,bmp,tiff}`. Store as `SOURCE_FILE`.
**0b. Output folder** -- Ask if output goes to same folder or different. Create `<dest>/output_docx/`. All generated files go there.
**0c. Model** -- Ask for the target Odoo model if not provided (`sale.order`, `account.move`, etc.).
**0d. Dependencies** -- Run `scripts/check_deps.py`. Required: `python-docx`. Conditional: `poppler-utils` (PDF), `Pillow` (color extraction).

### Phase 1: Analysis

**1a.** If PDF, convert with `scripts/pdf2img.py`. Multi-page: ask which pages matter.
**1b.** Read image with Read tool. Identify sections, dynamic fields, tables, typography, layout.
**1c. Color extraction (MANDATORY)** -- Run: `scripts/extract_colors.py <image>`. No bands = B&W. Logo color != header color.

### Phase 2: Field Mapping

**2a.** Map visual elements to Odoo field paths. Read target model source for relational fields.
**2b.** Identify `docx.variable` needs (complex formatting, computed values, conditionals).
**2c.** Check built-ins first: `monetary()`, `format_lang()`, `format_date()`, `pct()`, `Image()`, `Html2Docx()`, `Barcode()`. See `references/docxtpl-syntax.md`.
**2d.** Design variables with name, scope, type, python_code. See `references/docx-variables.md`.
**Present mapping + variables to user for confirmation before generating.**

### Phase 3: Template Generation

Write `generate_<name>.py` in `OUTPUT_DIR`. Execute to produce DOCX. See `references/report-layouts.md` for layout patterns.

### Phase 4: Configuration XML

Write `docx_config_<name>.xml` with `docx.variable`, `docx.template`, `ir.actions.report`, optional `docx.parameter` records.

### Phase 5: Variable Staging

Only when custom variables created. Ask where to save: staging local (append to `docx_variable_staging.xml`) or solo output.

### Phase 6: Validation

Checklist: colors from extract only, `{%tr %}` for table rows, `Image()` with mm, `{{ r Html2Docx() }}` for HTML, `monetary()`/`format_date()`/`format_lang()` for formatting, None-safe conditionals, `value` set on all variables.

## GOTCHAS

1. **Invented colors**: NEVER invent hex values. Only use colors from `extract_colors.py` output. No color bands detected = B&W document (white bg, gray borders, black text).
2. **Logo color applied to headers**: Logo colors do NOT imply header background colors. Ask when unsure.
3. **`{%p %}` inside tables**: Always use `{%tr %}` for table row loops. Using `{%p %}` causes broken table structure.
4. **Missing `r` prefix on RichText**: `{{ r Html2Docx() }}` requires the `r` prefix. Without it, the output is raw XML text.
5. **Variables without `value` set**: Every `docx.variable` MUST set `value`. Prefer functions: `def f(x): ...\nvalue = f`.
6. **Custom variable before checking built-ins**: Always check if a built-in function (`monetary()`, `format_date()`, etc.) already does what you need before creating a new variable.
7. **Auto-pushing git branches**: Never auto-push. Let user decide.
8. **`{%p %}` vs `{%tr %}` — scope matters**: `{%p %}` works at paragraph level (document body, conditionals, page-break loops). `{%tr %}` works at table-row level. A FOR loop that generates rows MUST use `{%tr %}`; using `{%p %}` inside a table body corrupts the XML structure silently.
9. **Page break between iterations — 3 paragraphs required**: The pattern is always: `{%p if not loop.last %}` paragraph → actual page-break run → `{%p endif %}` paragraph. Collapsing into fewer paragraphs causes docxtpl to render the tag as literal text.
10. **QWeb `t-field widget="monetary"` → `monetary()`**: In QWeb you get formatting for free; in docxtpl you must call `{{ monetary(doc.amount_total) }}`. Forgetting this outputs raw floats without currency symbol or thousand separators.
11. **QWeb `t-raw` (HTML) → `{{ r Html2Docx() }}`**: The `r` prefix is mandatory. Without it the XML tags appear as literal text in the document. The paragraph containing it must be a plain (non-styled) run.
12. **QWeb `t-foreach` migration**: `t-foreach` on `line_ids` inside a `<tr>` becomes three rows: its own `{%tr for line in doc.line_ids %}` row, the data row, and its own `{%tr endfor %}` row (see gotcha 24). Outside a table with page breaks, use `{%p for %}` / `{%p endfor %}`.
13. **None-safe all optional fields**: `{{ doc.partner_id.phone }}` renders the literal string `"False"` in the document when the field is empty on some records (Odoo represents an empty `Char` as `False`, and Jinja stringifies it). Always write `{{ doc.partner_id.phone or '' }}` for text fields and `{% if doc.partner_id.phone %}...{% endif %}` for conditional blocks. (The `UndefinedError` from #21 is a different scenario: the variable itself missing from the context.)
14. **LibreOffice ignores `cell.width`**: Setting `cell.width = Mm(x)` alone has no effect in LibreOffice. You MUST also set `<w:tblGrid>` via `set_table_grid(table, widths_mm)`. Without it, LibreOffice distributes columns equally regardless of your widths.
15. **LibreOffice ignores `autofit=True`**: `table.autofit = True` tells LibreOffice to override all widths. Always set `table.autofit = False` when using explicit column widths. Otherwise the tblGrid is silently ignored.
16. **Studio fields**: Fields created via Odoo Studio use an `x_studio_` prefix (e.g., `doc.x_studio_attention`). Custom-coded fields use a plain `x_` prefix (e.g., `doc.x_attention_off`). Check the actual field name in the model; do not guess the prefix. Many2one Studio fields traverse normally: `doc.x_studio_field_id.name`.
17. **Nested tables**: When placing a table inside a cell, the outer cell's default padding (usually 108 twips ~2mm) adds unexpected margin around the inner table. Call `set_cell_padding(cell, 0, 0, 0, 0)` on the outer cell before inserting the nested table to reset it.
18. **`set_table_grid()` twip conversion**: Use `int(Mm(w_mm).emu / 635)` for exact EMU→twip conversion (635 EMU per twip). The shortcut `int(w * 56.7)` introduces rounding errors that accumulate across many columns and can cause the last column to be clipped.
19. **`{%p %}` tags must be the only content in their paragraph**: If there is any other text in the same paragraph as a `{%p for %}` or `{%p endif %}` tag, docxtpl will not recognize it as a block tag and will render it as literal text. Always put block tags in their own dedicated paragraph.
20. **Jinja filter `|int` for display-only truncation**: `{{ doc.mrp_id.product_qty|int }}` removes the decimal part for display. Do NOT use it when passing values to `format_lang()` — `format_lang()` accepts floats and handles digits internally. Using `|int` first can hide decimal quantities.
21. **`doc` not `record` in indaws_report_docx templates**: The Jinja context variable is `doc`, NOT `record`. Using `{{ record.field }}` raises `jinja2.exceptions.UndefinedError: 'record' is undefined`. Source: `indaws_report_docx/models/ir_actions_report.py:368`, `IrActionsReport._render_docx_record` → `eval_ctx = {..., "doc": record}`. The variable `record` only exists inside `docx.variable` `python_code` as a function parameter name, not in template Jinja.
22. **`Image()` width must be smaller than the column**: If `Image(logo, width=X)` and the column is also `X` mm wide, LibreOffice clips the right edge of the image (the last few pixels are cut off). Always leave at least 2mm slack: `width = column_mm - 2` minimum. For logos, 4-6mm slack prevents edge clipping on all renderers.
23. **Auto-load `.docx` binary inside an Odoo module**: Use `<data noupdate="0">` (not `noupdate="1"`) for `docx.file.template` records so the binary is re-loaded on every module upgrade. Pattern: `<field name="file" type="base64" file="module_name/data/templates/file.docx"/>`. Place structural records (`docx.template`, `ir.actions.report`) in a separate `<data noupdate="1">` block.
24. **Every table cell needs a `<p>`; `{%tr%}` must be on its OWN row (docxtpl 0.16.7 AND 0.20)**: A "bare" cell (text without a paragraph) makes the export flatten/crash — every cell must contain at least one `<p>`. And `{%tr for%}`/`{%tr endfor%}` on the SAME physical row as the data raises `unknown tag 'tr'`/`endfor`; put them on separate tag-rows (above/below the data-row) or use `{%p%}`. Applies equally when the DOCX comes from a browser editor (SuperDoc). See `references/docxtpl-syntax.md` §Table Patterns.
24. **`tight()` everywhere collapses visual hierarchy**: Setting `space_before=0, space_after=0, line_spacing=1.0` on every paragraph produces a wall of text with no visual breathing room. Use `tight()` only for runs inside tables. For standalone paragraphs between sections use `spaced(p, before=0, after=4)` and insert explicit `add_spacer(doc, pt=8)` paragraphs between major sections.
25. **Label/value pairs need a 2-col table**: Writing bold label + normal value in a single paragraph (`p.add_run("Label:"); p.add_run(value)`) means values never align across rows. Use a 2-column borderless table with fixed column widths: labels in col 0, values in col 1. This is the only way to get consistent column alignment in LibreOffice.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Read target model fields | Bash inline |

## Consumed by

- **`report-designer-developer`** (developer) — delega aquí para **generar** la estructura docx limpia (python-docx) cuando la IA crea una plantilla DOCX; luego se edita/rellena en SuperDoc vía el módulo `indaws_docx_designer`.
- **`report-designer-author`** (consultant) — genera la plantilla DOCX base que el autor luego rellena/edita en el editor DOCX (SuperDoc).

Standalone si no. Used when creating DOCX report templates for `indaws_report_docx`.

## References

| File | Content | Load when |
|------|---------|-----------|
| `references/docxtpl-syntax.md` | Jinja2 syntax, loop levels, built-in functions, escaping | Phase 2-3 |
| `references/docx-variables.md` | Variable pipeline, 8 creation patterns, XML format, eval context | Phase 2d, 4 |
| `references/field-mapping.md` | Field paths for sale.order, account.move, stock.picking, purchase.order | Phase 2a |
| `references/report-layouts.md` | Page setup, typography helpers, layout patterns, color handling | Phase 3 |
| `references/shared/field-mapping-syntax.md` | Field path conventions for Odoo model traversal | Phase 2a |

## Scripts

| Script | When |
|--------|------|
| `scripts/check_deps.py` | Phase 0d |
| `scripts/pdf2img.py` | Phase 1a (PDF input) |
| `scripts/extract_colors.py` | Phase 1c (always) |
