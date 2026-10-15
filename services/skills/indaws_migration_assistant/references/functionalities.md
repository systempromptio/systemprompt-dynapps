# Functionalities — Detailed F1-F16

## Functionality Index

| # | Functionality | Triggers | Roles | API | Template / Reference |
|---|---|---|---|---|---|
| F1 | Context detection | Always first | All | Optional | — |
| F2 | Phase guide | "¿Qué se hace?", "¿En qué consiste?" | All | No | `templates/phase-summary.md` |
| F3 | Role guide | "¿Qué tengo que hacer?", "¿Qué me toca?" | All | No | `templates/role-guide.md` |
| F4 | Gate validation | "Gate X", "¿Puedo avanzar?" | All | No | `templates/gate-validation.md` |
| F5 | Technical planning | "¿Por dónde empiezo?", "Plan técnico" | Dev, TL | Optional | `templates/technical-planning.md` |
| F6 | Process queries | Free questions about migration | All | No | — (direct answer) |
| F7 | Communication reports | "Informe Dev→BC", "Resumen TL→PM" | Dev, TL | No | `templates/dev-to-bc-report.md` / `templates/tl-to-pm-report.md` |
| F8 | Progress report | "¿Cómo vamos?", "informe de horas" | All | Yes | `templates/migration-status-report.md` |
| F9 | Gate progress view | "¿Qué gates faltan?", "progreso de gates" | All | Yes | `templates/gate-progress.md` |
| F10 | Gate creation | "Crea los gates", "Monta los gates" | PM, TL | Yes | `references/gate-templates.md` |
| F11 | Rollback Playbook | "Rollback", "Plan de abort" | TL, Dev, PM | No | `references/rollback-playbook.md` |
| F12 | Health Dashboard | "Dashboard", "Riesgos" | PM, COO, TL | Yes | `scripts/risk_dashboard.py` + `templates/risk-dashboard.md` |
| F13 | Proactive reminders | (automatic in F8/F9) | All | Yes | `references/gate-reminders.md` |
| F14 | Multi-project Portfolio | "Portfolio", "Comparar proyectos" | PM, COO, TL | Yes | `scripts/multi_project_status.py` + `templates/multi-project-report.md` |
| F15 | Resource Advisor | "¿Qué recursos necesito?" | TL, PM | Yes | `scripts/resource_advisor.py` + `templates/resource-advisor.md` |
| F16 | Client report | "Informe para cliente" | BC, PM | No | `templates/bc-to-client-report.md` |

---

## F1: Context Detection

**Trigger:** Always executes as first step of any interaction.

**Process:**
1. Analyze user message to infer: role, phase, project.
2. Apply "Smart Detection" logic from `references/initialization-protocol.md`.
3. If enough data to infer phase (contextual clues), DO NOT query API.
4. If user explicitly asks about progress or gates, offer to query Odoo.
5. If essential data missing, ask conversationally and friendly.
6. Once context identified, route to corresponding functionality (F2-F16).

**Key rule:** Minimize questions. If inferable from context, don't ask.

---

## F2: Phase Guide

**Trigger:** "¿En qué fase estoy?", "¿Qué se hace en esta fase?", "¿En qué consiste la fase X?"

**Process:**
1. Read `references/phases.md` for full phase description.
2. Read `references/roles.md` to include ALL roles' actions in that phase.
3. Format with `templates/phase-summary.md`.

**Reads:** `references/phases.md`, `references/roles.md`
**Produces:** Phase summary with activities, gate, and actions per role.
**No API required.**

---

## F3: Role Guide

**Trigger:** "¿Qué tengo que hacer?", "¿Qué me toca?", "Soy [role], ¿qué hago?"

This is the skill's main flow. When a user says "Soy dev, estoy en una migración, ¿qué tengo que hacer?", the skill must:

1. Detect role (Dev in this case)
2. Determine phase (by clues, gates in Odoo, or asking)
3. Read `references/roles.md` for the role's numbered actions in the phase
4. Read `references/communication.md` to include communication protocol
5. Include reference to the phase's gate with user's role highlighted
6. Format with `templates/role-guide.md`

