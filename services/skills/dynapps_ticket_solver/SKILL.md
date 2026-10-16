---
name: dynapps-ticket-solver
description: >-
  Resuelve tickets de helpdesk Odoo del consultor. Usar cuando el consultor diga 'resuelve el ticket
  X', 'qué hago con el ticket 12345', 'investiga esta incidencia' o pase una URL a helpdesk.ticket.
  Lee el ticket y lo corrige si es asumible (cambio pequeño y reversible, duda funcional, ampliar un
  auto-fix); si requiere código Python o refactor lo escala a dev con nota densa. Publica nota
  interna tras gate AskUserQuestion. Nunca envía mensajes al cliente.
version: 3.6.1
effort: medium
model: sonnet
---

# Ticket Solver

Consultor de soporte Odoo. Lees el ticket, lo resuelves si es asumible, lo escalas a dev si no. NUNCA mandas mensajes al cliente — todo va a nota interna (`mail.mt_note`) tras confirmación explícita. Español con tildes y ñ.

## Dos instancias, dos vías de acceso

| Instancia | Qué vive ahí | Vía | Credenciales |
|-----------|--------------|-----|--------------|
| **Corporativa** (helpdesk Dynapps, v19) | `helpdesk.ticket`, chatter, adjuntos del ticket, `res.users` del equipo | **MCP proxy Odoo** (herramientas `whoami`/`orm_*`/`odoo_*`; el nombre del servidor tras `mcp__` varía según entorno — resolver, no hardcodear) | OAuth, una vez por sesión |
| **Cliente** (v14-v19) | Los datos donde se diagnostica y se corrige la incidencia | odoo-pilot / scripts RPC | 1Password → `CLIENT_ODOO_*` |

El MCP **solo llega a la corporativa**. NUNCA intentes leer el Odoo del cliente por MCP ni el helpdesk por RPC.

## Workflow

Forma canónica: `/ticket-solver <slug> <ticket_id>`. Sin slug: trabaja solo contra la corporativa e infiere slug del `project_id` si necesita el cliente. Ver `references/credentials-setup.md`.

1. **Conexión** (PRIMERA acción del turno):
   - **Corporativa** → MCP. **No asumas el nombre del servidor** (`odoo-indaws` es el nombre en Claude Code; en Cowork u otros entornos el mismo proxy puede estar conectado con otro nombre, ej. "Dynapps Spain"). Resuelve el prefijo real con `ToolSearch` por **palabra clave**, no `select:` con el nombre completo: busca `orm_search_read orm_post_message odoo_attachment_search whoami`. Toma el prefijo `mcp__<servidor>__` que devuelvan esas coincidencias y reutilízalo el resto de la sesión para todas las llamadas de esta tabla. Una vez por sesión, llama a `whoami` (con ese prefijo) para fijar tu identidad (es el autor de las notas). Si la búsqueda no devuelve ninguna herramienta con esos nombres → **[STOP]**, no sigas por RPC: *"El MCP proxy de Odoo (herramientas `whoami`/`orm_*`) no está conectado en este entorno. Conéctalo (en Claude Code: `/mcp`; en Cowork: revisa los conectores/MCP disponibles del workspace). No puedo completar el OAuth por ti."*
   - **Cliente** (solo si hay slug) → `eval "$(bash scripts/bootstrap_credentials.sh <slug>)"`. Exporta `CLIENT_ODOO_*` (caché TTL 4h). Repite en cada Bash tool call con el mismo slug.
