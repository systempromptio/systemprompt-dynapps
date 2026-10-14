# Gotchas — Website Controllers (verificados con path:line)

Rutas relativas a `$ODOO_SRC/odoo18/` salvo que se indique v17/v19.

## WC-01 — `website=True` es lo que crea el contexto frontend
`http_routing/models/ir_http.py:386-387`, `IrHttp._match`:
```python
request.is_frontend = routing.get('website', False)
request.is_frontend_multilang = request.is_frontend and routing.get('multilang', routing['type'] == 'http')
```
Sin `website=True`: no hay `request.website`, ni multi-website, ni multilang, ni
`website.layout` correcto. Síntoma típico: la página renderiza pero sin header/footer del
sitio o falla `request.website`. Idéntico en v17 (`:414-415`) y v19 (`:375-376`).

## WC-02 — csrf activo por defecto en `type='http'` → POST da 403
`odoo/http.py:725-727`, `route` (default) y `odoo/http.py:2282-2287`, `HttpDispatcher.dispatch` (validación). Un formulario POST sin
`csrf_token` recibe 403. Solución: incluir en el form
`<input type="hidden" name="csrf_token" t-att-value="request.csrf_token()"/>`
o `csrf=False` en la ruta (solo si la entrada es de confianza). `request.csrf_token()` en
`odoo/http.py:1776`, `Request.csrf_token`.

## WC-03 — `auth='public'` ⇒ usuario público sin permisos
`website/models/ir_http.py:176-182`, `Http._auth_method_public` (se asigna el public user del website). Con público,
`request.env.user` no puede leer datos de negocio. Usar `request.env['x'].sudo()` y filtrar
manualmente la visibilidad (p.ej. `is_published`). Olvidarlo da AccessError o, peor, expone
datos al hacer sudo sin filtrar.

## WC-04 — `type='json'` no sirve páginas HTML
`odoo/http.py:702-704`, `route`: `'json'` es JSON-RPC, serializa el retorno como JSON. Para una página
usar `type='http'` (default). Confundirlos hace que el navegador descargue/ muestre JSON.

## WC-05 — `sitemap` puede ser callable, no solo bool
Ejemplo real `website_sale/controllers/main.py:145-161`, `WebsiteSale.sitemap_shop`: una función
`def sitemap_shop(env, rule, qs): ... yield {'loc': '/shop'}` enumera URLs dinámicas. Si se
pasa un callable mal firmado, la generación del sitemap falla silenciosamente para esa ruta.

## WC-06 — heredar un método con `@route` exige re-decorar
`odoo/http.py:692-695`, `route` (warning explícito en el docstring): "It is mandatory to re-decorate any
method that is overridden in controller extensions". Olvidar `@http.route()` al hacer
`super()` deja el método sin ruta (404) o usando un routing inesperado.

## WC-07 — `auth='bearer'` es de v18+, y lo exclusivo de v19 es otra cosa
Implementado en `odoo18/odoo/addons/base/models/ir_http.py:204`, `IrHttp._auth_method_bearer` (`_auth_method_bearer`) y
documentado en `odoo/http.py:709`, `route`, v18. En **v17 no existe** (cero ocurrencias de `bearer` en
su `http.py` y ningún `_auth_method_bearer` en todo `odoo/`).
Lo **exclusivo de v19** es que ese auth fuerce `save_session=False` (`odoo/http.py:828`, `route`, v19,
doc en `:807`): en v18 el modo funciona pero la sesión se guarda como en cualquier otra ruta.
No aplica a páginas web —es para tokens de API—, pero conviene no copiar de v19 a v18 dando por
hecho que la ruta es stateless.
<sub>Cita corregida 2026-08-06: la línea 750 que se citaba antes es la propiedad `env` de
`Controller`, nada que ver con bearer. **Corregido de nuevo 2026-08-14**: esta entrada afirmaba
«no está en v17/v18», y en v18 sí está — verificado contra fuente.</sub>

