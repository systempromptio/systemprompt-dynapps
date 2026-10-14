# Seguimiento diario: alta y baja

Una tarea programada **por persona**, y la da de alta la propia persona ejecutando la skill. Dar de alta a alguien no es dar de alta a su equipo: cada uno acepta solo lo suyo.

## Alta

1. Resuelve quién ejecuta (usuario de Odoo de la sesión → empleado). Solo se da de alta a esa persona.
2. Si `schedule.json` ya tiene un alta sin baja, no crees otra: di cuál existe.
3. Haz una pasada completa primero, para que la persona vea lo que le llegará cada mañana.
4. Comprueba que los conectores responden (documentación, Odoo, Outlook). Una tarea programada no tiene a nadie delante para autorizar uno; si alguno falla ahora, dilo antes de crearla.
5. Crea la tarea con la herramienta de tareas programadas de la sesión (identifícala por su función):
   - Frecuencia: días laborables, por la mañana (08:00 en su zona horaria, salvo que diga otra hora).
   - Instrucción: `Ejecuta employee-planner en modo diario para mí. Solo proponer, no escribir nada.`
   - Nombre: `planificacion-<nombre-de-la-persona>`.

   Si la sesión no ofrece esa herramienta, dale esos tres datos para que la cree a mano en la app, y que vuelva a ejecutar el alta para registrarla.
6. Anota el alta en `schedule.json`.

La pasada programada **nunca escribe**: propone y guarda `proposals.json`. La persona acepta después, en una sesión suya.

## Baja

- A petición: retírala y anota la fecha. El estado se conserva.
- Cuando la persona se queda sin tareas activas asignadas, la pasada lo dice y propone la baja. No se retira sola.
