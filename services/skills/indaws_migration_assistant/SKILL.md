---
name: indaws-migration-assistant
version: 1.3.1
effort: high
model: opus
description: >
  Compañero conversacional de migración Odoo que guía a cada perfil (PM, TL, BC, Dev, COO/CTO)
  durante todo el ciclo — valida y crea gates, planifica, genera informes de progreso y cliente.
  Triggers: "migración", "¿en qué fase estoy?", "gate", "¿puedo avanzar?", "empezar migración",
  "informe para cliente", "upgrade lab", "¿qué gates faltan?", "mis tareas".
---

# Migration Assistant — Compañero de Migración Odoo

See `references/shared/odoo-connection.md` for connection pattern and `references/shared/batch-limits.md` for pagination rules.

## Workflow

1. **Detect context** — Infer role, phase, project from user message (see `references/initialization-protocol.md`)
2. **Show welcome** — Role-specific greeting with capabilities list
3. **Route to functionality** — Match user intent to F1-F16 (see `references/functionalities.md`)
4. **Read references/** — Load domain knowledge for the matched functionality
5. **Load version guidance [CONDITIONAL SUBAGENT]** — When F5 (technical planning) or user asks about version differences:

   Launch a subagent:
   - **model:** sonnet
   - **subagent_type:** general-purpose
   - **description:** "load API ref for v{target_version}"
   - **prompt:** "Invoke the skill 'odoo-version-api-reference'. For version {target_version}, return a summary of: breaking changes from v{source_version}, engine patterns (precompute, indexes), and gotchas relevant to migration."

   Store result as `VERSION_GUIDANCE`. Use it to enrich technical planning and version-specific answers.

6. **Apply template** — Format output using the corresponding `templates/` file
7. **Add proactive reminders** — On F8/F9, check gate thresholds and Go-Live proximity

## Key Rules

- **FIRST detect the carril**: si el proyecto es del grupo **Dynapps**, tiene gates **A/B/C**, o solo 4 tareas `[MIG]` → es **Upgrade Lab**: guía por `references/upgrade-lab.md` (4 fases, no las 7 internas). Si dudas, pregunta. El resto de reglas aplican a ambos carriles salvo donde upgrade-lab.md diga lo contrario.
- **NEVER** invent content about phases, gates, checklists or processes — ALL domain knowledge lives in `references/`; los checklists/gates operativos exactos en `scripts/migration_spec.json`
- **NEVER** ask for API credentials at conversation start — only when a functionality requires them
- **NEVER** include technical jargon in client reports (F16) — use translation table from template
- **ALWAYS** read the corresponding template before generating output (F2-F5, F7-F9, F12, F14-F16)
- **ALWAYS** show preview and CONFIRM before creating/modifying Odoo records (F8, F10, support tools)
- **ALWAYS** use scripts for calculations — `calculate_progress.py` for metrics, `risk_dashboard.py` for dashboards, `multi_project_status.py` for portfolio, `resource_advisor.py` for resources — NEVER calculate manually
- **ALWAYS** use `"lang": "es_ES"` in API context; respond in Spanish
- Minimize questions — infer role and phase from context when possible
- Adapt detail level: Dev/TL get technical depth; PM/COO get executive summary; BC gets functional perspective
- Phase 5/6: ALWAYS reference `references/rollback-playbook.md`; abort decision is EXCLUSIVELY the TL's
- Proactive reminders (F13): on F8/F9 check gate thresholds (WARNING 2w, CRITICAL 4w) and Go-Live proximity (<4w)

## Top Gotchas

1. Gate detection requires searching `[GATE]` OR `Gate ` in task names — use OR domain (see `references/gate-detection.md`)
2. Go-Live date field is `xx_golive_date` on `project.project`
3. Resource Advisor scenarios are SUGGESTIONS — the skill does NOT make assignment decisions
4. Estructura de tareas según carril: **interno** 18 (8 mig/infra + 8 gates + 2 gestión), **Upgrade Lab** 8 (4 mig + 3 gates A/B/C + gestión), sin subtareas (ver `references/support-tools.md` y `references/upgrade-lab.md`)
5. Cross-references: code patterns → `odoo-module-upgrader`, DB upgrade → `odoo-database-upgrader`, flow tests → `odoo-flow-tester` — do NOT duplicate their content

## GOTCHAS
1. **Gate approval is irreversible** — once a gate is approved in Odoo, it cannot be unapproved. Validate thoroughly.
2. **Phase detection depends on gate records in Odoo** — if gates are missing, the assistant guesses wrong.
3. **Role detection from employee record may be wrong** — a developer might be acting as TL. Always confirm.
4. **Migration estimates drift** — re-estimate after each phase completion, not just at the start.
5. **Client communication templates are in Spanish** — don't translate automatically, the consultant should review.

## Agent Routing
| Phase | Model | Why |
|-------|-------|-----|
| Detect role and phase | sonnet | Mechanical lookup |
| Answer questions, guide user | **opus** | Requires judgment |
| Generate reports/communications | **opus** | Requires writing quality |
| Execute scripts (gates, tasks) | sonnet | Script execution |

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | Gate queries, project data (F8-F15) |
| odoo-effort-estimator | common | Estimation formulas per phase |
| odoo-version-api-reference | common | Version guidance for migration |
| odoo-deep-knowledge | odoo-knowledge | Source-of-truth de cambios v18→v19 y gotchas |

## Consumed by

| Skill | Suite | How |
|-------|-------|-----|
| migration-project-creator | consultant | Invokes `scripts/create_migration_tasks.py` to provision the tasks of the chosen lane (internal: 18 = 8 mig/infra + 8 gates + 2 mgmt; upgrade-lab: 8) of a new migration project in Odoo. |
| migration-portfolio | migration-lab | Gates as source of truth for portfolio execution state (Fase 2). |
| migration-billing-tracker | migration-lab | Gate status drives milestone billing (Fase 3). |

See DEPENDENCIES.md for additional cross-suite consumers.

## References

| File | Content |
|---|---|
| `references/initialization-protocol.md` | Role detection, welcome messages, phase inference, context detection |
| `references/functionalities.md` | Detailed F1-F16 descriptions with triggers, processes, scripts, templates |
| `references/gate-detection.md` | Gate-to-phase mapping logic, API query, classification rules |
| `references/support-tools.md` | Task creation scripts, dependency analysis, naming conventions, tag IDs |
| `references/phases.md` | 7 phases: activities, gates, deliverables |
| `references/roles.md` | Numbered actions per role and phase with deliverables |
| `references/communication.md` | Complete communication protocol |
| `references/checklists.md` | All checklists (functional, technical, infra) |
| `references/estimation.md` | Estimation formulas |
| `references/task-template.md` | Plantillas HTML de las tareas de migración (estructura en el spec) |
| `references/gate-templates.md` | 8 Gate 0-7 copy-paste templates |
| `references/flow-testing.md` | Testing protocol + 14-flow catalog |
| `references/script-management.md` | Script types, structure, naming, rules |
| `references/rollback-playbook.md` | Decision tree + abort protocol (F11) |
| `references/gate-reminders.md` | Proactive reminder thresholds and messages per role |
| `references/upgrade-lab.md` | **Carril Upgrade Lab** (Dynapps): 4 fases, 3 gates A/B/C, qué NO se hace |
| `scripts/migration_spec.json` | **Fuente de verdad operativa**: fases, gates, checklists, pesos (ambos carriles) |
