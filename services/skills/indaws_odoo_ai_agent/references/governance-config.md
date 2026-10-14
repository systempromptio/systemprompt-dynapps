# Gobernanza y configuración en capas (admin vs usuario)

> Patrón del build **dynapps_ai_assistant** (2026-07). Separa lo que el usuario
> ajusta (preferencia) de lo que el admin impone (gobernanza) — clave para exponer
> el agente a usuarios reales sin perder control.

## Principio: GOBERNANZA vs PREFERENCIA
- **Preferencia** (autoservicio, `res.users`, en Preferencias): tono, longitud de
  respuesta, instrucciones personales, imagen/avatar. El usuario las cambia.
- **Gobernanza** (admin, en una fila per-usuario tipo `web.shell.quota`): acceso de
  escritura on/off, política de confirmación **forzada**, proveedor **bloqueado**,
  límites de recurso. El usuario NO puede aflojarlas.

## Resolución (orden de precedencia)
`admin-forzado (quota) > preferencia-usuario (clamp al techo) > default sistema`.
Ejemplos de resolvers:
- `_write_enabled_for(user)`: override per-usuario > switch global. Un admin activa
  escritura solo a usuarios de confianza, o la veta a uno concreto.
- `_resolve_confirm_policy(user)`: valor forzado por admin gana (sin clamp); si no,
  preferencia del usuario **acotada al techo** del sistema; si no, default.
- `_resolve_provider(user)`: lock del admin > preferencia > default.

## Política de confirmación en 3 niveles (configurable, sin abrir agujero)
- `always` (defecto, más seguro) · `risky_only` (auto lo rutinario, **SIEMPRE**
  confirma borrado/método arbitrario/lote grande/modelo sensible) · `never`
  (auto todo salvo el suelo intocable; opt-in, gateado por techo admin).
- **Innegociable pase lo que pase**: la blacklist del write-server, el cap de lote,
  la auditoría y `su=False`. Auto-aprobado **NO es silencioso**: se emite un aviso
  (`auto_write`) + refresco de vista.
- "Aprobar y no volver a preguntar en esta conversación" = confianza por-hilo
  (lo peligroso sigue confirmando; se resetea al reconectar).

## System prompt / comportamiento (dónde se entrega)
El bloque de comportamiento (longitud, tono, aviso de coste, instrucciones
empresa+usuario) se construye **agnóstico** en la sesión y se entrega por el
**system_prompt del SDK** / system message del provider — NO por un `CLAUDE.md`
en el cwd (ver GOTCHAS de la skill: con `--cap-drop ALL` el fichero es frágil).

## Techo del admin
`confirm_policy_ceiling` (y equivalentes): el admin fija cuán suelto puede ir un
usuario; `never` off hasta que el admin lo permita. La empresa fija instrucciones
que SIEMPRE aplican; las del usuario van encima.
