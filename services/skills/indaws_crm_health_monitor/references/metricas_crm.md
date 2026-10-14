# Metricas CRM - Algoritmos y Formulas

Documento de referencia con todos los algoritmos de calculo del CRM Health Score,
scoring por comercial, analisis de demos y prevision mensual.

---

## 1. Datos de Entrada

> Fuente PRIMARIA: **Indaws Odoo MCP** (`orm_search_read`). Fallback: **odoo-pilot**
> (`fetch_crm_data.sh`). Las consultas exactas (model/domain/fields por clave JSON)
> estan en `workflow_detail.md` Paso 1.

### 1.1 Modelos Odoo

| Modelo | Contenido | Clave |
|--------|-----------|-------|
| `crm.lead` (activas) | Oportunidades abiertas (type=opportunity, active=True) | Pipeline actual |
| `crm.lead` (ganadas) | Oportunidades ganadas ultimos 12 meses (stage_id=won) | Win rate |
| `crm.lead` (perdidas) | Oportunidades perdidas ultimos 12 meses (active=False) | Win rate |
| `crm.lead` (todas 12m) | TODAS las opp ultimos 12 meses (activas + archivadas) | Win rate total |
| `crm.stage` | Etapas del pipeline CRM | Clasificacion de avance |
| `res.users` | Comerciales (user_id de oportunidades) | Scoring individual |

### 1.2 Query de Oportunidades Totales (12m)

Para obtener TODAS las oportunidades (activas e inactivas) se usa el filtro:

```
["|", ["active", "=", True], ["active", "!=", True], ["type", "=", "opportunity"], ["company_id", "=", 1], ["create_date", ">=", "FECHA_12M_ATRAS"]]
```

El operador `|` en notacion polaca aplica OR a las 2 siguientes condiciones,
asegurando que se obtienen tanto las activas como las archivadas.

### 1.3 Campos Clave de crm.lead

| Campo | Uso |
|-------|-----|
| `user_id` | Comercial asignado |
| `team_id` | Equipo de ventas |
| `stage_id` | Etapa actual en el pipeline |
| `probability` | Probabilidad de cierre (0-100) |
| `expected_revenue` | Ingreso esperado |
| `date_deadline` | Fecha prevista de cierre |
| `create_date` | Fecha de creacion |
| `write_date` | Ultima actualizacion |
| `date_last_stage_update` | Ultimo cambio de etapa |
| `date_closed` | Fecha de cierre (ganadas) |
| `priority` | Prioridad (0=normal, 1=alta, 2=muy alta) |
| `activity_state` | Estado de la próxima actividad: `False`/null=sin actividad, `planned`=programada, `today`=vence hoy, `overdue`=vencida |
| `activity_date_deadline` | Fecha límite de la próxima actividad programada |
| `activity_summary` | Descripción/resumen de la próxima actividad |
| `last_activity_date` | **Calculado**, no es un campo de Odoo: fecha del último mensaje del chatter del lead. Es la señal de estancamiento (ver 1.5) |

### 1.4 Demos via actividad (fuente única)

Los dos campos de Studio que guardaban "demo realizada" y "fecha de demo" fueron
**borrados sin sustituto**. Las demos se derivan del tipo de actividad **"Demo"**,
ligado a `crm.lead`.

El tipo se resuelve **POR NOMBRE**, nunca por id: `mail.activity.type` con
`name='Demo'` y `res_model='crm.lead'`. En producción es el 38, pero cambia entre
instancias.

| Dato | Modelo | Dominio |
|------|--------|---------|
| Demo **realizada** | `mail.message` | `[["model","=","crm.lead"],["mail_activity_type_id","=",<tipo>]]` (fecha = `date`) |
| Demo **agendada** | `mail.activity` | `[["res_model","=","crm.lead"],["activity_type_id","=",<tipo>]]` (pendiente) |

Una demo realizada genera un `mail.message` al marcar la actividad como hecha. El
motor mapea `res_id` → lead y usa la fecha de la PRIMERA demo.

