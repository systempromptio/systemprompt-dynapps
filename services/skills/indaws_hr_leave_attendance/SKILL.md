---
name: indaws-hr-leave-attendance
description: >
  Monitorizacion de ausencias y asistencia. Saldos de vacaciones, who-is-off,
  anomalias de asistencia, Bradford Factor, absentismo por departamento.
  Fuentes: HiBob TimeOff + Attendance, Odoo hr.leave + hr.attendance.
  Usar cuando el usuario pida: vacaciones, ausencias, who is off, quien falta,
  asistencia, fichajes, Bradford Factor, absentismo, saldos de vacaciones,
  control de presencia, horas extras, anomalias de asistencia.
version: 1.1.1
effort: low
model: sonnet
---

# HR Leave & Attendance

**CONFIDENTIAL - HR TEAM ONLY** · **Document ID:** HR-SKILL-005

Monitorizacion de ausencias y presencia de la plantilla. Genera informes de vacaciones,
ausencias, presencia y anomalias; calcula Bradford Factor y tasa de absentismo por
departamento. Fuente primaria HiBob (TimeOff + Attendance); Odoo (hr.leave + hr.attendance)
como fuente secundaria/complementaria.

**Herramientas:** Bash, Read, Write, Glob, Grep.

## Workflow (resumen)

Detalle completo, comandos y formulas en `references/workflow.md`.

1. **Paso 0 - Recopilar (DETENTE):** tipo de consulta, periodo, alcance, credenciales.
   Esperar respuesta antes de seguir.
2. **Paso 1 - Extraer:** `fetch_leave_data.py` (ausencias/saldos) y/o
   `fetch_attendance_data.py` (asistencia). Si Odoo no responde, seguir solo con HiBob y avisar.
3. **Paso 2 - Calcular (OBLIGATORIO):** `anomaly_detector.py` calcula absentismo, Bradford
   Factor, anomalias y cumplimiento. Nunca calcular a mano.
4. **Paso 2.5 - Validar (DETENTE):** presentar resumen y esperar aprobacion.
5. **Paso 3 - Informe:** solo tras aprobacion, con `templates/leave_report.md` o
   `attendance_report.md`. Guardar `informe_*_{periodo}.md`; los JSON intermedios son temporales.

Los scripts son portables: resuelven la raiz de la suite con `parents[3]` +
`sys.path.insert(0, ...)`. Ejecutar desde la raiz del repo, sin rutas de maquina.

## Agent Routing

| Tarea | Agente / modelo |
|-------|-----------------|
| Extraccion, calculo y generacion de informe (flujo estandar) | SONNET (este skill, `effort: low`) |
| Localizar codigo de cliente/politica en `lib/` o referencias | Explore |
| Diseno de nuevas metricas o cambios en `anomaly_detector.py` | OPUS (revision) |

## GOTCHAS

- **Saldos de vacaciones**: HiBob los devuelve por **politica y devengo (accrual)**; no se
  recalculan, se leen tal cual. No sumar dias entre politicas distintas.
- **Who-is-off**: depende de **timezone y half-days** de HiBob; una media jornada cuenta como
  ausente parcial. Validar el rango de fechas en la zona horaria del empleado.
- **Bradford Factor** requiere **historico de ausencias de 12 meses**; con menos datos el
  resultado no es representativo (S y D quedan infraestimados).
- **Festivos vs ausencias**: los festivos NO son ausencias. No inflar el absentismo contando
  festivos como dias de baja.
- **Datos medicos (RGPD)**: el motivo de una baja medica esta **prohibido** en cualquier
  salida. Nunca mostrar "Baja medica" como detalle; agrupar como "Otros".

## NEVER

- NUNCA mostrar el tipo/motivo de baja medica ni dato de salud (RGPD / LOPDGDD). Agrupar "Otros".
- NUNCA mostrar saldos de vacaciones individuales a quien no sea RRHH o manager directo.
- NUNCA calcular metricas manualmente: siempre via `anomaly_detector.py`.
- NUNCA generar el informe sin la aprobacion del Paso 2.5.
- NUNCA inventar datos: todos los numeros vienen del JSON calculado.
- NUNCA mostrar departamentos con <3 empleados sin agregar (agrupar como "Otros").

## ALWAYS

- SIEMPRE tratar HiBob como fuente primaria y Odoo como secundaria.
- SIEMPRE leer (solo lectura): este skill no escribe en HiBob ni Odoo, solo extrae y reporta.
- SIEMPRE aplicar `report_utils.sanitize_employee_data()` en salidas de datos de empleados.
- SIEMPRE usar tipos "Vacaciones", "Asuntos propios", "Otros"; fechas DD/MM/YYYY.
- SIEMPRE alertar si las horas extras superan 80 h/mes (Art. 35 ET).
- SIEMPRE eliminar los JSON intermedios tras generar el informe.

## Delegates to

- `lib/hibob_client.py` — extraccion de TimeOff, balances, policies, Who's Out y Attendance.
- `lib/odoo_hr_client.py` — extraccion complementaria de `hr.leave` y `hr.attendance`.
- `lib/report_utils.py` — formato y enmascaramiento RGPD de la salida.
- `Explore` — busqueda de codigo/politicas en `lib/` y `references/`.

## Consumed by

- **hr-people-report** — consume las metricas de absentismo y Bradford Factor por departamento
  para el HR Health Score y el informe mensual de RRHH.

## REFERENCES

- `references/workflow.md` — pasos completos, comandos, formulas (Bradford Factor, absentismo,
  saldos) y reglas de generacion.
- `references/leave_policies.md` — legislacion laboral y convenio colectivo (asuntos propios).
- `lib/hibob_client.py`, `lib/odoo_hr_client.py`, `lib/report_utils.py`, `lib/config.py` —
  clientes y umbrales compartidos de la suite.
- `../../security/gdpr_guidelines.md` — directrices RGPD obligatorias (HR-SEC-001).
- `common:references/shared/` — N/A: HiBob via `lib/`, no hay conexion Odoo compartida.

## Success Criteria

- [ ] Datos de ausencias y asistencia extraidos de HiBob (o aviso si Odoo no disponible)
- [ ] `anomaly_detector.py` ejecutado sin errores
- [ ] Resumen presentado y aprobado (Paso 2.5)
- [ ] Absentismo y Bradford Factor calculados con umbrales correctos
- [ ] Anomalias detectadas (llegadas tarde, fichajes omitidos, horas extras)
- [ ] Informe Markdown generado con todas las secciones aplicables
- [ ] RGPD cumplido: sin baja medica, saldos restringidos, deptos <3 agregados
- [ ] Fichero guardado y JSON intermedios eliminados
