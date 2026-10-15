---
name: indaws-claude-onboarding-dynapps
description: >
  Ejecuta el onboarding interactivo de Claude para nuevos consultores de Dynapps Spain: audita
  el setup (credenciales, conector Odoo, plugins), lee la ficha real del consultor en el Odoo
  corporativo, genera un itinerario por perfil, guia el primer parte real y verifica hitos a
  1/7/30 dias con informe de adopcion para el mentor.
  Usar cuando digan: "empieza mi onboarding", "soy nuevo", "prepara la sesion de onboarding de X",
  "como va la adopcion de X". NOT solo credenciales (usar onboarding); NOT dudas de metodologia
  (usar consultant-assistant); NOT mentoria continua (usar odoo-mentor).
version: 1.1.1
effort: medium
model: opus
allowed-tools: [Read, Write, Bash, Glob, Grep, AskUserQuestion]
---

# Claude Onboarding Dynapps

Principio: **el onboarding no se lee, se practica**. Un consultor no adopta Claude por leer una
guía; lo adopta la primera vez que resuelve algo real con sus manos. Este skill convierte a
Claude en quien ejecuta el onboarding: comprueba, personaliza, acompaña el primer éxito y mide
si el hábito cuaja. Caso que lo motiva: un experto con dos semanas resolviendo tickets reales y
cero partes de horas. No le faltaba información, le faltaba circuito: trabajo que ni se medía
ni se facturaba.

## Reglas de oro (no negociables)

1. **Producción real de cliente: solo lectura, siempre.** Ni durante demos.
2. **El Odoo corporativo sí se escribe** para lo propio del consultor (partes, tareas, notas),
   siempre con confirmación previa. Por eso el primer parte es el quick win seguro.
3. **Credenciales nunca inline.** Si faltan, delega en `onboarding` (common). No pidas API
   keys por chat.
4. **El informe de adopción habla de proceso, no de personas.** Va al mentor y señala huecos
   de circuito, no juicios de rendimiento. Delante del consultor: acompañar, no auditar.
5. **Provenance marker.** Cualquier fichero generado que admita comentarios acaba con
   `Made with claude-onboarding-dynapps · v<version>` (el estado en JSON está exento). Regla:
   `references/shared/skill-provenance-marker.md`.

## Modos (detección)

| Señal | Modo |
|-------|------|
| El consultor nuevo arranca: "empieza mi onboarding", "soy nuevo" | **NUEVO** |
| Mentor/PM o tarea programada: "cómo va la adopción de X", "informe 7/30 días" | **SEGUIMIENTO** |
| Mentor prepara la sesión: "prepara la sesión de onboarding de X" | **MENTOR** |

Si no está claro, pregunta. Un mentor puede arrancar NUEVO en nombre de otro: pide el email
del consultor y trabaja sobre su ficha, no sobre la del mentor.

## Conexión a Odoo

Conector MCP del Odoo corporativo si existe; si no, `odoo-pilot` con credenciales corporativas
(odoo-corporate del sistema de onboarding, patrón en `references/shared/odoo-connection.md`).
Sin credenciales: delega en `onboarding` y detente. Lecturas solo lectura; única escritura: los
partes/tareas propios del consultor en el corporativo, confirmados.

## Agent Routing

| Fase | Modelo | Por qué |
|------|--------|---------|
| Fase 1 NUEVO / paso 2 SEGUIMIENTO — recolectar ficha y actividad Odoo | sonnet (subagente) | Lectura mecánica de datos, sin decisiones |
| Itinerario, diagnóstico, informes, conducción de la sesión | opus (principal) | Personalización y juicio sobre datos ambiguos |
| Escritura del parte guiado | opus (principal) | Escritura en corporativo con gate humano |

Para la recolección, lanza `Agent(model: sonnet)` con: email del consultor, fecha de arranque
estimada y la lista de consultas de `references/modo_nuevo.md` (o `references/hitos.md` en
seguimiento). El subagente devuelve datos, nunca conclusiones.

## Workflow

### Modo NUEVO — detalle completo en `references/modo_nuevo.md`

1. **Setup en verde** — conector, credenciales (delega a `onboarding`), plugins del perfil.
   No avances con algo en rojo.
