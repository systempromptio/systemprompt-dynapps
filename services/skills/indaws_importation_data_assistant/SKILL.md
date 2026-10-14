---
name: indaws-importation-data-assistant
description: >-
  Asiste en la importación masiva de datos a Odoo. Localiza archivos adjuntos en client requests,
  analiza Excel/CSV, mapea columnas a campos Odoo, normaliza datos, importa vía método load en
  batches y documenta el resultado. Usar cuando haya que importar datos masivos a Odoo desde
  Excel/CSV.
version: 1.2.1
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob, Grep, Agent, common-skills:odoo-pilot, common-skills:odoo-importation, consultant-skills:odoo-configurator]
---

# Importation Data Assistant

Asistente de Importacion de Datos de Indaws (Odoo Gold Partner). Flujo completo: localizar archivo, analizar, mapear campos, normalizar, importar via `load` en batches, y documentar en tarea y client request.

## Workflow

### Paso 0: Recopilacion de Datos

1. **Task ID** (`project.task`) en Odoo
2. **Que importar** (contactos, productos, tarifas, facturas, etc.) -- modelo se infiere automaticamente
3. **Credenciales Odoo** si no estan en variables de entorno

See references/shared/odoo-connection.md for connection pattern. DETENTE y espera respuesta.

### Paso 1-6: Secuencia completa

| Paso | Accion | Validacion |
|------|--------|------------|
| 1 | Localizar archivo (scripts/fetch_import_context.sh + download_attachment.py) | Usuario confirma archivo |
| 2 | Analizar archivo (scripts/analyze_file.py) -- estructura, duplicados, inconsistencias | Informativo |
| 3 | Mapear campos (fields_get + references/mapeo_campos_comunes.md) | OBLIGATORIO: usuario aprueba mapeo |
| 4 | Preparar importacion -- entorno (PROD/TEST), normalizar CSV, test con 5-10 filas | Usuario confirma entorno + test OK |
| 5 | Importar en batches de max 1000 registros via metodo `load` | Automatico con progreso |
| 6 | Documentar -- nota interna en tarea + actualizar client request | Automatico |

Full step-by-step details in `references/workflow_detail.md`.

### Key Decision: Campos no mapeados

For unmapped columns, the user chooses:
- **(a) Ignorar**: Skip column
- **(b) Mapear a campo existente**: User indicates Odoo field
- **(c) Crear campo nuevo**: Create checklist item for odoo-configurator (this skill NEVER creates fields)

## GOTCHAS

1. **Crear campos directamente** -- NUNCA crear campos. Si falta un campo, crear item en checklist para odoo-configurator
2. **Importar en produccion sin confirmacion** -- SIEMPRE advertencia explicita con numero de registros y confirmacion del usuario
3. **Saltarse test con muestra** -- SIEMPRE importar 5-10 filas primero y verificar antes de importacion completa
4. **Olvidar external IDs** -- SIEMPRE generar `__import__.{modelo}_{id}` para idempotencia
5. **JSON2 message_post bug** -- Usar proceso de 2 pasos: message_post con body vacio + mail.message/write con HTML real
6. **Batch size excesivo** -- Maximo 1000 registros por llamada a `load`. Batches mas grandes pueden causar timeouts
7. **Marcar checklist como completado** -- NUNCA marcar items como completados. Solo el consultor lo hace

### Agent() real — Pattern C "Post-action"

Para la importacion masiva, invocar odoo-importation con el plan de mapeo:

```
# Importacion batch via load method
Agent({
  description: "Import {n_records} {model} records",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Import data into Odoo instance {odoo_url}, database: {db_name}.
    Model: {model} (e.g. res.partner, product.template).
    Total records: {n_records}.
    Source file: {file_path} ({file_format: CSV/XLSX}, encoding: {encoding}, separator: {separator}).
    Field mapping (approved by user in Paso 3): {mapping_json — column_name: odoo_field pairs}.
    Normalization rules applied: {summary of normalizations from Paso 4, e.g. 'country codes → res.country IDs', 'phone → E.164'}.
    Environment: {PROD|TEST} — {if PROD: 'CAUTION: production environment, user has confirmed.'}.
    Use the odoo-importation skill. Batch size: 1000. Generate external IDs with prefix __import__.{model}.
    MANDATORY: test first with 5 rows and verify before full import.
    Return: total imported, total errors, error details per row, external IDs created."
})
```

### Agent() condicional — odoo-configurator (solo si usuario decide crear campos)

```
Agent({
  description: "Create {field_name} on {model} for import",
  subagent_type: "general-purpose",
  model: "sonnet",
  prompt: "Create a custom field needed for data import on {odoo_url}, database: {db_name}.
    Context: importing {model} records, column '{column_name}' from source file has no matching Odoo field.
    Field specification: {field_spec — name, type, label, help text, required, default value}.
    Target model: {model}.
    Environment: {PROD|TEST}.
    Use the odoo-configurator skill to create the field via Studio.
    After creation, verify the field exists with fields_get.
    Return: field technical name, field ID, creation status."
})
```

**NO es Agent()**: odoo-pilot (Bash inline), analyze_file.py (script local).

## Delegates to

| Skill | Suite | When | Agent()? |
|-------|-------|------|----------|
| odoo-pilot | common | CRUD, auth, fields_get | No |
| odoo-importation | common | Metodo load, batching, external IDs | **Yes — Pattern C** |
| odoo-configurator | consultant | Crear campos custom si mapeo lo requiere | **Condicional — Pattern C** |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| consultant-daily-planner | consultant | Detecta importaciones pendientes (data_file con respuesta) |

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Full 6-step workflow with API calls and templates |
| `references/mapeo_campos_comunes.md` | Known column-to-field mappings per model |
| `references/normalizacion_datos.md` | Normalization rules: countries, provinces, phones, dates |
| `templates/informe_importacion.md` | HTML template for internal note on task |
| `templates/validacion_cliente.md` | HTML template for client request update |
| `scripts/fetch_import_context.sh` | Extracts task context and finds attached files |
| `scripts/analyze_file.py` | Parses and analyzes Excel/CSV files |
| `scripts/download_attachment.py` | Downloads ir.attachment from Odoo |
