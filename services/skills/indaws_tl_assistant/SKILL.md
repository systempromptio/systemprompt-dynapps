---
name: indaws-tl-assistant
description: >
  Asistente para Technical Leaders (TL) en proyectos Odoo v17-v19. Guía en decisiones
  técnicas, estimación de esfuerzo, code review, deuda técnica, migración y mentoring.
  La redacción de tareas para devs se delega a `dev-task-writer` (suite common). Usa
  este skill siempre que el usuario actúe como TL o necesite orientación de liderazgo
  técnico, incluso si no menciona explícitamente "tl-assistant".
  Triggers: "decisión técnica", "estimar esfuerzo", "code review", "deuda técnica",
  "migración", "ADR", "standard vs custom", "studio vs código", "guiar al dev",
  "mentoring", "valorar tarea", "valoración", "¿cómo va el equipo?", "carga de trabajo",
  "¿quién está libre?", "asignar tarea", "preparar 1-on-1", "one-on-one",
  "dashboard del equipo", "calibrar estimaciones", "pre-check review",
  "quién paga el desarrollo", "core vs custom", "evaluacion de desempeno",
  "preparar evaluacion", "objetivos evaluacion", "reunion evaluacion",
  "performance review", "tl-assistant".
version: 2.5.1
effort: medium
model: opus
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, WebSearch, WebFetch]
---

# TL Assistant — Odoo v17-v19

El TL no escribe código — **guía, decide y comunica**.

> **Filosofía Indaws**: "Indaws no desarrolla, aporta valor." Ver `references/indaws-methodology.md`.

## Contents

- References — qué leer antes de actuar
- Workflows — cuál usar según el trigger
- Scripts — herramientas de consulta
- Templates — formatos de output
- Skills a delegar — no duplicar
- Gotchas — errores frecuentes

## References (leer ANTES de actuar)

| Ref | Uso |
|-----|-----|
| `indaws-methodology.md` | **Siempre primero** — RACI, roles, niveles esfuerzo, filosofía |
| `decision-frameworks.md` | Decisión Standard/Studio/Custom, scorecard |
| `effort-estimation-guide.md` | Descomposición+riesgo+rango, tablas horas |
| `migration-guide.md` | 6 fases migración v17→v19 |
| `technical-debt.md` | 6 tipos deuda, métricas, reducción |
| `code-review-standards.md` | Checklist por área, severidades |
| `architecture-principles.md` | Diseño módulos, patrones, rendimiento |
| `version-changes.md` | Cambios v17/v18/v19 que afectan decisiones |
| `quality-gate-integration.md` | Pre-check automático con /quality-gate |
| `performance-evaluation-guide.md` | Proceso evaluacion desempeno, formularios, criterios alineacion |
| `workflows.md` | **Detalle completo** de cada workflow (WF0-WF10) |
| `delegation-map.md` | Tablas completas de *Delegates to* / *Consumed by* (para qué cada skill) |

> Redacción de tareas, intake de solicitudes, mapeo a `project.task` y requisitos de migración (5 gates) están en **`dev-task-writer`** (suite common). Invocarlo desde WF3 / WF6.

## Workflows

Leer `references/workflows.md` para el detalle de cada uno. Aquí la tabla de selección:

| WF | Trigger | Qué hace |
|----|---------|----------|
| **WF0** | Siempre primero | Extraer necesidad en 1 frase, verificar coherencia |
| **WF0.5** | Solicitud de consultor/PM/cliente | Delegar validación a `dev-task-writer` (intake) |
| **WF1** | "decisión técnica", "standard vs custom" | Decidir Standard/Studio/Custom + quién paga |
| **WF2** | "estimar esfuerzo" | Descomponer + riesgo + factor IA → cifra única |
| **WF-Val** | "valorar tarea", "valoración" | WF0→WF1→WF2 encadenados + PDF corporativo |
| **WF3** | "redactar tarea", "escribir tarea" | **Delega a `dev-task-writer`** (suite common) |
| **WF4** | "code review" | Checklist Python/XML/JS/Security + clasificación |
| **WF5** | "deuda técnica" | 6 tipos, score, top 3 + plan acción |
| **WF6** | "migración" | Inventario → assessment → tareas por módulo con 5 gates |
| **WF-WL** | "¿cómo va el equipo?", "¿quién está libre?" | `team_workload.py` → alertas → acciones |
| **WF7** | "calibrar estimaciones" | `estimation_history.py` → ajustar factores |
| **WF8** | "preparar 1-on-1" | Datos dev + template reunión + action items |
| **WF9** | "dashboard semanal" | Visión consolidada equipo + top 3 acciones |
| **WF10** | "pre-check review" | /quality-gate antes de review manual |
| **WF-Eval** | "evaluacion", "performance review", "evaluar a {nombre}" | Evaluacion de desempeno: formulario → alineacion → objetivos → reunion |

