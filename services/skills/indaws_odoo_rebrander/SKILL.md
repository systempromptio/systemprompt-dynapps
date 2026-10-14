---
name: indaws-odoo-rebrander
description: Use this skill when the user needs to refactor and migrate Odoo modules from Indaws branding to Dynapps branding. Activates for tasks involving module renaming, code refactoring (models, views, XML IDs, translations), data migration script creation, and ensuring referential integrity during brand transitions. Handles complete rebranding workflow from analysis to migration execution.
version: 1.0.4
allowed-tools: [Bash, Read, Write, Glob, Grep, Edit]
effort: medium
model: sonnet
---

# Odoo Rebrander Skill

Manages the complete rebranding process from Indaws to Dynapps: code refactoring, naming cleanup, and data migration script generation. Ensures referential integrity and smooth transition between brand identities.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Scan module components | sonnet | Delegate to scanner |
| Plan rename strategy | **opus** | XML ID, deps, translations |
| Execute rename | **opus** | Code transformation |
| Generate migration script | **opus** | Data migration |

## References (read on demand)

| Ref | When to read |
|-----|-------------|
| [references/workflow-steps.md](references/workflow-steps.md) | **Always** — Full 3-step workflow (analyze → refactor → migrate) + the real migration mechanism (in-module `migrations/<version>/` scripts, rename helpers) |
| [references/naming-rules.md](references/naming-rules.md) | **Always** — Naming cleanup rules, DO/DON'T, validation checklist, examples |

## Project Context

- **Legacy:** `/dynapps/indaws-odoo-ce` and `/dynapps/indaws-odoo-ee`
- **Target:** `/dynapps/dynapps-odoo-ce` and `/dynapps/dynapps-odoo-ee`
- **Migration Scripts:** live INSIDE each module → `<module>/migrations/<version>/{pre,post,end}-migrate.py` (no central orchestrator)
- **From:** Indaws Business Solutions → **To:** Dynapps

## Quick Start

1. Read [references/workflow-steps.md](references/workflow-steps.md)
2. STEP 1: **Scan module [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "scan module") — invoke odoo-module-scanner on {path}. Output: models, fields, views, security, controllers, JS inventory. Analyze module + propose plan [STOP — wait for confirmation]
3. STEP 2: Copy + refactor code (apply [naming-rules.md](references/naming-rules.md))
4. STEP 3: Create in-module migration scripts (`<module>/migrations/<version>/pre-migrate.py`). Pick rename helper per env — see workflow-steps.md STEP 3

## Migration Mechanism (read before generating scripts)

- Scripts live in `<module>/migrations/<version>/{pre,post,end}-migrate.py`; `MigrationManager` auto-discovers them. Signature: `def migrate(cr, version):` (`version` = installed version, string).
- Triggered by bumping `version` in `__manifest__.py` + `odoo-bin -u <module> -d <db>`. NOT click-odoo, NOT `--load`, NO central `upgrade.py`.
- Phases: model/table/column + xmlid renames → **pre-migrate**; data fixes → **post-migrate**; cross-module → **end-migrate**.
- Rename helpers — TWO supported paths (full detail + fallback in workflow-steps.md):
  - **`upgrade-util` installed (NOT an EE thing):** `from odoo.upgrade import util` → `util.rename_model/rename_xmlid/rename_module/rename_table/rename_field`. `odoo/upgrade-util` is a **PUBLIC** repo (`github.com/odoo/upgrade-util`, LGPL-3.0) that simply isn't bundled; `odoo/upgrade/` is a **mount point** whose `__path__` fills from `--upgrade-path` (`odoo/modules/module.py:154-171`, `initialize_sys_path`), not an empty dir. Installable on **Community** (clone + `--upgrade-path`, `pip`, or `requirements.txt` on Odoo.sh) — and worth it here, because it handles the **indirect references** a rename leaves behind.
  - **CE-portable (default):** OCA `from openupgradelib import openupgrade` → `openupgrade.rename_models/rename_xmlids/rename_tables/rename_fields`.
  - **Fallback no-deps:** `from odoo.tools.sql import rename_column, table_exists, column_exists` + manual UPDATEs on `ir_model`/`ir_model_fields`/`ir_model_data`/`ir_module_module`.

## Core Naming Rule

**ELIMINATE `indaws` from model names. DO NOT use `dynapps` in models unless strictly necessary.**
Use functional names: `indaws.stock.picking` → `stock.process.picking` (NOT `dynapps.stock.picking`)

## Prohibitions

- NEVER delete old modules before successful migration
- NEVER use direct SQL without checking table/column existence
- NEVER leave `indaws` references in refactored code
- NEVER skip referential integrity checks

## GOTCHAS
0. **Code stays in English** — rebranding renames identifiers/IDs but never translates them to another language; source code is always English. See `references/shared/coding-standards.md`.
1. **XML IDs contain the module name** — renaming module means updating ALL xml_id references.
2. **ir.model.data records persist after rename** — must create migration script to update module field.
3. **Translation files (.po) contain module name in paths** — regenerate, don't find-replace. Terms load from PO via `_load_module_terms`; there is no `ir.translation` model to patch (legacy ≤v15 only).
4. **Security groups reference module name** — ir.model.access.csv and ir.rule XML IDs must be updated.
5. **Depends in __manifest__.py must use new names** — if dependent modules also rebranded.

For shared ecosystem patterns, see common:references/shared/.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-module-scanner | common | Scan components before rebranding |
| dynapps-brand | common | Dynapps naming and branding rules |
| odoo-version-api-reference | common | Version-specific patterns/APIs if rebranding also upgrades version |

## Consumed by

None — standalone rebranding skill.

## Related Skills

- **odoo-module-scanner** — Analyze components before rebranding
- **odoo-version-api-reference** — Version-specific patterns if also upgrading version
- **dynapps-brand** — Target brand tokens and naming conventions

Rebranding is not just search-and-replace. It's an opportunity to clean technical debt while maintaining data integrity.
