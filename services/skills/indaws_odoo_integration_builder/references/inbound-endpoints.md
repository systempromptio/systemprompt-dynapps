# Inbound Endpoints — Controllers y Webhooks

## Estructura de ficheros

```
my_module/
├── controllers/
│   ├── __init__.py
│   └── main.py
├── __manifest__.py   # debe declarar 'controllers' si usa rutas
```

En `__init__.py` del módulo raíz añadir `from . import controllers`.

## Tipos de `@http.route`

| Parámetro | Opciones | Cuándo usarlo |
|-----------|----------|---------------|
| `type` | `'http'` (default) / `'json'` (v17/18) / `'jsonrpc'` (v19) | `'json'`/`'jsonrpc'`: body es JSON, respuesta JSON automática. En v19 `'json'` es alias deprecado de `'jsonrpc'` (DeprecationWarning). `'http'`: body libre, control total |
| `auth` | `'public'` / `'user'` / `'bearer'` (v18+) / `'none'` | Ver tabla abajo |
| `methods` | `['GET']`, `['POST']`, `['GET','POST']` | POST para webhooks y creación |
| `csrf` | `True` (default) / `False` | `False` obligatorio en webhooks `type='http'`. En `type='json'`/`'jsonrpc'` **no hace nada** (ver abajo) |
| `cors` | `'*'` / origen | Para endpoints llamados desde browser externo |

### Tabla auth modes

| Mode | Cuándo | Nota |
|------|--------|------|
| `'public'` | Webhooks de terceros, APIs públicas | No requiere sesión. Usar verificación de firma propia |
| `'user'` | Endpoints de portal/usuario autenticado | Requiere sesión Odoo válida |
| `'bearer'` | Endpoints B2B con API key de Odoo | Solo **v18+** (`_auth_method_bearer` en `odoo18/odoo/addons/base/models/ir_http.py:204`, `IrHttp._auth_method_bearer`, v19 `:212`; **no existe en v17**). Valida el header `Authorization: Bearer <key>` contra `res.users.apikeys`. NO existe `'api_key'` |
| `'none'` | NUNCA para datos de negocio | Sin ninguna verificación |

### `csrf=False`: obligatorio en `http`, decorativo en `json`/`jsonrpc`

La validación CSRF vive **solo** en `HttpDispatcher.dispatch`:
`if method not in CSRF_FREE_METHODS and endpoint.routing.get('csrf', True)`
(`odoo18/odoo/http.py:2282`, `HttpDispatcher.dispatch`, clase en `:2263`). `JsonRPCDispatcher` (`:2327`) **no menciona `csrf`
ni una vez**: nunca lo comprueba.

Consecuencia práctica:

- `type='http'` + POST externo → **`csrf=False` es obligatorio**, si no da **400**, no 403. Es
  `werkzeug.exceptions.BadRequest('Session expired (invalid CSRF token)')` (`odoo18/odoo/http.py:2292`, `HttpDispatcher.dispatch`),
  medido con una sonda en v18 y v19. El número importa: un 403 se lee como «permisos» y lleva al
  decorador; un 400 se lee como «payload mal formado» y manda a depurar el JSON, que es donde no
  está el problema. En el log del servidor sí queda la pista buena:
  `WARNING odoo.http: No CSRF validation token provided for path '...'`.
- `type='json'` / `'jsonrpc'` → `csrf=False` **no cambia nada**, porque nadie lee ese valor. No es un
  error ponerlo, pero no es la razón de que funcione, y creer que ahí "desactivas una protección"
  lleva a razonar mal sobre la seguridad del endpoint: **una ruta jsonrpc nunca tuvo CSRF que
  desactivar**. Lo que protege ese endpoint es tu HMAC/api-key, nada más.

El docstring de Odoo (*"disabled by default for json-type requests"*, `odoo18:725-727`,
`odoo19:793-795`) describe el efecto, no el mecanismo: no hay un default distinto por tipo, hay un
dispatcher que no comprueba. Verificado 2026-08-07 en v18 y v19.

### v19: `auth='bearer'` es stateless por defecto (y en v18 NO)

En v19 el decorador fuerza `save_session=False` cuando `auth='bearer'`:

```python
# odoo19/odoo/http.py:828-829
if routing.get('auth') == 'bearer':
    routing.setdefault('save_session', False)  # stateless
```

Es justo lo que se quiere en API máquina-a-máquina: no se emite cookie de sesión por llamada.
**En v18 este atajo no existe**: `save_session` sigue por defecto a `True`
(`odoo18/odoo/http.py:2214`, `Dispatcher.pre_dispatch`, `routing.get('save_session', True)`) y no hay caso especial para
bearer, así que en v18 hay que poner `save_session=False` **a mano** si no quieres sesión.

## Controller JSON básico