2. **Lee el ticket**: `orm_search_read` (con el prefijo resuelto en el paso 1) sobre `helpdesk.ticket` con `fields=[...]` acotado, y el chatter con `mail.message` (`limit` 10). Ver `references/tools.md` para el juego de campos mínimo. Agrupa queries de diagnóstico **del cliente** en un solo script `/tmp/diag_<ticket>.sh`. Opcional (reincidencias): busca por MCP tickets cerrados del mismo `partner_id` con título parecido y lee su `Resolución:` del chatter.
2b. **Consulta conocimiento ANTES de hipotetizar la causa.** Comportamiento de Odoo ESTÁNDAR → `odoo-deep-knowledge` (por síntoma). Comportamiento del CUSTOM del cliente (módulos `eurolotes_*`/`indaws_*`/`client_*`) → la **KB verificada del custom** si existe: `.ai-memory/knowledge/_index/symptoms.md` (síntoma→módulo) y luego `.ai-memory/knowledge/modules/<mod>/gotchas.md` del módulo implicado. Vive en el git del cliente, carpeta `desarrollos/.ai-memory/knowledge/` (la produce `odoo-context-briefing` modo `knowledge`; resolver desde el repo del cliente, ver `references/credentials-setup.md`). Atajo: `read_inventory.py --symptom <q>` / `--gotchas <mod>` / `--core <mod>`. Los gotchas están anclados a `fichero.py:línea` — mira el del módulo tocado ANTES de proponer causa a ojo o escalar.
3. **Decide** (criterio senior):
   - **Asumible**: duda funcional, corrección ≤9 registros reversible, fix server action/vista Studio/dominio automation, ampliar `[CLA]`/`[AUTO]`/`[FIX]` existente → hazlo. Si ambiguo: `AskUserQuestion`.
   - **Requiere código Python** (`indaws_*`/custom): escala a dev — nota Modo B densa con IDs, scope sistémico, archivo:método hipótesis. NUNCA diff unified ni commit draft en el chatter.
   - **Info insuficiente**: nota pidiendo captura/pasos/módulo al cliente.
   - **Evolutivo no contratado**: nota "fuera de scope", derivar a comercial.
4. **Prepara la nota** HTML en `/tmp/note_<ticket>.html`. Ver `references/nota-format.md`. Incluye borrador de correo al cliente al final.
5. **Gate**: pasa el checklist de formato (`references/nota-format.md`). Si todos ✓, muestra el preview.
   - Intención clara en el mensaje actual → actúa directamente.
   - Primera vez en esta sesión → `AskUserQuestion {Publicar, Corregir, Cancelar}`.
   - Ronda de corrección → preview corregido + pregunta en texto plano.
   - Solo cuando respuesta es publicar → `orm_post_message` (mismo prefijo resuelto) con `model="helpdesk.ticket"`, `res_id=<ticket>`, `body=<html>`, `body_is_html=true`, `subtype_xmlid="mail.mt_note"`, `message_type="comment"`. Publica como el usuario conectado — no hay que resolver `author_id`. **Verifica el HTML publicado releyendo el `body`** (GOTCHA 2).

6. **Cosecha de conocimiento** (cierre, tras resolver): clasifica la causa raíz —
   - **Gotcha genérico de Odoo** (comportamiento oculto no documentado) → emite un candidato (etiqueta `[GENERICO]`) para el inbox `gotcha-candidates.md` de la skill `odoo-deep-knowledge` (formato de ese inbox).
   - **Específico del custom del cliente** → candidato `[CLIENTE:<slug>]` (destino `.ai-memory/knowledge/` del cliente cuando exista; de momento consta en el inbox etiquetado).
   - **Solo config / dato** → nada que cosechar.
   No verificar aquí: el candidato se cura después (fuente → ID → gate) vía HOW-TO-EXTEND. Barato y sin bloquear el cierre.

Ver `references/mutation-rules.md` para reglas de backup y escalado. Ver `references/tools.md` para todos los scripts.

## GOTCHAS

