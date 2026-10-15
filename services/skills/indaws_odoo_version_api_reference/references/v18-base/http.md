<!-- verified: v18.0 @ 2026-06 -->
# HTTP Layer — Controllers, Routing & Request (Odoo 18.0)

Source: `$ODOO_SRC/odoo18/odoo/http.py` (2624 lines)

## Table of Contents

1. [Controller & Route](#controller--route)
2. [Request Object](#request-object)
3. [Response](#response)
4. [Stream](#stream)
5. [Auth Types](#auth-types)
6. [Dispatchers](#dispatchers)
7. [Session](#session)
8. [Common Patterns](#common-patterns)

---

## Controller & Route

### Defining a controller

```python
from odoo import http
from odoo.http import request

class MyController(http.Controller):

    @http.route('/my/endpoint', type='http', auth='user', website=True)
    def my_page(self, **kwargs):
        return request.render('module.template', {'data': ...})

    @http.route('/api/data', type='json', auth='user')
    def api_data(self, **kwargs):
        return {'result': 'ok'}
```

### @http.route parameters

```python
@http.route(
    route='/path/<int:id>',      # str or list of str — URL pattern(s) (first positional arg)
    type='http',                  # 'http' | 'json' — request type. Default: 'http'
    auth='user',                  # 'user' | 'bearer' | 'public' | 'none' — see Auth Types
    methods=['GET', 'POST'],      # list — allowed HTTP methods. If omitted, all methods are allowed
    cors=None,                    # str — Access-Control-Allow-Origin value (e.g., '*')
    csrf=True,                    # bool — CSRF protection. Default True for http, False for json
    readonly=False,               # bool|callable — open cursor on a read-only replica
)
```

CORE `@http.route` params (verified in `route()` docstring, http.py:687):
`type`, `auth`, `methods`, `cors`, `csrf`, `readonly`, `handle_params_access_error`.

> **NOTE (website module, not core http.py):** `website=True`, `sitemap`,
> `multilang` are added by the `website` addon (`addons/website/models/ir_http.py`),
> not by core `http.route`. They only work when `website` is installed.
> `request.website` / `request.lang` / `request.is_frontend` likewise come from
> the website module, not core Request.

> **`auth='bearer'`** (new, present in v18): authenticate via an
> `Authorization: Bearer <api-token>` header; falls back to the session if the
> header is missing. Documented in the `route()` docstring.

### URL converters

```python
'/path/<name>'              # string (default)
'/path/<int:id>'            # integer
'/path/<float:amount>'      # float
'/path/<model("res.partner"):partner>'  # ORM record by ID
'/path/<path:subpath>'      # catches remaining path including /
```

### Multiple routes on one method

```python
@http.route(['/path/a', '/path/b', '/path/<int:id>'], ...)
def handler(self, id=None, **kwargs):
    ...
```

---

## Request Object

`request` is a thread-local proxy to the current HTTP request.

### Key attributes

```python
request.db           # str — current database name
request.env          # Environment — ORM environment (with user, context, cursor)
request.uid          # int — current user ID (None if auth='none')
request.context      # dict — request context (lang, tz, etc.)
request.session      # Session — user session (dict-like)
request.cr           # Cursor — database cursor (shortcut for request.env.cr)

request.httprequest  # werkzeug.Request — raw HTTP request
request.params       # dict — request parameters (set by the dispatcher).
                     #   http: merged query string + form + files (get_http_params)
                     #   json: the 'params' object of the JSON-RPC body
```

> **GOTCHA — these do NOT exist on `request` in v18:**
> - `request.args` — use `request.httprequest.args` (werkzeug query-string MultiDict).
> - `request.jsonrequest` — REMOVED from the Request object. The parsed JSON body
>   lives on the `JsonRPCDispatcher` instance, not `request`. To read the raw JSON
>   body in a handler use `request.get_json_data()`. Already-parsed `params` are in
>   `request.params`.
> - `request.website` / `request.lang` / `request.is_frontend` — provided by the
>   `website` addon, not core Request (see route NOTE above).

### Key methods

```python
# Render QWeb template — render(template, qcontext=None, lazy=True, **kw)
# kw are forwarded to werkzeug Response (e.g. status=200). There is NO status_code param.
request.render('module.template_id', values_dict)

# Redirect — redirect(location, code=303, local=True)
request.redirect('/target/url', code=303)

# Raise HTTP error
from werkzeug.exceptions import NotFound, Forbidden
raise NotFound()
raise Forbidden()

# Get/set session values
request.session['key'] = value
value = request.session.get('key')

# File upload
uploaded_file = request.httprequest.files.get('file_field')

# Make URL
request.httprequest.url_root  # e.g., 'https://example.com/'
```

### not_found helper

```python
# not_found(description=None) RETURNS a werkzeug NotFound (HTTP 404) — you must raise it
raise request.not_found()
```

### make_response / make_json_response

```python
# make_response(data, headers=None, cookies=None, status=200) — non-HTML / custom headers
request.make_response(body, headers=[('Content-Type', 'text/plain')], status=200)

# make_json_response(data, headers=None, cookies=None, status=200) — json.dumps + Content-Type
request.make_json_response({'ok': True})
```

---

## Response

```python
from odoo.http import Response

# Return plain response
return Response("OK", status=200, headers={'Content-Type': 'text/plain'})

# Return JSON (type='json' routes return dicts directly, framework wraps in JSON-RPC)
return {'result': 'ok'}

# Return file download
return request.make_response(
    file_content,
    headers=[
        ('Content-Type', 'application/pdf'),
        ('Content-Disposition', content_disposition('report.pdf')),
    ],
)
```

### content_disposition

```python
from odoo.http import content_disposition

# Generates proper Content-Disposition header for downloads
headers = [('Content-Disposition', content_disposition('my_file.xlsx'))]
```

---

## Stream

For streaming file responses (large files, attachments):

```python
from odoo.http import Stream

# Constructors in v18 (only two classmethods — verified http.py:486/511):
#   Stream.from_path(path, filter_ext=('',), public=False)        — addon resource path
#   Stream.from_binary_field(record, field_name)                  — Binary/Image field
# There is NO Stream.from_attachment() in v18. For an ir.attachment, read its
# binary field via from_binary_field, or build a path-based stream.

# From file path (addon resource, validated by file_path)
stream = Stream.from_path('module/static/report.pdf')

# From a binary/image field on a record
stream = Stream.from_binary_field(record, 'datas')

# get_response(as_attachment=None, immutable=None,
#              content_security_policy="default-src 'none'", **send_file_kwargs)
return stream.get_response(as_attachment=True)
```

> Constructing `Stream(...)` manually is discouraged (docstring). Instances expose
> attributes like `type` ('data'|'path'|'url'), `data`, `path`, `mimetype`,
> `download_name`, `as_attachment`, `etag`, `size`, `public`.

---

## Auth Types

| Type | `request.uid` | DB cursor | Use case |
|------|---------------|-----------|----------|
| `'user'` | Logged-in user ID | Yes | Authenticated endpoints, portal, backend |
| `'bearer'` | User from `Authorization: Bearer <token>` (else session) | Yes | Token/API access (v18) |
| `'public'` | Public user ID or logged-in user | Yes | Website pages (visible to everyone) |
| `'none'` | None | No | Always active even with no DB; framework/health/webhooks |

### auth='none' gotchas

- No `request.env` — you must create it manually:
  ```python
  @http.route('/webhook', type='json', auth='none', csrf=False)
  def webhook(self, **kwargs):
      registry = odoo.registry(db_name)
      with registry.cursor() as cr:
          env = api.Environment(cr, SUPERUSER_ID, {})
          # ... use env
  ```
- No CSRF protection needed (set `csrf=False`)
- You are responsible for authentication (HMAC, token, API key)

### auth='public' vs 'user'

- `'public'`: always works. If user is logged in, `request.uid` is their ID. If not, it's the public user
- `'user'`: requires login. Redirects to login page if not authenticated

---

## Dispatchers

Odoo has two request dispatchers:

### HttpDispatcher (type='http')

- Receives standard HTTP requests (GET/POST form data)
- Returns HTML, files, redirects
- CSRF token required for POST (unless `csrf=False`)
- Template: `request.render('template', values)`

### JsonRPCDispatcher (type='json')

- Selected when the request mimetype is JSON (`mimetype in JSON_MIMETYPES`).
- Request body: `{"jsonrpc": "2.0", "method": "call", "params": {...}, "id": 1}`
- Returns: `{"jsonrpc": "2.0", "result": ..., "id": 1}` (errors → `error` key).
- The JSON-RPC `method` member is **ignored** — routing is by HTTP path. `params`
  MUST be a JSON object (by-name), not an array.
- **CSRF is never checked here.** The check lives only in `HttpDispatcher.dispatch`
  (`odoo/http.py:2282`, `HttpDispatcher.dispatch` — `endpoint.routing.get('csrf', True)`); this class
  (`:2327`) contains no reference to `csrf` at all. So `csrf=False` on a `type='json'`
  route is **decorative** — it is not what makes the route work, and there was never a
  CSRF protection there to disable. Odoo's own docstring ("disabled by default for
  json-type requests", `:725-727`) describes the effect, not the mechanism. The
  `csrf=False` still present in some examples below is harmless noise; what protects a
  public json endpoint is your own HMAC/api-key check. Verified 2026-08-07 (v18, v19).
- A special `context` key inside `params` replaces the session context.
- The dispatcher stores the parsed body on itself (`dispatcher.jsonrequest`),
  sets `request.params = params`, and passes them as kwargs to your method.
  In a handler, read the raw body via `request.get_json_data()` if needed
  (there is NO `request.jsonrequest` in v18).
- CSRF: not required by default.

---

## Session

```python
request.session  # Session object — a collections.abc.MutableMapping (dict-like),
                 # persisted to the filesystem session store
```

### Session attributes

`Session` is a MutableMapping: most fields are dict keys exposed as attributes.
`sid` and `should_rotate` are real slots.

```python
request.session.uid          # int — logged-in user ID (dict key)
request.session.login        # str — user login (dict key)
request.session.db           # str — current database (dict key)
request.session.context      # dict — user context (lang, tz) (dict key)
request.session.sid          # str — session ID (slot)
request.session.should_rotate  # bool — slot; set True to regenerate the sid
```

### Session methods

```python
# v18: authenticate takes a CREDENTIAL DICT, not (db, login, password)
request.session.authenticate(dbname, {'login': login, 'password': password, 'type': 'password'})

request.session.logout(keep_db=False)   # log out

# Regenerate the session id (NOT `.rotate`): set should_rotate; the framework
# rotates the sid after the request when should_rotate is True.
request.session.should_rotate = True
```

> `authenticate()` delegates to `res.users.authenticate(dbname, credential, wsgienv)`
> and handles MFA (finalizes immediately only when 2FA is disabled/skipped).

### Session timeout

Configurable via `web.session.max_inactivity` system parameter (default: 7200 seconds = 2 hours).

---

## Common Patterns

### Portal controller

```python
from odoo.addons.portal.controllers.portal import CustomerPortal, pager

class MyPortal(CustomerPortal):

    @http.route('/my/orders', type='http', auth='user', website=True)
    def portal_orders(self, page=1, sortby='date', **kwargs):
        Order = request.env['sale.order']
        domain = [('partner_id', '=', request.env.user.partner_id.id)]
        count = Order.search_count(domain)
        pager_data = pager(url='/my/orders', total=count, page=page, step=10)
        orders = Order.search(domain, limit=10, offset=pager_data['offset'])
        return request.render('module.portal_orders', {
            'orders': orders,
            'pager': pager_data,
        })
```

### REST-like JSON controller

```python
class ApiController(http.Controller):

    @http.route('/api/v1/partners', type='json', auth='user', methods=['POST'])
    def get_partners(self, domain=None, fields=None, limit=10):
        partners = request.env['res.partner'].search_read(
            domain or [], fields=fields or ['name', 'email'], limit=limit
        )
        return {'status': 'ok', 'data': partners}
```

### File download

```python
@http.route('/download/<int:id>', type='http', auth='user')
def download(self, id):
    attachment = request.env['ir.attachment'].browse(id)
    if not attachment.exists():
        raise request.not_found()
    # No Stream.from_attachment in v18 — stream the attachment's binary field:
    return Stream.from_binary_field(attachment, 'datas').get_response(as_attachment=True)
```

### Webhook (no auth)

```python
@http.route('/webhook/payment', type='json', auth='none', csrf=False, methods=['POST'])
def payment_webhook(self, **data):
    # Verify signature
    if not self._verify_hmac(data, request.httprequest.headers.get('X-Signature')):
        raise Forbidden()
    db_name = data.get('db')
    registry = odoo.registry(db_name)
    with registry.cursor() as cr:
        env = api.Environment(cr, SUPERUSER_ID, {})
        env['payment.transaction'].sudo()._handle_notification(data)
    return {'status': 'ok'}
```

### Database utilities

```python
from odoo.http import db_list, db_filter

db_list()                    # list all databases
db_filter(dbs, host=None)    # filter by host-based DB filter
```
