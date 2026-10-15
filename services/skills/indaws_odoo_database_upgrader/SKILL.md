---
name: indaws-odoo-database-upgrader
description: >
  Migrates Odoo databases between versions using the official Upgrade Service (upgrade.odoo.com).
  Use when migrating an Odoo database between versions, preparing a dump for submission, analyzing
  installed modules, diagnosing Upgrade Service errors, or validating the post-upgrade result.
  Supports v8.0–v19.0, CE and EE. Orchestrates indaws_migration_kit + playbooks + replay 1-step (local submit→download→restore exec delegated to dynapps-db-upgrade-runner).
version: 2.4.10
effort: high
model: opus
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, WebFetch, WebSearch, AskUserQuestion]
---

# Odoo Database Upgrader

Migración Odoo end-to-end con `indaws_migration_kit` como motor + recetarios (playbooks). **El estudio previo del consultor dirige el flujo**, no la BD.

## Agent Routing

| Phase | Tipo | Model | Sub-agente |
|---|---|---|---|
| FASE 1 — parse estudio | B (lector) | sonnet | `estudio-parser` |
| FASE 3.1 — detect BD | B | sonnet | `db-detector` |
| FASE 3.2 — parse iterate | B | sonnet | `iter-parser` |
| FASE 3.2 — post-`suggest` | C (post-action) | sonnet | `suggest-validator` |
| FASE 3.3 — port módulo | A (delegación) | opus | `module-port` → `odoo-module-upgrader` |
| FASE 2 — fit Studio | A | opus | → `odoo-studio-migration` |
| FASE 3.3 / 4 — RPC validate | A | sonnet | → `odoo-pilot` |
| FASE 0.6 — bootstrap docker | A | sonnet | → `common-skills:docker-deployment` |
| FASE 4 — compare 8 niveles | B | sonnet | `compare-runner` |
| FASE 4 — smoke post-`run --apply` | C | sonnet | `smoke-validator` |
| FASE 5 — gate readiness | B | sonnet | `gate-checker` |

Prompts + contracts (JSON schemas) de cada agent: **`references/agent-tree.md`**.

For shared ecosystem patterns, see `common:references/shared/`.

## GOTCHAS (top 5)

1. **`arch_db` jsonb en v18** — `LIKE` no funciona; usar `jsonb_each_text` + `jsonb_object_agg`.
2. **`aim=production` 1 vez por contrato** — en odoo.sh SIEMPRE `test`, nunca `production` local.
3. **`migrated_modules/` fuera del workdir** — `docker-compose.yml` los monta como addons-path.
4. **No borrar `ir_model_data` sin preguntar** — usar `PreserveOrphanXmlid`.
5. **`tax_totals` (dict) reemplaza `tax_totals_json` (str)** — templates QWeb v15 rompen.

Lista completa (26 patrones, incl. P5 unowned_views + P6 disabled_inherited_views + gotchas 19-26 de la validación end-to-end Tecniman: update-list/`-u base`, mark_keep_to_install, NEUTRALIZE force-uninstall, módulos installed sin código, campos compute non-store en res.partner, FKs legacy, sale_type jsonb): `references/gotchas/index.md` — **READ FIRST**.

## Workflow

**Paso 0 — Localizar motor del kit** (una vez por entorno):

```bash
bash <SKILL_DIR>/scripts/locate_kit.sh
```

Salida `OK:<cli>` → `export INDAWS_MIGRATE_CLI=<cli>` y continuar. Cualquier otra salida (`NOT_CONFIGURED`, `PATH_MISSING`, `VENV_MISSING`) → **[STOP]** `AskUserQuestion` con 3 opciones (ruta existente / clonar default / clonar otra). Detalle: `references/kit-setup.md`. La ruta se guarda en el fichero de contexto global del agente (`AGENTS.md` / `CLAUDE.md`) para futuras sesiones.

**Paso 1 — Detectar sesión en curso**: `$INDAWS_MIGRATE_CLI playbook-session 2>/dev/null`. Si existe `migration_kit_*/` saluda con status + TODOs; si no, arranca FASE 0. Busca el workdir subiendo en árbol (estilo git).

