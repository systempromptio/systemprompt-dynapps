# Credentials & Setup — Ticket Solver

## Detección de entorno

`scripts/_os_detect.sh` (sourced por todos los scripts). Detecta:
- `windows` (Git Bash / Claude Code Desktop) — usa `$TEMP` para caché.
- `wsl` (Ubuntu en WSL) — `/tmp` estándar; rutas Windows en `/mnt/c/...`.
- `linux` / `macos` — `/tmp` estándar.
- Localiza `op` (1Password CLI) en PATH.

## Instancia corporativa (helpdesk) — MCP proxy Odoo (nombre de servidor variable)

El helpdesk vive en el portal corporativo Dynapps (Odoo 19). **No usa 1Password ni API key**: se accede por un MCP proxy de Odoo, autenticado con OAuth.

**El nombre del servidor no es fijo.** En Claude Code está registrado como `odoo-indaws` (`mcp__odoo-indaws__*`); en Cowork u otros entornos el mismo proxy puede estar conectado bajo otro nombre (ej. "Dynapps Spain"), lo que cambia el prefijo `mcp__<servidor>__` de cada herramienta. Este skill identifica el servidor correcto por las herramientas que expone (`whoami`, `orm_search_read`, `orm_post_message`, `orm_write`, `orm_create`, `orm_execute_kw`, `odoo_attachment_search`, `odoo_attachment_link`), no por el nombre del servidor. NUNCA hardcodees `odoo-indaws` al llamar una herramienta — usa el prefijo que resultó de la búsqueda del paso 1.

> **Precedencia**: para la conexión corporativa manda ESTE fichero, no el preflight
> de `references/shared/odoo-connection.md`. Aquel sigue nombrando el conector
> anterior (`mcp__claude_ai_INDAWS_MCP__*`) y ordena parar y pedir «claude.ai INDAWS
> MCP», que ya no existe: seguirlo bloquea el trabajo con una instrucción imposible
> de cumplir. De ese doc sigue siendo válido todo lo relativo a **odoo-pilot y al
> Odoo del cliente**.

Preflight, PRIMERA acción del turno:

1. Resuelve el prefijo del servidor: `ToolSearch` **por palabra clave** (NO `select:` con el nombre completo, que asume `odoo-indaws` y falla si el entorno usa otro nombre) → busca `orm_search_read orm_post_message odoo_attachment_search whoami`. Las coincidencias traen su nombre completo `mcp__<servidor>__<herramienta>`; ese `<servidor>` es el prefijo a reutilizar el resto de la sesión para cualquier herramienta de esta tabla.
2. Una vez por sesión, llama a `whoami` (con el prefijo resuelto). Devuelve `user` (login, nombre), `company` y `employee`. Ese usuario es **el autor de las notas** que publiques — verifícalo antes de publicar en nombre de nadie.
3. Si la búsqueda por palabra clave no devuelve ninguna herramienta con esos nombres → **[STOP]**. No caigas a RPC. Di al consultor, literal:

   > El MCP proxy de Odoo (herramientas `whoami`/`orm_*`) no está conectado en este entorno. Conéctalo (en Claude Code: `/mcp`; en Cowork: revisa los conectores/MCP disponibles del workspace). No puedo completar el OAuth por ti.

**Si `/mcp` falla con "rejected them on reconnect"**: el OAuth está bien pero el usuario no tiene permiso en el registro `mcp.server.config` del portal (HTTP 403, `-32003 Access denied to this MCP server`). No es un problema de credenciales y reautenticar no lo arregla — hay que darle acceso en Odoo (grupos/usuarios permitidos del servidor MCP).

La URL del servidor la gestiona la configuración MCP del entorno (en Claude Code: `claude mcp get <nombre-del-servidor>`, que en ese entorno es `odoo-indaws`; en Cowork u otros, la configuración de conectores propia del entorno), no este skill. Si el portal cambia, se actualiza ahí — no hay endpoint ni nombre de servidor hardcodeado en el skill.

