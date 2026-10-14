# Integración Odoo — usuario, endpoints, UI, cuotas

> Wiring Odoo-específico del host del agente. Patrones del build **web_shell** (v19, 2026-07).
>
> **Alcance real de la evidencia:** todo lo medido aquí sale de builds v19. La skill declaraba
> "v17-v19" y se corrigió a **v18-v19** el 2026-08-07: no hay ni una comprobación contra v17 en
> ninguna referencia, y dos mecanismos centrales no llegan tan atrás — el default readonly de
> `auth='none'` es **v18+** y `all_group_ids` es **v19 puro** (0 ocurrencias en
> `odoo17/odoo/addons/base/models/res_users.py` y en la de v18; 17 en v19). Lo que se afirme para
> v18 aquí es extrapolación desde v19 salvo que lleve cita de v18.

## Identidad y permisos

- El agente corre como el **usuario autenticado** (`su=False`); las herramientas ORM respetan
  record rules y access rights de ese usuario. No lo eleves a sudo "por comodidad".
- Deny-by-default sobre quién puede usar el agente: `group_ids`/`user_ids` en la config; vacío =
  privado. En v19 la comprobación de grupos usa `all_group_ids` (directos + implícitos), no
  `group_ids` (ver `odoo-mcp-builder/references/security-defaults.md`).

## Endpoints

- Rutas del host (lanzar turno, confirmar tool, stream): `type='json'`. **Desde v18** (no desde v19,
  como decía esta línea hasta 2026-08-07), cualquier ruta que escriba y sea `auth='none'` necesita
  `readonly=False`: mismo código en `odoo18/odoo/http.py:890`, `_check_and_complete_route_definition` y `odoo19/odoo/http.py:974`.
  Preferible `auth='user'` para la UI (sesión real).
- Callbacks del orquestador de sandbox → `auth='none'` + token single-use/TTL (ver
  `sandbox-isolation.md`).

## UI y streaming

- Client action (pantalla del agente) o systray, construidos con `odoo-owl-component-builder`.
- Progreso server→cliente: `bus.bus._sendone(partner_or_channel, "odoo_ai_agent.stream", payload)`
  (`bus/models/bus.py:110`, `ImBus._sendone`) + `bus_service.subscribe("odoo_ai_agent.stream", cb)`
  (`bus/static/src/services/bus_service.js`). No polling manual.
- El gate humano (`canUseTool`, ver `agent-loop.md`) publica la propuesta por el mismo bus y espera
  la confirmación desde la UI.

## Cuotas / metering

- Mide y limita **por usuario** ANTES de lanzar el bucle (el coste de tokens es real). Guarda
  consumo por conversación/usuario en un modelo Odoo; corta al superar la cuota.
- Registra qué herramientas con efecto se aprobaron (auditoría).

## Cliente MCP (si el agente usa un server MCP externo o el propio de Odoo)

Wire de las herramientas por `.mcp.json` (0600):
```json
{ "mcpServers": { "<name>": { "type": "http", "url": "https://host/mcp/<db>/<slug>",
  "headers": { "Authorization": "Bearer <token>", "X-Odoo-Database": "<db>" } } } }
```
Naming de tools resultante: `mcp__<server>__<tool>`. El server MCP se construye con
`common-skills:odoo-mcp-builder`; aquí solo se consume.
