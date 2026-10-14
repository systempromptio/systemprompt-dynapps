# Initial Setup - Questions, Credentials, Deployment Modes

## ROUND 1: Core choices (use AskUserQuestion with 4 questions)

```
Question 1 (header: "Type", single-select):
  - "Staging -> Production (Recommended)" - Copy full config from staging to fresh production DB
  - "Audit only" - Verify existing migration without writing
  - "Incremental sync" - Update production with latest staging changes

Question 2 (header: "Deploy mode", single-select):
  - "Local" - Odoo runs on THIS machine. Full access to psql, odoo-bin, filesystem
  - "Remote + SSH" - Remote server with SSH access
  - "Remote (API only)" - Remote server, XML-RPC only, no shell access

Question 3 (header: "Scope", multi-select):
  - "Configuration" - Company, accounting, warehouses, carriers, products config, POS, connectors
  - "Automations" - Server actions, automated actions, crons — all created INACTIVE
  - "Studio" - Custom models, fields, views, menus, ACLs
  - "Credentials & Security" - Copy real credentials + ir.rule, ir.model.access

Question 4 (header: "Tracking", single-select):
  - "Yes, create GO-LIVE tasks" - Create parent task + subtasks with checklists
  - "Yes, use existing tasks" - Update existing task by ID
  - "No tracking" - Just execute and report
```

If "Audit only" selected -> switch to audit-only mode (see references/audit-mode.md).

## ROUND 2: Credentials (ask as text after Round 1)

"Now I need the connection details. You can paste them all at once or say 'use memory':

**Staging:** URL, DB, user, API key
**Production:** URL, DB, user, API key
**Client name:** (for reports)
**If tracking:** Indaws URL, DB, user, API key, project ID"

If "use memory" -> read from MEMORY.md or migration_config.json.

After receiving credentials, **TEST BOTH CONNECTIONS immediately** via XML-RPC and report:
"Staging connected (uid=2, v17.0)" / "Production auth failed"

If "Local" mode, also ask for .cfg path and Odoo source path.

## ROUND 3: Repository access and code investigation

"To properly analyze modules, detect hardcoded IDs, check noupdate flags, and plan phases, I need access to the source code:

**Custom modules repo path:** `_____________` (e.g. ~/odoo17/src/client/addons-custom)
**Indaws Core repo path:** `_____________` (if v18+, e.g. ~/odoo18/src/indaws/indaws-odoo)
**OCA repos path:** `_____________` (e.g. ~/odoo17/src/OCA/)
**Odoo addons_path from .cfg:** `_____________` (or I'll read it from the .cfg file)"

With repo access, Claude can:
- Read `__manifest__.py` to understand dependencies and install order
- Read `models/*.py` to map custom models, fields, and computed fields
- Read `data/*.xml` to detect `noupdate` issues and understand initial data
- Read `views/*.xml` to understand UI customizations
- Grep for hardcoded IDs in Python code (browse(N), search([('id','=',N)]))
- Read `security/*.csv` to understand custom ACLs and groups
- Analyze server action code embedded in XML data files
- Understand the FULL picture BEFORE touching any database

**If no repo access** (remote API only): Claude works with XML-RPC only but CANNOT detect noupdate issues, hardcoded IDs in source code, or analyze module dependencies from manifests. Warn the user about this limitation.

## Save and confirm

Display summary table and ask "Proceed?" before saving to `scripts/migration_config.json`.

## Deployment mode reference

| Step | LOCAL | REMOTE+SSH | REMOTE (API only) |
|------|-------|------------|-------------------|
| Restore backup | `createdb` + `psql` | SSH: same | User runs script |
| `-u all` | `odoo-bin` direct | SSH: same | User runs script |
| Install modules | `odoo-bin -i` | SSH: same | `odoo-pilot install_module.sh` |
| import_studio.py | Direct (PYTHONPATH) | SSH: scp + run | User runs script |
| Password reset | `psql` direct | SSH: same | User runs script |
| Migration scripts | XML-RPC | XML-RPC | XML-RPC |
| Copy credentials | XML-RPC | XML-RPC | XML-RPC |
| Audit | XML-RPC | XML-RPC | XML-RPC |

**LOCAL**: Claude runs everything autonomously.
**REMOTE+SSH**: Claude runs server commands via SSH, XML-RPC from local.
**REMOTE (API only)**: Claude generates a **server preparation script** with all shell commands. User executes it on the server, confirms, then Claude continues with XML-RPC steps.

## Config persistence

Save confirmed answers to `scripts/migration_config.json`. This file persists across sessions. Always check if it exists before asking questions — if it does, show the summary and ask "Continue with this config?"