> **GOTCHA.** Al cerrarse, una actividad queda **ARCHIVADA** (`active=False`).
> Cualquier consulta a `mail.activity` que deba ver las cerradas necesita el contexto
> `{"active_test": False}`, o devuelve casi nada. Aquí solo se piden las PENDIENTES
> (que están activas), por lo que no hace falta; si algún día se leen las cerradas, hay
> que añadirlo. Y la fecha de una demo hecha es **`date_done`**, no `date_deadline`.

Sin actividades Demo, la sección 4 se **omite sin penalizar** (antes la condición era
que faltase el campo; ahora, que no haya actividades).

### 1.5 Estancamiento: la fecha del chatter, no `write_date`

El campo de Studio que marcaba la última actualización fue borrado. Lo sustituye la
**fecha del último mensaje del chatter**, que el extractor calcula y mete en cada lead
como `last_activity_date`:

| | |
|---|---|
| Modelo | `mail.message` |
| Dominio | `[["model","=","crm.lead"],["res_id","in",<ids>]]` |
| Campos | `res_id`, `date` |
| Se toma | el `date` **máximo** por `res_id` |

Cascada de respaldo: `last_activity_date` → `date_last_stage_update` → `write_date`.

> **Por qué `write_date` va el ÚLTIMO, y no el primero.** `write_date` se mueve con
> CUALQUIER escritura, incluidas las automáticas de un cron o una importación masiva,
> así que rejuvenece oportunidades que nadie ha tocado. Medido en esta migración:
> estimando con `write_date` salían **2.645 oportunidades estancadas**; las reales eran
> **16**. El chatter solo se mueve cuando alguien hace algo — una nota, un correo, una
> llamada registrada.

Los umbrales por etapa se mantienen (Propuesta 10d, Negociación 7d, En espera 20d),
pero **la señal ha cambiado**: las cifras de estancamiento no son comparables con las de
informes anteriores. El informe debe decirlo.

---

## 2. CRM Health Score Global (0-100)

### 2.1 Formula Principal

```
S = clamp(100 - Pen_rendimiento - Pen_gobernanza, 0, 100)
```

Donde:
- `Pen_rendimiento` = suma de penalizaciones de rendimiento
- `Pen_gobernanza` = suma de penalizaciones de gobernanza

### 2.2 Penalizaciones de Rendimiento

| # | Penalizacion | Formula | Max | Cuando |
|---|-------------|---------|-----|--------|
| 1 | Win rate bajo | `max(0, (30 - win_rate)) * 0.5` | 15 | win_rate < 30% |
| 2 | Pipeline estancado | `stale_pct * 0.3` | 20 | Opps sin actividad segun umbral de etapa |
| 3 | Opp vencidas | `overdue_pct * 0.4` | 20 | Opps pasadas de date_deadline |
| 4 | Pipeline bajo | `max(0, (100 - coverage_pct)) * 0.15` | 15 | Solo si hay objetivo mensual |

> **NOTA**: El demo rate NO penaliza. Es informativo pero no obligatorio hacer demo.

**Calculos intermedios**:

```
win_rate = ganadas / total_oportunidades_12m * 100
  → total_oportunidades_12m = TODAS las opp (activas + archivadas) creadas en ultimos 12 meses
  → Si total_oportunidades_12m < 5: win_rate = 50 (dato insuficiente, sin penalizacion)

stale_pct = opp_estancadas / total_opp_pipeline * 100
  → "estancada" = sin actividad segun umbral por etapa (por id, Odoo 19):
    - Propuesta/Proposition (3): 10 dias
    - Negociación/Negotiation (27): 7 dias
    - En espera/On Hold (28): 20 dias
    - Análisis/Analysis (29): 30 dias
    - Otras etapas: stale_days (default 21)

overdue_pct = opp_vencidas / total_opp_pipeline * 100
  → "vencida" = date_deadline < hoy Y opp sigue abierta

coverage_pct = forecast_ponderado / objetivo_mensual * 100
  → Solo se calcula si el usuario proporciona objetivo_mensual
  → Si no hay objetivo: penalizacion = 0
```

