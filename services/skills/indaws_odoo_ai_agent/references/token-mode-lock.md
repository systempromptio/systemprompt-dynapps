# Token mode-lock — el token del agente NUNCA puede abrir un shell

> Patrón del build **web_shell** (2026-07), verificado en vivo. Cierra un RCE real:
> si el mismo token de sesión sirve para modo-agente Y para un PTY shell, un token
> filtrado (o un usuario no-admin al que abres el bubble) = ejecución de comandos.

## El fallo
Si el handshake deja que el **cliente** elija el modo (p.ej. `?mode=agent` en la URL
del WebSocket), nada impide conectar con `mode=` vacío y obtener un **shell PTY** con
el mismo token. El bubble era admin-only precisamente por esto.

## El fix: atar el modo al token, enforce server-side
1. La sesión (`web.shell.session`) lleva un campo `mode` (`shell`|`agent`, **default
   `shell`** = fail-safe), fijado al **mintar** el token (`connection_info(mode=...)`).
   Un valor desconocido → `shell` (nunca escala).
2. El handshake devuelve ese `mode` al sidecar (`_consume_token` → controller
   `verify` → `info["mode"]`).
3. El **sidecar IGNORA el `?mode=` del cliente** y usa `info["mode"]`. Un token
   agent → siempre agente, jamás PTY.
4. Abrir un `shell` real (RCE-capable) exige el grupo privilegiado; `agent` puede ir
   a un grupo menor — es SEGURO porque un token agent no puede convertirse en shell.

## Verificación (pentest, sin gastar tokens del modelo)
Minta un token `agent`, conéctate al WS **sin** `mode=` → el primer frame debe ser
`{"type":"ready"}` (agente), nunca bytes de terminal. Si sale un prompt de shell →
BREACH. Ver `odoo-ai-security-pentest`.

## Regla
El modo (y en general **toda capacidad**) se ata al token en el servidor; el cliente
nunca elige privilegio. `default=shell` + unknown→shell = fail-safe al modo estricto.
