# Reglas de Priorizacion

## 3 Niveles de Prioridad

### Nivel 1: URGENTE (hoy si o si)

Acciones que deben ejecutarse hoy sin excepcion. Si no se hacen hoy, impactan directamente al cliente o bloquean el proyecto.

| Condicion | Deteccion | Skill | Accion |
|-----------|-----------|-------|--------|
| Client requests vencidos >3 dias | `x_state=sent AND x_deadline < hoy-3` | `client-request-manager` FOLLOW-UP | Enviar recordatorio al cliente |
| Tareas con deadline hoy o vencidas | `date_deadline <= hoy AND checklists incompletos` | `odoo-configurator` | Ejecutar checklists pendientes |
| Tickets sin respuesta >24h | `helpdesk.ticket abierto AND create_date < hoy-1` | `service-desk` | Diagnosticar y responder |
| Reunion esta semana sin preparar | Checklist "Reunion/Formacion" con fecha esta semana, sin preparacion detectada | `odoo-flow-tester` (seguimiento) o `training-planner` (formacion) | Preparar reunion |
| Reunion pasada sin acta | Checklist "Reunion/Formacion" con fecha pasada, sin articulo en "Actas de reunion" | `project-meeting-minutes` | Generar acta |
| Viernes: weekly update pendiente | Dia de la semana = viernes AND no ejecutado esta semana | `project-weekly-update` | Ejecutar para cada proyecto activo |

### Nivel 2: IMPORTANTE (si hay tiempo)

Acciones que mejoran la gestion del proyecto y previenen problemas futuros. Deben hacerse esta semana.

| Condicion | Deteccion | Skill | Accion |
|-----------|-----------|-------|--------|
| Health score < 60 | `x_studio_health_score < 60` en ultimo project.update | `project-health-fixer` | Diagnosticar y proponer plan de accion |
| Weekly update pendiente >7 dias | `x_studio_last_health_check < hoy-7` | `project-weekly-update` | Ejecutar update (prerequisito de health-fixer) |
| Client requests draft con deadline <=10d y tarea iniciada | `x_state=draft AND x_deadline <= hoy+10 AND task.planned_date_begin <= hoy` | `client-request-manager` FOLLOW-UP | Revisar borradores y enviar |
| Client requests con respuesta sin procesar | `x_state=sent AND x_has_response=true` | `client-request-manager` CERRAR o `importation-data-assistant` | Procesar respuesta |
| Formacion proximos 2 dias | Checklist "Formacion:" con fecha <= hoy+2 | `training-planner` CONTENIDO | Preparar materiales |
| Desarrollo entregado pendiente de test | Tarea [DEV] con checklists completados pero sin validacion | `technical_dev_validation` (futuro) | Testear desarrollo |

### Nivel 3: PLANIFICADO (esta semana)

Acciones que mantienen el ritmo del proyecto. Se planifican para cuando haya disponibilidad.

| Condicion | Deteccion | Skill | Accion |
|-----------|-----------|-------|--------|
| Validaciones pendientes de documentar | `x_client_request` tipo validation en draft sin descripcion completa | `project-validation-drafting` DOCUMENTAR | Preparar documento de validacion |
| Checklists que vencen esta semana | `date_deadline` (o `date_done` si no hay) entre hoy y fin de semana, `state != done` | `odoo-configurator` | Ejecutar items pendientes |
| Preparar dev-assessment | Tarea [DEV] proxima a empezar sin descripcion detallada | `dev-assessment` | Evaluar y documentar desarrollo |
| Go-Live proximo (<=30 dias) | Tarea "Go-Live" con deadline <= hoy+30 | `go-live-validation` (futuro) | Preparar checklist de salida |
| Importaciones pendientes | Client requests tipo data_file con respuesta y adjunto | `importation-data-assistant` | Importar datos |
| Estructura Knowledge incompleta | Falta algun articulo obligatorio | Manual o `project-odoo-creator` | Crear articulos faltantes |

---

## Reglas de Ordenacion dentro de cada Nivel

1. **Dentro de URGENTE**: Ordenar por impacto al cliente (tickets > requests vencidos > reuniones > tareas)
2. **Dentro de IMPORTANTE**: Ordenar por deadline mas cercano
3. **Dentro de PLANIFICADO**: Ordenar por dependencias (ej: dev-assessment antes de desarrollo)

## Reglas de Contexto Cruzado

- Si un proyecto tiene health score < 40 (EMERGENCIA), TODAS sus acciones suben un nivel
- Si hay Go-Live en <14 dias, las acciones de ese proyecto suben un nivel
- Si hay reunion manana, la preparacion sube a URGENTE independientemente del nivel original
- Si un client request data_file tiene respuesta Y hay tarea de migracion proxima, la importacion sube a IMPORTANTE

## Estimacion de Tiempos

| Accion tipo | Tiempo estimado |
|-------------|----------------|
| Enviar recordatorio (client-request-manager FOLLOW-UP) | 10-15min |
| Preparar reunion de seguimiento | 20-30min |
| Preparar formacion (training-planner CONTENIDO) | 45-60min |
| Resolver ticket (service-desk) | 15-45min |
| Ejecutar weekly update | 15-20min por proyecto |
| Health fixer | 30-45min |
| Dev-assessment | 30-60min |
| Importacion de datos | 30-60min |
| Validacion post-implementacion | 20-30min |
| Generar acta de reunion | 15-20min |