**El MCP NO corre con sudo**, pese a lo que anuncia su propia descripción ("every model with sudo"). Verificado: una lectura de `base.automation` se rechaza con el error de permisos del usuario. Corre como el usuario autenticado, con reglas de registro y ACLs aplicadas — el mismo acceso que tendría el consultor en la interfaz. Si una lectura falla por permisos no es un fallo del skill: hay que pedir el grupo en Odoo.

## Instancia del cliente — 1Password + RPC

El MCP no llega a los Odoo de cliente. Ahí siguen las credenciales de bóveda:

```bash
eval "$(bash scripts/bootstrap_credentials.sh <slug>)"
```

Carga `ensure_credentials.sh <slug> Indaws-Clientes-API CLIENT_` y exporta `CLIENT_ODOO_URL/DB/USER/KEY`. Caché en `${TS_TMPDIR}/.odoo_creds_*` (TTL 4h, escritura atómica). La app de 1Password pide biometría/PIN en el primer `op item get` del día.

Sin slug de cliente no hace falta bootstrap: si el ticket se resuelve solo con el helpdesk, el MCP basta.

### Modo headless (cron / Workflow)

Ahí no hay MCP autenticado. Para ese caso —y solo para ese— `bootstrap_credentials.sh --with-indaws [<slug>]` carga además `indaws_api` de la bóveda `Employee` en `ODOO_*`, y los scripts legacy (`fetch_ticket_bundle.py`, `post_internal_note.py`, …) siguen funcionando por RPC. Requiere `eval "$(python3 scripts/ts_auth.py)"` después, para resolver el UID real del dueño del key.

## Inferencia de slug desde proyecto

El campo `project_id` tiene formato `[TAG] cliente` o `cliente descripción`. Pasos:
1. Elimina el prefijo `[TAG]` si existe.
2. Toma solo la **primera palabra**. Ej: `[MIG] piber bolsa de horas` → `piber`.
3. `bash scripts/vault_index.sh resolve <slug> Indaws-Clientes-API` para obtener nombre canónico.
4. Si `resolve` falla: pregunta al usuario primero. Solo si confirma que debe existir, corre `bash scripts/vault_index.sh list Indaws-Clientes-API`.

## Ítems con secciones master/staging

`load_client_credentials.sh` lo gestiona automáticamente: usa `master` por defecto. Para staging: pasa 4º argumento `staging` a `ensure_credentials.sh`.

## Flujo de auth 1Password

1. Primer `op item get` dispara la auth desktop (biometría/PIN).
2. Los siguientes `op item get` del mismo proceso son transparentes.
3. Si la app no está corriendo: error claro pidiendo abrirla.
4. Las credenciales se cachean (TTL 4h) → 1Password no se vuelve a consultar hasta que expire.

## Repitiendo en cada Bash tool call

Cada `Bash` es un subshell nuevo. Repite el bloque `eval` al inicio de cada call usando SIEMPRE el MISMO slug. Las llamadas posteriores leen del fichero (sin contactar 1Password ni Odoo). Coste: ~2 lecturas de fichero.

Esto **solo aplica al cliente**. Las llamadas MCP no son subshells: no hay nada que reexportar entre tool calls.

## Convenciones inDAWS

Para hosts `*.indaws.cloud` y `www.indaws.es`: URL **sin** sufijo `/odoo` y DB **en minúsculas**.

## WSL + CRLF

Si `TS_OS=wsl` con odoo-pilot (one-time por máquina):
```bash
mkdir -p /tmp/odoo-pilot-clean && for f in /mnt/c/Users/*/.claude/plugins/cache/dynapps-spain-ai/common-skills/*/skills/odoo-pilot/scripts/*.sh; do tr -d '\r' < "$f" > "/tmp/odoo-pilot-clean/$(basename "$f")" && chmod +x "/tmp/odoo-pilot-clean/$(basename "$f")"; done
```
