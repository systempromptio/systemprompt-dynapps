# Workflow Steps — Detailed Reference

## STEP 1: Reception and Analysis [STOP]

1. **Ask User:** "Which module(s) from `/dynapps/indaws-odoo-ce` or `/dynapps/indaws-odoo-ee` should we refactor today?"

2. **Analyze Source Code:**
   - Read `__manifest__.py` to understand dependencies
   - List all models defined in the module
   - Identify views, actions, menus, and security rules
   - Check for external integrations or data files

3. **Propose Refactoring Plan:**
   - New technical module name
   - List of models to rename (Old → New)
   - List of critical views to modify
   - Dependencies to update in manifest
   - Migration complexity assessment

4. **STOP HERE** — Wait for user confirmation.

---

## STEP 2: Code Refactoring

1. **Copy Module** to target directory with new folder name:
   - Source: `/dynapps/indaws-odoo-ce/<old_module>`
   - Target: `/dynapps/dynapps-odoo-ce/<new_module>`

2. **Apply Text Substitutions:**
   - Replace `indaws_` prefix in module name
   - Update model names (database tables)
   - Refactor XML IDs and external references
   - Clean up view inheritance xpath references
   - Update action and menu references
   - Fix security rules in CSV
   - Update translations in `.po` files

3. **Update `__manifest__.py`:**
   - New module name, description, summary
   - Clean dependencies (remove `indaws_*`, add `dynapps_*`)
   - Update author from "Indaws" to "Dynapps"

4. **Rename Python Files** if needed to match new naming conventions

5. **Validate Syntax:** Python imports, XML well-formed, CSV structure

---

## STEP 3: Migration Script Creation

Renaming a module/model/table on top of an existing database is a DATA MIGRATION.
Odoo runs migration scripts that live **inside the module**, discovered automatically
by `MigrationManager`. There is NO central orchestrator and NO `--load` mechanism.

### How Odoo discovers and runs migration scripts (the real mechanism)

1. Scripts live in the module:
   ```
   <new_module>/migrations/<version>/pre-migrate.py
   <new_module>/migrations/<version>/post-migrate.py
   <new_module>/migrations/<version>/end-migrate.py
   ```
   `<version>` is the manifest `version` you are bumping TO (e.g. `18.0.1.1.0`).
   File names can be anything matching `pre-*.py` / `post-*.py` / `end-*.py`.

2. They are triggered by **bumping the `version` field** in `__manifest__.py` and
   running an **update** of the module:
   ```bash
   odoo-bin -u <new_module> -d <database> -c <config> --stop-after-init
   ```
   Odoo only runs migrations when the installed version is LOWER than the manifest
   version. No click-odoo, no `--load`.

3. Each script exposes:
   ```python
   def migrate(cr, version):   # version = currently-INSTALLED version (string)
       ...
   ```

### Which phase does what
- **pre-migrate.py** — runs BEFORE the new schema is loaded. Do model/table/column
  renames and xmlid renames here, so the new ORM definitions bind to the existing
  (renamed) tables instead of creating empty new ones / dropping old ones.
- **post-migrate.py** — runs AFTER the new schema is loaded. Data fixes, recompute,
  re-point references.
- **end-migrate.py** — runs at the very end of the whole `-u` run (all modules).
  Use for cross-module cleanup that depends on other modules already migrated.

### Renaming helpers — TWO supported paths

**The current `util.rename_*` code only works if `upgrade-util` is installed** — which is a matter of
*installing it*, not of edition. Corrected 2026-08-07; this said `odoo/upgrade/` is EMPTY and that
`odoo.upgrade.util` does NOT exist in CE. Both are wrong:

