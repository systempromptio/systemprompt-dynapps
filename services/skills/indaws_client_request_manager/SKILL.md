---
name: indaws-client-request-manager
description: >-
  Gestiona peticiones al cliente en proyectos de implantación Odoo. Analiza tareas, coteja con el
  análisis funcional y crea registros x_client_request. Centraliza envío de emails, follow-up y
  cierre. Usar cuando haya que crear, seguir o cerrar peticiones al cliente.
version: 1.2.1
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# Client Request Manager

Gestor de Peticiones al Cliente de Indaws (Odoo Gold Partner). Centraliza y profesionaliza la comunicacion con el cliente durante las implantaciones de Odoo, creando peticiones estructuradas y rastreables en el modelo `x_client_request`.

## Workflow

Three modes of operation. See `references/workflow_detail.md` for full step-by-step instructions.

### Paso 0: Recopilacion de Datos

Collect before continuing:
1. **Modo**: GENERAR | FOLLOW-UP | CERRAR
2. **Credenciales Odoo**: URL, DB, API Key, Usuario
3. **Segun modo**: GENERAR (task_id + analisis funcional), FOLLOW-UP/CERRAR (project_id)

See references/shared/odoo-connection.md for connection pattern.

DETENTE y espera respuesta del usuario.

## Field contract (preflight OBLIGATORIO)
Antes de leer Odoo, verifica que existen (ver `references/shared/field-contract-preflight.md`):
- modelo Studio `x_client_request` (debe existir): campos `x_name`, `x_task_id`, `x_project_id`, `x_partner_id`, `x_description`, `x_type`, `x_priority`, `x_state`, `x_deadline`, `x_date_sent`, `x_date_received`, `x_is_overdue`, `x_has_response`, `x_active`
Si falta algo → STOP con mensaje accionable (qué falta, dónde se configura en Studio). NO continúes con datos parciales.

### Mode GENERAR

1. **Extraer contexto** -- `scripts/fetch_task_context.sh <task_id>` obtiene tarea, checklists, proyecto y requests existentes
2. **Analisis con cotejo** -- Cruzar 5 fuentes (descripcion tarea, checklists, patrones por tipo, analisis funcional, requests existentes). Si info ya en analisis -> DESCARTAR peticion
3. **Validacion consultor** -- Presentar tabla con justificacion. DETENTE. Esperar aprobacion
4. **Crear registros** -- `x_client_request` con campos completos. Deadlines basados en `planned_date_begin` de la tarea

### Mode FOLLOW-UP

1. **Dashboard** -- `scripts/fetch_requests.sh <project_id>` con resumen por estado
2. **Borradores** -- Generar email con template, validar con consultor, enviar via proceso 3 pasos (bug JSON2)
3. **Vencidas** -- Recordatorio, extender deadline, cancelar o marcar recibida
4. **Respondidas** -- Marcar como received con notas

### Mode CERRAR

1. Filtrar requests con respuesta no procesada, cerrar en batch
2. Resumen de lo cerrado, sugerir FOLLOW-UP si quedan vencidas

Full details in `references/workflow_detail.md`. Rules and validations in `references/rules_and_validations.md`.

## GOTCHAS

1. **JSON2 HTML escaping** -- `message_post` escapa el HTML del body. Usar proceso de 3 pasos (mail.message/create + mail.mail/create+send + mail.notification/create) en lugar de message_post directo
2. **Deadline desde planned_date_begin** -- Los deadlines se calculan desde la fecha planificada de inicio de la tarea, NO desde hoy. Si no tiene fecha planificada, usar +14 dias conservador
3. **Duplicar peticiones** -- Siempre verificar requests existentes (Fuente E) antes de crear. Si ya hay una peticion similar para la misma tarea, DESCARTAR
4. **Pedir acceso a software cliente** -- NUNCA solicitar acceso a Redmine, SAP, Sage, banca online, etc. Los datos vienen en Excel/CSV, la banca es self-service
5. **Enviar email sin validacion** -- NUNCA enviar sin que el consultor apruebe el borrador. Las peticiones son la cara visible del equipo
6. **Olvidar cotejo con analisis** -- SIEMPRE cruzar contra el documento de analisis funcional completo. Leer TODAS las secciones, no solo la app correspondiente
7. **Tono formal distante** -- Redactar en segunda persona (tu/vosotros), tono cercano y profesional. Nunca "se solicita a la empresa que proporcione..."

## Delegates to

| Skill | Suite | When | Via |
|-------|-------|------|-----|
| odoo-pilot | common | CRUD en x_client_request, mail.message, mail.mail | Bash inline |
| dynapps-brand | common | (not used -- emails are plain text) | -- |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| consultant-daily-planner | consultant | Detecta requests vencidos, con respuesta, draft proximos |
| importation-data-assistant | consultant | Actualiza client request tras importacion completada |
| project-meeting-minutes | consultant | Crear peticiones formales al cliente desde acciones pendientes del acta |
| project-validation-drafting | consultant | Generar peticiones de validacion/aceptacion al cliente |

## References

| File | Content |
|------|---------|
| `references/workflow_detail.md` | Full workflow for GENERAR, FOLLOW-UP, CERRAR modes |
| `references/rules_and_validations.md` | Rules, email guidelines, validations |
| `references/analisis_peticiones.md` | Patrones de peticiones por tipo de tarea |
| `references/tipos_peticion.md` | Tipos de peticion y prioridades |
| `templates/email_peticion.md` | Template email de peticion inicial |
| `templates/email_followup.md` | Template email de recordatorio |
| `scripts/fetch_task_context.sh` | Extrae contexto de tarea para GENERAR |
| `scripts/fetch_requests.sh` | Extrae requests de proyecto para FOLLOW-UP/CERRAR |
