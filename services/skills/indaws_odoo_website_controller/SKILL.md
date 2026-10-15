---
name: indaws-odoo-website-controller
version: 0.3.10
effort: high
model: opus
description: >-
  Escribe controllers de páginas web públicas en módulos Odoo (v17-v19):
  @http.route (website=True, auth public/user, sitemap, multilang),
  request.render con website.layout, redirects, multi-website, SEO/canonical,
  páginas de error y herencia de controllers. NOT portal /my/* (→
  odoo-portal-developer); NOT webhooks/API externa (→ odoo-integration-builder).
  Triggers: "controller de página web", "ruta pública en Odoo", "@http.route
  website", "render QWeb frontend", "sitemap de mi ruta", "multi-website routing",
  "website controller", "public web page route", "serve a website page".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode, Agent]
---

# Odoo Website Controller

Diseña e implementa controllers que sirven páginas web públicas en módulos Odoo v17-v19.
El controller orquesta: declara la ruta `@http.route(... website=True)`, prepara el
qcontext y delega el render a una plantilla QWeb que extiende `website.layout`. Impone
los flags correctos (auth, csrf, sitemap, multilang) y los patrones de multi-website/SEO
desde el primer scaffold.

Refs compartidas: `references/shared/coding-standards.md`, `references/shared/odoo-connection.md`

## Workflow

1. **Definir la página** [STOP — AskUserQuestion]
   - Path(s) y si lleva parámetros de URL (`<int:id>`, `<model("x"):rec>`, `<string:slug>`)
   - Público anónimo (`auth='public'`) o requiere login (`auth='user'`)
   - ¿GET (mostrar) y/o POST (formulario)? → define `methods` y `csrf`
   - ¿Debe aparecer en `sitemap.xml`? ¿Multi-website / multilang?
   - Versión objetivo (v17/v18/v19)

2. **Analizar el módulo existente** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` → `odoo-module-scanner` sobre el módulo destino: controllers existentes, modelos, dependencias del `__manifest__.py` (necesita `website`).

3. **Proponer arquitectura** [STOP — confirmación]
   - Fichero `controllers/main.py` + `controllers/__init__.py`
   - Plantilla(s) QWeb en `views/` que extienden `website.layout`
   - Nombres de ruta y de método definitivos

4. **Scaffold del controller**
   - Leer [`references/routing.md`](references/routing.md): firma completa de `@http.route`, objeto `request`, parámetros de URL, herencia de controllers (re-decorar `@route`), multi-website y multilang, `website.page` vs ruta de controller.

5. **Render y SEO**
   - Leer [`references/rendering-and-seo.md`](references/rendering-and-seo.md): `request.render(template, values)`, `t-call="website.layout"`, valores de contexto (`main_object`, `additional_title`), meta/canonical, páginas de error 404/403/500, `assets_frontend`.

6. **Robustez y revisión**
   - Leer [`references/gotchas.md`](references/gotchas.md) antes de cerrar: `sudo()` con `auth='public'`, csrf en POST, `website=True` obligatorio, orden de rutas, cache de página.

7. **Tests** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` → `odoo-test-generator` con `HttpCase` (tour o `url_open`) para validar status, render y multilang.

## GOTCHAS

1. **`website=True` define el contexto frontend** — Sin él la ruta NO es frontend: `request.website` no se setea, no hay multi-website ni multilang. Verificado en `http_routing/models/ir_http.py` — **y la línea cambia en cada major**: v17 `:414`, v18 `:386`, v19 `:375` (`request.is_frontend = routing.get('website', False)`, idéntica en las tres; remedido 2026-08-18). Esta nota citaba `:386-387` **sin versión**, y en v19 esa línea es otra cosa (`allow_redirect = (...)`): quien la comprobara contra un checkout de 19 concluiría que la skill miente.
2. **csrf por defecto en `type='http'`** — POST sin `csrf_token` se rechaza con **400** (MEDIDO en v18 y v19; la skill decia 403). El cuerpo si nombra el token. El POST necesita `<input type="hidden" name="csrf_token" t-att-value="request.csrf_token()"/>` o `csrf=False`. Ver `odoo/http.py:725-727`, `route` (la doc del parámetro `csrf`), y la comprobación real en `odoo/http.py:2282-2287`, `HttpDispatcher.dispatch`.
3. **`auth='public'` → `request.env.user` es el usuario público** — Sin permisos sobre datos de negocio. Usar `request.env['x'].sudo()` para leer/escribir, y filtrar manualmente lo que el anónimo puede ver. Ver `website/models/ir_http.py:176-182`, `Http._auth_method_public`.
4. **`type='json'` no es para páginas** — Sirve JSON-RPC, no HTML. Para páginas siempre `type='http'` (es el default). Ver `odoo/http.py:702-704`, `route`.
5. **`sitemap` puede ser callable** — `sitemap=False`/`True` o una función `def f(env, rule, qs): yield {'loc': ...}`. Ejemplo real `website_sale/controllers/main.py:145-161`, `WebsiteSale.sitemap_shop`.
6. **Re-decorar al heredar — y el síntoma NO dice que falte el decorador (MEDIDO v18/v19).** Al sobrescribir un método con `@route` hay que volver a poner `@http.route(...)` (puede ir sin args para heredarlos). Si se omite: Odoo avisa al construir el mapa (`WARNING ... is not decorated by @route(), decorating it myself`) y **la petición muere con 500** si el método devuelve un `str` — que es el caso normal en `type='http'`. El traceback apunta a `_inject_future_response` con `AttributeError: 'str' object has no attribute 'headers'`: habla de cabeceras, no del decorador. Medido aparte: **el override SÍ se ejecuta** y, si devuelve una `Response` de verdad (`request.make_response`), responde 200 — o sea que la auto-decoración resuelve el enrutado pero pierde el `type='http'` que envuelve el valor. Fuente: `odoo/http.py:692-695`, `route` (la advertencia de la docstring), y el log en `odoo/http.py:844`, `_generate_routing_rules`; sonda: `tools/bench-orm::TestControladores`.
7. **`auth='bearer'` existe desde v18, no solo en v19** — implementado en `odoo18/odoo/addons/base/models/ir_http.py:204`, `IrHttp._auth_method_bearer` (`_auth_method_bearer`) y documentado en `odoo/http.py:709`, `route`, v18. En **v17 no está** (cero ocurrencias). Lo que sí es exclusivo de v19 es que fuerce `save_session=False` para ese auth (`odoo/http.py:828`, `route`, v19, doc en `:807`): en v18 el modo existe pero la sesión se guarda igual. Para páginas no se usa; relevante solo si se confunde con auth de API.
8. **`readonly=True` para rutas GET es de v18, no de v17** — abre cursor en réplica de solo lectura y mejora el rendimiento de páginas públicas: documentado en `odoo/http.py:728`, `route`, v18 y **consumido** en `:2101` (`rule.endpoint.routing['readonly']`) y `:1693`. En v17 la clave existe en los valores por defecto del routing (`odoo17/odoo/http.py:880`, `_generate_routing_rules`) pero **no la lee nadie**: ponerla ahí no hace nada y no avisa.
9. **404 personalizado solo para diseñador** — `_serve_page` y `_get_exception_code_values` muestran `page_404` editable solo si el user tiene `website.group_website_designer`; el anónimo ve un 404 plano. Ver `website/models/ir_http.py:386-395`, `Http._get_exception_code_values`.
10. **Página estática → `website.page`, no controller** — Si el contenido es CMS editable sin lógica, créalo como `website.page` (lo sirve `_serve_page`, `website/models/ir_http.py:302-346`, `Http._serve_page`). El controller es para rutas con lógica/parámetros.
11. **Cache de página frontend** — `website.layout` cachea el `<head>` (`website/views/website_templates.xml:75`, `layout`). Contenido dependiente de usuario/sesión no debe ir en zonas cacheadas; usar `t-cache="None"` donde aplique.
12. **`request.make_response` para no-HTML** — PDF, CSV, XML (sitemap) requieren `request.make_response(data, headers=[('Content-Type', ...)])`, no `render`. Ver `odoo/http.py:1893-1914`, `Request.make_response`.

## NEVER

- Omitir `website=True` en una ruta que deba ser multi-website, multilang o usar `website.layout`
- Usar `type='json'` para servir una página HTML
- Acceder a datos de negocio con el usuario público sin `sudo()` y sin filtrar visibilidad
- Servir POST sin csrf_token ni `csrf=False` explícito
- Heredar un método con `@route` sin volver a decorarlo con `@http.route`
- Meter lógica de negocio pesada en el controller (delegar al modelo); el controller orquesta
- Hardcodear el idioma o el `website_id` en la ruta o el render
- Crear un controller para contenido estático que debería ser un `website.page`

## ALWAYS

- Leer la reference de cada fase antes de generar código
- Confirmar arquitectura y nombres de ruta con el usuario (Step 3)
- Declarar dependencia `website` en `__manifest__.py`
- Renderizar con `t-call="website.layout"` para que la página tenga header/footer del sitio
- Marcar `readonly=True` en rutas que solo leen (**v18+**; en v17 se acepta y se ignora)
- Pasar `main_object` en el qcontext si la página representa un registro (habilita editor/SEO)
- Definir `sitemap=` explícito (True/False/callable) en cada ruta pública
- Usar `request.redirect()` (werkzeug) para 3xx, no construir Response a mano
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-website-controller · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-portal-developer | developer | Si la página es del portal de cliente (`/my/*`, `CustomerPortal`, access_token) |
| odoo-integration-builder | developer | Si es un webhook / endpoint de API / cliente REST externo (no una página) |
| odoo-frontend | common | Para diseñar el HTML/Bootstrap del contenido de la plantilla QWeb |
| odoo-version-api-reference | common | Diffs de routing/http entre v17/v18/v19 |
| odoo-module-scanner | common | Inventariar controllers/modelos del módulo antes de ampliar |
| odoo-test-generator | developer | HttpCase / tours para validar las rutas |

## Consumed by

- `odoo-module-creator` — cuando un módulo nuevo necesita páginas web públicas

## REFERENCES

| Fichero | Contenido |
|---------|-----------|
| [`references/routing.md`](references/routing.md) | `@http.route` completo, objeto `request`, params de URL, herencia de controllers, multi-website/multilang, `website.page` vs ruta |
| [`references/rendering-and-seo.md`](references/rendering-and-seo.md) | `request.render`, QWeb frontend, `website.layout`/`web.frontend_layout`, qcontext, SEO/canonical/meta, errores 404/403/500, assets_frontend |
| [`references/gotchas.md`](references/gotchas.md) | 12 gotchas reales con path:line: auth+sudo, csrf, website=True, orden de rutas, cache, sitemap callable |
