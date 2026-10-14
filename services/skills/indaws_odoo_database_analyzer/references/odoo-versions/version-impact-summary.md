# Version Impact Summary for Migration Estimation

> For complete API changes per version, see `odoo-version-api-reference/references/diffs/v{version}-diff.md`

---

## Effort Multipliers by Module Type

Use these multipliers on top of the version multiplier:

| Module type | Multiplier | Reason |
|-------------|-----------|--------|
| Python only (models, no views) | 1.0x | Only API changes |
| Python + XML views | 1.2x | View syntax changes |
| Python + XML + JS | 1.5x | Full stack migration |
| JS-heavy (portal, website) | 2.0x | OWL/RPC/widget migration |
| Theme (SCSS + snippets) | 1.8x | SCSS variables + snippet system changes |
| External API integration | 1.1x | Mostly unchanged |
| Report (QWeb PDF) | 1.1x | Minor template changes |

---

## How to use with source code

For precise per-module estimation, use source code symlinks (from `indaws-project-setup`):

```bash
# Diff a specific model between versions
diff base_v15/sale/models/sale_order.py base_v18/sale/models/sale_order.py

# Find deprecated patterns in client module
grep -rn "name_get\|group_operator\|@api.returns" /path/to/client/module/

# Count JS files needing migration
find /path/to/client/module/static -name "*.js" | xargs grep -l "odoo.define\|require(" | wc -l
```

This gives exact impact per module instead of generic estimates.
