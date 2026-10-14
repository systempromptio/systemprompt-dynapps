---
name: indaws-attendance-monitor
version: 1.0.3
effort: medium
model: sonnet
description: >-
  Controla semanalmente que los empleados estén fichando correctamente en Odoo. Verifica registros
  por tipo de jornada (partida/intensiva), suma de horas (8h/7h ±1h) y pausa de comida (30min-1h30),
  y excluye días con vacaciones o festivos. Genera informe de incidencias por empleado. Solo
  lectura. Usar cuando haya que controlar el fichaje semanal de empleados.
---

# Monitor de Fichajes Semanal

**CONFIDENTIAL — MANAGEMENT TEAM ONLY**

## Workflow

1. **Paso 0 — Recopilación**: Confirmar credenciales Odoo + semana a analizar (default: semana ISO anterior). DETENTE si la semana por defecto no es correcta.
2. **Paso 1 — Extracción**: Ejecutar `scripts/fetch_attendance_data.sh [--week-start YYYY-MM-DD]`. Obtiene 5 conjuntos: asistencias, empleados, vacaciones validadas, festivos calendario, calendarios laborales.
3. **Paso 2 — Análisis**: Ejecutar `python3 scripts/analyze_attendance.py < /tmp/att_data.json`. Aplica reglas por empleado y día (L–V). Ver `references/attendance_rules.md`.
4. **Paso 3 — Presentación**: Mostrar informe Markdown en pantalla. Si el usuario pide guardarlo, redirigir a fichero.

Ver `references/workflow-detail.md` para reglas de negocio completas.

> Note: `common:references/shared/` — auth pattern via `auth.sh`, context `es_ES`.

## GOTCHAS

1. **Sesiones abiertas (`check_out=False`)** → ignorar el registro para el cálculo de horas y reportarlo como aviso, no como error; no rompe el análisis.
2. **Festivos globales vs. por empleado** → `resource.calendar.leaves` con `resource_id` vacío son festivos globales; los que tienen `resource_id` son del empleado concreto. Ambos eximen del fichaje.
3. **Jornada intensiva en viernes y julio/agosto** → el tipo de jornada se calcula por `weekday==4` OR `month in (7,8)`, no por la configuración del empleado.
4. **Empleado sin asistencias esa semana** → no aparece en el informe; solo se analizan quienes tienen al menos un registro en el rango.
5. **Pausa de comida**: se mide entre `check_out` del primer registro y `check_in` del segundo del mismo día. Si hay >2 registros, avisar pero no bloquear.
6. **Semana por defecto** → lunes a domingo de la semana ISO anterior. Si el usuario lanza el skill el lunes, la semana pasada es la correcta.

## NEVER
- Modificar datos en Odoo. Este skill es solo lectura.
- Inventar incidencias. Si faltan datos, marcarlo en el informe.
- Analizar sábado y domingo — no son días laborables.

## ALWAYS
- Usar odoo-pilot (`auth.sh`) con contexto `{"lang": "es_ES"}`.
- Mostrar el informe en la conversación (no guardar fichero salvo petición explícita).
- Confirmar la semana antes de lanzar el análisis si no se indicó explícitamente.

## Delegates to

| Skill | Suite | Para qué |
|-------|-------|----------|
| `odoo-pilot` | common | Autenticación y consultas a Odoo |

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| — | — | Skill terminal de dirección |

## REFERENCES

- `references/attendance_rules.md` — Matriz completa de incidencias y reglas de negocio
- `references/workflow-detail.md` — Detalle completo del workflow y reglas de cálculo
- `scripts/fetch_attendance_data.sh` — Extracción de los 5 conjuntos de datos
- `scripts/analyze_attendance.py` — Análisis y generación del informe
