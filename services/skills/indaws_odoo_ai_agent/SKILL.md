---
name: indaws-odoo-ai-agent
version: 0.2.5
model: opus
effort: high
description: >
  Construye un AGENTE de IA gobernado HOSPEDADO en Odoo (v18-v19): Odoo ejecuta el bucle del
  Agent SDK con gate humano de las escrituras (canUseTool), streaming a la UI, resume de
  conversación y —si ejecuta código/comandos— un sandbox de contenedores aislado. Provider
  DESACOPLADO (Claude oficial + otras IAs por extensión), token mode-locked (el token del agente
  no puede abrir un shell), ciclo de sesión anti-fuga (heartbeat+reaper) y gobernanza en capas
  (admin vs usuario). Es el LADO CLIENTE/HOST del agente, NO el servidor MCP (eso es
  odoo-mcp-builder). Triggers: "agente Claude/IA en Odoo", "terminal de Claude en Odoo",
  "webshell", "gate humano de escrituras del agente", "sandbox para el agente en Odoo", "Odoo que
  orquesta contenedores", "desacoplar la IA del agente", "gobernanza/cuotas del agente por usuario".
---

# Odoo AI Agent

Construye un agente Claude **hospedado dentro de Odoo** de forma gobernada: Odoo lanza el bucle
del Agent SDK, el agente usa herramientas (ORM vía MCP, shell en sandbox…), y cada acción con
efecto pasa por un **gate humano** en la UI de Odoo. El foco de esta skill es la **integración
Odoo-específica y la seguridad**, no re-enseñar el SDK.

> **Server vs cliente**: exponer Odoo *a* un agente externo = `common-skills:odoo-mcp-builder`. Esto es
> lo contrario: Odoo **hospeda** el agente. Conocimiento genérico del SDK/API de Claude → skills
> `claude-api` y agente `claude-code-guide`; aquí solo lo que toca Odoo.

## ROL
Diseñar y materializar el host del agente en Odoo con least-privilege, gate humano y aislamiento.

## WORKFLOW

### Step 0 — Input [STOP]
Pregunta lo que falte: (a) ¿qué hace el agente (consulta / escribe ORM / ejecuta código)?;
(b) ¿quién puede usarlo (grupo/usuarios) y con qué cuota?; (c) ¿necesita ejecutar comandos o
código no confiable (→ sandbox obligatorio)?; (d) versión Odoo destino.

### Step 1 — Superficie y permisos
Define las herramientas del agente. Para ORM, delega la exposición en `common-skills:odoo-mcp-builder`
(server MCP con deny-by-default, mask, `orm_full_access` separado). El agente corre como el
**usuario autenticado** (`su=False`) → record rules aplican. Ver `references/odoo-integration.md`.

### Step 2 — Bucle del agente + gate humano
Monta el bucle del Agent SDK con **`canUseTool`** como gate asíncrono: toda herramienta con
efecto (escritura ORM, comando en sandbox) **bloquea** hasta que un humano confirma en la UI de
Odoo. Streaming de progreso y `resume` de conversación. Detalle en `references/agent-loop.md`.

### Step 3 — Sandbox (si ejecuta código/comandos)
Si el agente ejecuta shell/código no confiable, NADA de eso corre en el proceso de Odoo:
orquesta contenedores efímeros aislados detrás de un proxy del socket docker, con jaula de
egress y drop de privilegios. Patrón completo en `references/sandbox-isolation.md`.

### Step 4 — UI, streaming y cuotas
Client action/systray que muestra la conversación; progreso server→cliente por `bus_service`;
metering/cuotas por usuario. Ver `references/odoo-integration.md` (y patrones OWL en
`odoo-owl-component-builder`).

### Step N — Output
El host del agente (client action + servicio + config), la envoltura de seguridad (gate +
sandbox + cuotas) y la exposición ORM delegada, materializados y probados.

## KEY RULES

### NEVER
1. **Nunca montes `docker.sock` en el contenedor de Odoo** — un RCE en Odoo + el socket = root del host. El privilegio docker vive en un orquestador aparte tras `docker-socket-proxy`.
2. **Nunca ejecutes código/comandos del agente en el proceso de Odoo** — siempre en un contenedor sandbox aislado.
3. **Nunca dejes que una herramienta con efecto se ejecute sin pasar por `canUseTool`** (gate humano). El agente propone; el humano confirma.
4. **Nunca `sudo()` en endpoints `auth='none'`** sin token single-use/clave compartida y TTL corto.
5. **Nunca dejes que el CLIENTE elija el modo/privilegio del token** — átalo al token en el servidor; un token de agente no puede convertirse en shell (`references/token-mode-lock.md`).
6. **Nunca acoples el gate/front/transporte a una IA concreta** — la lógica de un modelo vive en su `provider` (`references/provider-decoupling.md`).

