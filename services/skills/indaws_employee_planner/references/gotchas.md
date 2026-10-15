# Gotchas de datos, fuentes y escritura

## Doctrina

1. **No te quedes con el primer resultado de la búsqueda.** El índice de Planificación y Dedicación mencionan lo mismo y pueden puntuar por encima de Colocación. Elige por título y ruta, y verifica el contenido.
2. **Los artículos pueden contradecirse.** Para la herramienta manda la tabla de parámetros de Colocación; la contradicción se cita en la cabecera. Ver `parametros.md`.

## Odoo

3. **El MCP corporativo corre como superusuario.** Odoo no te va a impedir escribir el turno de otra persona: la única barrera es la regla de la skill. Con `odoo-pilot` como el propio usuario pasa al revés: puede que no tenga permiso para crear turnos (Planificación reserva crear a sus gestores). Si falla por permisos, dilo; no busques otra vía.
4. **`project.task.state` no dice si una tarea está terminada**: se queda en `01_in_progress` en tareas cerradas hace meses. Manda si la etapa está plegada.
5. **`date_deadline` de la tarea es datetime; la ventana del script es por fecha.** Normaliza en la zona de la persona: una fecha límite a las 23:00 UTC es el día siguiente en Madrid.
6. **El campo de consultores del proyecto no sirve para saber quién trabaja en él.** Los asignados son los de cada tarea (la doctrina dice por qué).
7. **El rol de planificación puede estar vacío** aunque la tabla diga `rol_desde = rol_planificacion`: `colocar.py` cae entonces al departamento para esa persona y lo reporta en `rol_deducido_del_departamento`.
8. **Publicar con `action_planning_publish_and_send` envía correo.** Escribir el estado no. Elige a propósito.
9. **Los turnos recurrentes (`recurrency_id`) se regeneran.** Mover uno de una serie no mueve la serie, y borrarlo puede no impedir que vuelva. Trátalos como ajenos y fijos, y si estorban, señálalo.

## Fuentes fuera de Odoo

10. **Una reunión de Outlook sin asistentes externos puede ser trabajo de cliente** («Rotger · Preparar demo compras»). Con `reunion_interna_por_asunto`, el asunto decide si nombra un proyecto o cliente; si no lo nombra, es interna y no se adivina por el contexto. Las que suenan a preparación sin nombrar proyecto se señalan y se preguntan.
11. **Reuniones solapadas**: el tramo se fusiona y cuenta una vez; se asigna a la primera. Dilo si el solape es entre dos clientes.
12. **Una reunión fuera de jornada** no consume capacidad y el script la ignora. Si es recurrente, señálalo: está comiendo horas que el plan no ve.

14. **Unas fechas de tarea vencidas que no son reales dominaban la semana.** En la v2.1, las vencidas pasaban delante de todo y un proyecto con cinco tareas de julio dejaba a cero al proyecto principal. Ahora (`vencidas_dentro_del_reparto`) solo consumen el reparto de su proyecto, y si no caben sale `vencidas_superan_reparto` con la propuesta de actualizar fechas, cerrar tareas o revisar el reparto. **La skill nunca toca las fechas.**
15. **El go-live corta el trabajo de implantación** (`cortar_en_go_live`). Las tareas que acaban después del go-live dejan de recibir horas en esa fecha, y las semanas siguientes pueden salir vacías aunque el proyecto tenga cientos de horas: sale `trabajo_cortado_en_go_live` por tarea. Casi siempre es que la fecha de go-live o las de las tareas ya no son reales.

## Ejecución programada

13. **La pasada de la mañana no escribe nada.** Si alguien le pide "aplícalo" en la propia tarea programada, no hay nadie que acepte: se guarda y se espera a una sesión de la persona.
