# CRM Health Monitor - Workflow Detail

Reference file with full workflow steps, data extraction, validation, and report generation details.

---

## Paso 0: Recopilacion de Informacion

PREGUNTAR al usuario los siguientes datos:

1. **Credenciales de Odoo** (solo si NO hay conector MCP en la sesion):
   - Si existe una tool `mcp__*__orm_search_read`, NO se necesita credencial alguna
     (el conector ya viene autenticado) — saltar este punto.
   - Solo para el fallback odoo-pilot: URL real de la instancia Odoo (pasada
     explicitamente en cada run; NUNCA `https://www.indaws.es`, que redirige a la
     web de marketing dynapps.es), base de datos, API key o credenciales.

2. **Objetivo mensual de ventas** (opcional):
   - Revenue target mensual en EUR
   - Si no se proporciona, se omite el indicador de cobertura

3. **Umbral de estancamiento** (opcional, default: 21 dias):
   - Dias sin actividad para considerar una oportunidad estancada
   - Valor por defecto: 21 dias

> **DETENTE** aqui y espera la respuesta del usuario antes de continuar.

---

## Paso 1: Conexion y Extraccion de Datos

**Deteccion de transporte (orden ESTRICTO):**

1. Si existe una tool `mcp__*__orm_search_read` en la sesion → usarla SIEMPRE
   (Opcion A). NO intentar odoo-pilot ni `auth.sh` en absoluto: el conector ya
   viene autenticado y no requiere `ODOO_URL/ODOO_DB/ODOO_KEY/ODOO_USER`.
2. Solo si esa tool NO esta disponible → fallback odoo-pilot (Opcion B),
   exigiendo `ODOO_URL` explicita en cada run (nunca hardcodeada en un
   scheduled task, nunca `www.indaws.es`).

### Opcion A (preferente) — Indaws MCP

Ejecutar estas consultas con `orm_search_read` y ensamblar el JSON que consume
`calculate_crm_health.py`. `DATE_LIMIT` = hoy menos 12 meses. `LEAD_FIELDS` =
`["id","name","partner_id","user_id","team_id","stage_id","probability","expected_revenue","date_deadline","create_date","write_date","date_last_stage_update","date_closed","priority","active","activity_date_deadline","activity_state","activity_summary","activity_type_id"]`.

Los dos campos de Studio de demo y de ultima actualizacion fueron BORRADOS: ya no se
piden. La senal de estancamiento se calcula aparte (`last_activity_date`, ver abajo).

| Clave JSON | model | domain | fields |
|------------|-------|--------|--------|
| `leads_active` | crm.lead | `[["type","=","opportunity"],["company_id","=",1]]` | LEAD_FIELDS |
| `leads_won` | crm.lead | `[["type","=","opportunity"],["company_id","=",1],["stage_id","=",4],["date_closed",">=",DATE_LIMIT]]` | LEAD_FIELDS |
| `leads_lost` | crm.lead | `[["type","=","opportunity"],["company_id","=",1],["active","=",false],["stage_id","!=",4],["date_last_stage_update",">=",DATE_LIMIT]]` | LEAD_FIELDS |
| `leads_all` | crm.lead | `["|",["active","=",true],["active","!=",true],["type","=","opportunity"],["company_id","=",1]]` | LEAD_FIELDS |
| `stages` | crm.stage | `[]` | `["id","name","sequence","is_won","fold"]` |
| `sale_orders` | sale.order | `[["state","in",["sale","done"]],["company_id","=",1],["date_order",">=",DATE_LIMIT]]` | `["name","partner_id","user_id","date_order","amount_untaxed","state"]` |
| `demo_type` | mail.activity.type | `[["name","=","Demo"],["res_model","=","crm.lead"]]` | `["id","name"]` |
| `demos_done` | mail.message | `[["model","=","crm.lead"],["mail_activity_type_id","=",DEMO_TYPE_ID],["date",">=",DATE_LIMIT]]` | `["res_id","date","author_id"]` |
| `demos_scheduled` | mail.activity | `[["res_model","=","crm.lead"],["activity_type_id","=",DEMO_TYPE_ID]]` | `["res_id","date_deadline","user_id","state"]` |
| `chatter_last` | mail.message | `[["model","=","crm.lead"],["res_id","in",<ids de los leads>]]` | `["res_id","date"]` |
| `users` | res.users | `[["id","in",<ids unicos de user_id de las opp>]]` | `["name","login"]` |

`DEMO_TYPE_ID` se RESUELVE de `demo_type`, nunca se escribe a pelo: es 38 en produccion
pero cambia entre instancias. Si no existe el tipo, se omiten las demos (seccion 4 fuera,
sin penalizar).

