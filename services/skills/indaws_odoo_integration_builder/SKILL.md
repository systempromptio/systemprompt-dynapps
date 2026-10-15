---
name: indaws-odoo-integration-builder
version: 0.1.16
effort: high
model: opus
description: >-
  Construye integraciones externas en Odoo: cliente REST/SOAP con reintentos, webhook entrante firmado, idempotencia y queue.job. Usala cuando integres una API. Incluye cron de sincronización y secretos en ir.config_parameter. NOT MCP (→ odoo-mcp-builder); NOT carriers (→ odoo-carrier-developer).
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode, Agent]
---
<!-- verified: v18.0 @ 2026-06 -->

# Odoo Integration Builder

Diseña e implementa integraciones externas completas en módulos Odoo v17-v19. Cubre los dos
sentidos: cliente saliente (outbound) hacia APIs de terceros y endpoint/webhook entrante
(inbound). Impone patrones de robustez, seguridad de secretos y trazabilidad desde el primer
scaffold.

Refs compartidas: `references/shared/odoo-connection.md`, `references/shared/batch-limits.md`

## Workflow

1. **Definir la integración** [STOP — AskUserQuestion]
   - Dirección: outbound / inbound / bidireccional
   - Protocolo: REST JSON, REST form-data, SOAP, GraphQL
   - Auth: API key, OAuth2, Basic, HMAC, JWT
   - Odoo models afectados y versión objetivo (v17/v18/v19)
   - ¿Asíncrono requerido? (`queue.job` OCA o cron periódico)

2. **Analizar el módulo existente** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` — invocar `odoo-module-scanner` sobre el módulo destino. Salida: modelos, campos, controllers existentes, dependencias.
   - Cargar patrones de versión: `odoo-version-api-reference` (diff v17/v18/v19 según target)

3. **Proponer arquitectura** [STOP — confirmación usuario]
   - Estructura de archivos: `services/`, `controllers/`, `data/` (cron, config_parameter)
   - Clase service (outbound) o controller (inbound) con nombres definitivos
   - Campos de idempotencia en el modelo receptor

4. **Scaffold de clases**
   - Outbound → leer [`references/outbound-clients.md`](references/outbound-clients.md): sesión requests, retry/backoff, auth, timeout, parseo, mapeo
   - Inbound → leer [`references/inbound-endpoints.md`](references/inbound-endpoints.md): `@http.route`, `type='json'` (v17/18) / `type='jsonrpc'` (v19), HMAC, api-key, `csrf=False`

5. **Secretos y configuración**
   - `ir.config_parameter` para URLs y tokens; acceso con `sudo()`
   - Nunca hardcodear credenciales ni URLs de producción en código

6. **Robustez**
   - Leer [`references/robustness.md`](references/robustness.md): idempotencia, `queue.job`, cron, rate limiting, logging, reproceso
   - Añadir campo `external_ref` (o similar) en modelo para idempotencia

7. **Tests** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` — invocar `odoo-test-generator` con mock de la API externa (responses fixtures)

## GOTCHAS