**Reads:** `references/roles.md`, `references/communication.md`, `references/phases.md`
**Produces:** Numbered action list with concrete deliverables.
**API:** Optional (for phase detection via gates).

---

## F4: Gate Validation

**Trigger:** "Gate X", "¿Puedo avanzar?", "¿Qué necesito para el gate?", "dame la plantilla del gate"

**Process:**
1. Read `references/gate-templates.md` for the gate's copy-paste template.
2. Read `references/roles.md` for each role's deliverables for that gate.
3. Pre-fill fields (Project, Client, Version) if project context available.
4. Format with `templates/gate-validation.md`.

**Reads:** `references/gate-templates.md`, `references/roles.md`, `references/phases.md`
**Produces:** Pre-filled copy-paste template + deliverables list per role.
**No API required.**

**Rule:** Gate templates are ALWAYS delivered in ready-to-use copy-paste format.

**Note Gate 5:** Include reference to Rollback Playbook (F11). The deliverable "Documented and tested rollback plan" is formalized with `templates/rollback-decision.md`.

---

## F5: Technical Planning

**Trigger:** "¿Por dónde empiezo?", "Orden de migración", "Plan técnico"

**Process (two modes):**

### Mode with API (project exists in Odoo):
1. Execute `scripts/analyze_task_dependencies.py` for topological order.
2. Include script management from `references/script-management.md`.
3. Include communication protocol from `references/communication.md`.
4. Format with `templates/technical-planning.md`.

### Mode without API (no project in Odoo):
1. Apply ordering criteria from `references/phases.md`:
   - Standard first → Core/Studio → Custom last (migration ordering by change surface, not a preference)
   - No dependencies → with dependencies (topological order)
   - Tie: lower risk first
2. Include script management from `references/script-management.md`.
3. Format with `templates/technical-planning.md`.

**Reads:** `references/phases.md`, `references/script-management.md`, `references/communication.md`
**Produces:** Technical plan with order, scripts, environment and communication.
**API:** Optional (improves result if Odoo data available).

---

## F6: Process Queries

**Trigger:** Free questions about the migration process.

**Process:**
1. Search answer in `references/` (phases.md, roles.md, communication.md, checklists.md, estimation.md, flow-testing.md, script-management.md).
2. Answer directly without template, citing the source.
3. If question not covered in references/, say so explicitly.

**Reads:** Any `references/` file as needed.
**Produces:** Direct answer with source reference.
**No API or template required.**

---

## F7: Communication Reports

**Trigger:** "Informe Dev→BC", "Resumen TL→PM", "Plantilla de informe"

**Process:**
1. Detect report type by role:
   - **Dev:** → `templates/dev-to-bc-report.md`
   - **TL:** → `templates/tl-to-pm-report.md`
2. Show template with usage instructions.
3. If project data available, pre-fill what possible.

**Reads:** `references/communication.md` for frequency and format context.
**Produces:** Ready-to-fill report template.
**No API required.**

---

## F8: Progress Report

**Trigger:** "¿Cómo vamos?", "Estado de la migración", "Informe de progreso", "Informe de horas"

**Process:**
1. Get API credentials (Step 3 of initialization protocol).
2. Execute `scripts/calculate_progress.py` with project data.
3. Script calculates: % completed, hours consumed/billable/non-billable, alerts, forecast.
4. Format with `templates/migration-status-report.md`.

**Reads:** Project data via API.
**Produces:** Progress report with real metrics.
**Requires API:** Yes.

### Script
```bash
python3 scripts/calculate_progress.py \
    --url "$ODOO_URL" --db "$ODOO_DB" --user "$ODOO_USER" --key "$ODOO_KEY" \
    --project-id {project_id} --version "$ODOO_VERSION" \
    --budget-hours {budget_hours}
```

### Automatic Alerts
- **CRITICAL:** Hours consumed > 80% AND tasks completed < 80%
- **WARNING:** Hours vs progress deviation > 15 percentage points
- **WARNING:** Blocked tasks > 0