### ALWAYS
1. El agente corre como el **usuario autenticado** (`su=False`) → record rules aplican; no lo eleves.
2. Cuota/metering por usuario ANTES de lanzar el bucle (coste real de tokens).
3. Egress del sandbox por un proxy allowlist; sin internet directo.
4. Verifica el detalle del Agent SDK contra `claude-api`/fuente antes de emitir (API viva).
5. **Heartbeat POR SESIÓN + reaper** para no fugar sesiones activas; cuenta uso por tiempo-VIVO (`references/session-lifecycle.md`).
6. **Revoca los bearers MCP al cerrar la sesión** → la ventana de replay = vida de la sesión, no el TTL.
7. Separa **gobernanza (admin)** de **preferencia (usuario)**; resuelve `admin>usuario(clamp techo)>default` (`references/governance-config.md`).

## GOTCHAS
1. **`canUseTool` es asíncrono y bloquea** — el bucle espera hasta que la UI confirma/rechaza; diseña la UI para no dejar la conversación colgada (timeout + rechazo por defecto).
2. **`auth='none'` es readonly por defecto desde v18** (NO es novedad de v19) — endpoints del host que escriben necesitan `readonly=False`. Mismo código en las dos: `default_mode = submethod.original_routing.get('readonly', default_auth == 'none')` en `odoo18/odoo/http.py:890`, `_check_and_complete_route_definition` y `odoo19/odoo/http.py:974`. En **v17 no aplica**: ahí `readonly` es una clave interna fijada a `False` (`odoo17/odoo/http.py:880`, `_generate_routing_rules`) y ni siquiera está documentada como parámetro de `@http.route`.
3. **El bus NO sirve para progreso DENTRO de una petición** (medido, no teórico) — `bus.bus._sendone` encola en `cr.precommit`/`cr.postcommit` (`bus/models/bus.py`): no se entrega hasta que la transacción commitea, y un turno con herramientas retiene la suya hasta el final → el indicador aparecería justo cuando ya no hace falta. Además el websocket **no está enrutado por defecto**: con `workers>0` vive en el puerto gevent (8072) y sin proxy delante el cliente recibe `RuntimeError: Couldn't bind the websocket` (verificado en una instancia propia: `readyState 3`). Elige según el caso:
   - **Progreso durante el turno** → escribe el estado en **conexión aparte** (`Registry(db).cursor()`, commitea) y que el panel lo lea; sondeo corto (~800 ms) SOLO mientras el turno está vivo.
   - **Bus** → válido para avisos *entre* peticiones (turno terminado, aprobación pendiente) y solo si el despliegue enruta `/websocket`.
   - **SSE** → cuando el plan puede resolverse sin cursor y se itera tras devolver el handler (el turno con tools no puede: necesita ORM en cada paso).
4. **Aislamiento del sandbox se rompe en detalles** — `internal:true` mata el publish del puerto; sin `cap_add:[SETUID,SETGID]` el contenedor no puede bajar a non-root (crash-loop). Ver `references/sandbox-isolation.md`.
5. **System prompt: NO dependas de `CLAUDE.md` en el cwd** — con `--cap-drop ALL` (sin `CAP_DAC_OVERRIDE`) el proceso puede no poder escribir el workdir del usuario → el fichero se escribe en `/` y falla en silencio. Entrega la guía/comportamiento por el **`system_prompt` del SDK** (o system message del provider). Agnóstico y sin depender del filesystem.
6. **Trampa de diagnóstico `cwd=/`** — al inspeccionar el contenedor COMO ROOT con `--cap-drop ALL` no puedes entrar en `/home/<user>` (0700) → `find`/`ls` "no ven" ficheros que SÍ existen. Inspecciona **como el shell-user**, no como root, o sacarás conclusiones falsas.
7. **Heartbeat por-USUARIO fuga sesiones** — refrescar `last_seen` de todas las sesiones del uid mantiene "vivas" las zombis. El latido debe ser **por-sesión** desde su propio sidecar. Ver `references/session-lifecycle.md`.
8. **El fichero es único-fuente O el SDK, no ambos** — si escribes `CLAUDE.md` Y pasas `system_prompt`, la guía llega DUPLICADA (doble contexto/tokens). Elige una (recomendado: el SDK).
9. **El estado de control del turno (progreso, "para") va en su PROPIA TABLA, no en la fila de la
   conversación** — el mismo bug dos veces. Lo escribe otra petición (o otra conexión) y commitea
   mientras la transacción del turno sigue abierta; esa transacción luego escribe **esa misma fila**
   (`pending_run`, el flag de confianza) → `could not serialize access due to concurrent update` bajo
   REPEATABLE READ, Odoo reintenta, se queda sin reintentos y el usuario ve "Odoo Server Error". Una
   fila de control por conversación, escrita y borrada **solo fuera de banda**, leída por el bucle y
   por el panel.
