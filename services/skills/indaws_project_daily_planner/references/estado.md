# Estado entre pasadas

El estado vive fuera del plugin, que es caché de solo lectura. Carpeta raíz, por precedencia:

1. `$PDP_DATA_DIR`
2. `.project-daily-planner/` en el directorio actual (la carpeta del proyecto)
3. `$HOME/dynapps-informes/project-daily-planner/`

Un subdirectorio por proyecto, nombrado con su id de `project.project`. JSON compacto, UTF-8, fechas ISO (`YYYY-MM-DD`, o `YYYY-MM-DDTHH:MM` cuando importa la hora).

## Ficheros

| Fichero | Contenido | Regla |
| --- | --- | --- |
| `baseline.json` | Primera foto del plan: tareas con sus fechas y horas, presupuesto, fecha de arranque, fecha de creación del baseline | **No se sobrescribe nunca.** Es contra lo que se detecta una replanificación |
| `history.json` | Una entrada por pasada: fecha, métricas del día, hallazgos por id de comprobación, fuentes disponibles, artículo raíz leído | Se conservan las 60 últimas |
| `proposals.json` | Las propuestas de la **última** guía | Se reemplaza entero en cada pasada |
| `commitments.json` | Propuestas aceptadas: id de comprobación, texto, fecha de aceptación, fecha comprometida, registros escritos | Solo se añade |
| `dismissals.json` | Comprobaciones descartadas en este proyecto: id, motivo escrito, fecha, quién | Un descarte se respeta y no se vuelve a proponer |
| `schedule.json` | Seguimiento programado: id de la tarea, carpeta desde la que se creó, horario, fecha de alta y de baja | Lo escribe el Paso 6 |

## `proposals.json`

Es lo que permite aceptar horas después, en otra sesión, lo que propuso la pasada de la mañana.

```json
{"generated":"2026-09-29T08:05","project_id":1234,"proposals":[
 {"n":1,"check":"D9","label":"la ventana de la tarea ya no es creíble","action":"reschedule_task",
  "records":[{"model":"project.task","id":22553,"name":"01 · Configuraciones generales"}],
  "payload":{"date_deadline":"2026-10-15"},"needs_separate_confirmation":false}
]}
```

- `action` es siempre una acción del catálogo cerrado de la skill. Una propuesta sin acción aplicable (solo informativa) lleva `"action":null`.
- `payload` es exactamente lo que se escribirá si se acepta: el consultor aceptó ese texto, no otro.
- `needs_separate_confirmation` es `true` para lo que no entra en una aceptación en bloque (la fecha de salida a producción).
- Si `generated` no es de hoy, la aceptación se para y se ofrece rehacer la pasada.

## Primera ejecución

No hay baseline: se crea con la foto actual y la guía dice que es la primera pasada, para que la ausencia de deriva no se lea como buena señal.
