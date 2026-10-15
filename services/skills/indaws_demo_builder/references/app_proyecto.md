# Proyecto (Project) - Guia de Configuracion y Datos para Demo

## Modulos

| Modulo | Descripcion | Cuando instalar |
|--------|-------------|-----------------|
| project | Gestion de proyectos | Siempre (si app seleccionada) |
| hr_timesheet | Partes de horas | Si registra horas trabajadas |
| sale_timesheet | Facturacion por horas | Si factura horas de proyecto |
| project_forecast | Planificacion de recursos | Si necesita asignacion de capacidad |

## Configuracion

### Ajustes Principales
- **Partes de horas**: Activar si el cliente registra horas (consultoria, servicios)
- **Subtareas**: Activar para desglose jerarquico
- **Etapas de proyecto**: Personalizar segun metodologia del cliente
- **Etapas de tareas**: Crear etapas relevantes (Nuevo, En Progreso, Revision, Hecho)
- **Facturacion por horas**: Activar si cobra servicios por hora

### Features Opcionales
- Hitos del proyecto: Para entregables clave
- Planificacion de recursos: Para gestion de capacidad
- Ratings/Satisfaccion: Para feedback del cliente
- Recurrencia de tareas: Para tareas periodicas

## Datos Maestros

### Proyectos (2-3 proyectos)
- 1 proyecto en progreso (con tareas en varias etapas)
- 1 proyecto recien creado (pocas tareas, inicio)
- 1 proyecto casi terminado (opcional, para mostrar cierre)
- Cada proyecto con partner_id del cliente asociado

### Tareas (15-25 tareas totales)
- Distribuidas entre los proyectos
- Asignadas a diferentes usuarios
- Con fechas planificadas (inicio y fin)
- Al menos 3 con partes de horas registrados
- Subtareas en al menos 2 tareas principales
- Tags relevantes (urgente, bug, mejora, etc.)

### Etapas de Tareas
- Nuevo
- En Progreso
- En Revision
- Hecho
- Cancelado (fold=True)

## Flujo de Negocio - Distribucion por Estado

### Nuevo - 15% de tareas
**Crear**: project.task en primer stage
**Para la demo**: Tareas recien asignadas

### En Progreso - 40%
**Crear**: Mover a stage "En Progreso" + registrar horas
**Para la demo**: Trabajo activo, mostrar timesheet

### En Revision - 15%
**Crear**: Mover a stage "Revision"
**Para la demo**: Pendiente de validacion del responsable

### Hecho - 25%
**Crear**: Mover a stage "Hecho" + marcar kanban_state=done
**Para la demo**: Completadas, para ver historico

### Cancelado - 5%
**Crear**: Mover a stage "Cancelado"
**Para la demo**: Contexto de gestion

## Tips para Demo Impactante
- Vista Kanban de tareas con columnas llenas y visuales
- Al menos 1 tarea con timer de horas para iniciar en demo en vivo
- Subtareas para mostrar desglose jerarquico
- Mostrar el grafico de burndown o progreso del proyecto
- Tener 1 tarea con deadline vencido (muestra alertas visuales)
- Si factura por horas, mostrar la facturacion desde el proyecto
- Registrar notas/comentarios en el chatter de 2-3 tareas

## Comandos API (odoo-pilot)

```bash
# Crear proyecto
create_record.sh project.project '{"name": "Nombre Proyecto", "partner_id": PARTNER_ID}'

# Crear tarea
create_record.sh project.task '{"name": "Nombre Tarea", "project_id": PROJECT_ID, "user_ids": [[6, 0, [USER_ID]]], "stage_id": STAGE_ID, "date_deadline": "2026-03-15"}'

# Registrar horas
create_record.sh account.analytic.line '{"project_id": PROJECT_ID, "task_id": TASK_ID, "name": "Descripcion trabajo", "unit_amount": 2.5, "user_id": USER_ID}'

# Mover de etapa
update_record.sh project.task TASK_ID '{"stage_id": NEW_STAGE_ID}'
```
