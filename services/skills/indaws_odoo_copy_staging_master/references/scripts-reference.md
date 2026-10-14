# Scripts Reference

## Migration scripts

| Script | Purpose | Requires |
|--------|---------|----------|
| `scripts/utils.py` | Shared utilities (OdooRPC, IDMapper, safe_create/write/search_read) | - |
| `scripts/analyze.py` | Compare staging vs production counts + missing modules + Studio | XML-RPC |
| `scripts/import_studio.py` | Import Studio models, fields, inverse m2o, ACLs | Odoo ORM |
| `scripts/migrate_studio_views.py` | Migrate Studio views, actions, menus | XML-RPC |
| `scripts/copy_credentials.py` | Copy real credentials replacing PLACEHOLDERs | XML-RPC |
| `scripts/migrate_extra_data.py` | Migrate taxes, accounts, rules, templates, automations, crons, sequences, mail servers | XML-RPC |
| `scripts/audit.py` | Post-migration audit with PLACEHOLDER check | XML-RPC |
| `scripts/orchestrator.py` | Master script running all phases end-to-end | XML-RPC |
| `scripts/migration_config.json` | Persisted config with credentials, scope, task IDs | Created in STEP 2 |
| `scripts/logs/migration_progress.json` | Step-by-step progress for resume mode | Auto-generated |

## odoo-pilot scripts (for Indaws tracking)

| Script | Purpose |
|--------|---------|
| `auth.sh` | Authenticate with Odoo instance |
| `create_record.sh` | Create a record in any model |
| `update_record.sh` | Update a record by ID |
| `search_records.sh` | Search records with domain filter |
| `execute_method.sh` | Execute a method on a model |
| `delete_record.sh` | Delete a record by ID |

Location: resolved at runtime — see `references/shared/skill-path-resolution.md`. Never pin the marketplace name or version in the path.
