---
name: indaws-odoo-website-verticals
version: 0.1.16
effort: high
model: opus
description: >-
  Desarrolla las apps verticales del website Odoo (v17-v19): blog, foro/karma,
  eLearning, eventos web, form builder y marketing (newsletter, link tracker/UTM).
  NOT eCommerce (→ odoo-ecommerce-developer); NOT snippets (→ odoo-website-snippets).
  Triggers: "extender el blog de Odoo", "karma del foro", "slide.channel",
  "registro de eventos en web", "website form builder", "popup de newsletter",
  "link tracker / UTM", "extend Odoo blog", "eLearning course",
  "web event registration".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode, Agent]
---

# Odoo Website Verticals

Desarrolla y extiende las apps website verticales de Odoo v17-v19: blog, foro,
eLearning, eventos, formularios web y marketing web (newsletter/popups, link
tracker/UTM). Cada vertical aporta modelos de negocio + controllers frontend
propios. Esta skill cubre los modelos clave, los mixins de extension y los
controllers de cada app, delimitando con las skills adyacentes.

Refs compartidas: `references/shared/coding-standards.md`, `references/shared/odoo-connection.md`

## Delimitacion (que NO cubre esta skill)

- Autoria de contenido HTML / Bootstrap del editor → `odoo-frontend`
- Tienda eCommerce (website_sale, carrito, pago) → `odoo-ecommerce-developer`
- Controllers base de pagina (`@http.route website=True`, `website.layout`, SEO genérico) → `odoo-website-controller`
- Snippets del editor (s_*, options.js) → `odoo-website-snippets`

Esta skill se centra en los **modelos de dominio** de cada vertical y sus
controllers especificos (registro de evento, voto de foro, completar slide, etc.).

## Workflow

1. **Identificar el vertical y el objetivo** [STOP — AskUserQuestion]
   - ¿blog / forum / slides / event / form builder / marketing (newsletter/links)?
   - ¿Extender modelo existente, anadir campos, nuevo controller, o modulo puente?
   - Version objetivo (v17 / v18 / v19) — afecta `type='json'` vs `type='jsonrpc'` y captcha.

2. **Analizar el modulo destino** [delega]
   - **[SONNET SUBAGENT]** `Agent(model: sonnet)` → `odoo-module-scanner` sobre el modulo custom: dependencias del `__manifest__.py` (¿declara website_blog / website_forum / website_slides / website_event / website / website_mass_mailing / website_links?), modelos y controllers existentes.

3. **Cargar la referencia del vertical** (solo la que aplique)
   - blog / forum → [`references/blog-forum.md`](references/blog-forum.md)
   - eLearning / eventos → [`references/elearning-events.md`](references/elearning-events.md)
   - formularios web / marketing (newsletter, link.tracker/UTM) → [`references/forms-and-marketing.md`](references/forms-and-marketing.md)

4. **Proponer arquitectura** [STOP — confirmación]
   - Modelos: `_inherit` del modelo del vertical + mixins necesarios (`website.seo.metadata`, `website.published.multi.mixin`).
   - Controllers: heredar la clase del controller del vertical y **re-decorar siempre** `@http.route` al sobrescribir un método (vacío si no cambias nada; los args se heredan del padre).
   - Seguridad: ¿afecta a karma (forum), captcha/whitelist (form), opt-in (newsletter)?

5. **Implementar**
   - Aplica los patrones de extension de la referencia cargada. Respeta los flags de los controllers (auth='public', csrf, website=True).

6. **Gotchas** — leer la seccion final de la referencia antes de cerrar (karma gating, publicacion de slides, estados de registro, whitelist del form builder, opt-in del newsletter, tracking UTM).

7. **Tests** [delega] — `odoo-test-generator` (HttpCase para flujos frontend).

## Diferencias de version (resumen)