De `chatter_last` se toma el `date` MAXIMO por `res_id` y se inyecta en cada lead como
`last_activity_date`: es la senal de estancamiento que sustituye al campo borrado. NO usar
`write_date` como senal principal — se mueve con escrituras automaticas de cron o
importacion (medido: 2.645 estancadas falsas frente a 16 reales).

> **GOTCHA `mail.activity`**: las actividades cerradas quedan ARCHIVADAS (`active=False`).
> Cualquier consulta que deba verlas necesita el contexto `{"active_test": False}` o
> devuelve casi nada. `demos_scheduled` pide solo pendientes (activas), por lo que no le
> afecta. La fecha de una demo hecha es `date_done`, no `date_deadline`.

#### Paginacion OBLIGATORIA (el proxy MCP trunca a ~300 filas por llamada)

Cuando el resultado supera el limite, el proxy devuelve:

```json
{"truncated": true, "total": N, "shown": M, "hint": "...offset=M...", "records": [...]}
```

Para CADA query de la tabla:

- Añadir SIEMPRE `order: "id"` para que la paginacion sea estable.
- Repetir la llamada con `offset += shown` hasta que `truncated` sea `false`/ausente
  o la pagina devuelta tenga menos filas que el limite pedido.
- NUNCA asumir que el `total` reportado es el recuento real definitivo mientras siga
  habiendo `truncated: true` — seguir paginando hasta agotar.
- Afecta sobre todo a `leads_active` (~677 filas), `leads_lost` (puede superar 1000)
  y `leads_all` (varios miles, sin filtro de fecha).
- NO usar `odoo_aggregate` para contar `crm.lead`, `mail.message` ni `mail.activity`
  (bloqueado en este servidor: "Read not allowed") — contar sobre los registros paginados.

Ensamblar tambien `summary` con: `has_demo_activity` (`demos_done`/`demos_scheduled` no vacios), `has_activity_fields`
(algun lead trae `activity_state`), `lost_available` (`leads_lost` no vacio) y los counts.

### Opcion B (fallback) — odoo-pilot

Solo si NO existe la tool MCP. `ODOO_URL` debe ser la URL real del Odoo corporativo,
pasada explicitamente en cada run. NUNCA `https://www.indaws.es`: redirige (302) a la
web de marketing `dynapps.es` y la auth falla siempre con "Invalid credentials"
(recibe HTML de una landing, no JSON-RPC). Si no se conoce la URL real, preguntarla
al usuario en el Paso 0 — no adivinarla.

```bash
export ODOO_URL="<url real del Odoo>"; export ODOO_DB="<db>"; export ODOO_KEY="<key>"; export ODOO_USER="<user>"
bash scripts/fetch_crm_data.sh --pilot-dir /path/to/odoo-pilot/scripts > crm_data.json
```

El script hace exactamente las mismas consultas y produce el mismo JSON (y aborta
con error si `ODOO_URL` apunta a `indaws.es`).

**Verificar (ambas opciones)**:
- Que se obtuvieron oportunidades activas (si 0 -> pipeline vacio, el score sera 0)
- Que se obtuvieron etapas (minimo 2 para calcular "calificada")
- Si las oportunidades perdidas no se pudieron obtener, el win rate sera parcial

---

## Paso 2: Calculo de Metricas (OBLIGATORIO)

EJECUTAR el script Python `scripts/calculate_crm_health.py`:

```bash
# Sin objetivo mensual
python3 scripts/calculate_crm_health.py --input crm_data.json > crm_health.json

# Con objetivo mensual
python3 scripts/calculate_crm_health.py --input crm_data.json --target 50000 > crm_health.json

# Con umbral personalizado de estancamiento
python3 scripts/calculate_crm_health.py --input crm_data.json --target 50000 --stale-days 14 > crm_health.json
```

> **IMPORTANTE**: Este paso es OBLIGATORIO. NUNCA calcules las metricas manualmente.
> El script aplica todas las formulas documentadas en `references/metricas_crm.md`.

**El script calcula**:
- CRM Health Score global (0-100) con penalizaciones
- Scoring individual por comercial (0-100)
- Analisis de demos (si el campo existe)
- Prevision mensual con forecast ponderado
- 5 tipos de problemas de gobernanza
- Escenario de diagnostico

LEER el JSON generado (`crm_health.json`) para continuar.

---

## Paso 2.5: Validacion del Dashboard

PRESENTAR al usuario un resumen del dashboard con los datos calculados:

