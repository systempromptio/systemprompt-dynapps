# QWeb Report Changes by Version (v14→v19)

For the complete migration guide with code examples, detection script, and all fixes, read:
`$SKILLS_ROOT/ai-claude-developer/skills/odoo-module-upgrader/fixes/qweb_report_fixes.md`

This file is the quick-reference summary. The upgrader fixes file has the full details.

## Breaking Changes Matrix

| # | Change | From | To | Severity |
|---|--------|------|----|----------|
| 1 | Bootstrap 4 → 5 (class names) | v15 | v16 | CRITICAL |
| 2 | `t-raw` → `t-out` | v15 | v16 | HIGH |
| 3 | Tax totals evolution (4 different APIs) | v14 | v18 | CRITICAL |
| 4 | `<h2>` title → `layout_document_title` variable | v17 | v18 | CRITICAL |
| 5 | `<field t-field>` → `<span t-field>` in reports | v16 | v17 | BLOCKING |
| 6 | External layout CSS classes renamed | v17 | v18 | HIGH |
| 7 | Report render API: `report_ref` 1st param | v15 | v16 | CRITICAL |
| 8 | `_post_pdf` → `_render_qweb_pdf_prepare_streams` | v15 | v16 | CRITICAL |
| 9 | Assets: XML inheritance → `__manifest__.py` | v14 | v15 | HIGH |
| 10 | PyPDF2 → `odoo.tools.pdf` | v16 | v17 | HIGH |
| 11 | Paper format: `css_margins` | v17 | v18 | MEDIUM |
| 12 | `is_pro_forma` → `is_proforma` | v16 | v17 | MEDIUM |
| 13 | Layout XML IDs renamed (background→striped, clean→bold) | v14 | v15 | HIGH |
| 14 | `_run_wkhtmltopdf` new `report_ref` param | v17 | v18 | HIGH |
| 15 | `company_details` field: `Html` → `Html(translate=True)` | v17 | v18 | LOW |
| 16 | `layout_background`: 'Geometric' → 'Demo logo' | v17 | v18 | MEDIUM |

## v19-Specific Changes

Based on v19 source analysis (verify against actual release):

| Change | Impact | Action |
|--------|--------|--------|
| Bootstrap 5 unchanged (same as v16-v18) | None | No action |
| QWeb directives unchanged | None | No action |
| `external_layout` structure stable | None | No action |
| `ir.actions.report` model stable | None | No action |
| Paper format: `css_margins` default behavior | Check | Verify margin rendering |

v19 report engine is largely stable compared to v18. The major breaking changes happened in v16 (Bootstrap, render API) and v18 (layout_document_title, CSS margins).

## Quick Decision: What Needs Fixing When Migrating

### From v14/v15 → v19 (MOST WORK)
- [ ] Bootstrap 4 → 5 classes (12+ replacements)
- [ ] `t-raw` → `t-out`
- [ ] Tax totals: inline → model-specific templates + currency
- [ ] `<field t-field>` → `<span t-field>`
- [ ] Title: `<h2>` → `layout_document_title`
- [ ] Render API: add `report_ref` first param
- [ ] `_post_pdf` → `_render_qweb_pdf_prepare_streams`
- [ ] Assets: XML template → `__manifest__.py`
- [ ] Layout XML IDs renamed
- [ ] PyPDF2 → `odoo.tools.pdf`
- [ ] Paper format: check `css_margins`

### From v16/v17 → v19 (MODERATE)
- [ ] Title: `<h2>` → `layout_document_title` (v18)
- [ ] Tax totals: add model-specific template + currency (v18)
- [ ] External layout CSS classes (v18)
- [ ] Paper format: check `css_margins` (v18)
- [ ] `_run_wkhtmltopdf` new param (v18)
- [ ] `is_pro_forma` → `is_proforma` (v17)

### From v18 → v19 (MINIMAL)
- [ ] Verify no structural changes in target templates
- [ ] Test PDF rendering
- [ ] Check paper format behavior

## Detection Script

The complete detection script is in the upgrader fixes file. Quick inline check:

```bash
# Run all checks for target v19
bash $SKILLS_ROOT/ai-claude-developer/skills/odoo-module-upgrader/fixes/detect_qweb_issues.sh /path/to/module 19
```

Or manually check the most common issues:

```bash
MODULE=/path/to/module

# Bootstrap 4 classes
grep -rn "text-right\|text-left\|ml-[0-9]\|mr-[0-9]" --include="*.xml" "$MODULE"

# t-raw
grep -rn "t-raw" --include="*.xml" "$MODULE"

# <field t-field> in reports
grep -rn "<field t-field" --include="*.xml" "$MODULE/report/"

# H2 xpath (breaks in v18+)
grep -rn 'expr.*//h2' --include="*.xml" "$MODULE"

# Old tax totals
grep -rn "tax_totals_json\|amount_by_group" --include="*.xml" --include="*.py" "$MODULE"

# Old render API
grep -rn "_post_pdf\|from PyPDF2" --include="*.py" "$MODULE"
```