2. **Conocer al consultor** — subagente sonnet lee ficha y actividad. Propón perfil.
3. **(STOP) Confirmar perfil** — AskUserQuestion. Los datos engañan (GOTCHAS 1-3).
4. **Itinerario personalizado** — 3 casos de su perfil desde `references/perfiles.md`, no 15.
5. **(STOP) Primer parte con sus manos** — el corazón: parte real de hoy, confirmado antes de
   escribir. No termines la sesión sin esto.
6. **Ritual programado** — ofrece briefing matutino y cierre del día como tareas programadas
   (vía `consultant-daily-planner` si la suite consultant está instalada; si no, una tarea programada genérica).
7. **Registrar progreso** — estado en `${DYNAPPS_HOME:-~/.dynapps}/onboarding/dynapps-spain-ai/claude-onboarding-<login>.json`;
   entrega `references/prompts.md`.

### Modo SEGUIMIENTO (checkpoints día 7 y 30, one-shot)

1. Lee el fichero de progreso; si no existe, reconstruye la fecha de arranque (GOTCHA 1).
2. Subagente sonnet recolecta actividad real: partes (autoría real, GOTCHA 2), tareas,
   tickets, chatter.
3. Contrasta contra `references/hitos.md`.
4. **Horas fantasma**: trabajo visible (tickets, chatter) sin partes que lo cubran → bandera
   explícita. Es la fuga silenciosa de margen y la razón de ser del seguimiento.
5. Informe corto al mentor: verde/ámbar/rojo por hito, banderas, UNA siguiente acción con
   responsable. Prosa, media página.

Tras el checkpoint de día 30, **handoff a `odoo-mentor`** (common): la mentoría continua y los
nudges periódicos son suyos. Este skill no monta un segundo motor de recordatorios; su ciclo
termina cuando el hábito está medido.

### Modo MENTOR

Lee ficha y actividad del consultor (mismas consultas que NUEVO), después genera el guion desde
`references/guion_sesion.md` adaptado a su perfil y estado real. Un guion sin datos del
consultor es media sesión perdida.

## GOTCHAS

1. **Ficha antigua ≠ alta real.** Contactos de años atrás con alta efectiva reciente: para la
   fecha de arranque usa el máximo entre `create_date` del `res.users` y su primer login.
2. **Partes mal atribuidos.** `account.analytic.line` colgados de su empleado creados por
   otros (`create_uid` distinto): al medir SU actividad, filtra por autoría real.
3. **El perfil por datos engaña.** Un fiscal parece "helpdesk", un dev "sin actividad".
   Propón, pero confirma con la persona.
4. **El primer parte va al Odoo corporativo**, nunca al de un cliente. Verifica la instancia
   destino antes de escribir.
5. **No volcar el catálogo.** Tres casos por perfil el día 1; la sobrecarga mata la adopción.
6. **El informe de adopción no se comparte con el consultor sin contexto.** Lo entrega el
   mentor, que decide cómo conversarlo.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| onboarding | common | Credenciales faltantes (paso 1) |
| odoo-pilot | common | API si no hay conector MCP |
| odoo-mentor | common | Handoff de mentoría continua tras el día 30 |
| Skills por perfil | varias | Casos estrella (ver references/perfiles.md) |

**Delegaciones opcionales (suite consultant — común NO depende en firme de una hija):** si la
suite consultant está instalada, `consultant-daily-planner` monta el ritual matutino (perfiles 1,
2, 5) y `consultant-assistant` resuelve dudas de metodología durante la sesión. Si no está, degradar:
tarea programada genérica para el ritual y seguir sin bloquear.

## Consumed by

| Quién | Para qué |
|-------|----------|
| Mentores / PMs | Sesiones y checkpoints de adopción |
| Tareas programadas | Checkpoints automáticos día 7 y 30 |

## References

| Fichero | Contenido | Cuándo |
|---------|-----------|--------|
| `references/modo_nuevo.md` | Las 7 fases del modo NUEVO en detalle, con consultas Odoo | Modo NUEVO |
| `references/perfiles.md` | Itinerarios por perfil: quick win, 3 casos, skills, ritual | Fase 4 NUEVO, MENTOR |
| `references/hitos.md` | Hitos día 1/7/30 con verificación Odoo y formato del informe | SEGUIMIENTO |
| `references/guion_sesion.md` | Guion de sesión de acompañamiento (45-60 min) | MENTOR |
| `references/prompts.md` | Chuleta de prompts para el consultor | Cierre de NUEVO |
