---
name: indaws-odoo-configurator
description: >-
  Ejecuta y documenta items de checklist de tareas de implantación Odoo. Lee la tarea, analiza
  checklists pendientes, ejecuta configuraciones (settings, módulos, campos Studio) y documenta lo
  hecho en notas internas o Knowledge. Usar cuando haya que ejecutar y documentar items de checklist
  de una tarea.
version: 1.2.1
effort: medium
model: opus
allowed-tools: [Read, Write, Bash, Glob, Grep, Agent, common-skills:odoo-pilot, common-skills:odoo-studio, common-skills:odoo-flow-tester]
---

# Odoo Configurator

Configurador de Odoo especializado de Indaws (Odoo Gold Partner). Lee tareas de implantacion y sus checklists pendientes, clasifica items por tipo (CONFIG, STUDIO/AUTO, DEV), ejecuta configuraciones aprobadas por el consultor, y documenta todo en notas internas o Knowledge.

## Workflow

### Paso 0: Recopilacion de Datos

1. **Task ID** (`project.task`) en Odoo
2. **Credenciales Odoo**: URL, DB, API Key, Usuario (si no en env vars)
3. **Entorno**: Produccion o Test

See references/shared/odoo-connection.md for connection pattern. DETENTE y espera respuesta.

## Field contract (preflight OBLIGATORIO)
Antes de leer Odoo, verifica que existen (ver `references/shared/field-contract-preflight.md`):
- modelo `xx.checklist.item`: campo `state` (`pending` / `in_progress` / `waiting` / `done`)
Si falta algo → STOP con mensaje accionable (qué falta, dónde se configura en Studio). NO continúes con datos parciales.

### Paso 1-6: Secuencia completa

| Paso | Accion | Validacion |
|------|--------|------------|
| 1 | Leer tarea + checklists (scripts/fetch_task_checklist.sh). Filtrar: excluir completados y DEV/MIGR | Automatica |
| 2 | Plan de ejecucion: tabla items clasificados con accion propuesta y destino | OBLIGATORIO: usuario aprueba |
| 3a | Items CONFIG: ejecutar via odoo-pilot, verificar post-cambio | Advertencia en PROD |
| 3b | Items STUDIO/AUTO: propuesta detallada -> aprobacion -> ejecutar con odoo-studio -> verificar con odoo-flow-tester | OBLIGATORIO: usuario aprueba cambios Studio |
| 4 | Documentar: CONFIG simples -> 1 nota interna agrupada; STUDIO/AUTO -> articulo Knowledge "Adaptaciones" | Automatica |
| 5 | Actualizar checklists: NUNCA marcar completado, solo corregir tipo | Automatica |
| 6 | Resumen final con estadisticas | Informativo |

Full step-by-step details in `references/workflow_detail.md`.

### Documentation Routing

| Item type | Destination | Template |
|-----------|-------------|----------|
| CONFIG simple | Nota interna en chatter (agrupada) | `templates/nota_interna_config.md` |
| STUDIO/AUTO, complejos | Knowledge > Adaptaciones | `templates/adaptacion_knowledge.md` |

See `references/patrones_documentacion.md` for full routing rules.

## GOTCHAS

1. **Marcar checklist como completado** -- NUNCA cambiar `state` ni `date_done`. Solo el consultor lo hace manualmente
2. **Ejecutar en produccion sin confirmacion** -- SIEMPRE confirmacion explicita. Advertir antes de cambios irreversibles
3. **JSON2 message_post bug** -- Usar proceso de 2 pasos: message_post con body vacio (subtype_xmlid: mail.mt_note) + mail.message/write con HTML real
4. **Crear notas separadas por item CONFIG** -- Agrupar TODAS las configs simples en UNA SOLA nota por ejecucion
5. **Procesar items DEV/MIGR** -- Ignorar silenciosamente. No procesar, no mencionar al usuario
6. **Item checklist ambiguo** -- Buscar contexto en analisis funcional (Knowledge). Si sigue sin estar claro, PREGUNTAR al usuario
7. **Olvidar verificacion post-cambio** -- SIEMPRE verificar que la configuracion se aplico (lectura post-cambio para CONFIG, odoo-flow-tester para STUDIO)

### Agent() real — Pattern C "Post-action"

Para items STUDIO/AUTO del checklist, invocar odoo-studio y luego verificar:

```
# 1. Crear campos/modelos/vistas Studio
Agent({
  description: "Create Studio fields for {item_name}",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Create Studio customizations on Odoo instance {odoo_url}, database: {db_name}.
    Checklist item: '{item_name}' — {item_description from checklist}.
    Task context: {task_name} (project: {project_name}).
    Environment: {PROD|TEST}.
    Fields to create: {field_specs — list with model, field name, type, label, help text, required, default}.
    Use the odoo-studio skill. After creation, verify each field exists with fields_get on the target model.
    Return: list of created fields with their IDs, and any creation errors."
})

# 2. Post-verificacion con flow-tester
Agent({
  description: "Verify Studio changes for {item_name}",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Verify that the Studio customization '{item_name}' works correctly on {odoo_url}, database: {db_name}.
    What was created: {summary of fields/actions created in step 1 — field names, models, types}.
    Expected behavior: {test_description — what the user should be able to do after this change}.
    Use the odoo-flow-tester skill in 'test' mode. Test the specific flow affected by the change.
    Pass criteria: {specific pass/fail conditions, e.g. 'field visible on form', 'automated action triggers on write'}.
    Return: PASS/FAIL with evidence (screenshot path or API verification result)."
})
```

**NO es Agent()**: odoo-pilot (Bash inline — scripts CRUD), items CONFIG (se ejecutan con Bash directo).

## Delegates to

| Skill | Suite | When | Agent()? |
|-------|-------|------|----------|
| odoo-pilot | common | CRUD, execute, auth, fields_get | No |
| odoo-studio | common | Crear campos, modelos, vistas Studio | **Yes — Pattern C** |
| odoo-flow-tester | common | Verificar cambios Studio | **Yes — Pattern C** |
| odoo-deep-knowledge | odoo-knowledge | Entender por qué Odoo hace X al configurar flujos complejos | No |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| importation-data-assistant | consultant | Crear campos custom si mapeo lo requiere |
| consultant-daily-planner | consultant | Referencia para acciones de configuracion |

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Full 6-step workflow with execution phases |
| `references/configuraciones_estandar.md` | Playbook de configs por app (paths, verificaciones) |
| `references/patrones_documentacion.md` | Rules for nota interna vs Knowledge routing |
| `references/checklist_workflow.md` | Checklist states and transition rules |
| `templates/nota_interna_config.md` | HTML template for internal notes |
| `templates/adaptacion_knowledge.md` | HTML template for Knowledge articles |
| `scripts/fetch_task_checklist.sh` | Extracts task and checklist data |
