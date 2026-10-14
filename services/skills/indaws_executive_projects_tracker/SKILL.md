---
name: indaws-executive-projects-tracker
version: 2.7.4
effort: high
model: opus
description: >-
  Trackea la cartera de proyectos activos de Indaws desde la perspectiva del CEO. Lee Odoo
  (project.project en etapas whitelist), guarda snapshots locales con baseline + history + reviews,
  detecta 6 alertas (vencidas, sin fecha, sin movimiento, gap avance, sin checklist, update no
  enviado) y genera informe semanal con email por responsable. Usar cuando dirección pida el
  seguimiento de la cartera de proyectos.
---

# Executive Projects Tracker

## Workflow CEO (ciclo semanal)

```
LUNES  → /executive-projects-tracker review-all
           genera UN .md con email único POR RESPONSABLE
LUNES  → CEO copia emails y los envía (asunto: [PROJECT REVIEW] Seguimientos de proyecto <NOMBRE_PM>)
MARTES → Responsables contestan (máx. 24h) en el mismo hilo
MARTES → Agente lee hilo Outlook y registra respuestas con "respond <id> ..."
LUNES SIGUIENTE → review-all repite el ciclo con escalación si acción comprometida sigue activa
```

Ver `references/workflow-detail.md` para el detalle de cada modo (0–8).

> Note: `common:references/shared/` — auth via `auth.sh`, batch reads via `search_records.sh`.

## Field contract (preflight OBLIGATORIO)
Antes de leer Odoo, verifica que existen (ver `references/shared/field-contract-preflight.md`):
- modelo `project.project`: campo `x_studio_fecha_prevista_salida_produccin` (sin "ó" — no hardcodear con acento)
- modelo Studio `x_project_indaws_check` (debe existir): campos `x_studio_task_id`, `x_studio_peso`, `x_studio_completado`, `x_studio_hito`
Si falta algo → STOP con mensaje accionable (qué falta, dónde se configura en Studio). NO continúes con datos parciales.

## GOTCHAS

1. **El campo fin en Odoo se llama `date`, no `date_end`.** En snapshots locales lo renombramos a `date_end`.
2. **El campo de salida producción es `x_studio_fecha_prevista_salida_produccin`** (sin "ó", confirmado en `ir.model.fields`).
3. **JSON2 (Odoo 19) exige JSON estricto** — dominios con comillas simples se convierten en `search_records`.
4. **Movimiento se mide vs el último snapshot en `history`.** Si solo hay baseline (recién registrado), no disparará `no_movement` en la primera review.
5. **`set-current` mueve el current anterior a `history`** automáticamente (máx. 26 entradas).
6. **`add-review` siempre crea nueva entrada** — no sobreescribe.
7. **Si un proyecto sale de la whitelist**, no se filtra automáticamente. Marcar `complete` manualmente.
8. **Los emails NO se envían**, solo se redactan en el .md. El CEO los copia y envía manualmente.
9. **`fecha fin de proyecto` puede estar vencida sin que sea alerta.** Solo `production_date` es relevante para el recordatorio del email.
10. **"Enviado al cliente"** se detecta en `mail.message` con `subtype_id==1` (Discussions) y `partner_ids` no vacío. Notas internas no cuentan.
11. **`tasks.total` cuenta solo tareas no terminadas.** Las de `completed_task_stages` se excluyen de alertas.
12. **Para HTML Outlook-pegable usar SIEMPRE `render_consultant_report.py` (Modo 8).** No existe otro script HTML — `gen_html_email.py` fue eliminado en v2.7.0.
13. **`render_consultant_report.py` y deltas vs `history[-1]`**: si `set-current` se ejecuta dos veces el mismo día, la función `previous_snapshot()` cae a `history[-2]`. Si solo hay un snapshot, los deltas no se muestran.
14. **Carpeta de datos se resuelve en orden**: `EPT_DATA_DIR` > marcador `.executive-projects-tracker/projects/` en cwd (estilo git) > fallback `~/.indaws/executive-projects-tracker/projects/`.

## NEVER
- Modificar datos en Odoo. El skill es read-only sobre Odoo.
- Enviar el email — solo redactarlo en el .md.
- Borrar archivos JSON locales sin confirmación expresa.

## ALWAYS
- Validar `ODOO_URL`/`ODOO_DB`/`ODOO_KEY` antes de invocar scripts que consultan Odoo.
- Delegar la conexión a Odoo a `common-skills:odoo-pilot` (vía `auth.sh`).
- Confirmar al usuario antes de: alta (nombre/responsable/cadencia), alta masiva (lista previa), respuesta (qué proyecto + texto), cambio fecha (old→new).

## Delegates to

| Skill | Suite | Para qué |
|-------|-------|----------|
| `odoo-pilot` | common | Autenticación y búsquedas a Odoo (`auth.sh`) |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| migration-portfolio | migration-lab | Vista ejecutiva/roll-up de la cartera de migraciones |

## REFERENCES

- `references/odoo-fields-map.md` — Modelos y campos Odoo consumidos
- `references/drift-rules.md` — Detalle de las 6 alertas y lógica de escalación
- `references/workflow-detail.md` — Modos 0–8 completos, modelo de datos JSON, cadencia, estructura del informe
- `config.json` — Etapas whitelist, ruta de datos, umbrales, cadencia, etapas terminadas
- `scripts/discover_projects.sh` — Lista proyectos en etapas whitelist
- `scripts/fetch_project_state.sh` — Snapshot enriquecido de un proyecto
- `scripts/detect_alerts.py` — Detector de las 6 alertas + escalación
- `scripts/render_portfolio.py` — Informe Markdown agrupado por responsable
- `scripts/render_consultant_report.py` — **Único** generador HTML Outlook-pegable
- `scripts/manage_local.py` — CRUD de los JSON locales
