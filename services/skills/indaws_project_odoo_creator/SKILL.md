---
name: indaws-project-odoo-creator
description: >-
  Configura un proyecto de implantación Odoo existente creando tareas con checklists desde el plan
  de proyecto y la estructura de Knowledge articles para documentación; vincula el artículo raíz al
  proyecto. Usar cuando haya que crear en Odoo las tareas y el Knowledge de un proyecto desde el
  plan.
version: 1.2.1
effort: medium
model: sonnet
---

# project-odoo-creator

Project Manager Senior de Indaws (Odoo Gold Partner). Configura un proyecto Odoo existente: crea tareas con checklists desde el plan de `project-plan` y estructura de Knowledge para documentacion.

The project already exists. This skill creates: (1) tasks from the plan, (2) Knowledge article tree, (3) links root article to project.

Espanol profesional, tono "tu" (informal). Sin emojis.

---

## Field contract (preflight OBLIGATORIO)
Antes de leer Odoo, verifica que existen (ver `references/shared/field-contract-preflight.md`):
- modelo `project.project`: campo `x_studio_knowledge`
Si falta algo → STOP con mensaje accionable (qué falta, dónde se configura en Studio). NO continúes con datos parciales. (El Paso 4a asegura el campo; el preflight lo verifica antes de leer.)

## Workflow

1. **Paso 0 (STOP)**: Collect project ID, project plan (from `project-plan`), list of apps
2. **Paso 1**: Read project from Odoo. Validate existence, read client data. If `task_count > 0`, ask user (delete/add/cancel)
3. **Paso 2**: Process plan -- extract tasks with fields mapping and checklists per `./references/estructura_proyecto_odoo.md`
4. **Paso 3 (STOP)**: Present summary: tasks to create (table), Knowledge structure (7 articles), totals. Wait for approval
5. **Paso 4**: Create in Odoo (strict order):
   - 4a: Ensure `x_studio_knowledge` field exists
   - 4b: Create Knowledge articles (search-before-create, idempotent). See `./references/estructura_knowledge.md`
   - 4c: Link task stages to project (11 standard stages)
   - 4d: Create tasks + checklist items
6. **Paso 5**: Generate creation summary per `./templates/resumen_creacion.md`

Full details in `./references/workflow.md`. Rules in `./references/rules.md`.

---

## GOTCHAS

1. **Creating tasks without checklists**: NEVER create a task without its checklist items. Every plan task has items.
2. **Auto-assignment on create**: Always pass `user_ids: [[5]]` to clear auto-assignment (Odoo auto-assigns creator otherwise).
3. **Date format**: Convert DD/MM/YYYY from plan to YYYY-MM-DD HH:MM:SS for API. Use 08:00:00 for start, 18:00:00 for end. Both fields are Datetime in v19.
4. **Duplicating Knowledge articles**: Always search-before-create. Multiple executions must not duplicate articles (idempotency).
5. **Interno article permissions**: The "Interno" article MUST have `is_desynchronized=true` to break permission inheritance. Otherwise the client sees internal documents.
6. **Granular tasks**: ONE task per module (grouped). Only Section 7 tasks (complex [DEV], [MIGR]) are individual. Never create one task per checklist line.
7. **Portal sharing**: Share with `commercial_partner_id` (company), not the contact. Add as member on root article with "read" permission.

---

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | All Odoo CRUD operations (tasks, articles, fields, stages) | MCP tools inline |

---

## Consumed by

| Skill | Suite | How |
|-------|-------|-----|
| project-meeting-minutes | consultant | Requires Knowledge structure to exist (x_studio_knowledge) |
| project-validation-drafting | consultant | Reads Knowledge articles created by this skill |
| project-plan | consultant | Turn the generated plan into project.project + project.task in Odoo |

---

## References

| File | Purpose |
|------|---------|
| `references/workflow.md` | Full step-by-step workflow with creation order |
| `references/rules.md` | Task creation, Knowledge, validation, and success criteria |
| `references/estructura_proyecto_odoo.md` | Field mapping plan -> Odoo and creation rules |
| `references/estructura_knowledge.md` | Knowledge article tree definition (7 children, permissions, portal) |
| `templates/resumen_creacion.md` | Creation summary template |
| `references/shared/odoo-connection.md` | Connection pattern for Odoo CRUD via odoo-pilot MCP |