## WC-08 — `readonly=True` para GET de páginas públicas, y es de v18
`odoo/http.py:728`, `route`, v18 lo documenta y `:2101` lo **lee** de verdad
(`readonly = rule.endpoint.routing['readonly']`, con la variante callable justo debajo). De ahí
viaja a `_transactioning(..., readonly=readonly)` (`:2106`) y **el cursor se abre en `:2164`**:
`with contextlib.closing(self.registry.cursor(readonly=readonly_cr)) as cr:`.

> **Cita corregida 2026-08-17: esta línea apuntaba a `:1693`, que dice lo CONTRARIO.** Ahí Odoo
> fuerza un cursor de escritura —`cr_readwrite = registry.cursor(readonly=False)`— y el comentario
> de al lado explica por qué: *"use a RW cursor! Sequence data is not replicated and would be
> invalid if accessed on a readonly replica"* (`:1691-1692`). Quien fuera a comprobar el gotcha leía
> el argumento inverso al que se le estaba dando. Las otras dos citas (`:728`, `:2101`) sí eran
> correctas.
>
> En **v19** la cadena existe pero está reorganizada: el `routing['readonly']` se lee en `:2294` y
> ya no hay un `cursor(readonly=readonly_cr)`; se decide sobre el cursor ya abierto
> (`if readonly and cr.readonly:` → `cursor_mode = 'ro'`, `:2300-2301`). O sea que **las líneas de
> v18 no valen para v19**, aunque el atributo de ruta sea el mismo.
Recomendado en rutas que solo leen (listados, detalle) para descargar la primaria. NO usar en
rutas que escriben (POST de formulario) o dará error al intentar `commit`.
**En v17 no sirve de nada**: la clave está en los valores por defecto del routing
(`odoo17/odoo/http.py:880`, `_generate_routing_rules`, `'readonly': False`) pero ningún sitio la consume, así que ponerla
se acepta en silencio y no cambia el cursor.
<sub>Corregido 2026-08-14: decía «no existía como flag estable antes de v17», que se lee como
que en v17 ya vale.</sub>

## WC-09 — 404 editable solo para el diseñador del website
`website/models/ir_http.py:386-390`, `Http._get_exception_code_values`: `page_404` (plantilla editable) se activa solo si
`request.env.user.has_group('website.group_website_designer')`. El visitante anónimo ve el
404 estándar. No asumir que tu plantilla de error 404 personalizada se ve para todos.

## WC-10 — contenido estático debería ser `website.page`, no controller
`website/models/ir_http.py:302-346`, `Http._serve_page` (`_serve_page`): Odoo ya sirve cualquier `website.page`
por su `url`. Crear un controller para HTML estático editable duplica trabajo y le quita al
usuario la edición inline. Controller solo si hay lógica/parámetros/POST.

## WC-11 — `website.layout` cachea el `<head>`
`website/views/website_templates.xml:75`, `layout` ("Cache the <head> of the website.layout"). Datos
dependientes de la sesión/usuario colocados en zonas cacheadas se mostrarán obsoletos o
cruzados entre usuarios. Mantener lo personalizado fuera de la cabeza cacheada.

## WC-12 — respuestas no-HTML necesitan `make_response`, no `render`
`odoo/http.py:1893-1914`, `Request.make_response`. Para PDF, CSV, XML (sitemap propio), imágenes: construir
`request.make_response(data, headers=[('Content-Type', '...'), ('Content-Disposition', '...')])`.
`request.render` solo produce HTML vía QWeb. Devolver bytes crudos con `render` rompe el
Content-Type.

## Extra — `request.redirect` respeta multilang
`odoo/http.py:1942-1951`, `Request.redirect`: si hay DB delega en `ir.http._redirect`, que reescribe la URL con el
prefijo de idioma correcto. No construir el `Location` a mano ni redirigir con un `Response`
manual en frontend, o se pierde el idioma.