### Proactive Reminders (F13)
Apply F13 logic: check pending gates and Go-Live date, add alerts section if applicable (see `references/gate-reminders.md`).

---

## F9: Gate Progress View

**Trigger:** "¿Qué gates faltan?", "¿Qué gates tengo pendientes?", "Progreso de gates"

**Process:**
1. Get API credentials.
2. Search [GATE] tasks in the Odoo project:
   ```
   domain = [
     ['project_id', '=', project_id],
     '|',
     ['name', 'ilike', '[GATE]'],
     ['name', 'ilike', 'Gate ']
   ]
   fields = ['id', 'name', 'stage_id', 'user_ids', 'date_last_stage_update']
   ```
3. Classify each gate as approved/pending.
4. Determine current phase per `references/gate-detection.md`.
5. Format with `templates/gate-progress.md`.

**Produces:** Gate table with status, inferred current phase, and next actions per role.
**Requires API:** Yes.

### Output Format

```
Migración {Client} v{origin} → v{target}
Fase actual: {N} — {phase_name}

| Gate | Nombre                      | Estado    | Aprobado por | Fecha      |
|------|-----------------------------|-----------|--------------|------------|
| 0    | Alcance y Propuesta         | Aprobado  | COO          | 2026-01-15 |
| 1    | Análisis Técnico            | Aprobado  | TL           | 2026-01-28 |
| 2    | Código Migrado              | Pendiente | —            | —          |
| ...  |                             |           |              |            |

Progreso: [████████░░░░░░░░░░░░] 25% (2/8 gates aprobados)

Próximo gate: Gate 2 — Código Migrado
  Ejecuta: Dev | Aprueba: Tech Lead
  Entregables pendientes: [list per references/roles.md]
```

### Proactive Reminders (F13)
Apply F13 logic: after generating gate table, check thresholds from `references/gate-reminders.md` and Go-Live date (`xx_golive_date`). If applicable, add role-personalized reminder section.

---

## F10: Gate Creation in Odoo

**Trigger:** "Crea los gates", "Monta los gates del proyecto", "Crear tareas de gate"

**Process:**
1. Get API credentials.
2. Get project data: client name, versions, project_id.
3. For each gate (0-7):
   a. Name: `[GATE] Gate {N} — {gate_name}`
   b. Description: Copy-paste template from `references/gate-templates.md`, converted to HTML
   c. Tags: `Migration` (+ gate tag if exists)
   d. Gates are created directly in the project (no parent task).
4. Show preview (dry-run) BEFORE creating.
5. Request explicit user confirmation.
6. Create tasks and report result.

### Gate Names

| # | Task name in Odoo |
|---|---|
| 0 | `[GATE] Gate 0 — Alcance y Propuesta Comercial` |
| 1 | `[GATE] Gate 1 — Análisis Técnico` |
| 2 | `[GATE] Gate 2 — Código Migrado` |
| 3 | `[GATE] Gate 3 — Base de Datos Migrada` |
| 4 | `[GATE] Gate 4 — Testing Funcional` |
| 5 | `[GATE] Gate 5 — Ensayo General (Rehearsal)` |
| 6 | `[GATE] Gate 6 — Puesta en Producción (Go-Live)` |
| 7 | `[GATE] Gate 7 — Cierre del Proyecto` |

### HTML Description for Each Gate

Convert copy-paste template from `references/gate-templates.md` to HTML for the task `description` field.

Minimum format:
```html
<div class="migration-gate">
  <h3>Gate {N} — {name}</h3>
  <p><strong>Ejecuta:</strong> {executor} | <strong>Aprueba:</strong> {approver}</p>
  <h4>Entregables</h4>
  <ul class="o_checklist">
    <li>{deliverable_1}</li>
    <li>{deliverable_2}</li>
    ...
  </ul>
  <h4>Observaciones</h4>
  <p><em>(Rellenar al momento de solicitar aprobación)</em></p>
</div>
```

