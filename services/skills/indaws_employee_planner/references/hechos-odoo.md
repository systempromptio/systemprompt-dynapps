# Los hechos: de dónde sale cada uno

Odoo 19 corporativo. Verificado contra el código de `planning`, `project_forecast` e `indaws_project_forecast`; lo demás, compruébalo con la descripción del modelo antes de la primera pasada.

## La persona

- `res.users` de la sesión → `hr.employee` (`user_id`): quien ejecuta. Solo a esta persona se le escriben turnos.
- `hr.employee`: `resource_id`, `resource_calendar_id`, `company_id`, `department_id`, `tz`, `default_planning_role_id`.

## Días y jornada

- **Tramos**: `resource.calendar.attendance` del `resource_calendar_id`, con `dayofweek` (texto `'0'`–`'6'`), `hour_from`, `hour_to`. **Excluye `day_period = 'lunch'`**: es el descanso. Si el calendario es de dos semanas (`week_type`), toma la que corresponda a cada fecha. Respeta `date_from`/`date_to` del tramo si los trae.
- **Festivos**: `resource.calendar.leaves` con `resource_id = False` y `calendar_id` = el de la persona (o `False`, globales). El calendario de la persona, nunca el de la compañía: la doctrina explica por qué.
- **Ausencias aprobadas**: `resource.calendar.leaves` con `resource_id` = el de la persona. Traen la hora exacta, así que las medias jornadas salen solas. Contrástalas con `hr.leave` en `state = 'validate'` si necesitas el tipo para el nombre del turno.

## Trabajo

- **Tareas**: las activas tal como las define la doctrina (*De dónde salen las horas de una persona*). En Odoo: `project.task` con la persona en `user_ids` y `stage_id.fold = False` (nunca por `state`, gotcha 4 de `gotchas.md`). Campos: `name`, `project_id`, `planned_date_begin`, `date_deadline` (datetime: normaliza a fecha), `allocated_hours`, `effective_hours`, `user_ids`.
- **Checklist**: `xx_checklist_ids` de la tarea; hecho = `state = 'done'`.
- **Asignados de cada tarea**: sus empleados con `department_id` y `default_planning_role_id`. Se pasan tal cual a `colocar.py`, que decide el rol según la tabla de parámetros.
- **Proyecto**: `name`, `date` (fin), `company_id` y la fecha de go-live (en la corporativa, `xx_golive_date`; compruébalo describiendo el modelo, y si no existe, `go_live: null` y se dice).
- **Quien imputa sin tarea**: `account.analytic.line` de la persona en los últimos `imputacion_sin_tarea_dias` días (de la tabla de parámetros), en proyectos donde no tiene ninguna tarea asignada → hallazgo.

## Proyecto interno

`res.company.internal_project_id` de la compañía **de la persona**, y sus tareas de reunión interna, formación, ausencias (`res.company.leave_timesheet_task_id`) y **gestión interna**. Si alguna no existe o el proyecto es de otra compañía → hallazgo, y esos turnos no se escriben.

La **tarea de gestión interna** es la que recoge los huecos cortos sueltos (ver *Los huecos de menos de dos horas* en la doctrina). Se busca dentro del proyecto interno, activa y en una etapa no plegada, por su nombre: gestión, administración o correo. Si hay una, va en `internal.management_task_id` de la entrada de `colocar.py`, junto con `internal.project_id`. Si hay varias, **[STOP]** pregunta cuál; si no hay ninguna, se pasa sin tarea y el turno se propone sin ella, avisando. No se crea la tarea.

## Turnos existentes

`planning.slot` con `resource_id` = el de la persona y solape con el horizonte: `start_datetime`, `end_datetime` (UTC), `allocated_hours`, `state` (`draft`/`published`), `project_id`, `task_id`, `name`, `company_id`, `recurrency_id` y **la marca soft**.

- **Marca soft**: no es estándar. Se descubre describiendo `planning.slot` y buscando un booleano cuyo nombre o etiqueta diga *soft* (en la corporativa, `xx_soft_planning`, «Soft Planning?»). Nunca se escribe un nombre supuesto. Los turnos con la marca puesta son previsiones: se comparan con `split.json` y, si alguno sigue en soft pasada su fecha, se señala (ver *Soft planning* en la doctrina).

- **Propio** = su id está en `written.json` y sus fechas coinciden con lo anotado.
- **Propio movido a mano** = está en `written.json` pero sus fechas u horas ya no coinciden → se trata como ajeno y fijo.
- **Ajeno** = todo lo demás. Entra a `colocar.py` como `fixed_slots`.
- **Soft muy pasado de fecha**: se reporta, no se toca (ver *Soft planning* en la doctrina).

## Reuniones (Outlook)

Vista de calendario de Microsoft 365 de la persona en el horizonte: inicio, fin, asunto, asistentes, si es periódica. Ignora las canceladas y las rechazadas. Para asignarlas:

- Asistentes con dominio o correo de un contacto del partner de un proyecto suyo → ese proyecto, y su tarea de seguimiento o reuniones.
- Solo asistentes internos y `reunion_interna_por_asunto` → se busca en el **asunto** el nombre del proyecto, del cliente o un alias corto (la palabra distintiva del nombre del cliente: «Rotger» en «Distribuidora Rotger»). Una coincidencia → ese proyecto y su tarea de reuniones o seguimiento. Dos → se pregunta. Ninguna → proyecto interno, tarea de reunión interna.
- **Aviso**: reuniones internas que se repiten y cuyo asunto suena a preparación de cliente («preparar», «demo», «revisar»…) sin nombrar proyecto → se señalan y se pregunta a la persona contra qué proyecto van. No se adivina.
- Asistentes de un cliente donde la persona **no tiene tarea** → se reserva el hueco con su nombre (`needs_project: true`) y se pregunta.

En modo arranque, para los demás asignados, el calendario solo se lee si el conector da acceso a calendarios compartidos. Si no, su encaje va en horas por día, sin hora de reloj.