### 2.3 Penalizaciones de Gobernanza

| # | Penalizacion | Formula | Max | Cuando |
|---|-------------|---------|-----|--------|
| 1 | Sin fecha cierre | `pct_sin_fecha * 0.15` | 10 | Opps sin date_deadline |
| 2 | Sin revenue | `pct_sin_revenue * 0.15` | 10 | expected_revenue = 0 o vacio |
| 3 | Sin comercial | `count_sin_user * 3` | 10 | Opps sin user_id |
| 4 | Sin actividad 30d | `pct_inactivas * 0.15` | 10 | write_date > 30 dias |
| 5 | Sin actividad futura | `pct_sin_act_futura * 0.2` | 15 | Opps sin next_activity programada |
| 6 | Actividad vencida | `pct_act_vencida * 0.25` | 10 | Opps con activity_state=overdue |

**Calculos intermedios**:

```
pct_sin_fecha = opp_sin_deadline / total_opp_pipeline * 100
pct_sin_revenue = opp_sin_revenue / total_opp_pipeline * 100
count_sin_user = count(opp donde user_id es False o null)
pct_inactivas = opp_inactivas_30d / total_opp_pipeline * 100
```

### 2.4 Clasificacion de Estado

| Score | Estado | Icono |
|-------|--------|-------|
| 80-100 | SALUDABLE | `🟢` |
| 60-79 | EN RIESGO | `🟡` |
| 40-59 | CRITICO | `🟠` |
| 0-39 | EMERGENCIA | `🔴` |

### 2.5 Barra Visual

```
30 caracteres para la barra. Caracteres llenos = round(S / 100 * 30).
Ejemplo (score=75): ██████████████████████▒▒▒▒▒▒▒▒ 75/100
```

---

## 3. Scoring por Comercial (0-100)

Cada comercial recibe un score individual basado en 5 penalizaciones.

### 3.1 Formula

```
Score_comercial = clamp(100 - sum(penalizaciones), 0, 100)
```

### 3.2 Penalizaciones Individuales

| # | Penalizacion | Que Mide | Formula | Max |
|---|-------------|----------|---------|-----|
| 1 | Pipeline | Valor pipeline vs media equipo | Ver 3.3 | 20 |
| 2 | Win rate | Tasa de ganancia vs media equipo | Ver 3.4 | 20 |
| 3 | Gobernanza | Campos incompletos | Ver 3.5 | 15 |
| 4 | Stale | % opp estancadas (por umbral de etapa) | Ver 3.6 | 15 |
| 5 | Actividad | % opp actualizadas | Ver 3.7 | 15 |

> **NOTA**: Demo rate NO penaliza. Se muestra como informativo pero no resta puntos.

### 3.3 Penalizacion Pipeline

```
pipeline_comercial = sum(expected_revenue) de opp activas del comercial
pipeline_media = sum(expected_revenue de todas las activas) / num_comerciales

Si pipeline_media > 0:
  ratio = pipeline_comercial / pipeline_media
  pen_pipeline = max(0, (1 - ratio)) * 20
Sino:
  pen_pipeline = 0
```

### 3.4 Penalizacion Win Rate

```
wr_comercial = ganadas_comercial / total_opp_12m_comercial * 100
wr_media = ganadas_total / total_opp_12m * 100

Si total_opp_12m_comercial < 3:
  pen_wr = 5  (dato insuficiente, penalizacion reducida)
Sino:
  pen_wr = min(20, max(0, (wr_media - wr_comercial)) * 0.5)
```

### 3.5 Penalizacion Gobernanza

```
opp_comercial = opp activas del comercial
sin_fecha = count(opp sin date_deadline) / total_opp_comercial * 100
sin_revenue = count(opp sin expected_revenue) / total_opp_comercial * 100

pen_gov = min(15, (sin_fecha + sin_revenue) * 0.15)
```

### 3.6 Penalizacion Stale