**Fases**:

1. **[STOP] FASE 0** — destino (odoo.sh / indaws / custom) + credenciales · `references/phases/fase-0-deployment.md`
2. **[STOP] FASE 0.5** — repo workdir cliente (`odoo-migkit-<cliente>`): `AskUserQuestion` → existe / crear / solo local. Usar `indaws-migrate workdir-repo {clone|init+create-remote}` · `references/kit-setup.md` §"Convención repo del workdir"
3. **FASE 0.6** — restaurar dump (si falta compose: `Agent(model:sonnet) → docker-deployment`) · `references/restore.md`
4. **[STOP] FASE 1** — solicitar estudio previo → `Agent(model:sonnet, estudio-parser)` para extraer decisiones · `references/phases/fase-1-estudio.md`
5. **FASE 1.5** — `indaws-migrate init` + completar `from_version` + crear `$DUMP_PATH/original.dump` FUERA del workdir
6. **[STOP] FASE 2** — poblar `migration.yaml.modules` con `AskUserQuestion`. Para casos Studio: `Agent(model:opus) → odoo-studio-migration` · `references/phases/fase-2-planificacion.md`
7. **FASE 3** — `compile` → `Agent(model:sonnet, db-detector)` → `iterate` → `Agent(model:sonnet, iter-parser)` → `suggest` → `Agent(model:sonnet, suggest-validator)`. Port de módulos: `Agent(model:opus, module-port) → odoo-module-upgrader` · `references/phases/fase-3-compile-iterate.md`
8. **FASE 4** — `run --apply` → `Agent(model:sonnet, smoke-validator)` → `Agent(model:sonnet, compare-runner)` + RPC via `odoo-pilot` · `references/phases/fase-4-validacion.md`
9. **[STOP] FASE 5** — `Agent(model:sonnet, gate-checker)` → `ready` (8 gates) → `go-live` · `references/phases/fase-5-go-live.md`

## Reglas del agente
Categorías de autonomía (D1) + reglas globales (preservar v_prev, idempotencia, opt-in destructivos): `references/agent-rules.md`.

## FASES — leer el .md correspondiente

| FASE | Detalle | Comando clave |
|---|---|---|
| 0 — Deployment | `references/phases/fase-0-deployment.md` | (manual) |
| 0.5 — Workdir repo | `references/kit-setup.md` §"Convención repo workdir" | `workdir-repo {clone,init,create-remote}` |
| 0.6 — Restore dump | `references/restore.md` | `pg_restore` / `psql` |
| 1 — Estudio previo | `references/phases/fase-1-estudio.md` | `load-estudio` / `gen-estudio` |
| 1.5 — Init contextual | (en fase-1-estudio.md) | `indaws-migrate init` |
| 2 — Planificación módulos | `references/phases/fase-2-planificacion.md` | edición `migration.yaml` |
| 3 — Compile + iterate | `references/phases/fase-3-compile-iterate.md` | `compile`, `iterate`, `suggest` |
| 4 — Validación | `references/phases/fase-4-validacion.md` | `run --apply`, `compare` |
| 5 — Go-live | `references/phases/fase-5-go-live.md` | `ready`, `go-live` |

## Situación → playbook

Ver `references/playbooks/` (grupos `oca/`, `studio/`, `modules/`, `data/`, `go_live/`). El nombre de cada playbook es autodescriptivo: `neutralize`, `replace_with`, `custom_to_studio`, `custom_to_core`, `custom_to_stub`, `custom_port`, `to_drop`, `aeat_map_tax`, `preserve_orphan_xmlid`, `comparison`, `smoke_test`, `ready_checklist`.

## Comandos del kit

