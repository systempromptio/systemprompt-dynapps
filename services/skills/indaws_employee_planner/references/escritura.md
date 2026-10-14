# Escribir lo aceptado

## Antes del lote

1. `estado.py proposals-get <emp> <n…>`. Si falla por fecha, **[STOP]**: ofrece rehacer la pasada.
2. Comprueba que **todas** las operaciones son de turnos de quien ejecuta (`resource_id` suyo). Si alguna no, no se escribe nada de esa propuesta.
3. Comprueba que el proyecto y la tarea de cada `create` son de la compañía de la persona. Si no, sácalo del lote y dilo; no dejes que falle a mitad.

## Turno a turno

- **Huecos cortos de gestión** (`kind: interno`): contra el proyecto interno y su tarea de gestión, **con su nombre en `name`** (el `label` que devuelve el script, que es `hueco_corto_nombre` de la tabla). Sin nombre, el tablero se llena de bloques del proyecto interno que nadie sabe leer. Si no había proyecto interno (hallazgo `sin_proyecto_interno`), no se escriben.
- **Huecos cortos de una reunión de cliente**: son turnos de trabajo normales de ese proyecto y de la tarea de la reunión. Si la reunión no tenía tarea, se escriben sin ella y con el nombre de la reunión, y sale el hallazgo `reunion_sin_tarea`.
- **Previsiones** (`kind: soft`): igual que un turno, con **la marca soft puesta** (el campo descubierto en el preflight) y contra su proyecto si existe; si no, sin proyecto y con su nombre en `name`. Si el preflight no encontró la marca, no se escriben: se dice y quedan solo en la propuesta.
- **create**: `planning.slot` con `resource_id`, `start_datetime`, `end_datetime` (**UTC**, convertidos desde `tz` de la persona), `project_id` y `task_id` **en el mismo create**, `company_id` de la persona, `name` cuando no sale de una tarea («Holgura», «Vacaciones», «Comité interno», «Reunión sin proyecto: <asunto>»). Nunca escribas `allocated_hours`: es calculado.
- Nada más crear: `estado.py written-add <emp> '{"slot_id":…,"start":…,"end":…,"task_id":…,"published":false,"proposal":n,"date":"…"}'`. **Antes del siguiente create**, no al final.
- **write** (mover o cambiar horas): solo `start_datetime`/`end_datetime`. Actualiza su entrada en `written.json` si es propio.
- **unlink**: solo turnos aceptados en la propuesta. Si es publicado o ajeno, ya se enseñó con su antes y su después; si no se enseñó, no se borra.

## Publicar

Solo si la persona lo ha confirmado **para esta publicación**. Escribir `state = 'published'` publica sin avisar; `action_planning_publish_and_send` publica **y envía correo** a la persona. Usa la primera salvo que pida el aviso. Volver a borrador (`action_unpublish`) exige ser gestor de Planificación.

## Si algo falla a mitad

Lo creado ya está en `written.json`. Para, di qué se escribió y qué no, y ofrece **dos salidas**: terminar lo que falta, o deshacer lo creado en esta aceptación (borrar los `slot_id` de esa propuesta). Nunca reintentes el lote entero: duplicaría lo ya creado.

## Al terminar

Una línea: cuántos turnos publicados, cuántos en borrador, qué cambios pisaron algo publicado o ajeno, y horas de proyecto frente a horas que no son de proyecto. Registra en `commitments.json` y en `history.json`.