**Requires API:** Yes.
**Rule:** ALWAYS show preview and CONFIRM before creating.

---

## F11: Rollback Playbook

**Trigger:** "Rollback", "Plan de abort", "¿Qué pasa si falla?", "Plan de contingencia"

**Process:**
1. Detect context: Phase 5 (preparing plan) or Phase 6 (executing go-live)?
2. If Phase 5 (preparation):
   a. Read `references/rollback-playbook.md` to explain 3 abort levels.
   b. Deliver `templates/rollback-decision.md` pre-filled with project data.
   c. Indicate that times must be measured during rehearsal.
3. If Phase 6 (incident in progress):
   a. Guide TL through decision tree from `references/rollback-playbook.md`.
   b. Identify abort level based on reported error.
   c. Deliver execution protocol for the corresponding level.
   d. Include communication templates adapted to the level.
4. If generic context (process query):
   a. Explain 3 levels, decision tree and criteria.
   b. Reference that the plan is formalized in Gate 5.

**Reads:** `references/rollback-playbook.md`, `templates/rollback-decision.md`
**Produces:** Pre-filled rollback plan (Phase 5) or real-time decision guide (Phase 6)
**No API required.**

---

## F12: Project Health Dashboard

**Trigger:** "¿Cómo va el proyecto?", "Dashboard", "Estado del proyecto", "Riesgos", "Risk dashboard"

**Process:**
1. Get API credentials.
2. Execute `scripts/risk_dashboard.py` with project data.
3. Script generates practical dashboard with: KPIs, focus areas, hours breakdown, gates, active tasks and forecast.
4. If `--output` specified, generates self-contained exportable HTML.
5. Format conversational summary with `templates/risk-dashboard.md`.

**Reads:** Project data via API (tasks, timesheets, gates, hour allocation `project.project.hour.line`).
**Produces:** Project health dashboard with KPIs, actionable alerts, hours breakdown per employee and exportable HTML.
**Requires API:** Yes.

### Script

```bash
python3 scripts/risk_dashboard.py \
    --url "$ODOO_URL" --db "$ODOO_DB" --user "$ODOO_USER" --key "$ODOO_KEY" \
    --project-id {project_id} --version "$ODOO_VERSION" \
    --lang es \
    --output /tmp/dashboard.html
```

Optional parameters:
- `--budget-hours N`: Manual budget override (default uses project hour allocation)
- `--lang es|en`: Label language and date format (default: es)

### Dashboard Sections

| Section | Content |
|---------|---------|
| KPIs | Tasks completed, hours consumed/available, gates, days until Go-Live |
| Focus areas | Auto alerts: forecast, hours, blocked tasks, unassigned |
| Hours | Breakdown per employee (billable/non-billable), active allocation |
| Gates | Status of each gate (approved/pending/in-progress) |
| Tasks | Active task list with stage, planned/consumed hours |
| Forecast | Average/recent velocity, estimated end date for tasks and hour exhaustion |

---

## F13: Proactive Gate Reminders

**Trigger:** Activates automatically within F8 and F9. No independent trigger.

**Process:**
1. When generating progress report (F8) or gate view (F9), check `date_last_stage_update` of each pending gate.
2. Check `xx_golive_date` from `project.project` for Go-Live date.
3. Calculate pending weeks per gate and days until Go-Live.
4. If any gate exceeds WARNING threshold (2 weeks) or CRITICAL (4 weeks), generate alerts section.
5. If Go-Live in < 4 weeks with pending gates, include proximity alert.
6. Personalize messages per user role (from `references/gate-reminders.md`).
7. Add "Reminders" section at end of generated report.

**Reads:** `references/gate-reminders.md`, gate data via API, `project.project` for Go-Live date
**Produces:** Additional section in F8/F9 reports with role-specific alerts
**Requires API:** Yes (uses F8/F9 data)

---

## F14: Multi-project Portfolio

**Trigger:** "¿Cómo van las migraciones?", "Comparar proyectos", "Portfolio", "¿Cuál va peor?"