- `odoo/upgrade/` is a **namespace mount point**, not an empty dir. Its only file in v19 is a
  `.gitkeep` whose contents are literally `(python namespace)`, and its `__path__` is appended at
  startup from `--upgrade-path` (`odoo19/odoo/modules/module.py:154-171`, `initialize_sys_path`, comment: *"hook odoo.upgrade
  on upgrade-path"*, with `base/maintenance/migrations` as the legacy default).
- **`odoo/upgrade-util` is PUBLIC**: `github.com/odoo/upgrade-util`, LGPL-3.0 (verified via API
  2026-08-07, `private: false`). Install it on **Community** by cloning + `--upgrade-path`, with `pip`,
  or via `requirements.txt` on Odoo.sh.

Why the distinction matters for a *rebrand* specifically: `util.rename_*` exists to fix the
**indirect references** a rename leaves behind (`ir_model_data`, `ir_model_fields`, stored domains,
views). That is the expensive, data-losing part. Believing it's unavailable pushes you to hand-rolled
SQL for the riskiest operation in the whole job.

Pick the path that matches the target environment:

#### Path A — `upgrade-util` available (installed; NOT limited to Enterprise)
```python
from odoo.upgrade import util

def migrate(cr, version):
    util.rename_module(cr, 'indaws_inventory', 'dynapps_inventory')
    util.rename_model(cr, 'indaws.stock.picking', 'stock.process.picking')
    util.rename_table(cr, 'indaws_stock_picking', 'stock_process_picking')
    util.rename_field(cr, 'stock.process.picking', 'indaws_ref', 'process_ref')
    util.rename_xmlid(cr,
        'indaws_inventory.view_picking_form',
        'dynapps_inventory.view_process_picking_form')
```
Only available when the deployment ships `upgrade-util` (typical in Odoo
Enterprise / official upgrade pipeline). Will `ImportError` on plain CE.

#### Path B — CE-portable → OCA `openupgradelib` (works on CE **and** EE)
External OCA dependency (`pip install openupgradelib`):
```python
from openupgradelib import openupgrade

def migrate(cr, version):
    openupgrade.rename_models(cr, [('indaws.stock.picking', 'stock.process.picking')])
    openupgrade.rename_tables(cr, [('indaws_stock_picking', 'stock_process_picking')])
    openupgrade.rename_xmlids(cr, [
        ('indaws_inventory.view_picking_form',
         'dynapps_inventory.view_process_picking_form'),
    ])
    # env is available via openupgrade.api.Environment(cr) when needed:
    env = openupgrade.api.Environment(cr, SUPERUSER_ID, {})
    openupgrade.rename_fields(env, [
        ('stock.process.picking', 'stock_process_picking', 'indaws_ref', 'process_ref'),
    ])
```
Note: `openupgradelib` has no `rename_module`; rename the module via the
`ir_module_module` UPDATE shown below (or rename the folder + let `-u` handle it).

#### Path C — Fallback, zero external deps (CE, stdlib + odoo.tools.sql)
```python
from odoo.tools.sql import rename_column, table_exists, column_exists

def migrate(cr, version):
    if table_exists(cr, 'indaws_stock_picking'):
        cr.execute("ALTER TABLE indaws_stock_picking RENAME TO stock_process_picking")
    # model record
    cr.execute("UPDATE ir_model SET model=%s WHERE model=%s",
               ('stock.process.picking', 'indaws.stock.picking'))
    # field model pointers
    cr.execute("UPDATE ir_model_fields SET model=%s WHERE model=%s",
               ('stock.process.picking', 'indaws.stock.picking'))
    # xmlids / module ownership
    cr.execute("UPDATE ir_model_data SET module=%s WHERE module=%s",
               ('dynapps_inventory', 'indaws_inventory'))
    cr.execute("UPDATE ir_module_module SET name=%s WHERE name=%s",
               ('dynapps_inventory', 'indaws_inventory'))
    if column_exists(cr, 'stock_process_picking', 'indaws_ref'):
        rename_column(cr, 'stock_process_picking', 'indaws_ref', 'process_ref')
```
Manual UPDATEs over `ir_model`, `ir_model_fields`, `ir_model_data`,
`ir_module_module`. Use only when neither upgrade-util nor openupgradelib is an
option — you own referential-integrity correctness.

### Decision: which path?
- **Odoo Enterprise / official upgrade pipeline** → Path A (`upgrade-util`).
- **Community (or anything portable across CE+EE)** → Path B (`openupgradelib`),
  the recommended default for rebranding CE modules.
- **Locked-down env, no extra deps allowed** → Path C (manual SQL).

### Execution & order
```bash
# 1. bump `version` in <new_module>/__manifest__.py
# 2. ensure migrations/<version>/pre-migrate.py exists
odoo-bin -u <new_module> -d <database> -c <config> --stop-after-init
```
Order: backup → bump version + add migration scripts → run `odoo-bin -u <module>`
→ verify (xmlids, tables, FK integrity) → keep old module until verified → test.