| Comando | Función |
|---|---|
| `indaws-migrate init` | Crea `migration_kit_<cliente>/` + `migration.yaml` |
| `indaws-migrate load-estudio` / `gen-estudio` | Parser y generador del estudio |
| `indaws-migrate detect` | Detecta patrones BD → propone steps |
| `indaws-migrate compile` | Genera `steps_generated` |
| `indaws-migrate inspect --plan migration.yaml` | Muestra pipeline + valida specs |
| `indaws-migrate run [--apply] [--resume]` | Ejecuta pipeline (dry-run o real) |
| `indaws-migrate iterate` | Loop restore → apply(pre) → update-list (`-u base`) → apply(pre_markers) → -u all → apply(post) → parse |
| `indaws-migrate suggest [--apply] [--commit]` | Lee última iter + propone fixes al yaml (auto-edit preservando comentarios) |
| `indaws-migrate modules list/order/set/blocker/history/report` | Tracking de módulos: status, blockers, orden topológico, historial |
| `indaws-migrate workdir-repo {init,clone,push,status,create-remote}` | Gestión repo cliente `odoo-migkit-<cliente>` (auto-push tras eventos clave con `--push`) |
| `indaws-migrate compare` | Comparison vs ref dump (8 niveles) |
| `indaws-migrate playbook-{init,sync,log,todo,session}` | `playbook.md` vivo |
| `indaws-migrate ack <gate>` / `ready` / `go-live` | Gates de salida + ejecución final |

## Delegates to

| Skill | Cuándo |
|---|---|
| `odoo-migration-parity` | FASE 4 paridad de capacidades origen↔migrado (además del `compare` de datos) |
| `odoo-module-upgrader` | FASE 3.3 port código custom v_X → v18 |
| `odoo-studio-migration` | FASE 2 decisiones custom → Studio |
| `odoo-pilot` | FASE 3.3 validación post-port via API |
| `common-skills:docker-deployment` | FASE 4 levantar entorno local de validación |
| `odoo-deep-knowledge` | Comportamiento cross-version y validación post-migración |

## Consumed by

Skill top-level del flujo migración. Puede ser invocada por:
- `common-skills:odoo-mentor` (recomendación proactiva al detectar una migración pendiente)
- `developer-skills:dynapps-migrator` (motor de upgrade de BD dentro del flujo de migración)
- Agente raíz desde mención directa en un proyecto Odoo

## REFERENCES

> **Helpers del kit**: 22 helpers en 7 grupos (validation, odoo, studio, data, spain, exec, reporting), auto-emitidos por el compiler vía `modules.action` + auto-inject contextual. Catálogo completo: `references/helpers-catalog.md`.

- `references/kit-setup.md` — **localizar/instalar el motor del kit** (apertura de sesión)
- `references/agent-tree.md` — **Agent() reales** (8 sub-agentes + 3 delegaciones) con prompts + JSON contracts
- `references/agent-rules.md` — reglas de autonomía y reglas globales del agente
- `references/gotchas/index.md` — **READ FIRST** — fallos reales (P1-P8, V1-V3, M17, etc.)
- `references/gotchas/v10_v18_migration.md` — gotchas v10→v18 con SQL fix-recipes
- `references/gotchas/code-port-patterns.md` — patrones código v_X→v_(X+M) (12 patrones Pedak v16→v19)
- `references/developer-workflow.md` — flujo del desarrollador (setup → iterate → cierre 1-step)
- `references/developer/auto-feedback.md` — `suggest` + PlaybookHook
- `references/developer/module-tracking.md` — status, blockers, orden topológico
- `references/developer/handoff.md` — entregar workdir a otro dev
- `references/developer/extend-kit.md` — subir mejoras al kit / ampliar la skill
- `references/helpers-catalog.md` — **catálogo completo de los 22 helpers del motor**
- `references/phases/*.md` — 6 fases del flujo (0→5)
- `references/restore.md` / `analysis.md` — detalle por FASE · `references/upgrade-api.md` / `common-upgrade-errors.md` / `pre-upgrade-checklist.md` — CLI servicio
- `references/post-upgrade-custom-fixes.md` — catálogo patrones (input del `detect`)
- `references/path-resolution.md` — convenciones workdir/pipeline_dir/absoluto
- `references/playbooks/` — recetarios + `fixes/` SQL