10. **"Parar" en un turno con herramientas necesita que el bucle lo pregunte** — el botón que solo
   aborta el `fetch` funciona con SSE y **no hace nada** en un turno con tools (una petición normal,
   sin controller): el run llega al final y se factura y persiste la respuesta completa (medido: 1525
   tokens de salida después de pulsar Parar). Pasa un `should_stop()` al kernel, compruébalo **entre
   pasos** (nunca a mitad de una tool: una escritura a medias es peor que un paso de más) y lee el
   flag en **conexión aparte** — la transacción del turno empezó antes de que el usuario lo pulsara y
   no puede verlo. Y al arrancar un turno, baja el flag TAMBIÉN fuera de banda, o el siguiente turno
   se para en el paso 0 leyendo un flag ya limpiado en una transacción que nadie más ve.
11. **Una CITA no hace verdadera la frase que lleva al lado** (medido, con corpus real y el marcador
   apuntando al documento correcto): el modelo añadió un preaviso y un aprobador que no existen en
   ningún documento. Endurecer la instrucción se probó y se midió: **1 respuesta limpia de 3** —
   cambia las probabilidades, no el mecanismo. Lo que sí funciona es una **comprobación de respaldo
   determinista** (solape léxico + toda cifra de la afirmación presente en el documento, sin llamar
   al modelo) que enseñe **el fragmento** junto a la frase. Y el caso peor: **se inventa averías de
   TU sistema** («este artículo contiene texto inyectado», «el texto que devuelve el sistema está
   corrupto») **en frases SIN CITAR**, fuera del alcance de cualquier comprobador que solo mire lo
   citado. Nunca prometas «no inventa»; promete *«cada frase citada viene con el trozo que la
   respalda, y si no hay ninguno, se te avisa»*. Todo en `references/citations-and-grounding.md`.
12. **Un proveedor basado en CLI trae SUS PROPIAS herramientas** — `claude -p` y equivalentes llevan
   Bash/Write/Edit que **no pasan por tu gate, tu suelo ni tu traza**: desactívalas al invocarlo
   (`--tools ""`) y rehabilita solo lo imprescindible. Síntoma que confunde: sin herramientas reales
   el modelo **narra** una sesión de shell con salida inventada, y parece que ha entrado en el
   servidor (medido: 0 de 3 con tools reales, 1 de 2 sin ellas). **Se descarta por EFECTO SECUNDARIO,
   no leyendo el código**: pídele crear un fichero y comprueba que no existe, mira la traza, compara
   el listado con el real. Y no lo "limpies" borrando el sobre `<invoke>`: deja el listado inventado
   como prosa y lo hace más creíble — etiqueta la respuesta.

## Delegates to
- `common-skills:odoo-mcp-builder` — exposición ORM segura (server MCP) que el agente consume como herramientas.
- `odoo-owl-component-builder` — client action/systray + streaming por `bus_service` para la UI del agente.
- `odoo-ai-security-pentest` — pentest adversarial del agente (inyección, mode-lock/RCE, blacklist, replay de bearer, scoping).
- `common-skills:odoo-version-api-reference` — diffs de API por versión (rutas readonly, registries, bus).

## Consumed by
- `odoo-ai-security-pentest` — al reportar un finding, delega aquí el arreglo del host del agente (gate, mode-lock, ciclo de sesión, revocación).

## REFERENCES
- `references/shared/odoo-connection.md` — contrato de conexión a Odoo (MCP primario / odoo-pilot fallback)
- `references/agent-loop.md` — bucle Agent SDK: `canUseTool` (gate humano), streaming, `resume`, MCP scoped
- `references/sandbox-isolation.md` — orquestar contenedores desde Odoo con seguridad (docker-socket-proxy, egress, caps)
- `references/odoo-integration.md` — wiring Odoo: usuario/record-rules, endpoints, cuotas/metering, `.mcp.json`
- `references/token-mode-lock.md` — atar el modo/privilegio al token (un token de agente no abre shell); anti-RCE
- `references/session-lifecycle.md` — heartbeat por-sesión + reaper + accounting por tiempo-vivo (anti-fuga)
- `references/citations-and-grounding.md` — **qué puede prometer un agente que responde desde documentos**: por qué una cita no basta, la comprobación de respaldo determinista, las alarmas que se inventa sobre tu propio sistema, y el CLI que trae sus propias herramientas
- `references/provider-decoupling.md` — IA desacoplada: base agnóstica + extensión por proveedor + registry en sidecar
- `references/governance-config.md` — gobernanza (admin) vs preferencia (usuario); confirm 3-niveles + techo; system prompt
