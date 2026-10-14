---
name: indaws-project-weekly-update
description: >-
  Genera actualizaciones semanales de proyecto Odoo y crea registro project.update con contenido
  HTML (español o inglés) para enviar al cliente. Ejecuta automáticamente project-health-monitor
  como primer paso. Un proyecto por invocación. Usar cuando haya que generar la actualización
  semanal del proyecto para el cliente.
version: 2.3.0
effort: medium
model: sonnet
---

# Actualizacion Semanal de Proyecto Odoo

Genera una actualizacion semanal profesional para enviar al cliente, creando
un registro `project.update` en Odoo. Lee datos frescos de
`project-health-monitor` (ejecutado automaticamente como Paso 1) + timesheets
semanales. El unico rol creativo del LLM es redactar narrativa y proximos
pasos; todas las cifras vienen precalculadas y se escriben via
`create_update.py` (no se permite al LLM componer la llamada a Odoo).

**Regla dura**: un proyecto por invocacion del skill. Para procesar varios
proyectos de golpe, ver `references/batch_mode.md` (subagentes paralelos).

## Workflow

See `references/workflow-detail.md` for step-by-step commands and field mappings.

1. **Recopilar datos** (Paso 0): Credenciales Odoo + ID proyecto + semana. STOP.
2. **Ejecutar project-health-monitor** (Paso 1, AUTOMATICO): Lanza
   `fetch_project_data.sh` + `calculate_health.py` escribiendo en
   `/tmp/pwu_{id}_phm_*.json`. Valida score/estado. Si falla, aborta.
3. **Extraer datos semanales** (Paso 2): `fetch_weekly_data.sh` → `/tmp/pwu_{id}_weekly.json`.
4. **Detectar update existente** (Paso 2.5): Si ya existe, STOP y preguntar.
5. **Preparar datos** (Paso 3): `prepare_update.py` → `/tmp/pwu_{id}_update_data.json`.
6. **Generar HTML** (Paso 4): UNICO paso LLM. Redactar narrativa en espanol.
   Guardar en `/tmp/pwu_{id}_description.html`.
7. **Validar borrador** (Paso 4.5): STOP obligatorio. Presentar HTML + resumen.
8. **Escribir registro** (Paso 5): Via `create_update.py` (obligatorio). El
   script valida salud, idioma y nombre canonico (fail-fast), crea el
   `project.update` solo con campos estandar y publica la salud como NOTA
   INTERNA en el chatter del proyecto (cero correos).

See references/shared/odoo-connection.md for connection pattern.

### Content Rules (for Step 4)

- **Idioma**: TODO el HTML en el idioma solicitado (`es` default, `en` si
  el cliente es internacional). `create_update.py` aborta si detecta
  contaminacion del idioma opuesto (marcadores mezclados).
- NUNCA mostrar horas al cliente. Los timesheets se leen para entender QUE
  se hizo, no se muestran.
- Tono: profesional, voz pasiva ("Se ha configurado...").
- NO mencionar: "checklists ponderados", "Health Score numerico", "risk score",
  "bottleneck", "penalizacion".
- Health Score numerico se traduce a lenguaje cualitativo (ver tabla en
  workflow-detail.md).
- HTML: estilos inline obligatorios, color principal `#875A7B`.

## GOTCHAS

1. **Batch sin subagentes**: El fallo mas documentado. Procesar varios
   proyectos en una sola conversacion provoca que el LLM salte
   `project-health-monitor`, reutilice datos del proyecto anterior y
   reescriba el nombre en ingles. SIEMPRE usa subagentes paralelos para
   multiples proyectos — ver `references/batch_mode.md`.
2. **project-health-monitor saltado**: Es el Paso 1, no un prerequisito
   opcional. Se ejecuta automaticamente. Si se omite, `create_update.py`
   detecta `health_score=0` + estado vacio (o sin JSON de metricas) y aborta.
3. **Paths `/tmp/phm_*.json` fijos**: Colisionan en batch. Este skill usa
   `/tmp/pwu_{project_id}_*.json` para aislamiento total.
4. **Crear registro con llamada manual**: Prohibido. Usa siempre
   `create_update.py`, que garantiza nombre canonico, idioma espanol y
   nota interna de salud. El LLM NO debe componer `create_record.sh` ni
   `message_post` directamente.
5. **Horas expuestas al cliente**: Los timesheets son datos internos. NUNCA
   mostrar tabla de horas, desglose ni horas en narrativa.
6. **Recalcular metricas manualmente**: Todas las cifras vienen precalculadas
   en el JSON. Sumar horas o recalcular porcentajes introduce errores.
7. **Update duplicado**: Si ya existe `project.update` para la semana,
   actualizar (con `--update-id`), no crear uno nuevo.
8. **CSS externo en emails**: Odoo no procesa CSS externo en
   `project.update.description`. Todo debe ser estilos inline.
9. **Escribir sin validacion**: En modo interactivo (un proyecto), NUNCA
   saltar Paso 4.5. En modo batch con subagentes, la validacion se delega
   al orquestador (que valida la lista de proyectos con el usuario antes
   de lanzar) + `create_update.py` como red de seguridad final. Ver
   `references/batch_mode.md`.
10. **Salud fuera de project.update**: los 16 campos `x_studio_*` de salud
    (`x_studio_health_score`, `x_studio_internal_report`, `x_studio_fecha_inicio`...)
    se eliminaron en Odoo; escribirlos da "Invalid field". NUNCA leerlos ni
    escribirlos. `create_update.py` aborta si se cuela alguno en `odoo_values`.
    La salud (y el informe interno si se paso `--health-report`) va en la nota
    interna del chatter de `project.project`, via `health_note.py` de
    project-health-monitor.
11. **Nota de salud duplicada**: `create_update.py` ya publica la nota. NO
    ejecutar ademas el Paso 4.5 de project-health-monitor en el mismo ciclo.

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| project-health-monitor | consultant | Paso 1 automatico (cada invocacion) + nota interna de salud en Paso 5 | Llamada directa a sus scripts (`health_note.py` importado por `create_update.py`) |
| odoo-pilot | common | Fetch semanal (Paso 2) | fetch_weekly_data.sh |

## Consumed by

Standalone. Se invoca directamente por el PM al final de cada semana, o
en paralelo por subagentes para procesar varios proyectos (ver batch_mode.md).

## References

| File | Content | Load when |
|------|---------|-----------|
| `references/workflow-detail.md` | Pasos detallados y comandos | Always |
| `references/contenido_actualizacion.md` | Guia de contenido narrativo para el cliente | Paso 4 |
| `references/odoo-field-mapping.md` | Campos `project.update` escritos + contenido de la nota interna | Solo si se debuggea Paso 5 |
| `references/batch_mode.md` | Patron de subagentes paralelos para varios proyectos | Solo si el usuario pide procesar varios proyectos |

## Scripts

| Script | When |
|--------|------|
| `scripts/fetch_weekly_data.sh` | Paso 2 — extrae timesheets + checklists |
| `scripts/prepare_update.py` | Paso 3 — agrega datos numericos y genera odoo_values canonicos |
| `scripts/create_update.py` | Paso 5 — escribe el registro en Odoo con fail-fast y publica la nota interna de salud |

## Templates

| Template | When |
|----------|------|
| `templates/weekly_update_html.md` | Paso 4 — estructura HTML del update |
