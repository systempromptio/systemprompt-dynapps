---
name: indaws-crm-health-monitor
description: >
  Evalua el estado de salud del CRM de Odoo analizando pipeline, demos, win rate
  y rendimiento por comercial. Incluye forecast con escenarios (best/likely/worst),
  priorizacion de deals por impacto/closability, revision de higiene del pipeline,
  y genera emails semanales por comercial con revision de fechas de entrega y
  cierre de forecast (commit). Usar tambien cuando el usuario pida: forecast,
  pipeline review, revision de pipeline, prevision de ventas, deals prioritarios,
  emails a comerciales, revision de fechas de entrega, cierre de forecast semanal.
version: 1.7.1
model: opus
effort: high
---

# CRM Health Monitor

Evalua la salud del CRM de Odoo y genera un informe con CRM Health Score (0-100), forecast con escenarios, priorizacion de deals y plan de accion por escenario diagnosticado.

---

## Workflow

> Full detail in `references/workflow_detail.md`

Acceso a datos — orden ESTRICTO: si existe una tool `mcp__*__orm_search_read` en la sesion, usarla SIEMPRE (no intentar odoo-pilot/`auth.sh` en absoluto; el conector ya viene autenticado, sin credenciales). Solo si esa tool NO esta disponible, fallback **odoo-pilot** (`scripts/fetch_crm_data.sh`), y en ese caso `ODOO_URL` debe pasarse explicitamente en cada run — nunca hardcodeada y nunca `www.indaws.es` (ver GOTCHAS). Queries y paginacion en `references/workflow_detail.md`. See `references/shared/odoo-connection.md` for connection pattern.

1. **Recopilar info** - Preguntar objetivo mensual (opcional) y umbral estancamiento (default 21d). Con Indaws MCP no hacen falta credenciales; solo si se usa el fallback odoo-pilot. **STOP: esperar respuesta.**
2. **Extraer datos** - Via Indaws MCP (`orm_search_read` con `order:"id"` y paginacion por `offset` hasta agotar `truncated`, ver tabla de queries y paginacion en workflow_detail.md) y ensamblar el JSON; solo si el MCP no esta, fallback a `scripts/fetch_crm_data.sh` (odoo-pilot, `ODOO_URL` explicita). Verificar opp activas > 0 y etapas >= 2.
3. **Calcular metricas** - Ejecutar `scripts/calculate_crm_health.py` (OBLIGATORIO, nunca calcular manualmente). Genera CRM Health Score, scoring comerciales, forecast, gobernanza.
4. **Validar dashboard** - Presentar resumen al usuario (score, pipeline, win rate, penalizaciones, scoring comerciales). **STOP: esperar aprobacion.**
5. **Evaluar escenario** - Leer `references/patrones_diagnostico_crm.md`, personalizar acciones con datos reales del JSON.
6. **Generar informe** - Leer `templates/informe_crm.md`, generar 8 secciones. Guardar como `informe_salud_crm_{fecha}.md`.
7. **Generar emails comerciales** - Primero ejecutar `python3 scripts/analisis_deadlines.py --health crm_health.json --data crm_data_clean.json --date {fecha}` para obtener, por comercial, las 5 secciones deterministas: commit del mes (>= 70%), Negociacion en riesgo (etapa 27 sin actividad futura o estancada >7d), fechas de entrega En analisis (SIN FECHA / VENCIDA / vence en Nd), demos agendadas (proximos 7d) y demos sin avance (>14d). Con esa salida (nunca a mano), redactar para cada comercial con >= 3 opp en pipeline un email con la estructura por decisiones (cabecera+score / Cierra esta semana / No dejes que se enfrien / Higiene / Demos / cierre firmado Victor). Plantilla en `templates/email_comercial.md`. Guardar como `emails_comerciales_crm_{fecha}.md`. Detalle en `references/workflow_detail.md`.

---

## RULES

