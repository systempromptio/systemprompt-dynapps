---
name: indaws-data-import
description: Advanced skill for creating and managing mass data imports into Odoo using the indaws_data_import module. Use when the user needs to import large volumes of data from CSV, Excel, JSON, or external APIs; set up scheduled imports; create data collectors; configure SFTP connections; or implement complex data transformations. This skill covers template creation, field mapping, Python code integration, parsers, schedulers, and troubleshooting. For simple Odoo load method imports, use the 'odoo-importation' skill instead.
version: 1.1.5
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion, EnterPlanMode]
effort: medium
model: opus
---

# inDAWS Data Import Skill

**Module:** `indaws_data_import` (Odoo 14.0-19.0) -- Enterprise-grade mass data imports.

## ⚠️ Version awareness (v18 vs v19) — READ FIRST

The reference files below describe the **v14–v18** API. In **v19** the module was refactored onto a
new shared **`indaws_dataflow`** layer and the connection/variable models were **moved**:

- `indaws.import.connection` → **`indaws.dataflow.connection`** (the v18 model no longer exists)
- `indaws.import.variable` → **`indaws.dataflow.variable`**; connection params → **`indaws.dataflow.connection.param`**
- scheduler/template now `_inherit` `indaws.dataflow.*`; `connections_ids` comodel → `indaws.dataflow.connection`
- `md5_*` → `sha1_*`; `defer_compute` removed; `safe_eval` lost `nocopy`; depend on **`indaws_dataflow`**

**For any v19 work, read [references/v19-dataflow.md](references/v19-dataflow.md) FIRST** — generating
v18 XML on a v19 DB fails at install.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Analyze source data | sonnet | Parse CSV/Excel structure |
| Design import mapping | **opus** | Complex field mapping decisions |
| Generate code | **opus** | Python code stages |

## When to Use This Skill vs odoo-importation

| Use Case | This Skill | odoo-importation |
|----------|-----------|------------------|
| Simple CSV/Excel import | Overkill | Perfect |
| Transformations / Custom Python | Yes | No |
| External API / SFTP / Scheduled | Yes | No |
| Millions of records (chunks) | Yes | Slow |
| Complex matching rules | Yes | Basic only |

**RULE:** Any advanced feature -> this skill. Basic imports -> recommend `odoo-importation`.

## Module Dependencies

```python
'depends': ['indaws_base', 'queue_job']
# external_dependencies (python): v18 = pandas, dateutil
#                                 v19 = pandas, dateutil, openpyxl, requests, xlrd
# pip install pandas python-dateutil   (+ openpyxl requests xlrd on v19)
```

**CRITICAL:** Verify deps before creating templates. xlrd/openpyxl/requests are manifest
deps **only in v19**; on v18 install them manually only if your own code needs them.

## Reference Files

Read these on demand -- do NOT load all at once.

| Reference | When to Read |
|-----------|-------------|
| [v19-dataflow.md](references/v19-dataflow.md) | **FIRST, if target is Odoo 19** — connection/variable moved to `indaws.dataflow.*`, eval context, XML examples |
| [architecture.md](references/architecture.md) | Models, data flow (Connection->Variable->Collector), streaming (v18 model names) |
| [workflow.md](references/workflow.md) | Full 4-step workflow: gather info, select use case, execute, validate |
| [python-code-stages.md](references/python-code-stages.md) | 6 code stages, `cell_value` vs `value`, eval context variables |
| [python-code-context.md](references/python-code-context.md) | Full eval context details and examples |
| [match-system.md](references/match-system.md) | Deduplication: `match=True` + `match_line_ids` (both required) |
| [variables-functions.md](references/variables-functions.md) | Variables returning functions, reusable helpers |
| [xml-noupdate.md](references/xml-noupdate.md) | noupdate gotcha during development |
| [performance-settings.md](references/performance-settings.md) | chunksize, commit_every, defer_compute, context_code |
| [common-pitfalls.md](references/common-pitfalls.md) | Critical gotchas and how to avoid them |

## Use Case Guides

| # | Guide | Covers |
|---|-------|--------|
| 1 | [Simple CSV Import](use-cases/01-simple-csv-import.md) | Products/contacts from CSV, basic field mapping |
| 2 | Excel with Relations | Many2one/Many2many fields, relational syntax |
| 3 | Scheduled Import | Cron, automatic file import, email notifications |
| 4 | [API Data Collector](use-cases/04-api-data-collector.md) | REST API, OAuth, JSON transformation |
| 5 | SFTP Automated Import | SFTP connection, download, process, archive |
| 6 | Custom Parser | Pre-process files, fixed-width, normalization |
| 7 | Complex Transformations | Multi-stage Python, computed values, cross-record |
| 8 | Migration Project | Legacy system, multiple tables, external IDs |

## Quick Start

```python
# 1. Create template
template = env['indaws.import.template'].create({
    'name': 'Import Products from CSV',
    'model_id': env.ref('product.model_product_product').id,
    'file_type': 'csv',
    'csv_separator': ';',
    'header_row': 1,
})

# 2. Add field mappings
env['indaws.import.template.line'].create({
    'template_id': template.id,
    'names': 'Code',
    'field_id': env.ref('product.field_product_product__default_code').id,
    'sequence': 10,
})

# 3. Execute
import_record = env['indaws.import'].create({
    'template_id': template.id,
    'file': base64.b64encode(open('products.csv', 'rb').read()),
    'filename': 'products.csv',
})
import_record.action_import()
```

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | API connection |
| odoo-importation | common | load() method patterns |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| importation-data-assistant | consultant | Advanced import workflows |
| odoo-shell-scripting | developer | Bulk import from external CSV/Excel |

## GOTCHAS
0. **Python code stages in English** — the Python you write for transform/match stages (variables, comments) always in English regardless of client locale. See `references/shared/coding-standards.md`.
1. **Match system uses ir.model.fields** — if field doesn't exist in target, match silently fails.
2. **Python code stages run in safe_eval** — no imports, no file I/O, use provided env.
3. **noupdate=1 on XML records skips updates** — force write if data needs refreshing.
4. **Large CSV files must use batch mode** — >1000 rows without batching causes memory issues.
5. **Date formats vary by source** — always normalize to YYYY-MM-DD before import.
6. **`cell_value` not `value`** — field code reads `cell_value`, writes `value`. #1 bug source.
7. **Match needs BOTH** — `match=True` on lines AND `match_line_ids` records, or duplicates on every run. `match_field_ids` is computed — NEVER write it; use `match_line_ids`.
8. **Connection params** — injected into the variable's eval context, NOT the collector's. Variable output reaches the collector under `variable.name`.

See `references/shared/field-mapping-syntax.md` for field types and command tuples.

## Related Skills

- **odoo-pilot** (required): Test imports via API
- **odoo-importation** (alternative): Simple bulk imports via load · **odoo-code-auditor**: audit Python quality
- **indaws-data-export** (complementary): Export data from Odoo (counterpart)
