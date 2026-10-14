# Arquitectura del Portal Odoo 18

## Tabla de Contenidos

- [Módulo portal — Estructura](#módulo-portal--estructura)
- [Jerarquía de Templates](#jerarquía-de-templates)
- [CustomerPortal Controller](#customerportal-controller)
- [Flujo de una Request Portal](#flujo-de-una-request-portal)
- [Pager (Paginación)](#pager-paginación)
- [Searchbar](#searchbar)
- [JavaScript Portal](#javascript-portal)
- [Ficheros Clave](#ficheros-clave)

---

## Módulo portal — Estructura

**Ubicación**: `addons/portal/` (rutas relativas al root de fuente Odoo; resuélvelo vía `references/shared/odoo-source.md`, NO asumas `/opt/odoo`)
**Dependencias**: `web`, `web_editor`, `http_routing`, `mail`, `auth_signup` (`odoo18/addons/portal/__manifest__.py`, literal)

```
portal/
├── controllers/
│   ├── portal.py      # CustomerPortal, pager(), get_records_pager()
│   ├── mail.py        # PortalChatter (init, fetch, avatar)
│   ├── thread.py      # Message posting desde portal
│   ├── web.py         # Redirect portal users /web → /my
│   └── message_reaction.py
├── models/
│   ├── portal_mixin.py   # access_url, access_token, get_portal_url()
│   ├── mail_thread.py    # website_message_ids, _sign_token()
│   └── mail_message.py   # portal_message_format()
├── views/
│   └── portal_templates.xml  # 847 líneas — todos los templates base (v18, exacto)
├── wizard/
│   ├── portal_wizard.py   # Crear/revocar portal users
│   └── portal_share.py    # Compartir documentos con link
├── static/src/
│   ├── scss/portal.scss   # 475 líneas — estilos portal
│   ├── js/portal.js       # PortalDetails, PortalHomeCounters
│   ├── js/portal_sidebar.js  # Delay labels, print
│   └── chatter/           # Portal chatter (shadow DOM, OWL)
└── utils.py               # validate_thread_with_hash_pid/token
```

---

## Jerarquía de Templates

El portal tiene una cadena de templates que se anidan:

```
web.frontend_layout (base HTML: head, body, scripts)
  └─ portal.frontend_layout (añade: logo, user_dropdown, RTL, is_portal)
       └─ portal.portal_layout (añade: breadcrumbs, record_pager, sidebar, content area)
            ├─ portal.portal_my_home (dashboard con cards de documentos)
            ├─ portal.portal_my_details (formulario de cuenta)
            ├─ portal.portal_my_security (contraseña, API keys)
            └─ [templates de módulos hijos: sale, account, project...]
```

### Templates base clave

| Template ID | Propósito | Variables principales |
|------------|-----------|----------------------|
| `portal.portal_layout` (`odoo18/addons/portal/views/portal_templates.xml:162`, `portal_layout`) | Wrapper de toda página portal | `page_name`, `no_breadcrumbs`, `my_details` |
| `portal.portal_my_home` (`odoo18/addons/portal/views/portal_templates.xml:225`, `portal_my_home`) | Dashboard /my con categorías | `portal_alert_category_enable`, `portal_client_category_enable` |
| `portal.portal_docs_entry` | Card individual (counter + link) | `icon`, `title`, `url`, `placeholder_count`, `text` |
| `portal.portal_breadcrumbs` (`odoo18/addons/portal/views/portal_templates.xml:146`, `portal_breadcrumbs`) | Navegación breadcrumb | `page_name` |
| `portal.portal_searchbar` (`odoo18/addons/portal/views/portal_templates.xml:326`, `portal_searchbar`) | Barra filtros/ordenación/búsqueda | `searchbar_sortings`, `searchbar_filters`, `searchbar_groupby` |
| `portal.portal_table` (`odoo18/addons/portal/views/portal_templates.xml:279`, `portal_table`) | Tabla responsive con paginación | contenido como child + `pager` |
| `portal.portal_record_sidebar` (`odoo18/addons/portal/views/portal_templates.xml:290`, `portal_record_sidebar`) | Sidebar de detail pages | `title`, `entries`, `classes` |
| `portal.side_content` (`odoo18/addons/portal/views/portal_templates.xml:826`, `side_content`) | Info usuario + contacto (offcanvas en mobile) | `sales_user` |
| `portal.message_thread` | Container del chatter | `object`, `token`, `pid`, `hash`, `allow_composer` |
| `portal.pager` (`odoo18/addons/portal/views/portal_templates.xml:732`, `pager`) | Controles de paginación | `pager` dict |
| `portal.record_pager` (`odoo18/addons/portal/views/portal_templates.xml:723`, `record_pager`) | Navegación prev/next entre registros | `prev_record`, `next_record` |

### Categorías de /my/home

Las cards se agrupan en categorías. Cada categoría se muestra solo si tiene cards:

| Categoría | Variable enable | Uso típico |
|-----------|----------------|------------|
| `portal_alert_category` | `portal_alert_category_enable` | Alertas (documentos a firmar, facturas vencidas) |
| `portal_client_category` | `portal_client_category_enable` | Documentos del cliente (pedidos, facturas) |
| `portal_vendor_category` | `portal_vendor_category_enable` | Documentos del proveedor (compras) |
| `portal_service_category` | `portal_service_category_enable` | Servicios (proyectos, tickets) |
| `portal_common_category` | siempre visible | Común |

---

## CustomerPortal Controller

**Archivo**: `odoo18/addons/portal/controllers/portal.py:131-466`, `CustomerPortal` — clase `CustomerPortal` (el siguiente símbolo de nivel superior, `get_error`, empieza en `:467`)

### Rutas base

| Ruta | Método | Auth | Tipo | Propósito |
|------|--------|------|------|-----------|
| `/my`, `/my/home` | `home()` | user | http | Dashboard portal |
| `/my/account` | `account()` | user | http | Editar datos partner |
| `/my/security` | `security()` | user | http | Contraseña + API keys |
| `/my/deactivate_account` | `deactivate_account()` | user | http | Borrar cuenta |
| `/my/counters` | `counters()` | user | **v18 `json` · v19 `jsonrpc`** | Actualizar contadores async |

### Métodos core

**`_prepare_portal_layout_values()`** — `odoo18/addons/portal/controllers/portal.py:135-153`, `CustomerPortal._prepare_portal_layout_values`
- Valores base para TODAS las páginas `/my/*`
- Resuelve `sales_user` desde el partner
- Define `page_name`

**`_prepare_home_portal_values(counters)`** — `portal/controllers/portal.py:155`, `CustomerPortal._prepare_home_portal_values` (v18) / **`:166` (v19)**
- Devuelve contadores para las cards del dashboard
- Vacío por defecto — cada módulo hijo lo extiende con `super()`
- `counters` es una lista de strings indicando qué contadores calcular
- **El "lazy" no es doctrina, está en el código** (verificado 2026-08-07): la carga de `/my/home` lo
  llama con **lista vacía** (`:176` v18 / `:187` v19), y hay una ruta aparte `/my/counters` que pasa
  la lista real (`:164` v18 / `:175` v19). Por eso omitir el chequeo de `counters` no da un bug
  visible: en la carga de la página tu contador simplemente no se pide.
- ⚠️ **Esa ruta cambia de `type` en v19**: `type='json'` (v18, `:164`) → **`type='jsonrpc'`**
  (v19, `:175`). Si extiendes/reemplazas la ruta, usa el `type` de la versión objetivo. `type='json'`
  sigue valiendo en v19 como alias deprecado (con `DeprecationWarning`), así que un módulo v18 arranca
  en v19 — pero no al revés.

**`_document_check_access(model_name, document_id, access_token)`** — v17 `portal/controllers/portal.py:439-459`, `CustomerPortal._document_check_access`,
v18 `:383-402`, v19 `:961-980`

> **La comprobación normal cambia de API en v18** (medido 2026-08-17): v17 llama **dos** métodos —
> `document.check_access_rights('read')` y `document.check_access_rule('read')` (`:454-455`)— y
> v18/v19 uno solo, `document.check_access('read')` (v18 `:398`, v19 `:976`). Si sobrescribes este
> método, el `super()` es igual pero **el bloque que replicas no**.
>
> Y los dos viejos **no han desaparecido del ORM**: `check_access_rights` y `check_access_rule`
> siguen definidos en v18 (`odoo/models.py:4490`, `BaseModel.check_access_rights`, `:4507`) y en v19 (`odoo/orm/models.py:4162`,
> `:4176`). Lo que cambió es qué usa `portal`, no qué existe.
- Valida acceso a un documento individual
- Primero intenta acceso normal (`check_access('read')`)
- Si falla y hay token → compara con `consteq()`
- Devuelve documento con SUPERUSER (sudoed) para renderizar

**`_get_page_view_values(document, access_token, values, session_history, no_breadcrumbs)`** — `odoo18/addons/portal/controllers/portal.py:404-439`, `CustomerPortal._get_page_view_values`
- Prepara valores para detail pages
- Inyecta: `token`, `pid`, `hash` (para chatter)
- Inyecta: `prev_record`, `next_record` (del session history)
- Inyecta: `error`, `warning`, `success` messages

**`_show_report(model, report_type, report_ref, download)`** — `odoo18/addons/portal/controllers/portal.py:441-455`, `CustomerPortal._show_report`
- Renderiza reportes desde portal (HTML, PDF, text)
- Configura headers HTTP (Content-Type, Content-Disposition)

### Redirect de portal users

`portal/controllers/web.py` intercepta:
- `/` → redirige a `/my` si `user.share=True`
- `/web` → redirige a `/my` si no es usuario interno
- Login redirect → `/my` por defecto

---

## Flujo de una Request Portal

### List page (ej: /my/orders)

```
1. Browser → GET /my/orders?sortby=date&filterby=all
2. Odoo routing → sale.CustomerPortal.portal_my_orders()
3. Controller:
   a. _prepare_portal_layout_values() → valores base
   b. _prepare_orders_domain(partner) → domain filtrado por partner
   c. Aplicar sortby/filterby del querystring
   d. pager() → calcular paginación (offset, pages, urls)
   e. SaleOrder.search(domain, order=sort, limit=80, offset=pager['offset'])
   f. request.session['my_orders_history'] = orders.ids[:100]
   g. request.render('sale.portal_my_orders', values)
4. QWeb renderiza template → HTML response
```

### Detail page (ej: /my/orders/123)

```
1. Browser → GET /my/orders/123?access_token=uuid
2. Odoo routing → sale.CustomerPortal.portal_order_page()
3. Controller:
   a. _document_check_access('sale.order', 123, access_token)
      → Si user tiene permiso OR token válido → devuelve order_sudo
      → Si no → redirect /my
   b. Si report_type → _show_report() (PDF/HTML)
   c. Preparar valores: sidebar, payment, messages
   d. _get_page_view_values(order_sudo, token, values, 'my_orders_history')
      → Añade chatter tokens, record pager
   e. request.render('sale.sale_order_portal_template', values)
4. QWeb renderiza template 2 columnas → HTML response
```

---

## Pager (Paginación)

**Función**: `portal.controllers.portal.pager()` (líneas 17-82)

```python
pager(url, total, page=1, step=30, scope=5, url_args=None)
```

**Parámetros**:
- `url`: URL base (ej: `/my/orders`)
- `total`: Total de registros
- `page`: Página actual (1-indexed)
- `step`: Registros por página (default 30, los controllers usan `_items_per_page=80`)
- `scope`: Páginas visibles en el pager (default 5)
- `url_args`: Query params adicionales (sortby, filterby, etc.)

**Retorna dict**:
```python
{
    'page_count': int,           # Total páginas
    'offset': int,               # Offset para search()
    'page': {'url': str, 'num': int},       # Página actual
    'page_first': {...},         # Primera página
    'page_previous': {...},      # Anterior
    'page_next': {...},          # Siguiente
    'page_last': {...},          # Última
    'pages': [{'url': str, 'num': int}, ...]  # Páginas visibles
}
```

Las URLs de página usan el patrón `/page/{n}` (página 1 no tiene sufijo).

### Record Pager (prev/next entre registros)

**Función**: `get_records_pager(ids, current)` (líneas 85-110)

- Navega entre registros individuales en una lista
- Usa `access_url` o `website_url` del registro
- Añade `access_token` si el registro lo tiene
- Los IDs vienen del session storage (guardados en la list page)

---

## Searchbar

**Template**: `portal.portal_searchbar` (portal_templates.xml, líneas 326-411)

Tres tipos de filtros, todos opcionales:

### Sort By (searchbar_sortings)
```python
searchbar_sortings = {
    'date': {'label': _('Date'), 'order': 'create_date desc'},
    'name': {'label': _('Reference'), 'order': 'name'},
    'state': {'label': _('Status'), 'order': 'state'},
}
```

### Filter By (searchbar_filters)
```python
searchbar_filters = {
    'all': {'label': _('All'), 'domain': []},
    'open': {'label': _('Open'), 'domain': [('state', '=', 'open')]},
    'done': {'label': _('Done'), 'domain': [('state', '=', 'done')]},
}
```

### Group By (searchbar_groupby)
```python
searchbar_groupby = {
    'none': {'label': _('None'), 'sequence': 10},
    'state': {'label': _('Status'), 'sequence': 20},
    'project_id': {'label': _('Project'), 'sequence': 30},
}
```

### Search Input (searchbar_inputs)
```python
searchbar_inputs = {
    'all': {'label': _('All'), 'input': 'all'},
    'name': {'label': _('Name'), 'input': 'name'},
    'description': {'label': _('Description'), 'input': 'description'},
}
```

**Variables del template**:
- `sortby` — criterio de ordenación actual
- `filterby` — filtro actual
- `groupby` — agrupación actual
- `search` — texto de búsqueda
- `search_in` — campo de búsqueda
- `default_url` — URL base para los filtros

El searchbar colapsa en móvil con `navbar-expand-lg`.

---

## JavaScript Portal

### PortalHomeCounters (`portal.js`)

- Selector: `.o_portal_my_home`
- Hace RPC a `/my/counters` de forma asíncrona
- Actualiza los placeholders de las cards
- Muestra/oculta cards según si el counter > 0
- Máximo 3 RPCs, 5 counters por RPC (batching)

### PortalDetails (`portal.js`)

- Selector: `.o_portal_details`
- Adapta formulario de dirección según país seleccionado
- Muestra/oculta opciones de estado dinámicamente

### PortalSidebar (`portal_sidebar.js`)

- Formatea delay labels: "Due today", "Due in X days", "X days overdue"
- Parsea atributos `datetime` en elementos `.o_portal_sidebar_timeago`
- Función de impresión via iframe oculto

### PortalSearchPanel (`portal.js`)

- Maneja el panel de búsqueda y filtros
- Construye URLs con los parámetros de búsqueda seleccionados

---

## Ficheros Clave

| Propósito | Ruta |
|-----------|------|
| Controller principal | `addons/portal/controllers/portal.py` |
| Chatter controller | `addons/portal/controllers/mail.py` |
| Portal mixin | `addons/portal/models/portal_mixin.py` |
| Mail thread portal | `addons/portal/models/mail_thread.py` |
| Templates base | `addons/portal/views/portal_templates.xml` |
| SCSS portal | `addons/portal/static/src/scss/portal.scss` |
| JS portal | `addons/portal/static/src/js/portal.js` |
| JS sidebar | `addons/portal/static/src/js/portal_sidebar.js` |
| Security utils | `addons/portal/utils.py` |
| Sale portal (ejemplo) | `addons/sale/controllers/portal.py` |
| Account portal (ejemplo) | `addons/account/controllers/portal.py` |
| Project portal (ejemplo) | `addons/project/controllers/portal.py` |