```
Para cada opp del comercial:
  umbral = stale_thresholds[stage_id] (10, 20 o 30 dias segun etapa)
  si ultima_actividad < hoy - umbral: es estancada

stale_pct = estancadas / total_opp_comercial * 100
pen_stale = min(15, stale_pct * 0.2)
```

### 3.7 Penalizacion Actividad

```
inactivas = count(opp del comercial con ultima_actividad < hoy - 30d)
inact_pct = inactivas / total_opp_comercial * 100

pen_actividad = min(15, inact_pct * 0.2)
```

---

## 4. Analisis de Demos

### 4.1 Metricas de Demos

| Metrica | Formula |
|---------|---------|
| Demos realizadas este mes | `count(mail.message tipo 38 con date en mes actual)` |
| Demos agendadas | `count(mail.activity tipo 38 pendiente)` |
| Demos por comercial | Agrupar demos realizadas por user_id del lead (via res_id) |
| Tasa de demo | `opp_pipeline_con_demo / opp_calificadas * 100` |
| Conversion con demo | `ganadas_con_demo / total_opp_12m_con_demo * 100` |
| Conversion sin demo | `ganadas_sin_demo / total_opp_12m_sin_demo * 100` |
| Dias demo→cierre | `avg(date_closed - fecha_demo)` para ganadas con demo |

> **NOTA**: La conversion se calcula usando el total de oportunidades de 12 meses
> (misma base que el win rate), separando las que tuvieron demo de las que no.

### 4.2 Determinacion de "Calificada"

```
etapas = crm.stage ordenadas por sequence ASC
min_sequence = etapas[0].sequence
calificada = opp donde stage.sequence > min_sequence
```

No se hardcodea ningun nombre de etapa. La deteccion es dinamica basada en sequence.

### 4.3 Sin Datos de Demo

Si no hay actividades Demo (`demos_done` y `demos_scheduled` vacios) — ya no existe
campo legacy al que recurrir:
- Skip completo del analisis de demos (`demo_analysis = None`)
- Penalizaciones de demo = 0 (el demo rate nunca penaliza, es informativo)
- Aviso en el informe: "Sin datos de demos. Analisis omitido."

---

## 5. Prevision Mensual

### 5.1 Oportunidades del Mes

```
opp_mes = opp activas donde date_deadline esta en el mes actual
```

### 5.2 Forecast Ponderado

```
forecast = sum(expected_revenue * probability / 100) para opp_mes
```

### 5.3 Cobertura

```
Si objetivo_mensual > 0:
  cobertura = forecast / objetivo_mensual * 100
Sino:
  cobertura = N/A (no se puede calcular)
```

### 5.4 Tabla de Oportunidades del Mes

Para cada opp_mes, mostrar TODAS (sin limite):
- Nombre de la opp
- Comercial (user_id)
- Revenue esperado
- Probabilidad
- Revenue ponderado (revenue * prob / 100)
- Etapa actual

---

## 6. Problemas de Gobernanza

5 tipos detectados automaticamente:

### 6.1 Sin Fecha de Cierre

```
opp donde date_deadline es False/null
Impacto: No aparece en previsiones mensuales
```

### 6.2 Sin Revenue Esperado

```
opp donde expected_revenue = 0 o es null/False
Impacto: Pipeline sin visibilidad economica
```

### 6.3 Sin Comercial Asignado

```
opp donde user_id es False/null
Impacto: Sin responsable
```

### 6.4 Vencidas y Abiertas

```
opp donde date_deadline < hoy Y sigue activa
Impacto: Pipeline inflado artificialmente
```

### 6.5 Sin Actividad 30+ Dias

```
opp donde ultima_actividad < (hoy - 30 dias)
Impacto: Pipeline muerto, oportunidades fantasma
```

### 6.6 Sin Actividad Futura (norma de actividad)

```
opp donde activity_state es False, null o vacío
Norma: toda oportunidad activa debe tener una actividad futura programada (next_activity).
Sin actividad programada = incidencia de gobernanza.
Impacto: Oportunidad sin seguimiento activo. Penalización: min(15, pct * 0.2)
```