1. **NUNCA** calcules metricas manualmente. Siempre usa `calculate_crm_health.py`.
2. **NUNCA** generes el informe sin aprobacion del dashboard (paso 4).
3. **NUNCA** inventes datos. Todos los numeros vienen del JSON calculado.
4. Demo rate es INFORMATIVO, no penaliza (ni global ni por comercial).
5. Demos = actividad **"Demo"**, con el tipo resuelto POR NOMBRE (`mail.activity.type` con `name='Demo'` y `res_model='crm.lead'`; es 38 en produccion, pero cambia entre instancias). Realizadas via `mail.message` (`mail_activity_type_id=<tipo>`), agendadas via `mail.activity` pendiente. No hay otra fuente de demos. Sin actividades Demo, omitir seccion 4 (demo nunca penaliza).
5b. **GOTCHA**: al cerrarse, una actividad queda ARCHIVADA (`active=False`). Toda consulta a `mail.activity` que deba ver las cerradas necesita `{"active_test": False}` o devuelve casi nada. La fecha de una demo hecha es `date_done`, no `date_deadline`.
5c. **Estancamiento = ultima fecha del chatter** (`last_activity_date`), no `write_date`. Cascada: `last_activity_date` → `date_last_stage_update` → `write_date`. `write_date` va el ultimo porque lo mueve cualquier escritura automatica y dispara falsos estancados.
6. Sin objetivo mensual: penalizacion de pipeline bajo = 0, omitir cobertura.
7. < 5 opp totales en 12m: win rate = 50% (neutro, sin penalizacion).
8. Opp perdidas en Odoo tienen `active=False`. Si no se obtienen, avisar.
9. Scoring por comercial es relativo al equipo (vs media), no absoluto.
10. El informe esta dirigido al director comercial / responsable de ventas.
11. El email se estructura por decisiones (no como lista de datos): Cierra esta semana (commit) / No dejes que se enfrien (Negociacion en riesgo + En analisis sin fecha) / Higiene (actividades vencidas) / Demos. Omitir el bloque sin contenido, salvo el de commit (siempre lleva la pregunta de cierre).
12. El bloque "Cierra esta semana" lista el commit del comercial (opp con probability >= 70% que cierran el mes en curso, con importe) y le pide responder antes del viernes que compromete para esta semana.
13. La comprobacion de fechas + cierre de forecast es SEMANAL: se ejecuta en cada corrida del skill, no es puntual.
14. Los importes, fechas y el commit de los dos bloques nuevos provienen SIEMPRE de la salida de `scripts/analisis_deadlines.py`, nunca se calculan ni se estiman a mano.
15. Toda oportunidad activa debe tener actividad futura programada (next_activity). Opp sin actividad = incidencia de gobernanza que penaliza el score global y el individual del comercial.
16. Si un comercial tiene actividades vencidas (activity_state=overdue), su email incluye SIEMPRE el bloque 5.2b de actividades vencidas, con nombre de opp y días de retraso.

---

## GOTCHAS

