---
name: indaws-hr-people-report
description: >
  Genera el informe mensual de RRHH con 15+ KPIs y HR Health Score (0-100).
  Analiza headcount, rotacion, antiguedad, staffing, ausencias y rendimiento por
  departamento. Incluye plan de accion basado en diagnostico automatico.
  Usar cuando el usuario pida: informe de personas, people report, HR report,
  headcount, rotacion, turnover, HR health, estado de RRHH, KPIs de personas,
  people analytics, informe mensual RRHH.
version: 1.1.1
model: sonnet
effort: medium
---

# HR People Report

**CONFIDENTIAL - HR TEAM ONLY** · **Document ID:** HR-SKILL-001

Informe mensual de RRHH: 15+ KPIs, HR Health Score (0-100), análisis por departamento y
plan de acción. Fuentes: HiBob (primaria) + Odoo hr.employee (secundaria), vía `../../lib/`.
Detalle de KPIs, fórmula del score y plantilla en `references/` (ver REFERENCES).

Todos los comandos se ejecutan desde el directorio de este skill. No uses rutas absolutas
de máquina; usa rutas relativas (`scripts/...`).

## Pipeline

**Paso 0 — Recopilar (DETENTE).** Preguntar: periodo (mes/año; default mes anterior),
confirmar credenciales HiBob+Odoo activas, filtros opcionales (departamento, ubicación,
incluir inactivos), objetivo de plantilla opcional, y destinatarios de email. Esperar respuesta.

**Paso 1 — Extraer.** `bash scripts/fetch_hr_data.sh > hr_data.json`. Extrae HiBob (people,
reports, timeoff, positions) + Odoo (hr.employee, hr.department). Verificar que el JSON trae
`people`, `reports`, `timeoff`, `positions`, `odoo_employees`, `odoo_departments`. Si HiBob
devuelve 0 → error de conexión. Si Odoo falla → continuar solo con HiBob y avisar.

**Paso 2 — Calcular (OBLIGATORIO, nunca manual).**
`python3 scripts/calculate_hr_health.py --input hr_data.json [--target-headcount N] > hr_health.json`.
Calcula el score (0-100) con desglose por dimensión, los 15+ KPIs con tendencia, distribución,
ranking de departamentos, alertas y escenario. Aplica las fórmulas de `references/`.

**Paso 2.5 — Validar dashboard (DETENTE).** Presentar al usuario score, periodo, headcount,
altas/bajas, rotación, antigüedad, desglose por dimensión, alertas y escenario. Esperar
aprobación antes de generar el informe.

**Paso 3 — Render.** Leer `templates/informe_hr.md` y generar las 8 secciones desde
`hr_health.json` siguiendo las reglas de formato/RGPD de `references/metrics.md`. Guardar como
`informe_hr_{periodo}.md`. (Cálculo + render pueden correr como subagent — ver Agent Routing.)

**Paso 4 — Enviar (opcional, requiere confirmación).** Tras CONFIRMAR los destinatarios con el
usuario: `python3 scripts/main.py --full --email --output .` envía vía SMTP/Gmail
(`lib/email_service.py`). No es la única forma de entrega; el informe local siempre se genera.

## Agent Routing

| Fase | Ejecutor | Por qué |
|------|----------|---------|
| Paso 0 / 2.5 (gates de usuario) | Main (sonnet) | Requieren interacción humana |
| Paso 1-2 (extracción + cálculo) | Subagent (sonnet) | I/O y scripts deterministas, aislable |
| Paso 3 (render del informe) | Subagent (sonnet) | Trabajo de plantilla sobre JSON ya calculado |
| Paso 4 (envío) | Main (sonnet) | Acción irreversible: confirmar destinatarios |

## GOTCHAS

- **Corte temporal del mes:** los KPIs son sensibles al snapshot de cierre. Fija siempre el
  último día del periodo; un corte distinto cambia headcount y rotación.
- **Altas y bajas en el mismo mes:** un empleado que entra y sale en el periodo distorsiona el
  headcount medio. El script lo maneja, pero verifica que `headcount_medio` no quede a 0.
- **Rotación anualizada vs mensual:** el script anualiza (`* 12 / meses_periodo`). No compares
  un valor anualizado con una rotación mensual cruda; siempre etiqueta el periodo.
- **Envío de email:** CONFIRMA los destinatarios antes de enviar (acción irreversible). El
  default es `rrhh@indaws.es` (`EMAIL_RECIPIENTS_HR`); no añadas correos sin pedirlos.
- **RGPD:** nunca expongas datos individuales en el informe. Solo agregados; departamentos
  con <3 empleados se agrupan como "Otros".
- **Dependencia doble:** los scripts requieren HiBob Y Odoo disponibles. Si Odoo cae, el
  informe es parcial (solo HiBob) y hay que avisarlo explícitamente.

## NEVER

- NUNCA calcules métricas a mano: siempre vía `calculate_hr_health.py`.
- NUNCA generes el informe sin aprobación del dashboard (Paso 2.5).
- NUNCA envíes el email sin confirmar destinatarios con el usuario.
- NUNCA expongas datos RGPD (individuales, NIF/NIE, salarios, banca, salud).
- NUNCA inventes nombres ni cifras: todo sale del JSON calculado.
- NUNCA uses rutas absolutas de máquina en comandos.

## ALWAYS

- SIEMPRE ejecuta los pasos en orden y respeta los gates (DETENTE en 0 y 2.5).
- SIEMPRE presenta solo métricas agregadas.
- SIEMPRE avisa si Odoo no está disponible (informe parcial).
- SIEMPRE genera el informe local; el email es complementario.

## Delegates to

- **hr-employee-lookup** — datos base de ficha/reporting line si el script necesita complemento.
- **hr-leave-attendance** — detalle de ausencias/absentismo para la Sección 6 si aplica.

## Consumed by

- Dirección de RRHH / dirección general (destinatarios del informe).
- Cuadros de mando y revisiones mensuales de people analytics del equipo HR.

## REFERENCES

- `references/metrics.md` — HR Health Score, catálogo de KPIs, estructura del informe, reglas de formato/RGPD.
- `references/kpi_definitions.md` — definición canónica de cada KPI (fórmula, fuente, umbrales).
- `references/org_structure.md` — organigrama y mapeo de departamentos.
- `templates/informe_hr.md` — plantilla del informe (8 secciones).
- `scripts/fetch_hr_data.sh` · `scripts/calculate_hr_health.py` · `scripts/main.py` — extracción, cálculo, CLI/envío.
- `../../lib/` — `hibob_client`, `odoo_hr_client`, `email_service`, `report_utils`, `config`, `constants`.
- GDPR: enmascaramiento en `lib/report_utils.py`; agregación obligatoria.
- `common:references/shared/` — N/A: HiBob vía lib/.

## Success Criteria

- [ ] Datos extraídos de HiBob (+ Odoo o aviso si no disponible)
- [ ] `calculate_hr_health.py` ejecutado sin errores
- [ ] Dashboard presentado y aprobado (Paso 2.5)
- [ ] Score (0-100) con desglose + 15+ KPIs con tendencia
- [ ] Análisis de rotación voluntaria/involuntaria y distribución por dpto/ubicación
- [ ] Plan de acción según escenario diagnosticado
- [ ] Informe con las 8 secciones, guardado como `informe_hr_{periodo}.md`
- [ ] RGPD cumplido (sin datos individuales)
- [ ] Si hubo envío: destinatarios confirmados antes de enviar