### 6.7 Actividad Vencida

```
opp donde activity_state = "overdue" o activity_date_deadline < hoy
Impacto: Compromiso con el cliente incumplido. Penalización: min(10, pct * 0.25)
```

---

## 7. Criterios de Estancamiento por Etapa

Los umbrales de estancamiento varian segun la etapa del pipeline (por id, Odoo 19
verificado 2026-06-22). Lead (1) y Cualificado (26) son etapas early: no penalizan
gobernanza y usan el umbral generico stale_days (default 21).

| Etapa (id) | Umbral | Justificacion |
|------------|:------:|---------------|
| Propuesta (3) | 10 dias | Debe seguirse para respuesta |
| Negociación/Negotiation (27) | 7 dias | Fondo de embudo: no debe enfriarse |
| En espera/On Hold (28) | 20 dias | Aparcada con fecha de retomar (parked) |
| Análisis/Analysis (29) | 30 dias | Analisis mas profundo permitido |

**Etapas aparcadas** (`parked_stage_ids`, hoy `[28]` En espera / On Hold): son oportunidades
congeladas con fecha prevista de retomar la negociacion. NO son avance del embudo:
se EXCLUYEN del ranking ordinal de `stage_progress_pct` y reciben un valor bajo fijo
(`parked_progress_pct = 15`). Asi una opp en En espera no infla su closability aunque
su `sequence` sea alta (45).

---

## 8. Dashboard Extra - Indicadores Adicionales

### 8.1 Pedidos Cerrados

```
Mes actual: opp ganadas con date_closed en mes actual
  → count + sum(expected_revenue)
  → Desglose por comercial

Mes anterior: opp ganadas con date_closed en mes anterior
  → count + sum(expected_revenue)
  → Desglose por comercial
```

### 8.2 Oportunidades Nuevas

```
Mes actual: count(opp en all_12m con create_date en mes actual)
Mes anterior: count(opp en all_12m con create_date en mes anterior)
```

### 8.3 Oportunidades Asignadas por Comercial

```
Por cada comercial:
  Mes actual: count(opp en all_12m con create_date en mes actual Y user_id = comercial)
  Mes anterior: count(opp en all_12m con create_date en mes anterior Y user_id = comercial)
```

---

## 9. Forecast con Escenarios

> **DETERMINISTA**: lo calcula `calculate_crm_health.py` y sale en la clave
> `forecast_scenarios` del JSON (`best_case`, `likely_case`, `worst_case`,
> `commit`, `upside`, `stretch`, `gap_analysis`). NO recalcular a mano: leer del JSON.

### 9.1 Tres Escenarios de Forecast

Para las oportunidades que cierran este mes (date_deadline en mes actual):

```
BEST CASE = sum(expected_revenue) de opp con probability >= 30%
  → Incluye todo lo que tiene alguna posibilidad razonable

LIKELY CASE = sum(expected_revenue * probability / 100) de todas las opp del mes
  → Forecast ponderado por probabilidad (ya existente)

WORST CASE = sum(expected_revenue) de opp con probability >= 70%
  → Solo lo practicamente seguro
```

### 9.2 Commit vs Upside

```
COMMIT = sum(expected_revenue) de opp con probability >= 70% (alta confianza)
UPSIDE = LIKELY_CASE - COMMIT (revenue adicional posible)
STRETCH = BEST_CASE - LIKELY_CASE (revenue optimista adicional)
```

### 9.3 Gap Analysis (solo si hay objetivo mensual)

```
Si objetivo_mensual > 0:
  GAP_TO_QUOTA = objetivo_mensual - LIKELY_CASE
  COVERAGE_PCT = LIKELY_CASE / objetivo_mensual * 100
  COMMIT_COVERAGE = COMMIT / objetivo_mensual * 100

  Si GAP_TO_QUOTA > 0:
    → "Faltan {GAP} EUR para el objetivo"
    → "Se necesitan {ceil(GAP / avg_deal_size)} deals adicionales"
  Si GAP_TO_QUOTA <= 0:
    → "Objetivo cubierto. Upside: {abs(GAP)} EUR"
```

