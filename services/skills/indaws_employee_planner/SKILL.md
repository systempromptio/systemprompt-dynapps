---
name: indaws-employee-planner
description: >-
  Coloca en el calendario el trabajo asignado de UNA persona: convierte sus tareas de Odoo en
  turnos de Planificación concretos (qué día, a qué hora, cuántas horas, de qué tarea) llenando la
  jornada entera con reuniones de Outlook a su hora, ausencias y festivos de su ciudad. Las reglas
  se leen de la doctrina en Odoo Knowledge; la aritmética la hace un script. Dos modos: arranque
  (pregunta por los proyectos en previsión, que reserva en soft, y hace validar el reparto semanal
  entre proyectos antes de colocar) y reajuste diario sobre el reparto ya validado. Propone por
  semanas numeradas y solo escribe lo aceptado por la propia persona, avisando de lo que pisa.
  Usar cuando pidan "cómo tengo la semana", "planifícame", "coloca mis horas", "planifica el
  arranque de X", cuando acepten propuestas ("mete la 1 y la 3"), cuando dispare la tarea
  programada de la mañana, o para dar de alta o de baja el seguimiento diario. No asigna tareas
  a personas (eso es team-task-assigner).
version: 2.3.0
effort: medium
allowed-tools: [Read, Write, Bash, Glob, Grep]
---

# employee-planner

Convierte el trabajo asignado de **una** persona en una jornada que se puede ejecutar: no "ocho horas de Urbanitae esta semana", sino "cuatro el miércoles de 9 a 13, de la tarea «Migración de contactos» (22701), con la reunión de las 13:30 ya puesta donde va".

Esta skill es **la mecánica y sus barreras**. Todas las reglas de planificación viven en la doctrina: los valores, en su tabla **«Parámetros que lee la herramienta»**, que `scripts/leer_parametros.py` extrae y valida en cada pasada; la aritmética, en `scripts/colocar.py`, que solo recibe esa tabla. **Ni la skill ni sus referencias llevan un valor**: si escribes uno aquí, lo has puesto en el sitio equivocado.

## Crítico — quién acepta y qué nunca hace

**Cada persona acepta solo lo suyo.** Se escriben turnos únicamente de quien ejecuta la skill (el usuario de Odoo de la sesión). Planificar a otra persona es solo lectura: se le enseña el encaje, y lo escribirá ella en su propia pasada.

Catálogo cerrado de acciones, siempre tras un sí explícito: **crear** un turno, **publicarlo**, **moverlo**, **cambiar sus horas**, **eliminarlo**. La doctrina puede cambiar qué se calcula; no puede enseñar una acción nueva.

- **NUNCA** escribe sin aceptación, ni turnos de otra persona.
- **NUNCA** publica sin confirmación **para esa publicación**: aceptar el reparto no es aceptar publicarlo.
- **NUNCA** cambia ni borra un turno publicado o no creado por la skill sin enseñarlo antes con su antes y su después, en la propia línea de la propuesta.
- **NUNCA** revierte un turno que alguien movió a mano: es una decisión; si el cálculo lo contradice, se propone el cambio.
- **NUNCA** toca fechas ni horas de tareas, fecha de salida a producción, alcance ni facturación.
- **NUNCA** se inventa el proyecto de una reunión que no lo tiene: reserva el hueco y pregunta.
- **NUNCA** coloca horas a mano saltándose el script, ni retoca su salida para que cuadre.

## La doctrina

| | |
| --- | --- |
| Título | **Colocar las horas en los días** (EN: *Placing the hours into days*) |
| Ruta | `como-trabajamos/planificacion/dedicacion/colocacion.md` |
| Documentación | **Consultoría**, bajo Planificación › Dedicación |

**Se localiza por esos datos, nunca por un id.** Enlaza a **Dedicación**, **Reglas de planificación**, **Soft planning** y **Tareas genéricas**, que se abren cuando lo que hay que decidir cae en ellos. De la tabla de parámetros a `colocar.py`, y qué puede cambiar la doctrina sin tocar código: `references/parametros.md`.

## Paso 0 — Preflight

