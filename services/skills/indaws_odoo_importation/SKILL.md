---
name: indaws-odoo-importation
description: Use this skill when the user asks to import mass data into Odoo from XLSX or CSV files, perform bulk imports, migrate data to Odoo, or use the load method for batch operations. Activates for tasks involving data normalization, external ID generation, batch processing, or importing contacts, products, invoices, or any Odoo model records in bulk. This is the OFFICIAL Odoo import method.
version: 1.0.1
effort: medium
model: sonnet
allowed-tools: [Bash, Read, Write]
---

# Odoo Mass Import Skill - load Method

## Fundamental Rules

- **ALWAYS** use the `load` method for mass imports
- **NEVER** use individual `create`/`write` unless explicitly requested by the user
- **ALWAYS** include external IDs (`'id'` field) in every import to prevent duplicates
- **ALWAYS** use XML IDs for relations (`field_name/id`) instead of numeric IDs
- **NEVER** exceed 1000 records per batch
- **ALWAYS** normalize data before importing (lowercase, no accents, trim spaces)
- **ALWAYS** test with 10-20 records first before full import

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.
See `references/shared/field-mapping-syntax.md` for field types and command tuples.

## Workflow

```
1. READ ORIGINAL FILE (XLSX/CSV)
   ↓
2. CONNECT TO ODOO (authenticate, get uid)
   ↓
3. GET ODOO CATALOGS (countries, states, etc.)
   ↓
4. NORMALIZE DATA (fix countries, validate fields)
   ↓
5. MAP TO XML IDs (convert relations to XML IDs)
   ↓
6. GENERATE NORMALIZED FILE (for audit)
   ↓
7. PREPARE DATA FOR LOAD (fields + data arrays)
   ↓
8. IMPORT WITH LOAD METHOD IN BATCHES
   ↓
9. REPORT RESULTS (total imported, errors)
```

## Quick Reference: load Method

```python
# fields: list of field names
fields = ['id', 'name', 'email', 'country_id/id']

# data: list of lists (values in same order as fields)
data = [
    ['__import__.contact_1', 'John Doe', 'john@email.com', 'base.us'],
    ['__import__.contact_2', 'Jane Smith', 'jane@email.com', 'base.uk'],
]

result = models.execute_kw(
    DB_NAME, uid, API_KEY,
    'res.partner', 'load',
    [fields, data], {}
)
# result = {'ids': [...], 'messages': [...]}
```

## External ID Format

```
__import__.{unique_identifier}
```

Prefer generating from record data: `__import__.contact_{normalized_name}_{index}`

## Field Type Syntax

| Field Type | Syntax | Example |
|------------|--------|---------|
| Simple | `'field_name'` | `'name'`, `'email'` |
| Many2one | `'field_name/id'` | `'country_id/id'` |
| Many2many | `'field_name/id'` | `'category_id/id'` |
| External ID | `'id'` | `'id'` (always include) |

## Gotchas

- `load` result with empty `messages` = success; check `type == 'error'` to filter warnings
- Row numbers in errors are 0-indexed; add 2 for spreadsheet row (header + 0-index)
- Empty string `''` in data means "leave blank"; use `False` to clear a field
- Many2many with multiple values: comma-separated XML IDs in a single string
- Re-running import with same external IDs updates records (idempotent)

## Best Practices

1. Always save the normalized file for audit trail
2. Use XML IDs from Odoo catalogs (e.g., `base.es` for Spain) -- never hardcode numeric IDs
3. Handle errors per batch: log failed rows, continue with next batch
4. For relations, fetch Odoo catalogs first and use fuzzy matching to resolve names

## Agent Routing
| Phase | Model | Why |
|-------|-------|-----|
| Read source data (CSV/XLSX) | sonnet | Mechanical parsing |
| Map fields and normalize data | **opus** | Complex mapping decisions |
| Execute load() batches | sonnet | Script execution |
| Error handling and retry | **opus** | Diagnosis needed |

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | API connection for import operations |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| indaws-data-import | developer | Mass data import workflows |
| importation-data-assistant | consultant | Guided import sessions |

## References

| File | Content |
|------|---------|
| `references/load-method-api.md` | Full load method API, field syntax details, advantages vs create/write |
| `references/external-ids-and-xml-ids.md` | External ID generation strategies, common XML IDs, lookup functions |
| `references/batch-processing.md` | Batch import implementation with BATCH_SIZE=1000 |
| `references/error-handling.md` | Response structure, interpreting results, common errors table |
| `references/data-normalization.md` | Text normalization, fuzzy matching with difflib |
