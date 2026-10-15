# [MIG-V19] indaws_sale_rappel (Capa 4 — Sale)

**Modulo**: `indaws_sale_rappel` | **Version**: 18.0 → 19.0
**Horas planificadas**: 14-22h | **Prioridad**: Normal (independiente, no bloquea otros modulos)
**Etiquetas**: `migracion`, `v19`, `capa-4`

---

## Context

### What it does

Bridge module that connects `indaws_rappel` (rappel definition) with `sale.order`. It:
- Adds rappel agreement Many2one to `res.partner` (buyer-side)
- Computes accumulated sale volume per quarter on confirmed sales
- Triggers credit note generation when threshold is reached

Inheritance: `res.partner`, `sale.order`, `sale.order.line`. Adds 1 wizard. 4 views. 6 unit tests.

### Why it must be migrated

Module is required by 4 active customer deployments. It depends on `indaws_rappel` (already migrated to v19, layer 3) and unblocks `indaws_purchase_rappel` migration (layer 5). Without it, sale-side rappel calculation is unavailable in v19.

**Decision**: Migrate — business logic is unique (Indaws-specific), no equivalent in standard or Core.

## Dependencies

- **Depends on (indaws)**: `indaws_rappel` (migrated to v19), `indaws_sale` (migrated to v19)
- **Depends on (Odoo)**: `sale_management`, `account`
- **Unblocks**: `indaws_purchase_rappel` (layer 5)

## Known v19 breaking changes

- `tree` view tag → `list` (verified in `$ODOO_SRC/odoo19/addons/sale/views/sale_order_views.xml`)
- `_read_group` signature change — multiple groupby returns dict instead of tuple
- `name_get()` deprecated → `_compute_display_name` already migrated in v18, OK

## Module inventory

| Component | Files | Detail |
|-----------|-------|--------|
| Python | 4 | 3 inherits (`res.partner`, `sale.order`, `sale.order.line`) + 1 wizard |
| XML | 4 | 3 view inherits + 1 wizard form |
| JS/OWL | 0 | — |
| Security | 1 | `ir.model.access.csv` — wizard access for sales group |
| Tests | 1 | `test_sale_rappel.py` — 6 unit tests |
| i18n | 2 | `es_ES.po`, `en_US.po` |

## Scope

### Gate 1 — Compatibility analysis

- Review all `_inherit` chains against v19 source
- Verify `tree` → `list` in 3 view inherits
- Check `_read_group` usage in `sale.order.line` compute
- Verify `account.move.reversal` API still matches

**Skills**: `odoo-module-scanner` to inventory components, `odoo-version-api-reference` for API changes v18→v19

### Gate 2 — Migration scripts

No schema changes detected. No renamed fields. **Document in commit**: "No migration scripts needed — only API and view tag changes."

### Gate 3 — Code

- Manifest: bump to `19.0.1.0.0`, verify `depends` (no rename in `indaws_rappel`)
- Python: refactor 1 `_read_group` call in `sale_order.py` (line 142) to v19 signature
- XML: 3 view inherits — replace `<tree>` with `<list>`, verify xpaths still match
- JS/OWL: N/A

**Skills**: `odoo-module-upgrader` for transformations, `indaws-core-developer` for layer 4 conventions

### Gate 4 — Tests

- 6 existing tests must pass green in v19
- Add 1 new test for the v19 `_read_group` signature
- Verify: `./odoo-bin -d testdb --test-tags /indaws_sale_rappel -i indaws_sale_rappel --stop-after-init 2>&1 | tail -30`

**Skills**: `odoo-test-generator` for v19 test patterns

## Acceptance criteria

- [ ] Compatibility analysis documented (file with breaking-changes table)
- [ ] No migration scripts needed — justification in commit message
- [ ] Installs cleanly in empty v19 DB
- [ ] Works in DB migrated from v18 (Upgrade Service path)
- [ ] All 7 tests pass green (6 existing + 1 new)
- [ ] No TODOs, no commented-out v18 code
- [ ] PR with description of API and view tag changes

**Final validation**: `quality-gate` — no Blockers; module is small, no `odoo-code-auditor` needed

## Resources

- Original v18 module: `$PROJECTS_ROOT/indawsodoo/18/indaws_sale_rappel`
- Odoo v19 source: `$ODOO_SRC/odoo19`
- Reference (already migrated, similar pattern): `$PROJECTS_ROOT/indawsodoo/19/indaws_purchase_stock_picking_invoice`

## Constraints

- Maintain backward compatibility of `res.partner.x_indaws_rappel_agreement_id` field name (clients have data)
- Do not refactor business logic during migration — only adapt to v19 APIs

---

## Output Odoo

**Title**: `[MIG-V19] indaws_sale_rappel (Capa 4 — Sale)`
**`x_studio_task_difficulty`**: `medium`
**`allocated_hours`**: 12 (factor IA aplicado sobre 18-20h estimadas)
**`tag_ids`**: `[[6, 0, [97, 83, 54]]]` (MIGR + inDAWS Core + Desarrollo)

Description en ingles, distribucion en campos Studio segun `references/odoo-task-field-mapping.md`.
