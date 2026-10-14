# Routing: `@http.route` y `request` para páginas web

Fuente verificada contra **v18**: `$ODOO_SRC/odoo18/odoo/http.py`,
`addons/website/controllers/main.py`, `addons/http_routing/models/ir_http.py`. Los números de
línea son de v18. **v17 no es equivalente en dos cosas que esta tabla marca**: `readonly` y
`auth='bearer'` son de v18. El resto se ha comprobado en v18 y no se ha vuelto a comprobar en
v17/v19 fila por fila.

## `@http.route` — firma completa (`odoo/http.py:687-734`, `route`)

```python
@http.route(route=None, **routing)
```

| Param | Valores | Notas para páginas web |
|-------|---------|------------------------|
| `route` | str o lista de str | Path(s) werkzeug. Una lista mapea varios paths al mismo método. |
| `type` | `'http'` (default) / `'json'` | Páginas SIEMPRE `'http'`. `'json'` es JSON-RPC, no HTML (`odoo/http.py:702-704`, `route`). |
| `auth` | `'public'` / `'user'` / `'bearer'` / `'none'` | `'public'` = usuario público compartido si no hay sesión (`odoo/http.py:715-717`, `route`). `'bearer'` desde **v18** (`odoo/addons/base/models/ir_http.py:204`, `IrHttp._auth_method_bearer`, doc `odoo/http.py:709`, `route`, v18); lo exclusivo de v19 es que fuerce `save_session=False` (`odoo/http.py:828`, `route`, v19). |
| `methods` | lista, p.ej. `['GET']`, `['POST']` | Si se omite, todos. Separa el GET de visualización del POST de formulario. |
| `website` | `True` | **Obligatorio para frontend.** Setea `request.website`, multi-website y multilang (`http_routing/models/ir_http.py:386-387`, `IrHttp._match`). |
| `csrf` | bool | Default `True` para `type='http'` (`odoo/http.py:725-727`, `route`). |
| `sitemap` | `True`/`False`/callable | Controla aparición en `/sitemap.xml`. Callable: `def f(env, rule, qs)` que hace `yield {'loc': ...}`. |
| `multilang` | bool | Default = `type=='http'`. `False` desactiva prefijo de idioma en la URL. |
| `save_session` | bool (default `True`) | `False` evita escribir la sesión en disco (`odoo/http.py:2214`, `Dispatcher.pre_dispatch`). Útil en rutas de alto tráfico sin estado. |
| `readonly` | bool/callable | **v18+.** `True` abre cursor en réplica de solo lectura (doc en `odoo/http.py:728`, `route`; se lee en `odoo/http.py:2101-2103`, `Request._serve_db`). Marcar en GET de páginas públicas. En v17 se acepta y no hace nada. |
| `cors` | str | Valor de `Access-Control-Allow-Origin`. |

### Ejemplos reales (`website/controllers/main.py`)

```python
# Home, en sitemap
@http.route('/', auth="public", website=True, sitemap=True)            # :86

# Lista paginada, solo lectura, fuera de sitemap
@http.route(['/pages', '/pages/page/<int:page>'], type='http',
            auth="public", website=True, sitemap=False, readonly=True) # :574

# Ruta no-multilang (idioma fijo)
@http.route(['/robots.txt'], type='http', auth="public",
            website=True, multilang=False, sitemap=False)              # :229

# sitemap callable
@http.route('/website/info', type='http', auth="public",
            website=True, sitemap=sitemap_website_info)                # :339
```

## Parámetros de URL (converters werkzeug)

- `<int:page>` → int. `<string:slug>` → str sin `/`. `<path:path>` → str con `/`.
- `<model("res.partner"):partner>` → resuelve el registro directamente (Odoo converter).
  Si no existe o no hay acceso, Odoo levanta 404/403 antes de entrar al método.
- Los `**kw` capturan query-string y campos de formulario adicionales.

## Objeto `request` (lo esencial para páginas)

| Atributo / método | Uso |
|-------------------|-----|
| `request.env` | ORM con el usuario de la request (público si `auth='public'`). Usar `.sudo()` para datos de negocio. |
| `request.render(template, qcontext)` | Render diferido de QWeb (`odoo/http.py:1958-1974`, `Request.render`). Devuelve `Response`. |
| `request.redirect(location, code=303, local=True)` | 3xx werkzeug; respeta multilang vía `ir.http._redirect` (`odoo/http.py:1942-1951`, `Request.redirect`). |
| `request.make_response(data, headers, cookies, status)` | Respuestas no-HTML: PDF/CSV/XML (`odoo/http.py:1893-1914`, `Request.make_response`). |
| `request.make_json_response(data)` | JSON serializado (`odoo/http.py:1916-1933`, `Request.make_json_response`). |
| `request.website` | Website actual (solo si `website=True`). |
| `request.lang` | Idioma resuelto de la URL. |
| `request.httprequest` | Werkzeug request crudo: `.method`, `.path`, `.args`, `.query_string`. |
| `request.params` | Dict combinado de query-string + body. |
| `request.csrf_token()` | Token a inyectar en formularios POST (`odoo/http.py:1776`, `Request.csrf_token`). |
| `request.not_found(desc)` | Atajo 404 (`odoo/http.py:1935-1940`, `Request.not_found`). |

## Herencia de controllers

Para extender un controller existente: heredar la clase y, **si se sobreescribe un método
con `@route`, volver a decorarlo** (obligatorio, `odoo/http.py:692-695`, `route`). Los args pueden
omitirse para heredar los del padre.

```python
from odoo.addons.website.controllers.main import Website

class WebsiteExt(Website):
    @http.route()  # re-decorado obligatorio; hereda routing del padre
    def index(self, **kw):
        res = super().index(**kw)
        # ... lógica extra
        return res
```

Para **añadir una ruta nueva** basta declarar un método nuevo con su `@http.route(...)`
en una clase `http.Controller` (o en la heredada).

## Multi-website y multilang

- `website=True` es el interruptor. Con él, `request.website` es el website del dominio/host
  actual y `request.env['website'].get_current_website()` lo resuelve.
- Filtrar por website en queries: `domain += request.website.website_domain()`
  (patrón en `website/models/ir_http.py:306`, `Http.page_domain`).
- Multilang: con `multilang` activo Odoo prefija la URL con el código de idioma
  (`/es/...`) y redirige. Los idiomas frontend salen de `res.lang._get_frontend()`
  (`http_routing/models/ir_http.py:228`, `IrHttp._is_multilang_url`). `multilang=False` fija el idioma (robots.txt, sitemap).
- El idioma por defecto del website: `ir.http._get_default_lang()`
  (`website/models/ir_http.py:289-293`, `Http._get_default_lang`).

## `website.page` vs ruta de controller

- **`website.page`**: página CMS editable, sin lógica. La sirve automáticamente
  `ir.http._serve_page()` buscando por `url` (`website/models/ir_http.py:302-346`, `Http._serve_page`).
  Crear como dato (`<record model="website.page">`) — no requiere controller.
- **Controller**: cuando hay lógica, parámetros de URL, queries, POST, o respuestas no-HTML.

Regla: contenido estático editable → `website.page`. Página con comportamiento → controller.
