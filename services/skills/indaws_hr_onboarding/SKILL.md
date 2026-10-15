---
name: indaws-hr-onboarding
description: >
  Gestiona el onboarding y offboarding de empleados en Indaws. Genera checklists
  personalizados por departamento, rol y ubicacion, planes de incorporacion
  (Day-1/Week-1/Month-1), emails de bienvenida, informes para managers,
  checklists de salida (devolucion de equipo, revocacion de accesos, transferencia
  de conocimiento) y plantillas de entrevista de salida.
  Usar cuando el usuario pida: onboarding, offboarding, incorporacion, alta empleado,
  baja empleado, checklist de entrada, checklist de salida, welcome email,
  plan de acogida, entrevista de salida, desvinculacion.
version: 1.1.2
effort: low
model: sonnet
---

# HR Onboarding & Offboarding

**CONFIDENTIAL - HR TEAM ONLY**
**Document ID:** HR-SKILL-002

Gestiona el ciclo completo de incorporacion y desvinculacion de empleados en Indaws.
Conecta con HiBob (fuente primaria de onboarding wizards y tareas) y Odoo (hr.employee)
para generar checklists personalizados, planes de acogida, emails de bienvenida,
informes para managers y checklists de salida.

## Prerequisitos

- **HiBob + Odoo**: credenciales en entorno (`HIBOB_*`, `ODOO_*`). Acceso via `../../lib/`
  (hibob_client, odoo_hr_client, config, report_utils, email_service).
- **Python 3** para los scripts. Plantillas en `templates/`, datos de referencia en `references/`.

## Workflow (resumen)

Detalle completo, comandos y plantillas de validacion en `references/workflow.md`.

| Paso | Accion |
|------|--------|
| 0 | **DETENTE**: pregunta tipo de operacion (onboarding/offboarding/status) y datos del empleado |
| 1 | Extrae datos: `scripts/fetch_onboarding_data.py` (HiBob wizards + Odoo employee) |
| 2 | Calcula checklist + plan: `scripts/onboarding_tracker.py` (NUNCA manual) |
| 2.5 | **DETENTE**: presenta resumen y espera aprobacion del usuario |
| 3 | Genera documentos desde `templates/`: checklist, plan Day-1/Week-1/Month-1, emails, informe manager |

Reglas de negocio (plazos, periodo de prueba, buddy, prevalencia HiBob): ver `references/workflow.md`.

## GOTCHAS

- **Onboarding antes de que el empleado exista en HiBob**: el wizard puede no estar creado
  el dia que se pide la incorporacion. `fetch_onboarding_data.py` devolvera vacio de HiBob;
  usa Odoo (hr.employee) como fallback y avisa al usuario de que el wizard no existe aun.
- **Offboarding y revocacion de accesos**: la revocacion se ejecuta el **ultimo dia efectivo**,
  no antes. Generar la notificacion a IT con esa fecha; nunca disparar la revocacion en el
  momento de la notificacion de baja.
- **Emails de bienvenida/notificacion via SMTP, NO Odoo**: esta suite envia por SMTP/Gmail
  (`lib/email_service.py`). Es el comportamiento esperado y NO debe cambiarse a envio por Odoo.
- **Checklist por departamento faltante**: si el departamento no tiene checklist en
  `references/onboarding_checklist.md`, NO inventes tareas; usa el bloque de tareas comunes
  como fallback y marca el departamento como pendiente de definir.
- **Discrepancia HiBob vs Odoo**: prevalece HiBob. No mezcles datos contradictorios en silencio.

## NEVER

- NUNCA generes checklists manualmente: siempre `onboarding_tracker.py`.
- NUNCA generes documentos sin la aprobacion del usuario (Paso 2.5).
- NUNCA incluyas datos sensibles (NIF/NIE, salario, banco, salud) — RGPD (ver `../../security/gdpr_guidelines.md`).
- NUNCA inventes datos: todo viene de HiBob y Odoo.

## ALWAYS

- SIEMPRE personaliza el checklist por departamento + rol + ubicacion.
- SIEMPRE detente en los Pasos 0 y 2.5.
- SIEMPRE envia el email de bienvenida minimo 3 dias habiles antes de la incorporacion.
- SIEMPRE programa la revocacion de accesos para el ultimo dia efectivo.

## Delegates to

- `hr-employee-lookup` — para resolver la ficha del empleado (manager, reporting line,
  historial) cuando faltan datos en el input.

## Consumed by

- `hr-people-report` — consume el estado de onboardings en curso como KPI de headcount.
- Ningun otro skill escribe sobre este; es punto de entrada de usuario para alta/baja.

## REFERENCES

- `references/workflow.md` — flujo detallado, comandos y plantillas de validacion.
- `references/onboarding_checklist.md` — proceso estandar por departamento.
- `references/offboarding_checklist.md` — proceso estandar de salida.
- `templates/` — `welcome_email.md`, `onboarding_report.md`, `offboarding_report.md`.
- `../../lib/` — hibob_client, odoo_hr_client, email_service, config, report_utils.
- `../../security/gdpr_guidelines.md` — enmascaramiento y cumplimiento RGPD.
- `common:references/shared/` — N/A: HiBob/Odoo se acceden via `lib/`.

## Success Criteria

- [ ] Datos extraidos de HiBob y/o Odoo; `onboarding_tracker.py` sin errores.
- [ ] Checklist personalizado por departamento/rol/ubicacion, presentado y aprobado.
- [ ] (Onboarding) plan Day-1/Week-1/Month-1, email de bienvenida e informe para manager.
- [ ] (Offboarding) checklist de salida completo y plantilla de entrevista de salida.
- [ ] Ningun dato sensible expuesto (RGPD). Documentos guardados en el directorio de trabajo.
