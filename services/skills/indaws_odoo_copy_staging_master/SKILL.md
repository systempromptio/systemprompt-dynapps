---
name: indaws-odoo-copy-staging-master
version: 1.1.1
effort: high
model: opus
description: >-
  Migra la configuracion completa de un Odoo de staging a produccion por XML-RPC: analiza repos y modulos, importa Studio, migra config por fases, copia credenciales reales, datos extra (acciones automatizadas, crons, reglas) y audita. Usala cuando toque el paso a produccion o go-live, copiar/migrar configuracion de staging, o auditar/verificar una migracion ya hecha (modo audit-only). Triggers: "migrate configuration", "staging to production", "go-live migration", "audit migration", "/odoo-copy-staging-master".

---

# Odoo Config Migrator - Execution Instructions

You are the **autonomous migration executor**. You EXECUTE the full pipeline: analyze, generate scripts, run them, track progress, and report. You make decisions based on data and only pause for genuine ambiguity. Every action is logged for session resume.

## Workflow Summary

0. **DISCOVER** — Scan repos, classify by origin, detect hardcoded IDs and noupdate risks
   **Scan repos [SONNET SUBAGENT]:** Launch Agent(model: sonnet, description: "scan staging repos for migration"):
   - **prompt:** "Scan all Odoo modules in these staging repository paths: {list of repo paths}. For each module found (has __manifest__.py), return: tech name, version, depends list, and classify its origin as one of: base (ships with Odoo CE), enterprise (ships with Odoo EE), OCA (matches known OCA repo), custom (everything else). Also detect: hardcoded database IDs in Python code (browse(N), (4,N,0) patterns), noupdate=1 in XML data files, and Studio-generated fields (x_studio_* prefix). Return a JSON inventory grouped by origin classification."
1. **ANALYZE** — Compare staging vs production counts, identify gaps
2. **PREPARE** — Install missing modules, import Studio (REPLAY approach), pre-migration DB fixes
3. **GENERATE** — Build phase plan from analysis, generate per-phase scripts, get user approval (MANDATORY)
4. **EXECUTE** — Run phases with pre-flight checks, dry-run option, auto-correction, per-phase validation
5. **CREDENTIALS** — Copy real credentials, verify 0 PLACEHOLDERs
6. **EXTRA DATA** — Migrate taxes, rules, automations (inactive), sequences, mail servers, payment providers
7. **VIEWS** — Migrate Studio views (2-pass), custom views, website content (COW key matching)
8. **POST-MIGRATION** — Deactivate automations, clear asset bundles, scan staging URLs
9. **AUDIT** — Full comparison + custom fields audit + smoke tests
10. **REPORT** — Summary, pending items, go-live runbook, UAT checklist, integration freeze window

**Audit-only mode**: If user says "audit migration" or "verify migration", skip steps 0-8 and run comparison only. See `references/audit-mode.md`.

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.
See `references/shared/field-mapping-syntax.md` for field types and command tuples.

## Key Rules (NEVER / ALWAYS)

- **NEVER** match by staging ID — always use name, code, xml_id, key
- **NEVER** copy `database.is_neutralized` from staging — es lo único que impide que un cron desactivado se reactive por efecto lateral (`ir_cron.toggle`, `odoo/addons/base/models/ir_cron.py:638`, `ir_cron.toggle` v18 / `:725` v19); detalle en `references/critical-rules-troubleshooting.md`
- **NEVER** execute writes without explicit user approval of the migration plan
- **ALWAYS** use `allow_none=True` on XML-RPC ServerProxy
- **ALWAYS** filter fields via `fields_get` before create/write (safe_create pattern)
- **ALWAYS** create automations/crons with `active=False`
- **ALWAYS** import Studio BEFORE migration (models -> inverse m2o -> fields -> ACLs -> views)
- **ALWAYS** request repo access before STEP 0 (needed for noupdate/hardcoded ID detection)
- **ALWAYS** migrate translations: v17+ use `context={'lang': False}` for raw JSONB dict

## Top Gotchas