1. **`csrf=False` en webhooks `type='http'`** — Sin esto Odoo rechaza POSTs externos con **400, no 403** (medido en v18 y v19: `dispatch` lanza `werkzeug.exceptions.BadRequest`, `odoo18/odoo/http.py:2292`, `HttpDispatcher.dispatch`). Importa porque quien busque un 403 en el log del proveedor no lo encuentra, y un 400 se lee como «mi payload está mal» — se depura el JSON durante horas en vez de mirar el decorador. Pero **solo aplica a `type='http'`**: la comprobación está únicamente en `HttpDispatcher.dispatch` —**una sola lectura de `csrf` en todo `http.py`**, verificado 2026-08-18: `odoo18:2282` y `odoo19:2493`, y **en v19 la constante de guarda se renombra**, `CSRF_FREE_METHODS` → `SAFE_HTTP_METHODS`—; `JsonRPCDispatcher` (v18 `:2327`) no lee `csrf` jamás, así que en `type='json'`/`'jsonrpc'` el parámetro es decorativo. Una ruta jsonrpc nunca tuvo CSRF que desactivar: lo que la protege es tu HMAC/api-key.
2. **`auth='public'` vs `auth='user'`** — Los webhooks de terceros usan `auth='public'`; los endpoints internos autenticados usan `auth='user'` o `auth='bearer'`. NO existe modo `auth='api_key'`; es `auth='bearer'` y solo desde **v18** (`_auth_method_bearer`, `odoo/addons/base/models/ir_http.py:204`, `IrHttp._auth_method_bearer`; no existe en v17). En **v19** una ruta `auth='bearer'` es además **stateless por defecto** (`odoo19/odoo/http.py:828-829`, `route.decorator` fuerza `save_session=False`); en **v18 NO** — ahí hay que poner `save_session=False` a mano si no quieres cookie de sesión por llamada.
3. **`sudo()` para leer `ir.config_parameter`** — El usuario que ejecuta el cron/webhook puede no tener permisos. Usar siempre `self.env['ir.config_parameter'].sudo().get_param(...)`.
4. **`queue.job` no disponible en CE sin OCA** — Si el módulo target no declara dependencia de `queue_job`, encolar con `.with_delay()` falla en install. Verificar en `__manifest__.py`. NO existe decorador `@job` en queue_job v14+: se encola llamando al método con `.with_delay()`.
5. **Timeout obligatorio en requests** — Sin `timeout=(connect, read)` una petición colgada bloquea el worker de Odoo indefinidamente. Mínimo `timeout=(5, 30)`.
6. **Idempotencia antes de procesar** — Comprobar `external_ref` ANTES de crear el registro, no después. Race condition en webhooks concurrentes.
7. **`numbercall` eliminado en v18+** — No usar en XML de cron. Usar solo `interval_number`/`interval_type`/`active`. Existía en v17 (`odoo/addons/base/models/ir_cron.py:67`, `ir_cron`, v17 — el campo `numbercall`) y desapareció en v18; un cron de sincronización migrado casi siempre lo trae. **Qué pasa si se cuela** (medido en v18 y v19): `ValueError: Invalid field 'numbercall' on model 'ir.cron'` **al crear**, no un aviso ni un valor ignorado — la fila no llega a existir, así que el cron simplemente no está y nadie lo echa de menos hasta que la sincronización no corre.
8. **Verificación HMAC en tiempo constante** — Usar `hmac.compare_digest()`, nunca `==` para comparar firmas; evita timing attacks.
9. **`type='json'` deprecado en v19** — En v19 es un alias de `type='jsonrpc'` con `DeprecationWarning` (`odoo/http.py:813-819`, `route.decorator`, v19 @d1d161033`: *"Since 19.0, @route(type='json') is a deprecated alias to @route(type='jsonrpc')"*, seguido de `routing['type'] = 'jsonrpc'`). Usar `type='jsonrpc'` para v19; `type='json'` sigue válido en v17/v18.

## NEVER

- Hardcodear URLs de API, tokens o contraseñas en código Python o XML
- Omitir `timeout` en cualquier llamada con `requests`
- Usar `auth='none'` en endpoints que reciban o devuelvan datos de negocio
- Comparar firmas HMAC con `==`
- Poner lógica de negocio directamente en el controller (el controller orquesta, la lógica va en el service)
- Procesar síncronamente en webhook si la operación puede tardar >5 s (usar `queue.job`)
- Capturar `Exception` genérico sin loggear el traceback completo

## ALWAYS

- Leer las 3 references antes de generar código de cada capa
- Confirmar arquitectura con usuario antes de crear ficheros (Step 3)
- Añadir campo de idempotencia al modelo receptor antes de implementar el procesado
- Usar `_logger = logging.getLogger(__name__)` en cada fichero
- Estructurar la respuesta JSON de error con `{"error": "...", "code": ...}` consistente
- Declarar dependencia `queue_job` en `__manifest__.py` si se usa `queue.job`
- Leer `odoo-version-api-reference` para el target de versión antes de generar código
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-integration-builder · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-mcp-builder | common | Si el objetivo es exponer Odoo como servidor MCP |
| odoo-carrier-developer | developer | Si la integración es un transportista de envíos |
| indaws-data-export | developer | Si basta un export configurable por cambios de registro (sin código a medida) |
| odoo-version-api-reference | common | Patrones específicos de versión (v17/v18/v19) |
| odoo-test-generator | developer | Generación de tests con mock de API externa |
| odoo-module-scanner | common | Analizar módulo existente antes de ampliar |

## Consumed by

- `guion` — el asistente de proyectos consume esta skill para tareas de integración
- `indaws-data-export` (developer) — cuando la integración excede los templates de export configurables y necesita código a medida (clientes outbound / webhooks inbound)
- `odoo-website-controller` (developer) — cuando el endpoint es un webhook / API REST externa en lugar de una página web

## REFERENCES

| Fichero | Contenido |
|---------|-----------|
| [`references/outbound-clients.md`](references/outbound-clients.md) | Patrón clase service con requests: sesión, auth, retry/backoff, timeout, parseo, mapeo a modelos Odoo |
| [`references/inbound-endpoints.md`](references/inbound-endpoints.md) | `@http.route` types, auth modes, api-key, csrf=False, verificación HMAC, respuestas JSON |
| [`references/robustness.md`](references/robustness.md) | Idempotencia, queue.job/cron, secretos en config, rate limiting, logging, reproceso |
