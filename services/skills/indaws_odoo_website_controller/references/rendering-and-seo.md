# Render QWeb frontend, layout y SEO

Fuente: `$ODOO_SRC/odoo18/odoo/http.py`, `addons/website/views/website_templates.xml`,
`addons/portal/views/portal_templates.xml`, `addons/website/models/ir_http.py`.

## `request.render`

```python
return request.render('mi_modulo.mi_template', {
    'records': records,
    'main_object': record,        # registro que representa la página (SEO/editor)
    'additional_title': record.name,
})
```

- `request.render(template, qcontext=None, lazy=True, **kw)` (`odoo/http.py:1958-1974`, `Request.render`).
  Render diferido: el QWeb se evalúa al final del dispatch, lo que permite que otros
  hooks alteren el qcontext.
- `mimetype=` en `kw` para servir otra cosa que `text/html` (ej. `_serve_page` usa
  `mimetype=...` para `.xml`, `website/models/ir_http.py:344`, `Http.response`).
- Para no-HTML (PDF/CSV/sitemap XML) NO usar `render`: usar `request.make_response(data,
  headers=[('Content-Type', 'application/xml')])` (`odoo/http.py:1893-1914`, `Request.make_response`).

## La plantilla debe extender el layout del sitio

```xml
<template id="mi_template" name="Mi Página">
    <t t-call="website.layout">
        <t t-set="additional_title">Mi título</t>
        <div id="wrap" class="oe_structure">
            <div class="container py-5">
                <h1 t-field="main_object.name"/>
                <t t-foreach="records" t-as="rec">
                    <p t-esc="rec.display_name"/>
                </t>
            </div>
        </div>
    </t>
</template>
```

### Cadena de layouts
- `web.frontend_layout` — base genérica del frontend.
- `portal.frontend_layout` hereda `web.frontend_layout`
  (`portal/views/portal_templates.xml:3`, `frontend_layout`).
- `website.layout` hereda `portal.frontend_layout`
  (`website/views/website_templates.xml:39`, `layout`) y añade header/footer/menús del website,
  selector de idioma, etc.

Para páginas de un sitio web público usa **`website.layout`** (header/footer del sitio).
`web.frontend_layout` solo si quieres una página frontend mínima sin la cáscara del website.

## Valores de contexto frecuentes (qcontext)

| Clave | Efecto |
|-------|--------|
| `main_object` | Registro que la página representa. Habilita el botón "Promote"/SEO del editor y el `<head>` canónico. |
| `additional_title` | Se concatena al `<title>` de la página. |
| `request` | Inyectado automáticamente al renderizar (`odoo/http.py:1431`, `_Response.render`). |
| `no_header` / `no_footer` | Ocultan header/footer del `website.layout`. |
| `pager` | Dict de `portal_pager(...)` para paginación. |

`oe_structure` en un contenedor del `<div id="wrap">` lo hace editable/arrastrable en el
editor del website.

## SEO / meta / canonical

- Con `main_object` seteado, `website.layout` añade automáticamente meta description,
  Open Graph y `<link rel="canonical">` a partir de los campos SEO del registro
  (mixin `website.seo.metadata`).
- Para SEO editable, el modelo de `main_object` debería heredar `website.seo.metadata`
  (campos `website_meta_title`, `website_meta_description`, `website_meta_keywords`).
- El canonical respeta multilang: Odoo genera `hreflang` por idioma frontend.
- Sitemap: declarar `sitemap=True` o un callable en `@http.route`. El callable hace
  `yield {'loc': '/ruta'}` (ej. `website_sale/controllers/main.py:145-161`, `WebsiteSale.sitemap_shop`).

## Páginas de error 404 / 403 / 500

- 404: levantar `werkzeug.exceptions.NotFound()` o `request.not_found()`. Para un usuario
  con `website.group_website_designer` Odoo muestra la plantilla editable `website.page_404`
  (`website/models/ir_http.py:386-390`, `Http._get_exception_code_values`, template `website/views/website_templates.xml:2449`, `protected_403`); el anónimo
  ve un 404 plano.
- 403: `werkzeug.exceptions.Forbidden()`. Caso especial de página protegida por contraseña
  → `protected_403` (`website/models/ir_http.py:391-394`, `Http._get_exception_code_values`, template `:2470`).
- 500: gestionado por `_get_values_500_error` (`website/models/ir_http.py:397`, `Http._get_values_500_error`); muestra
  plantilla de error. No capturar `Exception` para "tragarse" el 500 — dejar que Odoo lo
  renderice y loggear.

```python
from werkzeug.exceptions import NotFound, Forbidden

@http.route('/recurso/<int:rec_id>', type='http', auth='public', website=True)
def show(self, rec_id, **kw):
    rec = request.env['mi.model'].sudo().browse(rec_id).exists()
    if not rec or not rec.is_published:
        raise NotFound()
    return request.render('mi_modulo.tmpl', {'main_object': rec})
```

## Assets frontend

- El CSS/JS de páginas web va al bundle `web.assets_frontend` (no `web.assets_backend`):

```xml
<template id="assets" inherit_id="web.assets_frontend">
    <xpath expr="." position="inside">
        <link rel="stylesheet" type="text/scss"
              href="/mi_modulo/static/src/scss/page.scss"/>
        <script type="text/javascript"
                src="/mi_modulo/static/src/js/page.js"/>
    </xpath>
</template>
```

- En v17+ se puede declarar también vía `assets` en el `__manifest__.py`
  (`'web.assets_frontend': [...]`). El diseño visual del HTML lo cubre la skill `odoo-frontend`.