```
CRM Health Score: {icono} {estado} {score}/100

Pipeline: {total_active} oportunidades | {total_revenue} EUR
Win rate (12m): {win_rate}% ({total_won} ganadas / {total_all_12m} totales)
Estancadas: {stale_count} ({stale_pct}%) - umbrales por etapa: 10d/20d/30d
Vencidas: {overdue_count} ({overdue_pct}%)
Demos: {demo_rate}% de calificadas con demo (informativo, no penaliza)

Pedidos cerrados mes actual: {count} pedidos | {revenue} EUR
Pedidos cerrados mes anterior: {count} pedidos | {revenue} EUR
Opp nuevas este mes: {count}

Escenario: {scenario_code} - {scenario_desc}

Penalizaciones:
  Rendimiento: -{pen_rendimiento} (win rate: {p1}, stale: {p2}, overdue: {p3}, pipeline: {p4})
  Gobernanza: -{pen_gobernanza} (fecha: {g1}, revenue: {g2}, comercial: {g3}, actividad: {g4})

Scoring comerciales:
  {comercial_1}: {score_1}/100 ({estado_1})
  {comercial_2}: {score_2}/100 ({estado_2})
  ...
```

> **DETENTE** aqui y espera la aprobacion del usuario antes de generar el informe.
> El usuario puede pedir ajustes (ej: cambiar objetivo mensual, revisar datos).

---

## Paso 3: Evaluacion y Plan de Accion

LEER `references/patrones_diagnostico_crm.md` para:

1. **Identificar el escenario principal** (ya calculado en el JSON):
   - HEALTHY_PIPELINE, LOW_PIPELINE, STALE_PIPELINE,
     LOW_WIN_RATE, GOVERNANCE_ISSUES, UNBALANCED_TEAM

2. **Seleccionar acciones** del escenario correspondiente:
   - Acciones inmediatas (esta semana)
   - Acciones a corto plazo (proximas 2 semanas)
   - Acciones estrategicas

3. **Personalizar acciones** con datos reales:
   - Usar nombres de oportunidades/comerciales reales del JSON
   - Adaptar las recomendaciones al contexto especifico
   - Priorizar las acciones segun las penalizaciones mas altas

---

## Paso 4: Generacion del Informe

> **SOLO** generar el informe tras la aprobacion del Paso 2.5.

LEER `templates/informe_crm.md` y generar el informe completo:

1. **Seccion 1 - Dashboard CRM**: Score, indicadores clave, pedidos cerrados (mes actual/anterior), opp nuevas, opp asignadas por comercial, escenario
2. **Seccion 2 - Vision del Pipeline**: Tablas por etapa y por comercial (pipeline ponderado, columna cierre este mes)
3. **Seccion 3 - Prevision del Mes**: TODAS las oportunidades que cierran este mes, forecast, cobertura
4. **Seccion 3B - Forecast con Escenarios**: Best/likely/worst case, commit vs upside, gap-to-quota
5. **Seccion 3C - Priorizacion de Deals**: Top 10 deals por impacto x closability, deals de la semana
6. **Seccion 4 - Analisis de Demos**: Metricas de demo, por comercial (sin tabla de calificadas sin demo)
7. **Seccion 5 - Scoring por Comercial**: Tabla resumen con num opp, pipeline ponderado, scores individuales
8. **Seccion 6 - Detalle por Comercial**: Bloque detallado con opp vencidas (total + mes anterior)
9. **Seccion 7 - Problemas de Gobernanza**: 5 tipos con listas detalladas
10. **Seccion 8 - Plan de Accion**: Acciones del escenario detectado (no considerar falta de demos)

### Reglas de generacion del informe

- Si no hay actividades Demo: omitir seccion 4 con aviso (no penaliza)
- Si no hay objetivo mensual: omitir filas de objetivo/cobertura en seccion 3
- Si hay menos de 3 comerciales: omitir analisis de balance de equipo
- Usar datos reales del JSON, nunca inventar nombres o cifras
- Revenue siempre en EUR con 2 decimales
- Porcentajes con 1 decimal

GUARDAR el informe como `informe_salud_crm_{fecha}.md`.

---

## Paso 5: Generacion de Emails Comerciales (semanal)

Este paso se ejecuta en CADA corrida del skill (no es puntual) y produce un email por comercial con >= 3 opp en pipeline.

### 5.1 Calcular el insumo (obligatorio, nunca a mano)

EJECUTAR el script que deriva los dos bloques de forma determinista:

```bash
python3 scripts/analisis_deadlines.py \
  --health crm_health.json \
  --data crm_data_clean.json \
  --date {YYYY-MM-DD} > analisis_deadlines.txt
```

La salida agrupa, por comercial, CINCO secciones deterministas (todos los importes,
fechas, commit, negociaciones y demos provienen de aqui; no se recalculan a mano):