**Process:**
1. Get API credentials.
2. Get project IDs. If user doesn't provide them, ask.
3. Execute `scripts/multi_project_status.py` with project IDs.
4. Script generates: comparison, health score per project, most advanced, most at risk.
5. If `--output` specified, generates self-contained exportable HTML.
6. Format conversational summary with `templates/multi-project-report.md`.

**Context switching:** If user asks about a specific project by name (e.g., "¿Cómo va Eurolotes?"), filter portfolio output to show only that project.

**Requires API:** Yes.

### Script

```bash
python3 scripts/multi_project_status.py \
    --url "$ODOO_URL" --db "$ODOO_DB" --user "$ODOO_USER" --key "$ODOO_KEY" \
    --project-ids {ids_comma_separated} --version "$ODOO_VERSION" \
    --lang es \
    --output /tmp/multi-dashboard.html
```

### Health Score (0-100)

| Component | Weight | Description |
|-----------|--------|-------------|
| Task progress | 30% | % tasks completed |
| Budget health | 25% | Hours consumed vs progress deviation |
| Timeline | 25% | Days until Go-Live vs remaining work |
| Gates | 20% | % gates approved |

Indicators: green (>70), yellow (40-70), red (<40).

---

## F15: Resource Advisor

**Trigger:** "¿Cómo arreglo esto?", "¿Qué recursos necesito?", "El proyecto va mal", "Resource advisor"

**Process:**
1. Get API credentials.
2. Execute `scripts/resource_advisor.py` with project ID.
3. Script diagnoses project, checks developer availability from department, generates 3 recovery scenarios.
4. If `--output` specified, generates self-contained exportable HTML.
5. Format conversational summary with `templates/resource-advisor.md`.

**Reads:** Project data via API (tasks, timesheets, gates, allocation) + developer availability data (hr.employee, planning.slot, hr.leave).
**Produces:** Diagnosis + recovery scenarios (add resource, reduce scope, move go-live) with quantified date impact.
**Requires API:** Yes.

### Script

```bash
python3 scripts/resource_advisor.py \
    --url "$ODOO_URL" --db "$ODOO_DB" --user "$ODOO_USER" --key "$ODOO_KEY" \
    --project-id {project_id} --version "$ODOO_VERSION" \
    --weeks 8 --lang es \
    --output /tmp/resource-advisor.html
```

### Generated Scenarios

| Scenario | Input data | Result |
|----------|-----------|--------|
| Add resource | Dev pool + department availability | Top 5 devs with bandwidth and forecast impact |
| Reduce scope | Pending tasks without logged hours, low priority | 1-3 cut options with forecast impact |
| Move Go-Live | Current velocity + remaining hours | Minimum Go-Live date to finish on time |

### Availability Data

The script queries the same models as the dev-assistant skill:
- `hr.employee` — Dev Department (department_id=4), active, excludes CTOs (194, 193)
- `planning.slot` — Planned slots (allocated_hours, x_studio_done)
- `hr.leave` — Validated leaves (state='validate')

A developer "has bandwidth" if available > 10h in the planning horizon.

---

## F16: Client Report (BC → Client)

**Trigger:** "Informe para cliente", "Email al cliente", "Comunicar al cliente"

**Process:**
1. Detect current project phase (via F1).
2. Read `templates/bc-to-client-report.md` for template and phase adaptation guide.
3. Check adaptation guide for current phase: what to emphasize, what language to use, what client actions to include.
4. If F8 data (`calculate_progress.py`) available, simplify to: % global progress and estimated date. DO NOT include internal metrics.
5. Pre-fill template with project data if context available.
6. Apply technical→client translation table to eliminate all jargon.
7. Deliver to BC/PM the template ready to send by email.

**Reads:** `templates/bc-to-client-report.md`, `references/communication.md`, `references/phases.md`
**Produces:** Formal progress email in Spanish, no technical jargon, max 1 page.
**No API required.** (Can use F8 data if available, but not mandatory.)

**Key rule:** ZERO technical jargon. All internal terms are translated using the template's table.