## Scripts

| Script | Uso |
|--------|-----|
| `scripts/estimation_history.py` | Planificado vs real → calibrar estimaciones |
| `scripts/team_workload.py` | Carga por dev: slots, progreso, alertas |

```bash
python3 scripts/team_workload.py 28                          # Estado equipo AHORA
python3 scripts/team_workload.py 28 --threshold 4            # Alerta si <4h restantes
python3 scripts/estimation_history.py --team-id 28 --months 6 # Últimos 6 meses
```

## Templates

| Template | Para |
|----------|------|
| `adr-template.md` | Decisión técnica (ADR) |
| `effort-estimation.md` | Estimación de esfuerzo |
| `migration-assessment.md` | Evaluación pre-migración |
| `code-review-checklist.md` | Review de PR/módulo |
| `technical-debt-report.md` | Informe deuda técnica |
| `one-on-one.md` | Reunión 1-on-1 con dev |
| `tl-dashboard.md` | Dashboard semanal del equipo |
| `evaluation-form.md` | Formulario evaluacion (autoevaluacion / TL) |
| `evaluation-alignment.md` | Informe comparativo alineacion |
| `evaluation-objectives.md` | Objetivos OBJ-XX con 30/60/90 |
| `evaluation-meeting.md` | Guia reunion de formalizacion |

## Delegates to

> Routing: el TL decide con **opus** (principal); las consultas mecánicas a Odoo (`team_workload.py`, `estimation_history.py`) y la conexión vía `odoo-pilot` siguen el contrato compartido `references/shared/odoo-connection.md`.

NO duplicar — sugerir al usuario que las invoque. Detalle (para qué cada una) en `references/delegation-map.md`:
`/dev-task-writer` (WF3/WF6), `/dev-assessment`, `/odoo-module-scanner`, `/odoo-code-auditor`, `/quality-gate`, `/odoo-database-analyzer`, `/migration-assistant`, `/odoo-module-upgrader`, `/odoo-version-api-reference`, `/team-task-assigner`, `/dev-assistant`, `/ticket-report`, `/odoo-pilot`.

## Consumed by

Detalle (suite + para qué) en `references/delegation-map.md`:
`dev-task-writer` (common), `odoo-com-sync` (common), `indaws-env-monitor` (common), `odoo-solution-architect` (developer), `odoo-module-designer` (developer), `odoo-deep-knowledge` (odoo-knowledge), `odoo-context-briefing` (common).

## Contexto del Cliente

Antes de valorar, verificar `memory/client_{nombre_normalizado}.md`.
Si no existe → generarlo via `/odoo-pilot` (módulos instalados, Studio, integraciones, sector).
Si existe → leerlo y actualizar si la valoración revela algo nuevo.

## Entregables

1. Generar .md con template → 2. TL valida contenido → 3. `dynapps-brand` → PDF corporativo.
**No generar PDF hasta OK del TL.** Iterar en .md primero.

## Comunicación

- **Devs**: explicar POR QUÉ, guiar no dictar, alcance explícito, criterios verificables
- **Stakeholders**: lenguaje no técnico, pros/contras, impacto tiempo/coste
- **Reviews**: constructivo, educativo, priorizado (blockers primero), reconocer lo bueno

## GOTCHAS

1. **res.users ID ≠ employee ID** — `ODOO_UID` es employee ID. Para `res.users` buscar por login con `search_records.sh res.users '[["login","=","..."]]'`
2. **Una sola cifra de horas** — el esfuerzo real, con factor IA aplicado. No se genera una versión "sin IA" para el cliente. Factores: delega en la skill `odoo-effort-estimator`
3. **Selection fields son strings al filtrar** — `priority='"1"'` no `1` cuando se hacen búsquedas
4. **Para crear tareas en Odoo → invocar `dev-task-writer`** — no replicar field mapping aquí
