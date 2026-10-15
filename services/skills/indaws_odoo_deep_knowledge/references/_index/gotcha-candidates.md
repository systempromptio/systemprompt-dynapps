# Gotcha candidates — inbox (pre-curación)

> **Qué es:** cola de entrada de **candidatos** a gotcha, cosechados al resolver tickets/incidencias
> (`ticket-solver`) o tareas de dev. NO son gotchas verificados: se curan hacia el dominio real
> siguiendo [`HOW-TO-EXTEND.md`](HOW-TO-EXTEND.md) — verificar contra fuente, asignar ID, `gen_index.py --write`,
> `kb_release_check.py`. Al integrarlos, se mueven a "Curado" con su ID y se vacía "Abierto".
>
> **Qué NO es:** un gotcha ya bueno (esos van al `gotchas.md` de su dominio, verificados).
>
> **Formato por entrada:**
> `- **<síntoma/comportamiento observable>** — causa raíz · fuente/versión si se conoce · origen (ticket/dev)`
> Etiqueta: `[GENERICO]` (va a un dominio de este KB) o `[CLIENTE:<slug>]` (va al `project-knowledge` del cliente, cuando exista).

## Abierto

<!-- añadir candidatos aquí; una línea por candidato. Ejemplo:
- **Al desreservar no se libera el quant huérfano** — el move queda en estado intermedio · stock v18 · ticket 12345 · [GENERICO]
-->

## Curado

<!-- mover aquí con el ID asignado y fecha cuando se integre al KB (p.ej. "integrado como <PREFIJO><n> — 2026-07-01") -->
