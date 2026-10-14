---
name: indaws-hr-recruitment
description: >
  Gestiona el pipeline de seleccion de personal en Indaws. Genera informes de funnel
  (etapas, tasas de conversion, cuellos de botella), analisis de time-to-hire,
  scoring de candidatos (0-100), efectividad de fuentes de captacion, scorecards
  de entrevista, componentes de carta oferta y dashboard de posiciones abiertas.
  Usar cuando el usuario pida: recruitment, seleccion, candidatos, pipeline de
  seleccion, time-to-hire, scoring de candidatos, vacantes, posiciones abiertas,
  oferta de trabajo, entrevistas, funnel de contratacion, hiring.
version: 1.1.1
effort: medium
model: sonnet
---

# HR Recruitment Pipeline

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-004

Gestiona el pipeline de seleccion en Indaws. Conecta con HiBob (Workforce Planning,
Job Catalog) y Odoo (`hr.applicant`, `hr.recruitment.stage`, `hr.job`) para analizar
el funnel, calcular time-to-hire, puntuar candidatos (0-100) y generar informes para
los hiring managers.

## Prerequisitos

- **HiBob API** y **Odoo XML-RPC**: credenciales en entorno (ver `lib/config.py`).
- **Python 3** para los scripts de extraccion, analisis y scoring.
- **Libreria compartida** `../../lib/` (hibob_client, odoo_hr_client, config, report_utils).
- **Config de scoring** `config/scoring_config.json` (pesos y criterios; ver `references/scoring.md`).

## Workflow

### Paso 0: Recopilacion (DETENTE)

PREGUNTAR: **tipo de informe** (`pipeline`, `candidate-score`, `candidate-brief`,
`time-to-hire`, `source-effectiveness`, `open-positions`); **filtros** opcionales
(departamento, periodo [def. 12 meses], puesto, estado); para scoring/brief: nombre,
puesto, CV/notas, hiring manager destinatario. DETENTE y espera respuesta.

### Paso 1: Extraccion

EJECUTAR `scripts/fetch_recruitment_data.py` con `--mode {pipeline|candidate|open-positions}`
(+ `--department`, `--months`, `--candidate-name`, `--job-id`), redirigiendo a JSON.
Extrae HiBob (Workforce Planning, Job Catalog) y Odoo (`hr.applicant`, etapas, `hr.job`,
historial de movimientos). Verificar: candidatos > 0, etapas configuradas, fechas pobladas.

### Paso 2: Calculo (OBLIGATORIO - nunca manual)

- Pipeline / time-to-hire / source-effectiveness → `scripts/pipeline_analyzer.py --input <json> --report-type <tipo>`.
- Scoring de candidato → `scripts/candidate_scorer.py --input <json> --config config/scoring_config.json --job-id <id>`.

`pipeline_analyzer.py`: distribucion por etapa, conversiones, cuellos de botella
(<30%), time-to-hire por depto/puesto/etapa, efectividad de fuentes, ofertas
aceptadas/rechazadas, aging de posiciones. `candidate_scorer.py`: score 0-100,
desglose por dimension y recomendacion. Algoritmo y formulas en `references/scoring.md`.

### Paso 2.5: Validacion (DETENTE)

PRESENTAR resumen (funnel con volumen/conversion + time-to-hire vs objetivo, o
score desglosado por dimension + recomendacion). DETENTE y espera aprobacion; el
usuario puede ajustar filtros, periodo o pesos.

### Paso 3: Generacion del Informe

SOLO tras aprobacion. Leer la plantilla y generar: `templates/pipeline_report.md`
(dashboard, funnel, cuellos de botella, time-to-hire, fuentes, aging, plan de accion),
`templates/candidate_brief.md` (datos, scoring, entrevistas, recomendacion) o
`templates/offer_template.md` (componentes NO sensibles; nunca salario). Guardar en
el directorio de trabajo y eliminar los JSON intermedios.

## Agent Routing

| Tarea | Agente / modelo |
|-------|-----------------|
| Extraccion, calculo y generacion de informe (flujo estandar) | SONNET (este skill, `effort: medium`) |
| Localizar codigo/etapas/pesos en `lib/`, `config/` o referencias | Explore |
| Diseno de nuevas metricas o cambios en el algoritmo de scoring | OPUS (revision) |

