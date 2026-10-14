# Modo NUEVO — las 7 fases en detalle

## Fase 1 — Setup en verde (10 min máximo)

Comprueba en orden y no avances con algo en rojo:
1. Conector Odoo corporativo responde (`whoami` o llamada de prueba).
2. Credenciales del marketplace configuradas: delega la verificación al skill `onboarding`
   (common), que ejecuta su `check_prerequisites.sh`. No reimplementes esa lógica.
3. Skills del perfil del consultor instalados (contrasta con `perfiles.md`); si falta un
   plugin, anótalo y recomienda instalarlo.

El porqué de la rigidez: un onboarding sobre un setup roto quema la primera impresión, y la
primera impresión es la mitad de la adopción.

## Fase 2 — Conocer al consultor (leer, no preguntar lo que ya está en Odoo)

Lanza `Agent(model: sonnet)` con el email del consultor y estas consultas (solo lectura):
- `res.users` por email → login, create_date, último login.
- `hr.employee` → puesto, departamento, responsable (parent_id).
- `project.task` asignadas (abiertas y cerradas) → proyectos en los que está.
- `helpdesk.ticket` asignados → temática (fiscal, técnica, funcional).
- `account.analytic.line` desde su arranque → cuántos, qué días, quién los creó (`create_uid`).

El subagente devuelve los datos en crudo; el principal decide. Propón un perfil de los cinco
de `perfiles.md` a partir de las señales.

## Fase 3 — (STOP) Confirmar perfil

AskUserQuestion con el perfil propuesto y los otros como alternativas. Nunca lo des por hecho:
ficha antigua, partes ajenos y actividad heredada engañan (GOTCHAS 1-3 del SKILL.md).

## Fase 4 — Itinerario personalizado (3 casos, no 15)

Lee `perfiles.md` y presenta: el quick win, tres casos estrella con sus skills, y qué queda
para más adelante. La sobrecarga mata la adopción: mejor 3 casos dominados esta semana que 15
que suenan y no se usan. Lo avanzado (subagentes, tareas encadenadas, memoria de proyecto) se
menciona como "existe, lo verás con rodaje"; no se demuestra el día 1.

## Fase 5 — (STOP) El primer parte con sus manos

No termines la sesión sin esto:
1. Pregúntale qué ha hecho hoy (una cosa basta; la propia sesión de onboarding vale).
2. Busca el proyecto/tarea en el corporativo. Sin tarea asignada: búscala con él; si no hay
   ninguna, usa la tarea/proyecto interno de onboarding.
3. Muestra el parte propuesto (proyecto, tarea, descripción, horas) y confirma con
   AskUserQuestion antes de escribir. Verifica que la instancia destino es el corporativo.
4. Crea el `account.analytic.line` y enséñale el registro creado.

El porqué: el hábito no nace de entender el prompt, nace de ver SU parte creado en 30 segundos.
Ese momento es el objetivo de todo el día 1.

## Fase 6 — Programar el ritual

Ofrece (no impongas) dos tareas programadas:
- **Briefing matutino** (laborables, ~8:30): el planner/briefing de su perfil según
  `perfiles.md`.
- **Cierre del día** (~17:30): repaso de lo hecho y partes del día, con confirmación.

Un ritual impuesto se abandona; uno elegido se queda. Registra también el rechazo explícito:
decidir que no, es una decisión válida (cuenta como hito de día 1).

## Fase 7 — Registrar progreso y entregar la chuleta

Guarda en `${DYNAPPS_HOME:-~/.dynapps}/onboarding/dynapps-spain-ai/claude-onboarding-<login>.json`
(legacy: `~/.claude/onboarding/...`, que se sigue leyendo si existe):
fecha de arranque, perfil confirmado, hitos de día 1 (setup, primer parte, ritual decidido),
fecha del próximo checkpoint (día 7). Es lo que lee el modo SEGUIMIENTO.

Ofrece dejar nota en el chatter de su tarea de onboarding en Odoo si existe (transparencia con
su responsable, decidida por el consultor). Cierra entregando `prompts.md`.