- Opp perdidas requieren `active=False` en el domain, si no el win rate sera parcial
- Estancamiento usa umbrales por etapa (por id): 10d (Propuesta 3), 7d (Negociación 27), 20d (En espera 28), 30d (Análisis 29). La senal es `last_activity_date` (chatter), no `write_date`
- Forecast con escenarios (`forecast_scenarios`) y priorizacion (`prioritization`: top_deals/deals_week) son DETERMINISTAS, salen del JSON. No recalcular a mano.
- Priorizacion: usar `stage_progress_pct` de stages_summary (rango ordinal), NUNCA sequence/max_sequence (secuencias no contiguas: Ganado=70, En espera=80)
- En espera / On Hold (28) es etapa APARCADA (congelada con fecha de retomar): excluida del ranking ordinal, `stage_progress_pct` fijo bajo (15) via `parked_stage_ids`/`parked_progress_pct`. No infla closability.
- Gobernanza detecta 5 tipos de problemas automaticamente (sin duplicadas ni sin demo)
- < 3 comerciales: omitir analisis de balance de equipo
- Revenue siempre EUR con 2 decimales, porcentajes con 1 decimal
- Datos via Indaws Odoo MCP (`orm_search_read`); en runs headless/cron el conector puede faltar — entonces usar el fallback `fetch_crm_data.sh` (odoo-pilot). El fallback necesita Node.js (ensamblado JSON); el motor de calculo siempre Python 3.
- **indaws.es ya no es el Odoo.** `https://www.indaws.es` redirige (302) a la web de marketing `https://www.dynapps.es` (migracion de marca Indaws→Dynapps). Nunca usar `ODOO_URL=https://www.indaws.es` para auth/API — falla en silencio con "Invalid credentials" porque recibe HTML de una landing, no JSON-RPC. Para el fallback odoo-pilot, pedir la URL real del Odoo corporativo al usuario o usar el conector MCP.
- **El MCP es la fuente primaria, no un adorno.** Antes de intentar odoo-pilot/`auth.sh`, comprobar si hay una tool `mcp__*__orm_search_read` disponible en la sesion y usarla directamente. No requiere credenciales propias (sesion ya autenticada) y evita por completo la clase de fallo de auth.
- **El conector MCP trunca a ~300 filas por llamada.** Cualquier dataset con mas de ~300 registros (`leads_active`, `leads_lost`, `leads_all`) viene envuelto en `{"truncated": true, "total", "shown", "hint", "records"}`. Paginar con `offset += shown` y `order:"id"` fijo hasta agotar, o los totales/importes del informe saldran silenciosamente incompletos (falsos negativos de pipeline, forecast por debajo del real).
- **`odoo_aggregate` esta bloqueado para `crm.lead`, `mail.message` y `mail.activity`** en este servidor MCP ("Read not allowed"). No usarlo para contar estos modelos — usar `orm_search_read` paginado y contar en el propio script/agente.
- Opp "Análisis" / "Analysis" = stage_id 29 (Odoo 19). La fecha de entrega es `date_deadline` del crm.lead; False/vacio = SIN FECHA (incidencia a marcar en el email).
- Etapas reales (filtrar por id, NUNCA por nombre — los nombres cambiaron con One Odoo, los ids no): 1 Nuevo/New, 26 Cualificado/Qualified, 3 Propuesta/Proposition, 29 Análisis/Analysis, 27 Negociación/Negotiation, 4 Ganado/Won (is_won), 28 En espera/On Hold. NUNCA hubo etapa "Congelado": las perdidas se marcan con `lost_reason_id` y `active=False`, no con una etapa.
- Commit del comercial = sus opp con probability >= 70 y date_deadline dentro del mes en curso. Es la base de la pregunta de cierre semanal.
- activity_state en crm.lead: False/null = sin actividad (incidencia), "planned" = tiene actividad futura, "today" = vence hoy, "overdue" = vencida. Cualquier valor distinto de "planned"/"today" que no sea False se trata como "none".
- Si has_activity_fields=False en el summary del JSON, los campos de actividad no se extrajeron correctamente — informar al usuario y omitir las penalizaciones de actividad.

---

## Delegates to

- **Indaws Odoo MCP** (conector claude.ai) - Fuente primaria de datos via `orm_search_read`
- **odoo-pilot** (common) - Fallback de conexion/extraccion si el MCP no esta disponible

## Consumed by

- Ninguno actualmente

---

## Prerequisites

- **Indaws Odoo MCP**: Conector de Odoo (primario); si falta, se usa el fallback
- **odoo-pilot** (fallback): Skill de `ai-claude-common` para conexion via API. Requiere variables `ODOO_URL` (explicita en cada run, la URL real del Odoo — nunca `www.indaws.es`), `ODOO_DB`, `ODOO_KEY`, `ODOO_USER` y Node.js (solo si se usa este camino)
- **Python 3**: Para el motor de calculo

---

## References

| File | Description |
|------|-------------|
| `references/metricas_crm.md` | Algoritmos, formulas y pesos del CRM Health Score |
| `references/patrones_diagnostico_crm.md` | 6 escenarios de diagnostico y acciones recomendadas |
| `references/workflow_detail.md` | Pasos detallados del workflow, queries, validacion, formato dashboard |
| `scripts/fetch_crm_data.sh` | Extraccion de datos CRM (FALLBACK via odoo-pilot; primario = Indaws MCP) |
| `scripts/calculate_crm_health.py` | Motor de calculo - CRM Health Score, scoring, forecast, gobernanza |
| `scripts/analisis_deadlines.py` | Insumo semanal de emails: fechas de entrega (En analisis) y commit del mes (>= 70%) por comercial |
| `scripts/test_calculate_crm_health.py` | Tests del motor de calculo |
| `scripts/fixtures/sample_crm_data.json` | Datos de ejemplo para tests |
| `templates/informe_crm.md` | Template Markdown del informe (8 secciones) |
| `templates/email_comercial.md` | Template del email semanal por comercial (estructura por decisiones) |