---

## 10. Priorizacion de Deals

> **DETERMINISTA**: lo calcula `calculate_crm_health.py` y sale en la clave
> `prioritization` del JSON (`top_deals`, `deals_week`, `max_revenue`). Cada deal
> trae `impact`, `closability`, `deal_score`, `recency_score`, `days_to_deadline` y
> `flags` (stale/overdue/missing_data/no_future_activity). NO recalcular: leer del JSON.

### 10.1 Deal Score (Impacto x Closability)

Para cada oportunidad activa:

```
IMPACT = normalize(expected_revenue, 0, max_revenue_in_pipeline) * 100
  → 0-100, proporcional al revenue dentro del pipeline

CLOSABILITY = (
  probability * 0.4 +                    # Probabilidad Odoo
  recency_score * 0.3 +                  # Actividad reciente
  stage_progress_score * 0.3             # Avance en pipeline
)
  → 0-100

recency_score:
  - write_date < 3 dias: 100
  - write_date < 7 dias: 80
  - write_date < 14 dias: 50
  - write_date < 30 dias: 20
  - write_date >= 30 dias: 0

stage_progress_score:
  - USAR `stage_progress_pct` del JSON (campo en stages_summary).
  - Es el RANGO ORDINAL de la etapa entre todas las etapas (Lead=mas bajo,
    Ganado=100), NO sequence/max_sequence.
  - IMPORTANTE: las secuencias de crm.stage NO son contiguas (Lead 1, Cualificado 2,
    Propuesta 3, Análisis 4, Negociación 5, Ganado 70, En espera 80). Calcular
    avance con sequence/max_sequence esta MAL: daria En espera 100% y Negociación 6%.
  → Etapas mas avanzadas (por rango) = mayor closability

DEAL_SCORE = (IMPACT * 0.5 + CLOSABILITY * 0.5)
```

### 10.2 Top 10 Deals

Ordenar todas las opp activas por DEAL_SCORE DESC. Mostrar top 10 con:
- Nombre, comercial, revenue, probability, etapa, deal score
- Flag si estancada, vencida o sin datos

### 10.3 Deals de la Semana

Filtrar top deals por urgencia temporal:
```
DEALS_SEMANA = opp donde:
  - date_deadline <= hoy + 7 dias, O
  - probability >= 70%, O
  - stage_id == 27 (Negociación) SIN actividad futura programada, O
  - DEAL_SCORE >= 70 Y recency_score >= 50
  Ordenar por DEAL_SCORE DESC, max 5

Negociación (27) es fondo de embudo: una opp ahi sin next_activity es la maxima
prioridad de la semana (riesgo de enfriamiento; umbral de estancamiento 7d).
```

---

## 11. Casos Especiales

### 9.1 Sin Oportunidades Activas

```
Si total_opp_activas = 0:
  score = 0
  estado = "EMERGENCIA"
  escenario = "LOW_PIPELINE"
  aviso: "No hay oportunidades activas en el pipeline."
```

### 9.2 Sin Datos de Oportunidades Totales

```
Si total_opp_12m < 5:
  win_rate = 50 (neutro)
  pen_win_rate = 0
  aviso: "Datos insuficientes para calcular win rate (< 5 opp en 12 meses)."
```

### 9.3 Sin Datos de Demo

```
Si no hay actividades Demo (demos_done y demos_scheduled vacios):
  demo_analysis = None
  Todas las metricas de demo = 0 o N/A
  Penalizaciones de demo = 0 (siempre; el demo rate es informativo)
  Seccion 4 del informe indica que no hay datos de demo
```

### 9.4 Sin Objetivo Mensual

```
Si no se proporciona objetivo_mensual:
  cobertura = N/A
  pen_pipeline_bajo = 0
  Seccion 3 omite indicador de cobertura
```
