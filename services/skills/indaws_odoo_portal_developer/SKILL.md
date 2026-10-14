---
name: indaws-odoo-portal-developer
version: 1.2.8
effort: medium
model: opus
description: >
  Desarrolla y personaliza páginas del portal de Odoo 18 (CE + Enterprise) con UX profesional para
  PC y móvil. Cubre controllers (CustomerPortal, rutas /my/*), templates QWeb (layouts, herencia,
  sidebar), frontend responsive (Bootstrap 5, SCSS, mobile-first), seguridad (portal.mixin,
  access_token, record rules), chatter portal, pagos y patrones de extensión probados (counters,
  list pages, detail pages). Usa este skill para crear páginas de portal, añadir secciones a /my,
  personalizar vistas existentes, mejorar la UX móvil, integrar chatter o pagos, o debuggear
  acceso/seguridad del portal. Triggers: "portal", "página de portal", "portal page", "/my/",
  "customer portal", "portal mobile", "portal responsive", "portal access token", "portal sidebar",
  "portal chatter", "portal payment", "CustomerPortal", "portal.mixin", "portal list page",
  "portal detail page", "personalizar portal".
---

# Odoo Portal Developer

Skill para crear y personalizar páginas del portal de Odoo 18 con UX profesional para PC y móvil.

> **Alcance real: TODO el material está anclado a v18 (CE+EE) y no se ha verificado contra v19.** Lo
> comprobado de v19 hasta ahora (2026-08-07), como muestra de que sí hay deltas: las rutas jsonrpc
> cambian de `type='json'` a **`type='jsonrpc'`** (`portal/controllers/portal.py:164`, `CustomerPortal.counters` v18 → `:175`
> v19) y los métodos core se desplazan (`_prepare_home_portal_values` `:155` → `:166`). El `json`
> sigue valiendo en v19 como **alias deprecado**, así que lo que generes para v18 arranca en v19 con
> warning; lo que generes para v19 **no** arranca en v18. Para un proyecto v19, verifica antes de
> emitir — no extrapoles esta skill.

> **Filosofía**: El portal es la cara del cliente. Cada página debe ser rápida, clara en móvil,
> y seguir los patrones de Odoo para que las actualizaciones futuras no rompan nada.

## References (leer según necesidad)

| Ref | Cuándo leer |
|-----|-------------|
| `portal-architecture.md` | **Siempre primero** — Estructura del módulo portal, jerarquía de templates, controllers base, flujo de una request |
| `portal-extension-patterns.md` | Al crear nuevas páginas — Patrones para counters, list pages, detail pages, searchbar/filters |
| `portal-frontend-ux.md` | Al diseñar UI — Clases CSS, responsive design, mobile-first, SCSS, accesibilidad |
| `portal-security.md` | Al configurar accesos — portal.mixin, access tokens, record rules, portal users |
| `portal-chatter-payments.md` | Al integrar chatter o pagos — Mensajería portal, formularios de pago, firma digital |

## Anatomía de una extensión de portal

Cada módulo que extiende el portal sigue una estructura fija (manifest `depends: ['portal']`,
controllers, models con `portal.mixin`, templates, security, static). Estructura completa y el
patrón general de 5 pasos en `references/portal-extension-patterns.md` sección "Patrón General".

## Los 3 tipos de página portal

Todo portal page cae en una de estas categorías:

### 1. Home Counter (card en /my/home)

Añade una tarjeta con contador al dashboard `/my`. No es una página propia.

**Controller**: Override `_prepare_home_portal_values(counters)` (lazy + `has_access('read')`)
**Template**: Inherit `portal.portal_my_home` con `portal.portal_docs_entry`

Código completo (Python + XML) en `references/portal-extension-patterns.md` sección "Home Counter".

### 2. List Page (/my/things)

Tabla paginada con filtros, ordenación y búsqueda.

**Controller**: Ruta `@http.route(['/my/things', '/my/things/page/<int:page>'])` con `auth="user"`
**Template**: Usa `portal.portal_layout` + `portal.portal_searchbar` + `portal.portal_table`

Patrón completo en `references/portal-extension-patterns.md` sección "List Page".

### 3. Detail Page (/my/things/<id>)

Vista individual con sidebar, chatter y opcionalmente pagos.

**Controller**: Ruta con `auth="public"` (soporta access tokens)
**Template**: Layout 2 columnas — `col-lg-8` content + `col-lg-4` sidebar

Patrón completo en `references/portal-extension-patterns.md` sección "Detail Page".

## Reglas de oro

1. **Siempre `super()`** en `_prepare_home_portal_values()` — si no, borras los counters de otros módulos
2. **Lazy counters**: solo computar si `'mi_counter' in counters` — el frontend los pide por lotes
3. **`has_access('read')`** antes de cualquier `search_count` — previene errores de permisos
4. **`auth="public"` + token** en detail pages — permite compartir links sin login
5. **Domain por partner** en toda query portal — el usuario solo ve SUS registros
6. **Session history** limitada a 100 IDs — habilita el record pager (prev/next)
7. **`_get_page_view_values()`** en detail pages — inyecta chatter, pager, token automáticamente
8. **Herencia de templates** con `inherit_id` + `xpath` — nunca duplicar templates base
9. **Mobile-first**: diseñar primero para móvil, luego adaptar a desktop
10. **`consteq()`** para comparar tokens — previene timing attacks

## Workflow: Crear nueva sección de portal

1. **Leer** `portal-architecture.md` para entender la base
2. **Decidir** qué tipo de página(s) necesitas (counter / list / detail)
3. **Modelo**: añadir `_inherit = ['portal.mixin']`, implementar `_compute_access_url()`
4. **Seguridad**: crear record rules para `base.group_portal` filtradas por partner. **Ojo: eso
   cubre solo la puerta de identidad.** El `access_token` es una segunda puerta —autorización por
   capacidad— que se salta las record rules por diseño. Y **el `sudo` no es cosa del token**
   (precisado 2026-08-17): `_document_check_access` hace `return document_sudo` **incondicional**,
   así que *también* el usuario de portal que pasa la record rule limpiamente recibe un recordset
   **SUPERUSER**. El `try/except` decide si se lanza `AccessError`, no qué se devuelve
   (v17 `portal/controllers/portal.py:439-459`, `CustomerPortal._document_check_access`, v18 `:383-402`, v19 `:961-980 @d1d161033`; docstring: *":return:
   expected record, SUDOED, with SUPERUSER context"*). Consecuencia: a partir de esa llamada **no
   hay barrera del ORM en ninguna de las dos puertas**, y todo lo que el controlador haga con ese
   recordset —escribir, recorrer relacionales, pintar— va sin record rules. Ver `portal-security.md`
5. **Controller**: heredar `CustomerPortal`, añadir rutas
6. **Templates**: heredar `portal.portal_my_home` (counter) + crear list/detail templates
7. **Frontend**: aplicar patrones responsive de `portal-frontend-ux.md`
8. **Test**: verificar acceso portal user, acceso por token, y responsive

## GOTCHAS

1. **Olvidar `super()` en `_prepare_home_portal_values()`** — borras los counters de todos los demás módulos. Siempre `values = super()...` primero.
2. **`search_count` sin `has_access('read')`** — el portal user sin permiso revienta con AccessError en el dashboard. Guardar la query.
3. **Detail page con `auth="user"` en vez de `auth="public"`** — rompe el share por access_token (link sin login). Las detail pages van `auth="public"` + validación de token.
4. **Comparar tokens con `==`** — vulnerable a timing attack. Usar `consteq()` siempre.
4b. **El token es una credencial completa — MEDIDO (v18/v19).** Con un token válido entra **quien
   sea**: un usuario de portal al que la record rule excluye, y también un anónimo sin identificar.
   Y el registro vuelve **sudoed** (`env.su = True`), así que a partir de ahí no queda barrera del
   ORM: lo que el controlador pinte, se pinta. Matriz medida en `references/portal-security.md`.
   Corolario: el token **no existe hasta que se llama a `_portal_ensure_token()`**.
5. **Query portal sin domain por partner** — el usuario ve registros de otros. Todo `search`/`search_count` en portal filtra por `partner_id` del usuario.
6. **Duplicar templates base en vez de `inherit_id` + `xpath`** — la siguiente actualización de Odoo rompe la página. Heredar, nunca copiar.
7. **Counter no-lazy** — computar sin `if 'mi_counter' in counters` penaliza cada carga de `/my/home`; el frontend los pide por lotes.

## Delegates to

> Routing: diseño/generación de código = **opus** (principal). El scaffolding delega en `odoo-module-creator` y la conexión Odoo (cuando se prueba en vivo) sigue el contrato compartido `references/shared/odoo-connection.md`.

- **`/odoo-module-creator`** → Scaffolding del módulo completo
- **`/odoo-frontend`** → HTML/Bootstrap para contenido web (no portal controllers)
- **`/odoo-version-api-reference`** → Cambios de API entre versiones
- **`/quality-gate`** → Validar código antes de merge

## Consumed by

| Skill | Suite | Para qué |
|-------|-------|----------|
| `odoo-owl-component-builder` | developer | Componentes OWL embebidos en páginas de portal |
| `odoo-website-controller` | developer | Rutas y controllers frontend que solapan con /my/* |
| `odoo-ecommerce-developer` | developer | Páginas de cuenta/portal en flujos de e-commerce |