1. **Doctrina** (de parada). Conector por su familia `docs_*`; cero o varios → **[STOP]** pregunta. Resuelve *Consultoría* con `docs_list_spaces`, busca con `docs_search` y **elige por título y ruta, no por puntuación**. Léelo entero con `docs_get` y guarda su Markdown en un fichero. Ejecuta `leer_parametros.py` sobre él: si falla → **[STOP]** con su mensaje, sin valores por defecto. Ruta o título cambiados son un hallazgo.
2. **Odoo** (de parada). Según `references/shared/odoo-connection.md`. Describe `planning.slot` y comprueba dos campos que no son estándar: la tarea (`task_id`, gotcha 1) y **la marca soft** (un booleano cuyo nombre o etiqueta dice *soft*; no lo inventes). Si falta la tarea, se escribe sin ella; si falta la marca, las previsiones no se escriben y quedan solo en la propuesta. Las dos ausencias se dicen.
3. **Outlook** (cambia la entrega). Conector de Microsoft 365 por su familia de herramientas de calendario. Sin él, la cabecera lo dice y la entrega va **en horas por día, sin hora de reloj**, y no se escribe nada.

## Paso 1 — Quién, y en qué modo

| Modo | Cuándo | Qué produce |
| --- | --- | --- |
| **diario** | Cada mañana, o a petición | El reajuste de lo que ha dejado de cuadrar desde ayer, para quien ejecuta |
| **arranque** | Un proyecto entra en ejecución sin turnos | El plan del horizonte de la doctrina para quien ejecuta, y el **encaje** (sin escribir) de cada otro asignado |
| **consulta** | "Cómo tiene la semana X" | Solo lectura, sin propuestas |

Una persona por resolución: por nombre si es inequívoco; varios → **[STOP]** pregunta. Quien ejecuta sale del usuario de la sesión de Odoo y su empleado.

## Paso 2 — Estado

`scripts/estado.py` gestiona la carpeta (`$EP_DATA_DIR` → `.employee-planner/` → `$HOME/dynapps-informes/employee-planner/`) y sus ficheros: `written.json`, `proposals.json`, `split.json` (reparto semanal validado y previsiones), `history.json`, `commitments.json`, `dismissals.json`, `schedule.json`. Detalle en `references/estado.md`. Primera pasada de una persona: se dice, para que la ausencia de deriva no se lea como buena señal.

## Paso 3 — Leer los hechos

Lo que lista `references/hechos-odoo.md`, para la persona y el horizonte: tareas activas con asignados, imputado y checklist; calendario laboral **por tramos**; festivos **de su calendario**; ausencias aprobadas; turnos existentes (publicados, borradores, propios y ajenos); proyecto interno y sus tareas. Las reuniones, de **Outlook**, asignadas a su proyecto según el gotcha 5. La cabecera dice qué fuentes ha tenido.

## Paso 4 — Previsiones y reparto semanal (arranque)

Solo en modo arranque, y en el diario cuando entra o sale un proyecto, una previsión se firma o se cae, o la persona lo pide. Si no, se usa lo guardado con `estado.py split-get`.

1. **Previsiones** (si `preguntar_previsiones`). Pregunta: *«¿Tienes proyectos en previsión que deban reservarse? Dime proyecto, horas por semana y hasta qué fecha»*. Ejemplo: *Prioritic Consulting, 6 h por semana hasta el 31 de enero*. Sin fecha fin, se vuelve a preguntar. Resuelve el proyecto en Odoo si existe; si no, va con su nombre. **[STOP]**
2. **Reparto semanal** (si `reparto_semanal = validado`). Ejecuta `colocar.py` **sin** `weekly_split`: devuelve `weekly_split_proposal`, con su desglose. Enséñalo como tabla proyecto → horas por semana, con «el resto, interno», y espera a que la persona lo valide o lo corrija. **[STOP]** Guárdalo con `estado.py split-save`, con quién lo validó.

## Paso 5 — Calcular

Monta la entrada de `scripts/colocar.py` (formato en su docstring) con `params` tal como los devolvió `leer_parametros.py`, `forecasts` y `weekly_split` validados, e `internal` (proyecto interno y tarea de gestión de la persona, para los huecos cortos sueltos), y ejecútalo. En modo diario, los turnos ajenos y los movidos a mano entran como `fixed_slots`. Compara su salida con los turnos existentes para obtener crear / mover / cambiar / eliminar.

