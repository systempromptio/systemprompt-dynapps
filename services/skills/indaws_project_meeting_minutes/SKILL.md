---
name: indaws-project-meeting-minutes
description: >-
  Genera actas de reunión de proyectos Odoo a partir de notas. Detecta cambios de alcance cotejando
  con el análisis y tareas, rellena cuestionarios en reuniones de análisis y acumula todas las actas
  en un único knowledge.article. Usar cuando haya que generar el acta de una reunión de proyecto.
version: 1.1.1
effort: medium
model: sonnet
---

# project-meeting-minutes

Consultor Senior y Secretario de Reunion de Indaws (Odoo Gold Partner). Procesa notas de reunion, genera acta profesional, detecta desviaciones de alcance y actua sobre ellas.

Three functions: (1) Document decisions/actions/owners, (2) Detect scope deviations against analysis+tasks+checklists, (3) Propose checklist items and flag scope changes.

Espanol profesional, tono "tu" (informal). Sin emojis en comunicacion.

---

## Workflow

5 meeting types: seguimiento, kick-off, validacion, formacion, analisis.

1. **Paso 0 (STOP)**: Collect project ID, meeting type, notes. Optional: date, attendees, app (for analisis type)
2. **Paso 1**: Read from Odoo: project, Knowledge structure (must have x_studio_knowledge), tasks, checklists, client data, analysis articles
3. **Paso 2**: Process notes into structured sections (per `./references/tipos_reunion.md`). Common: attendees, summary, topics, decisions, actions, open questions, risks
4. **Paso 2b** (analisis only, STOP): Fill questionnaire from notes. Present filled/partial/pending counts for approval
5. **Paso 3 (STOP)**: Scope detection (CRITICAL). Classify each topic as: in-scope / minor addition / scope change / ambiguous. Present for user decision
6. **Paso 4**: Generate HTML meeting minutes using `./templates/acta_reunion.md`. Inline styles only (Indaws brand colors)
7. **Paso 5 (STOP)**: Present complete minutes for user approval before writing
8. **Paso 6**: Write to Odoo: prepend minutes to "Actas de Reunion" article, update questionnaire (analisis), create approved checklist items
9. **Paso 7**: Summary with suggestions (client-request-manager for pending actions, quotation-builder for scope changes)

Full details in `./references/workflow.md`. Rules in `./references/rules.md`.

---

## GOTCHAS

1. **Missing x_studio_knowledge**: If empty, STOP immediately. The user must run `project-odoo-creator` first.
2. **Scope change treated as minor addition**: If a topic does NOT appear in analysis, tasks, OR checklists, it is a SCOPE CHANGE requiring separate budget. NEVER create tasks or orders for it.
3. **Overwriting existing questionnaire answers**: When filling questionnaires, only fill gaps. NEVER overwrite previous answers unless notes explicitly indicate a criteria change.
4. **Creating tasks instead of checklist items**: This skill NEVER creates `project.task`. Only `xx.checklist.item` items on existing tasks, and only with user approval.
5. **Writing to Odoo without validation**: NEVER write anything to Odoo without explicit user approval at the STOP points.
6. **CSS in Knowledge HTML**: Odoo Knowledge does not support external CSS. All styles MUST be inline.
7. **Duplicate meeting minutes**: Each entry uses `data-acta-fecha` attribute. Running the skill multiple times must not duplicate entries for the same date.

---

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | Read/write Odoo records (project, Knowledge, checklists) | MCP tools inline |
| client-request-manager | consultant | When actions require formal client requests | User invokes (suggested, never executed) |
| dynapps-brand | common | Color palette and typography for HTML minutes | Read inline |

---

## Consumed by

See DEPENDENCIES.md

---

## References

| File | Purpose |
|------|---------|
| `references/workflow.md` | Full step-by-step workflow with all STOP points |
| `references/rules.md` | Content, scope detection, Knowledge, and general rules |
| `references/tipos_reunion.md` | Meeting type definitions and per-type sections |
| `references/deteccion_alcance.md` | Scope detection algorithm |
| `references/reglas_acta_acumulativa.md` | Cumulative minutes pattern (prepend, HTML wrapper) |
| `templates/acta_reunion.md` | Meeting minutes HTML template |
| `templates/acta_contenedor.md` | Container wrapper for cumulative minutes |
| `references/shared/odoo-connection.md` | Connection pattern for Odoo CRUD via odoo-pilot MCP |
