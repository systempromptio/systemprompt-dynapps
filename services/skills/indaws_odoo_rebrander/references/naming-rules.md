# Naming & Refactoring Rules

## File and Directory Naming
- CE modules: `/dynapps/dynapps-odoo-ce/<new_module>`
- EE modules: `/dynapps/dynapps-odoo-ee/<new_module>`

## Code Refactoring Scope
Search and replace in: XML Views, Python Code, Security CSV, Translations (.po)

## Naming Cleanup (Critical)

### Database Models
**ELIMINATE `indaws` from model names. DO NOT use `dynapps` unless strictly necessary.**
Use functional/contextual names:
- `indaws.stock.picking` → `stock.process.picking`
- NOT `dynapps.stock.picking`

### Views and XML IDs
**ELIMINATE `indaws`. DO NOT use `dynapps`. Use descriptive names.**
- `view_indaws_partner_form` → `view_partner_form_stock_extension`

### Literals, Licenses, Names
**ELIMINATE all forms of `indaws`** (inDAWs, Indaws, inDAWS).
Replace with `Dynapps` (titles) or `dynapps` (code).

## DO
- Always analyze module completely before refactoring
- Put migration scripts INSIDE the module: `<module>/migrations/<version>/pre-migrate.py` etc.
- Pick the rename helper that fits the env: `upgrade-util` (EE), `openupgradelib` (CE-portable, default), or manual SQL (fallback)
- Trigger migrations by bumping `version` in `__manifest__.py` + `odoo-bin -u <module>`
- Test migration scripts in staging before production
- Document complex migrations in script comments
- Keep old module installed until migration verified
- Use descriptive, functional names for models/views

## DON'T
- Never delete old modules before successful migration
- Never use direct SQL without checking table/column existence (use `table_exists`/`column_exists`)
- Never assume `from odoo.upgrade import util` works on CE — it does NOT (use openupgradelib there)
- Never invent a central `/dynapps/upgrade/upgrade.py` orchestrator or `click-odoo --load` — that is not how Odoo runs migrations
- Never leave `indaws` references in refactored code
- Never use `dynapps` in model names unless absolutely necessary
- Never skip referential integrity checks

## Validation Checklist
- [ ] New module directory in correct target path
- [ ] All `indaws` references replaced
- [ ] Model names cleaned (no `indaws`, minimal `dynapps`)
- [ ] XML IDs follow new convention
- [ ] `__manifest__.py` updated (incl. bumped `version`)
- [ ] Migration scripts created in `<module>/migrations/<version>/` (pre/post/end as needed)
- [ ] Rename helper chosen per env (upgrade-util / openupgradelib / manual SQL)
- [ ] Python/XML/CSV syntax validated

## Success Criteria
- Refactored module installs without errors
- No `indaws` references remain
- `odoo-bin -u <module>` runs the migration; module + data renamed correctly
- Referential integrity maintained
- Views, actions, menus, security all work
- Translations updated

## Examples

### Simple: Module Rename (`indaws_api`)
1. Analyze → 2. Plan `dynapps_api` → 3. Copy + replace → 4. Bump `version` + add `migrations/<version>/pre-migrate.py` with module rename

### Complex: Custom Models (`indaws_inventory`)
1. Analyze (custom `indaws.stock.picking`) → 2. Rename to functional (`stock.process.picking`) → 3. Refactor + pre-migrate rename_model/rename_table/rename_xmlid + field mapping → 4. Document complex migrations