1. **Commit del mes (>=70%)** — opp con `probability >= 70` que cierran el mes, con total.
2. **Negociacion en riesgo** — opp en etapa 27 (Negociacion) sin actividad futura o estancadas >7d.
3. **Fechas de entrega (En analisis)** — opp en etapa 29 con estado de `date_deadline` (SIN FECHA / VENCIDA / vence en Nd).
4. **Demos agendadas (proximos 7d)** — `mail.activity` tipo 38 pendientes esta semana.
5. **Demos sin avance (>14d)** — demos realizadas (`mail.message` tipo 38) cuya opp no ha avanzado de etapa.

### 5.2 Estructura del email (orden por decision, no por dato)

El email se ordena en tres decisiones para la semana, no como lista de datos. Todos
los datos salen del insumo (5.1); nunca se inventan nombres ni cifras.

**0. Cabecera**
- Saludo por nombre, tono tu (nunca usted).
- Score individual y estado (SALUDABLE / EN RIESGO / CRITICO) + una linea de lectura:
  que es lo que mas le esta restando puntos.

**1. Cierra esta semana** (bloque commit)
- Listar el commit del comercial (seccion 1 del insumo), con importe y total.
- Preguntar explicitamente que se compromete a cerrar esta semana; pedir respuesta antes del viernes.

**2. No dejes que se enfrien** (riesgo de resbalon)
- **Negociacion en riesgo** (seccion 2): cada opp con el motivo (sin actividad futura / Nd sin avance). Es el bloque de mayor valor: fondo de embudo enfriandose.
- **En analisis sin fecha o vencida** (seccion 3): marcar SIN FECHA como incidencia y pedir confirmar/reprogramar las que vencen <=7d.
- Argumentar por que cada caso importa (riesgo de forecast, compromiso con el cliente), no listar sin contexto.

**3. Higiene que te penaliza**
- **Actividades vencidas** (`activity_state = "overdue"`): nombre de opp, dias de retraso, resumen. Riesgo: el cliente interpreta el silencio como desinteres. Omitir si no hay.
- Opp sin actividad futura programada (norma de actividad).

**4. Demos** (solo si hay datos)
- **Demos agendadas esta semana** (seccion 4): recordatorio con fecha.
- **Demos sin avance >14d** (seccion 5): demos hechas cuya opp sigue parada; senal de seguimiento perdido.

**5. Cierre** firmado como Victor.

Respetar la marca Indaws: sin emojis, sin bullets superficiales (argumentar cada
recomendacion), vocabulario reto (no problema), inversion (no coste), propuesta de
valor (no presupuesto). Si un bloque no tiene contenido para ese comercial, omitirlo
(no escribir "ninguno"), salvo el commit, que siempre lleva la pregunta de cierre.

Plantilla de referencia del email: `templates/email_comercial.md`.

GUARDAR todos los emails como `emails_comerciales_crm_{fecha}.md`.

---

## Estructura de Archivos

```
crm-health-monitor/
├── SKILL.md                           # Skill principal
├── scripts/
│   ├── fetch_crm_data.sh             # Extraccion FALLBACK (odoo-pilot); primario = Indaws MCP
│   ├── calculate_crm_health.py       # Motor de calculo (OBLIGATORIO)
│   ├── analisis_deadlines.py         # Insumo semanal de emails (fechas + commit)
│   ├── test_calculate_crm_health.py  # Tests del motor de calculo
│   └── fixtures/
│       └── sample_crm_data.json      # Datos de ejemplo para tests
├── references/
│   ├── metricas_crm.md               # Algoritmos y formulas
│   ├── patrones_diagnostico_crm.md   # Escenarios y acciones
│   └── workflow_detail.md            # Este archivo - workflow detallado
└── templates/
    └── informe_crm.md                # Template del informe
```

---

## Success Criteria

- [ ] Datos extraidos correctamente (opp activas, ganadas, perdidas, etapas, usuarios)
- [ ] `calculate_crm_health.py` ejecutado sin errores
- [ ] Dashboard presentado y aprobado por el usuario
- [ ] CRM Health Score calculado (0-100) con penalizaciones detalladas
- [ ] Scoring individual por comercial calculado
- [ ] Escenario de diagnostico identificado correctamente
- [ ] Plan de accion personalizado con datos reales
- [ ] Informe Markdown generado con las 8 secciones
- [ ] Fichero guardado como `informe_salud_crm_{fecha}.md`
- [ ] `analisis_deadlines.py` ejecutado; emails generados por comercial con >= 3 opp
- [ ] Cada email contiene los dos bloques fijos: revision de fechas de entrega y cierre de forecast
- [ ] Emails guardados como `emails_comerciales_crm_{fecha}.md`
- [ ] Governance incluye sin_actividad_futura y actividad_vencida con sus listas de opp afectadas
- [ ] Emails de comerciales con actividades vencidas incluyen el bloque 5.2b
