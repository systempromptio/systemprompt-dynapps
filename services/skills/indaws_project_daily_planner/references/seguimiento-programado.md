# Seguimiento programado: alta y baja

La revisión diaria corre como una tarea programada **por proyecto**. La crea el propio consultor ejecutando esta skill **desde la carpeta del proyecto**. Si no se hace así, no funciona: la tarea corre en el directorio en el que se creó, y desde otro sitio cada mañana saldría una guía sin carpeta, sin estado y sin memoria de ayer.

## Alta

1. **Comprueba la carpeta.** El directorio actual tiene que ser la carpeta del proyecto: su nombre o su contenido (análisis, actas, cuestionarios, un `.project-daily-planner/` anterior) tienen que corresponder al proyecto resuelto en Odoo. Si no se puede afirmar, **[STOP]**: di en qué carpeta estás y pide al consultor que abra la sesión en la del proyecto. No lo crees "de momento" desde otra.
2. **Haz una pasada completa primero.** Así se crea el baseline y el consultor ve la guía que le va a llegar cada mañana antes de programarla.
3. **Comprueba los conectores** con los que correrá la tarea: documentación (`docs_*`), Odoo y Microsoft 365. Una tarea programada no tiene a nadie delante para autorizar un conector; si alguno no responde ahora, dilo antes de crearla y di qué bloques se quedarán sin comprobar.
4. **Crea la tarea programada** con la herramienta de tareas programadas que ofrezca la sesión (identifícala por su función, no por su nombre). Parámetros:
   - Carpeta de trabajo: la actual, la del proyecto.
   - Frecuencia: días laborables, por la mañana (por defecto 08:00, hora local del consultor).
   - Instrucción: `Ejecuta project-daily-planner para el proyecto <id> · <nombre>. Revisión diaria, sin aplicar nada.`
   - Nombre: `Revisión diaria · <nombre del proyecto>`.

   Si la sesión no ofrece esa herramienta, dale al consultor esos cuatro datos para que la cree él a mano en la app, **desde esta misma carpeta**, y pídele que vuelva a ejecutar el alta para registrarla.
5. **Registra el alta** en `schedule.json`: id de la tarea, carpeta, horario, fecha.

## Baja

- A petición: retira la tarea con la misma herramienta (o indica al consultor cómo hacerlo) y anota la fecha de baja en `schedule.json`. El resto del estado se conserva.
- Por etapa terminal: cuando el proyecto pasa a *Terminado* o *Cancelado*, la guía de ese día lo avisa y propone la baja. No se retira sin su sí.

## Errores típicos

- **Tarea creada desde la carpeta de inicio o desde otro proyecto.** Corre, pero sin carpeta y con el estado equivocado o vacío. Se detecta porque `schedule.json` registra otra carpeta que la actual: dilo en la cabecera de la guía.
- **Dos tareas para el mismo proyecto.** Si `schedule.json` ya tiene un alta sin baja, no crees otra: di cuál existe.
