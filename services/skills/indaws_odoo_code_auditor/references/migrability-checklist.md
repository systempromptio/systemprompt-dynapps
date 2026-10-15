# Migrability Analysis Checklist

Assess how easy or difficult it will be to migrate the code to newer Odoo versions.

## Key Assessment Criteria

### Deprecated API Usage
Check for deprecated decorators, methods, fields:
- `@api.one`, `@api.returns` (deprecated in 13+, **removed in 19.0**)
- `@api.model` instead of `@api.model_create_multi` for create() (Odoo 14+)
- `name_get()` (deprecated SaaS 16.4, **removed from core in 18.0** — use `_compute_display_name` from v18)
- `group_operator` (deprecated 18.0, use `aggregator`)
- `record._cr`, `record._context`, `record._uid` (deprecated 19.0, use `self.env.*`)
- Old-style fields (fields.function, etc.)
- Binary fields instead of Image fields (Odoo 14+)

### Version-Specific Patterns
Identify code tied to specific Odoo versions:
- Account move 'type' field (changed to 'move_type' in Odoo 15+)
- JavaScript jQuery/legacy widgets (OWL mandatory in Odoo 16+)
- Python 2 compatibility code (removed in Odoo 14+)
- `res.groups.category_id` → `privilege_id` via `res.groups.privilege` (Odoo 19)
- `res.groups.users` → `user_ids` (Odoo 19)
- `res.users.groups_id` → `group_ids` (Odoo 19)
- `base_import.Import` → `Base_ImportImport` (Odoo 19)
- Search view `<group>` wrapper removed (Odoo 19)
- Kanban `kanban-box` → `card` (Odoo 19)
- `ir.cron.numbercall` **y `doall`** removed (**Odoo 18**, no 19 — medido 2026-08-17: 0 ocurrencias
  en CE18 y CE19; existen en v17 `odoo/addons/base/models/ir_cron.py:67`, `ir_cron`). Un módulo que salte v17→v18 ya falla ahí
- Frontend RPC service removed (Odoo 19)

### Hard Dependencies on Core Changes
Code that relies on modified core modules:
- Direct core model modifications without proper inheritance
- Monkey patching core functionality

### Custom Reimplementations
Code that duplicates standard features (harder to migrate):
- Custom workflows that should use standard automation
- Custom fields that exist in standard

### Database Schema Dependencies
Direct SQL that may break with schema changes:
- Direct table access instead of ORM
- Hardcoded table/column names

## Migration Complexity Score

Calculate 0-10 score based on:
- Number of deprecated patterns found
- Code coupling to specific Odoo version
- Amount of custom vs standard code
- Complexity of modifications

## Output for Migrability

- Migration difficulty score (0-10, where 10 is extremely difficult)
- List of blocking issues for migration
- Estimated effort for migration (Low/Medium/High/Very High)
- Recommended migration path
- Risk assessment