1. **Studio = REPLAY, not copy** — Replay creation sequence resolving all references by natural key
2. **Carriers need product_id** — Create a "Delivery" service product as fallback
3. **Sendcloud carriers** — Force `delivery_type='fixed'`, set credentials after
4. **ftp.instance** matches by `host`, not `name` (field doesn't exist)
5. **prestashop.instance** matches by `url`, not `name`
6. **ir.actions.server** — Read in batches of 10 (singleton computed field bug)
7. **product.product/res.users Studio fields** — Delegated to product.template/res.partner
8. **one2many Studio fields** — Need inverse m2o created FIRST in target model
9. **Website views** — Match by `key` field (COW system), never by ID
10. **Clear asset bundles** after module install or view migration (`DELETE FROM ir_attachment WHERE url LIKE '/web/assets/%'`)
11. **Hardcoded IDs in server action code** — Must scan and remap after migration (browse(N), (4,N))
12. **noupdate=0 data XML** — Gets overwritten on `-u all`, needs SYNC step

## Resume Mode

On invocation, ALWAYS check for `scripts/migration_config.json` and `scripts/logs/migration_progress.json`. If found, reconstruct context from all persistent files (config, plan, progress, narrative, error_patterns, pending_decisions, id_mapping). Skip completed steps, re-run failed/in-progress. NEVER ask for credentials again.

## GOTCHAS
1. **XML IDs change between staging and production** — always remap by external_id, never by database ID.
2. **noupdate=1 records skip on module update** — you must force-write them via API, not module install.
3. **Studio customizations must be replayed, not copied** — use the REPLAY approach, not direct record copy.
4. **Credentials (SMTP, payment providers) are environment-specific** — never copy them blindly from staging.
5. **ir.config_parameter contains secrets** — filter out database_uuid, web.base.url, and auth tokens.
6. **Automated actions with cron may fire during migration** — disable all crons before starting, re-enable after.
7. **Views with priority conflicts cause silent failures** — audit view priorities after migration.

## Agent Routing
| Phase | Model | Why |
|-------|-------|-----|
| Discovery and scanning | sonnet | Mechanical inventory |
| Migration planning and decisions | **opus** | Complex trade-offs |
| Execute migration steps | sonnet | Script execution |
| Audit and validation | sonnet | Mechanical checks |
| Error resolution | **opus** | Requires diagnosis |

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | API operations for staging/production |
| odoo-module-scanner | common | Module inventory and classification |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| migration-closeout | migration-lab | Go-live / copia de configuración a producción al cerrar la migración |

See DEPENDENCIES.md for other cross-suite consumers.

## REFERENCES

- `references/audit-mode.md` — Audit-only mode: comparison tables, checks, report format
- `references/initial-setup.md` — Questions, credentials, deployment modes, config persistence
- `references/progress-tracking.md` — Prerequisites, Indaws task creation, checklists, logging
- `references/step-0-1-discovery-analysis.md` — Repo scanning, module classification, noupdate detection, staging comparison
- `references/step-2-3-prepare-generate.md` — Module install, Studio REPLAY strategy, phase plan generation, user approval gate
- `references/step-4-execution.md` — Pre-flight checks, dry-run, auto-correction patterns, decision framework
- `references/step-4.5-4.6-sync-remap.md` — XML data sync after -u all, hardcoded ID detection and remapping
- `references/step-5-6-7-credentials-extra-views.md` — Credential copy, extra data migration, Studio/custom/website view migration
- `references/step-8-9-10-post-audit-report.md` — Post-migration cleanup, audit, smoke tests, final report, go-live runbook
- `references/resume-mode.md` — Session resume logic, context reconstruction, compaction handling
- `references/scripts-reference.md` — Script inventory and odoo-pilot scripts for Indaws tracking
- `references/critical-rules-troubleshooting.md` — 24 critical rules from real migrations, troubleshooting table
- `references/migration-scope.md` — What gets migrated, what doesn't, data classification framework, NEVER_MIGRATE blacklist
- `references/lessons-learned.md` — Version differences, model particularities, field mapping quirks
- `references/beyond-technical.md` — Hardcoded URLs, go-live preparation, non-technical migration aspects