```python
import logging
from odoo import http
from odoo.http import request

_logger = logging.getLogger(__name__)


class MyWebhookController(http.Controller):

    @http.route(
        "/my_module/webhook",
        type="json",
        auth="public",
        methods=["POST"],
        csrf=False,
    )
    def webhook_receive(self):
        payload = request.get_json_data()  # v16+; en v15- usar request.jsonrequest
        _logger.info("Webhook received: %s", payload.get("event"))
        # Procesar o encolar
        try:
            request.env["my.model"].sudo()._process_webhook(payload)
        except Exception as exc:
            _logger.error("Webhook processing error: %s", exc, exc_info=True)
            return {"error": str(exc), "code": 500}
        return {"status": "ok"}
```

## Controller HTTP con respuesta explícita

```python
import json
from odoo import http
from odoo.http import request, Response


class MyApiController(http.Controller):

    @http.route(
        "/my_module/api/orders",
        type="http",
        auth="bearer",   # v18+; valida Authorization: Bearer <key> contra res.users.apikeys
        methods=["POST"],
        csrf=False,      # obligatorio: es type='http'
        # v18: añade save_session=False si no quieres cookie de sesion (v19 ya lo hace solo)
    )
    def create_order(self, **kwargs):
        try:
            body = json.loads(request.httprequest.data)
        except ValueError:
            return Response(
                json.dumps({"error": "Invalid JSON", "code": 400}),
                status=400,
                content_type="application/json",
            )
        record = request.env["sale.order"].sudo()._create_from_api(body)
        return Response(
            json.dumps({"id": record.id, "name": record.name}),
            status=201,
            content_type="application/json",
        )
```

## Verificación de firma HMAC

Obligatorio para webhooks de proveedores que firman el payload (Stripe, GitHub, Shopify…).

```python
import hmac
import hashlib
from odoo import http
from odoo.http import request

SECRET_PARAM = "my_module.webhook_secret"


class SignedWebhookController(http.Controller):

    @http.route(
        "/my_module/signed_webhook",
        type="http",
        auth="public",
        methods=["POST"],
        csrf=False,
    )
    def signed_webhook(self, **kwargs):
        secret = (
            request.env["ir.config_parameter"]
            .sudo()
            .get_param(SECRET_PARAM, default="")
        ).encode()

        # El proveedor envía la firma en un header (formato varía por proveedor)
        signature_header = request.httprequest.headers.get("X-Signature-256", "")
        payload = request.httprequest.data

        expected = "sha256=" + hmac.new(secret, payload, hashlib.sha256).hexdigest()

        # SIEMPRE usar compare_digest (tiempo constante)
        if not hmac.compare_digest(expected, signature_header):
            return request.make_response(
                '{"error": "Invalid signature", "code": 401}',
                headers=[("Content-Type", "application/json")],
                status=401,
            )

        body = json.loads(payload)
        request.env["my.model"].sudo()._process_webhook(body)
        return request.make_response(
            '{"status": "ok"}',
            headers=[("Content-Type", "application/json")],
        )
```

## Verificación por API Key propia

Para endpoints privados B2B sin usar el sistema `auth='api_key'` de Odoo:

```python
API_KEY_PARAM = "my_module.inbound_api_key"

def _check_api_key(self):
    provided = request.httprequest.headers.get("X-Api-Key", "")
    expected = request.env["ir.config_parameter"].sudo().get_param(API_KEY_PARAM, "")
    if not expected or not hmac.compare_digest(provided, expected):
        return False
    return True

# type='json': el csrf=False es decorativo (JsonRPCDispatcher no lo lee). Lo que protege es la api-key.
@http.route("/my_module/api/sync", type="json", auth="public", methods=["POST"])
def sync(self):
    if not self._check_api_key():
        return {"error": "Unauthorized", "code": 401}
    # ...
```

## `get_json_data()` vs `request.jsonrequest` por versión

| Versión | Forma correcta |
|---------|---------------|
| v17, v18, v19 | `request.get_json_data()` |
| v15, v16 | `request.jsonrequest` (deprecated en v17) |
| v14 | `request.jsonrequest` |

## Logging de entrada recomendado

```python
_logger.info(
    "Inbound webhook [%s] from %s event=%s",
    request.httprequest.url,
    request.httprequest.remote_addr,
    payload.get("event", "?"),
)
```

## Patrones de respuesta HTTP estándar

```python
# 200 OK con JSON
return {"status": "ok", "id": record.id}   # en type='json'

# 202 Accepted (webhook encolado, se procesará async)
return {"status": "queued"}

# Error estructurado
return {"error": "Order not found", "code": 404, "ref": external_ref}
```

## Registrar route en `__manifest__.py`

No es necesario declarar las routes en el manifest, pero sí asegurarse de que el directorio
`controllers/` está importado desde el `__init__.py` raíz:

```python
# __init__.py
from . import models, controllers
```
