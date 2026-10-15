---
name: indaws-demo-empleados-horas
description: >
  Dashboard de imputación de horas del equipo en un período (por defecto mes
  en curso). Para cada empleado que haya imputado horas calcula: compliance
  (horas imputadas vs 8h/día × días laborables), % tiempo en proyectos de
  cliente vs internos (partner_id vacío = interno), top proyectos y anomalías
  diarias. Identifica quién no imputa bien, quién está sobrecargado y si el
  ratio cliente/interno del equipo cumple el target. Output: HTML con matriz
  visual día-empleado, ranking por compliance y diagnóstico IA. Usar cuando
  el usuario pida: partes de horas, imputación de horas, cumplimiento de
  timesheets, ratio cliente interno, revisión de horas del equipo.
version: 1.0.2
model: sonnet
effort: medium
---

# Demo · Empleados y horas

Revisión de partes de horas del equipo. Mide compliance (imputas bien si cargas tus 8 h al día) y el ratio cliente/interno de cada empleado.

## Prerequisitos

- **odoo-pilot** para lectura de `hr.employee`, `account.analytic.line`, `project.project`, `hr.leave`
- **Credenciales Odoo** en env
- Python 3.8+ stdlib

## Workflow

| Paso | Accion | Output |
|------|--------|--------|
| 1 | Autenticar. Leer `config/config.json`. | -- |
| 2 | Ejecutar `python scripts/main.py` (default: mes en curso). Flags opcionales: `--month 2026-03`, `--from 2026-04-01 --to 2026-04-30`. El script saca timesheets + empleados + proyectos, calcula compliance + ratio cliente/interno por empleado + matriz día-empleado, genera HTML con placeholders LLM y `analysis.json`. | HTML + analysis.json |
| 3 | Leer `analysis.json`. Redactar **valoración** (3-4 frases: qué destaca del equipo esta semana/mes, top empleados por compliance o por ratio cliente, alertas clave) y **3 acciones** (concretas, con nombres: "chasear a X por los partes del viernes", "revisar con Y su baja ratio cliente 42%", etc.). Guardar en `output/report_narrative.json`. | narrative.json |
| 4 | Ejecutar `python scripts/main.py --finalize`. Aplica el narrative al HTML y guarda `output/report.html`. | HTML final |
| 5 | Presentar al usuario: ruta + hallazgos clave. | -- |

## Qué mide

### Compliance (cumplimiento de imputación)
Para cada empleado: `horas_imputadas / (días_laborables × 8h) × 100`.

- 🟢 95-105% → imputa bien
- 🟡 80-95% o 105-120% → roza el umbral
- 🔴 < 80% o > 120% → sub-imputa o sobre-imputa

### Ratio cliente / interno
Un proyecto es **cliente** si tiene `partner_id`. Si no → **interno**.

Por empleado:
- `pct_cliente = horas_cliente / horas_totales × 100`
- Target: ≥ 70% cliente (consultora sana)

### Anomalías diarias por empleado
- Día laborable con **0 horas** imputadas → flag "día sin cargar"
- Día con **< 4 h** → "día con pocas horas"
- Día con **> 10 h** → "día con muchas horas"

### Ratio cliente/interno del equipo (KPI estrella)
Suma total de horas cliente vs total. La demo se enfoca en este KPI — es la foto comercial del equipo.

## Narrativa IA (paso 3)

### `assessment` (3-4 frases)
Interpreta la foto. Ejemplos ilustrativos (varía estructura y énfasis según los datos):
> *"El equipo ha imputado 1.248 h este mes, un 92% del objetivo, con un ratio cliente 68% que queda ligeramente por debajo del target del 70%. Raquel y Víctor destacan con compliance verde y >80% cliente. Dos empleados (Adrián y el equipo de QA) concentran casi todo el tiempo interno — coherente con su rol, pero tira del ratio global hacia abajo. Marcos Beltrán lleva 3 días laborables sin imputar nada, requiere seguimiento."*
> *"Compliance alto en todo el equipo (98% de media), pero el ratio cliente cae al 55% por la preparación de la migración interna. Es un bache planificado: revisar que se recupere el mes que viene."*

### `actions` (3 items concretos)
- "Chasear a Marcos por los partes del 14-16 abril (0 h imputadas)"
- "Revisar con Adrián si su 5% cliente es correcto — quizá tiene horas cliente que está cargando a interno"
- "Proyecto 'Lumina SaaS - Plataforma Core' concentra 42% del tiempo interno del mes — revisar si justifica la dedicación"

## RULES

1. **SIEMPRE** mes en curso por defecto. El usuario puede pedir otros rangos explícitamente.
2. Solo empleados que hayan imputado **al menos 1 timesheet** en el período (no empleados sin actividad — sería ruido)
3. **Proyecto cliente** = tiene `partner_id`. Si aparece un proyecto con partner_id que conceptualmente es interno, añadir su id a `config.json` → `internal_project_override_ids.ids`
4. Días laborables = lunes a viernes (no contemplamos festivos oficiales, pueden falsear el compliance en un 5%)
5. Ignorar empleados del sistema tipo OdooBot o sin `name`
6. **NUNCA** cuentes `unit_amount` negativos como imputación — son ajustes

## GOTCHAS

1. Timesheets con `project_id = false` se cuentan como "sin proyecto" — los ignoramos del ratio cliente/interno pero sí cuentan para compliance (son horas imputadas al fin y al cabo)
2. `hr.leave` no se usa en v1 — asumimos que días de vacaciones tumban el compliance (se verá como rojo). En v2 cruzaríamos para ajustar las horas teóricas
3. Empleados que entran/salen en mitad del período: su "horas teóricas" se calcula desde el día laborable que están
4. El dashboard se hace largo con >20 empleados — la matriz día-empleado cabe bien hasta 25 filas

## Delegates to
- **common:odoo-pilot** — extracción de hr.employee, account.analytic.line, project.project

## References
| File | Content |
|---|---|
| `config/config.json` | Umbrales compliance + ratio + jornada teórica |