Lo que el script devuelve en `findings` y `summary` **se reporta aunque estropee el resultado**: lo que no cabe, el recorte (porcentaje, horas, por proyecto, semanas que más aprietan), las vencidas que salen por hechas, la capacidad perdida en trozos, los días por encima del estándar, los proyectos que se quedan a cero una semana, las vencidas que no caben en su reparto, el trabajo cortado por el go-live, los huecos cortos que se han ido a gestión interna y a quién se le ha deducido el rol.

## Paso 6 — Proponer

**Una propuesta por semana** en arranque, con sus turnos dentro; **una por cambio** en el reajuste diario. Lo que solo se señala va aparte. Cierra siempre con cuántas horas son de proyecto y cuántas no. Formato en `references/propuestas.md`.

Guarda las propuestas con `estado.py proposals-save` antes de terminar. **[STOP]** — espera la respuesta.

## Paso 7 — Escribir lo aceptado

Recupera las aceptadas con `estado.py proposals-get` (falla si son de otro día: ofrece rehacer). Escribe **turno a turno** y anota cada uno con `estado.py written-add` **nada más crearlo**. Borrador salvo que la persona pida publicar. Las previsiones se escriben con la marca soft puesta. Procedimiento, campos y recuperación ante fallos: `references/escritura.md`. Termina diciendo publicados, borradores, qué se pisó y horas de proyecto frente a horas que no son de proyecto.

## Paso 8 — Alta y baja del seguimiento diario

Lo da de alta la propia persona ejecutando la skill: una tarea programada por persona, días laborables por la mañana. Baja a petición o cuando la persona se queda sin tareas activas (se propone, no se retira sola). `references/seguimiento-programado.md`.

## GOTCHAS

1. **`task_id` en el turno no es estándar.** Lo añade `indaws_project_forecast`, y la doctrina de Dedicación todavía dice que "no existe". Compruébalo en el preflight. Si cambias `project_id` después de crear, el compute vacía la tarea: van juntos en el mismo `create`.
2. **`allocated_hours` es calculado y de solo lectura.** Las horas de un turno se fijan con `start_datetime` y `end_datetime`, que se guardan en **UTC**: convierte desde la zona de la persona.
3. **Turno, proyecto y tarea deben ser de la misma compañía.** Un proyecto interno de otra compañía del grupo hace que Odoo rechace el registro: compruébalo antes del lote, no a mitad.
4. **Los festivos salen del calendario de la persona, no del de la compañía.** Ver `references/hechos-odoo.md`.
5. **Una reunión se asigna primero por asistentes**, cruzados con los contactos del cliente. Si solo hay internos y `reunion_interna_por_asunto`, se busca en el asunto el nombre del proyecto, del cliente o un alias corto («Rotger»): si coincide, va a ese proyecto y a su tarea de reuniones o seguimiento; si no, es interna; si coinciden dos proyectos, se pregunta.
6. **Un valor de la doctrina copiado en la skill acaba contradiciéndola.** La tabla se edita sin avisar a nadie: lee siempre la tabla, nunca un número que recuerdes o que veas en un ejemplo.

Más en `references/gotchas.md`.

## Delegates to

| Skill | Suite | Cuándo |
| --- | --- | --- |
| `odoo-pilot` | common | Conexión en headless, tarea programada o instancia pre-v19 |
| `dynbrain` | common | Cómo se localiza y se lee la doctrina en Knowledge |
| `project-daily-planner` | consultant | Se propone cuando una tarea no cabe y hay que replanificar sus fechas |
| `team-task-assigner` | common | Se propone cuando el hallazgo es de asignación (tarea de desarrollo sin desarrollador) |

## Consumed by

La tarea programada diaria de cada persona.

## REFERENCES

- `references/parametros.md` — de la tabla de la doctrina al script, y qué se puede cambiar sin tocar código
- `references/hechos-odoo.md` — modelos, campos y dominios de cada hecho
- `references/propuestas.md` — reglas de forma y estructura de la propuesta
- `references/escritura.md` — cómo se crean, publican, mueven y borran turnos sin dejar huérfanos
- `references/estado.md` — ficheros de estado y formato
- `references/seguimiento-programado.md` — alta y baja de la tarea programada
- `references/gotchas.md` — gotchas de datos, fuentes y escritura