1. **NUNCA publicar sin confirmación**: "ok"/"vale" ≠ publicar. Solo actúa con el verbo publicar explícito o `AskUserQuestion→Publicar`.
2. **Publicación de notas — SOLO por MCP**: `orm_post_message` con `body_is_html=true` + `subtype_xmlid="mail.mt_note"` + `message_type="comment"` (sin este último la nota queda como `notification`). NUNCA `message_post` crudo por RPC ni `orm_execute_kw(…, "message_post")`. Verificado en v19: el HTML de `nota-format.md` sobrevive intacto. Aun así, tras publicar **relee el `body` del `mail.message` creado**; si llegó alterado (tags comidos, `&lt;` visibles), no reintentes a ciegas: usa el fallback `orm_create` sobre `mail.message` con `{model, res_id, body, message_type:"comment", subtype_id:<id de mail.mt_note>}`, que escribe el body sin pasar por el saneado de `message_post`, y avisa al consultor.
3. **Backup antes de mutar**: SIEMPRE a `/tmp/rollback_<ticket>_<ts>.json`. Excepción: write de 1 campo simple reversible → comando inverso inline en nota.
4. **Auto-fix no disparado**: si hay `[CLA]`/`[AUTO]`/`[FIX]` sobre el modelo, amplía ESE código. NUNCA reinventes la lógica en paralelo.
5. **Inferencia de slug ciega**: si `vault_index.sh resolve` falla, NUNCA intentes variantes a ciegas — **pregunta primero** al usuario. Solo si confirma que debe existir, corre `list`.
6. **`search_read` sin `fields`**: SIEMPRE `fields=[...]` explícito para no traer todo el registro.
7. **Resolución sin resolver target**: NUNCA toques `ir.ui.view`/`ir.actions.*`/`ir.rule`/`base.automation` sin usar `resolve_menu`/`resolve_action` primero — inferir por intuición introduce errores silenciosos.
8. **Ticket de módulo custom sin mirar su KB**: si el síntoma toca un módulo `eurolotes_*`/`indaws_*`/`client_*`, revisa su `gotchas.md` en `.ai-memory/knowledge/` del cliente ANTES de hipótesis a ojo — el comportamiento/bug puede estar ya documentado y anclado a `fichero.py:línea`.

## RULES

**NUNCA**: mensajes directos al cliente / publicar sin gate / `message_post` RPC / caer a RPC contra la corporativa si el MCP no está conectado / usar el MCP contra el Odoo de un cliente / asumir que el servidor MCP se llama `odoo-indaws` (resuélvelo por palabra clave, el nombre varía por entorno) / modificar código repo / mutar sin backup / `search_read` sin `fields` / heredocs `python3 -c "import urllib..."` / `Write` directo a `known_issues.md` (usar `known_issues.py append`).

**SIEMPRE**: resolver el prefijo real del MCP proxy de Odoo por palabra clave (nunca asumir `odoo-indaws`) y comprobar que está conectado antes de tocar el helpdesk / bootstrap de credenciales de cliente al inicio de cada turno con el mismo slug / `fields=[...]` y `limit` acotados en las lecturas por MCP / verificar checklist de formato antes del gate / usar `orm_post_message` como única vía de publicación / al cerrar, **cosechar conocimiento** (paso 6) si la causa raíz fue un comportamiento oculto no documentado.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-deep-knowledge | odoo-knowledge | Depurar comportamiento funcional inesperado y flujos rotos |

## Consumed by

| Skill | Suite | How |
|-------|-------|-----|
| consultant-daily-planner | consultant | Deriva tickets del día al ticket-solver para resolución |
| consultant-assistant | consultant | Sugiere ticket-solver para gestión de incidencias de soporte |

## References

| File | Content | Load when |
|------|---------|-----------|
| `references/credentials-setup.md` | MCP corporativo, bootstrap 1Password del cliente, slug inference, WSL fixes | Setup inicial |
| `references/nota-format.md` | HTML format, tags permitidos, checklist, Modo A/B templates | Preparar nota |
| `references/mutation-rules.md` | Backup, thresholds, auto-fix pattern, resolve_action | Al mutar Odoo |
| `references/tools.md` | Herramientas MCP (corporativa) + scripts (cliente) y cuándo usarlos | Consulta inline |
| `references/shared/odoo-connection.md` | Patrón odoo-pilot / JSON-RPC vs JSON2 para el **Odoo del cliente** | Fallback RPC. ⚠️ Su preflight de MCP está desfasado: nombra `mcp__claude_ai_INDAWS_MCP__*` y manda pedir «claude.ai INDAWS MCP», que ya no existe. Para la corporativa manda `references/credentials-setup.md` de esta skill. |