- **v17 → v18 → v19**: las rutas JSON de los controllers pasan de `type='json'`
  (v17/v18) a `type='jsonrpc'` (v19). Afecta a forum (vote), slides (set_completed),
  event (registration/new), newsletter (subscribe). **El riesgo es asimetrico, y solo en un
  sentido:** en v19 `type='json'` sigue funcionando —es un **alias deprecado** que avisa
  (`DeprecationWarning`: *"Since 19.0, @route(type='json') is a deprecated alias to
  @route(type='jsonrpc')"*) y se reescribe a `jsonrpc` (`odoo/http.py:813-819`, `route.decorator`, v19). Al reves **si
  rompe**: `type='jsonrpc'` no existe como tipo de ruta en v17 ni en v18. Y **falla pronto y
  fuerte**, no con un 404 en runtime: el `assert routing.get('type','http') in _dispatchers.keys()`
  (`odoo/http.py:784`, `route.decorator`, v17, `:739` v18) revienta **al registrar la ruta**, o sea al cargar el modulo.
  Escribir `jsonrpc` solo cuando el target es v19.
- **v19 tiene TRES tipos de ruta, no dos.** Los dispatchers se registran por su `routing_type`:
  v17 y v18 solo tienen `http` y `json` (`odoo/http.py:2098`, `HttpDispatcher.routing_type`, v17/`:2162` v17, `:2264`/`:2328` v18); v19
  tiene `http` (`:2473`), `jsonrpc` (`:2544`) y **`json2` (`:2639`)**. `json2` recibe y devuelve
  **JSON pelado**, sin el sobre JSON-RPC de `{"jsonrpc":"2.0","method":"call","params":…}`. Si en
  v19 escribes un endpoint para que lo consuma algo que no es el cliente web de Odoo —un fetch
  propio, un webhook, una integracion— `json2` es el que quieres; `jsonrpc` obliga al consumidor a
  hablar el sobre.
- **website_form captcha**: en v18 la verificacion reCAPTCHA es explicita dentro del controller
  (`if request.env['ir.http']._verify_request_recaptcha_token('website_form')`,
  `website/controllers/form.py:47`, `WebsiteForm.website_form`); en **v19** se declara como **flag de ruta**
  `captcha='website_form'` en el propio `@http.route` (`website/controllers/form.py:31`, `WebsiteForm.website_form`; el parametro esta documentado en
  `odoo19/odoo/http.py:802-803`, `route`). **Consecuencia:** un controller de formulario propio en v19 que no
  ponga el flag se queda **sin captcha y sin error** — la proteccion antispam desaparece en silencio.
  Portar no rompe: `_verify_request_recaptcha_token` **sigue existiendo** en v19
  (`google_recaptcha/models/ir_http.py:37`, `Http._verify_request_recaptcha_token`), asi que un controller v18 que lo llamaba explicitamente
  sigue funcionando; lo que hay que recordar es el caso nuevo, no el viejo.
- **website_slides**: el campo de estado de miembro es `member_status`
  (selection 'invited'/'joined'/'ongoing'/'completed') en v17-v19; NO existe un
  booleano `completed` plano en el canal.
- **website_forum**: 37 campos `karma_*` estables en v17/v18/v19 (sin cambios de count).

Detalle por vertical en cada reference.

## GOTCHAS

1. **Forum: karma gating.** Casi toda acción (votar, comentar, editar, cerrar)
   exige un mínimo de karma; sin él el método lanza o la ruta deniega. Verifica los
   `karma_*` del `res.groups`/`forum` antes de exponer una acción.
2. **Slides: estado por `member_status`**, no un booleano `completed`; un slide
   "completado" se deriva del progreso del canal, no de un flag plano.
3. **Form builder = whitelist estricta.** Ningún campo es escribible desde un
   website form salvo que tenga `website_form_blacklisted=False` explícito → un
   campo "no aparece"/"no guarda" suele ser esto, no un bug de la vista.
4. **JSON routes**: `type='json'` (v17/v18) → `type='jsonrpc'` (v19) en
   vote/registration/subscribe. Dejar `json` en v19 **no rompe** (alias deprecado con warning);
   poner `jsonrpc` en v17/v18 **si rompe** (el tipo no existe). El error caro es el backport.
5. **Newsletter/mailing opt-in y blacklist**: la suscripción respeta blacklist y
   double opt-in; un contacto "no recibe" suele estar en blacklist, no perdido.
6. **UTM/link.tracker**: el tracking depende de la short URL generada; enlazar la
   URL final salta el `link.tracker` y no cuenta clics.

## NEVER
- NUNCA autoría de contenido HTML del editor aquí → `odoo-frontend`.
- NUNCA lógica de tienda/carrito → `odoo-ecommerce-developer`.
- NUNCA **omitir** el `@http.route` al sobrescribir un método de controller: Odoo lo exige
  —*"It is mandatory to re-decorate any method that is overridden in controller extensions but the
  arguments can be omitted"* (`odoo/http.py:760-763`, `route`, v19)—. Re-decora con `@http.route()` **vacío**:
  los argumentos se fusionan con los del padre (`:721-726`, `_generate_routing_rules` `:849-856`),
  así que no duplicas la ruta. Si lo omites, el framework avisa y lo decora él
  (*"The endpoint … is not decorated by @route(), decorating it myself"*, `:928` v19, `:844` v18,
  `:889` v17) — funciona, pero por una vía de compatibilidad y con WARNING en el log.
- NUNCA exponer una acción de foro sin comprobar el karma requerido.

## ALWAYS
- SIEMPRE hereda el modelo/controller del vertical y respeta sus mixins
  (`website.published.multi.mixin`, `website.seo.metadata`).
- SIEMPRE confirma la versión (json vs jsonrpc, captcha como flag en v19).
- SIEMPRE abre los campos del form builder con `website_form_blacklisted=False`.
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-website-verticals · v<version del frontmatter de esta skill>`, wrapped in the file-type comment, idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-module-scanner | common | Inventariar el módulo/dependencias del vertical |
| odoo-version-api-reference | common | Diffs http/routing por versión |
| odoo-test-generator | developer | HttpCase de flujos frontend |
| odoo-website-controller | developer | Controllers base de página / routing |

## Consumed by
- **odoo-module-creator** (developer) — al crear un módulo que extiende un vertical website.
