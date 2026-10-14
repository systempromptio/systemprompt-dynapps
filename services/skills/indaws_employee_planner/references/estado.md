# Estado por persona

Lo gestiona `scripts/estado.py`. Carpeta, por precedencia: `$EP_DATA_DIR` → `.employee-planner/` en el directorio actual → `$HOME/dynapps-informes/employee-planner/`. Un subdirectorio por id de empleado. JSON compacto; escrituras atómicas.

| Fichero | Contenido | Regla |
| --- | --- | --- |
| `written.json` | Turnos creados por la skill: `slot_id`, inicio, fin, tarea, publicado, propuesta, fecha | **Se anota al crear cada turno, no al final.** Sin él, todos los turnos se tratan como ajenos |
| `proposals.json` | Las propuestas de la última pasada, con sus operaciones exactas | Se reemplaza en cada pasada; solo vale el mismo día |
| `split.json` | Reparto semanal validado (`weekly_split`, horas por semana y proyecto), previsiones (`forecasts`: nombre, proyecto o `null`, horas por semana, fecha fin), quién lo validó y cuándo | Lo escribe `split-save` en el arranque. El diario lo reutiliza; se vuelve a validar si entra o sale un proyecto, si una previsión se firma o se cae, o si la persona lo pide |
| `history.json` | Una entrada por pasada: modo, propuesto, aceptado, señalado, parámetros de doctrina usados | Se conservan 60 |
| `commitments.json` | Qué aceptó y cuándo | Solo se añade |
| `dismissals.json` | Propuestas descartadas, con el motivo escrito | Un descarte se respeta: no se vuelve a proponer igual |
| `schedule.json` | Tarea programada: id, horario, fecha de alta y de baja | Lo escribe el alta |

Un `written.json` perdido no es grave pero se nota: desde ese momento todo cambio sobre turnos existentes lleva su antes y su después, como si fueran ajenos. Dilo en la cabecera.