## GOTCHAS

- **Fuente de verdad doble**: Odoo `hr.applicant` es la fuente primaria de **datos de
  candidatos**; HiBob (Workforce Planning) es la primaria de **planificacion de plantilla**.
  No mezclar ni duplicar candidatos entre ambos.
- **Etapas del funnel personalizadas**: las etapas reales varian por proceso/departamento.
  Leer `hr.recruitment.stage` configurado en Odoo; no asumir el set estandar.
- **Time-to-hire: definir la fecha de inicio**. El computo cambia segun sourced vs applied
  vs apertura de `hr.job`. Por defecto Indaws usa apertura de la posicion; explicitarlo.
- **Scoring requiere scorecards completos**: `culture_fit` lee la media del scorecard de
  entrevista. Sin scorecards completados, esa dimension queda neutral (50) y sesga el total.
- **Datos de candidato (RGPD)**: candidato = dato personal con base legal y plazo de
  retencion propios. No tratar como empleado; respetar retencion y derecho de supresion.

## NEVER

- NUNCA calcular metricas ni scores manualmente: siempre via los scripts Python.
- NUNCA generar informes sin la aprobacion del Paso 2.5.
- NUNCA incluir salario, compensacion ni datos bancarios en ningun documento (RGPD).
- NUNCA exponer el scoring crudo (0-100 ni desglose) al candidato: es uso interno de RRHH.
- NUNCA compartir datos de un candidato con otro; cada brief es individual.
- NUNCA inventar datos: todo viene de HiBob y Odoo.

## ALWAYS

- SIEMPRE Odoo `hr.applicant` para candidatos; HiBob para planificacion de plantilla.
- SIEMPRE solo lectura: el skill extrae y reporta, no escribe en HiBob ni Odoo.
- SIEMPRE evaluar entrevistas con la rubrica de `references/interview_scorecard.md`.
- SIEMPRE tratar el scoring como apoyo, nunca decision automatica (validacion humana).
- SIEMPRE aplicar enmascaramiento RGPD (`report_utils`) y respetar la retencion del candidato.
- SIEMPRE eliminar los JSON intermedios tras generar el informe.

## Delegates to

- `lib/hibob_client.py` — Workforce Planning y Job Catalog de HiBob.
- `lib/odoo_hr_client.py` — `hr.applicant`, `hr.recruitment.stage`, `hr.job` de Odoo.
- `lib/report_utils.py` — formato y enmascaramiento RGPD de la salida.
- `Explore` — busqueda de codigo, etapas o pesos en `lib/`, `config/` y `references/`.

## Consumed by

- **hr-people-report** — consume metricas de funnel y time-to-hire para el informe mensual.
- **hr-onboarding** — toma el candidato seleccionado como entrada del pre-onboarding.

## REFERENCES

- `references/scoring.md` — algoritmo de scoring 0-100, pesos, clasificacion, etapas del funnel, time-to-hire.
- `references/hiring_process.md` — workflow end-to-end, SLAs y metricas de seguimiento.
- `references/interview_scorecard.md` — rubrica y plantilla de scorecard (escala 1-4).
- `references/job_catalog.md` — catalogo de roles por nivel.
- `lib/hibob_client.py`, `lib/odoo_hr_client.py`, `lib/report_utils.py`, `lib/config.py` — clientes y umbrales compartidos.
- `../../security/gdpr_guidelines.md` — directrices RGPD obligatorias (HR-SEC-001).
- `common:references/shared/` — N/A: HiBob/Odoo-HR via `lib/`.

## Success Criteria

- [ ] Datos extraidos de HiBob y Odoo; analisis ejecutado sin errores via script.
- [ ] Resumen presentado y aprobado (Paso 2.5).
- [ ] Funnel (conversion, volumen, time-to-hire) y cuellos de botella calculados.
- [ ] Scoring desglosado por dimension (si aplica).
- [ ] Informe generado segun plantilla; JSON intermedios eliminados.
- [ ] Ningun dato sensible expuesto (salario, bancarios, NIF/NIE); scoring no expuesto al candidato.
