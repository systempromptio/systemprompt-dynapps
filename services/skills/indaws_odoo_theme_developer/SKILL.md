---
name: indaws-odoo-theme-developer
version: 0.1.14
effort: high
model: opus
description: >-
  Desarrolla módulos theme_* Odoo (v17-v19): variables SCSS ($o-color-palettes,
  o-make-palette, fonts), bundles primary/secondary, theme.utils, customize_show.
  NOT snippets (→ odoo-website-snippets); NOT JS público (→ odoo-website-frontend-js).
  Triggers: "crear un tema Odoo", "módulo theme_*", "$o-color-palettes",
  "paleta de colores website", "fuentes del tema", "customize_show",
  "override de estilo se pierde al actualizar", "Odoo theme module",
  "theme SCSS variables".
allowed-tools: [Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion, EnterPlanMode, Agent]
---

# Odoo Theme Developer

Diseña módulos `theme_*` que controlan la apariencia del frontend website en Odoo v17-v19.
Un tema NO aporta contenido: aporta **variables SCSS** (colores, fuentes, presets) inyectadas
en los bundles de variables primarias y **plantillas QWeb** de layout (header/footer) activables
como opciones — respetando el orden de carga de Odoo y sin que se pierdan al actualizar.

Refs compartidas: `references/shared/coding-standards.md`, `references/shared/odoo-connection.md`

## Workflow

1. **Definir el tema** [STOP — AskUserQuestion] — ¿desde cero o ajustes sobre sitio existente?
   Paleta(s), fuentes (headings + body), estilo header/footer, versión (v17/v18/v19). Si aporta
   snippets/contenido, eso NO es este skill (delegar).
2. **Estructura** — Leer [`references/theme-module.md`](references/theme-module.md): `__manifest__.py`
   (`category: 'Theme'`, `depends: ['website']`), carpetas, declaración de SCSS en bundles, `theme.utils`/`_theme_load`.
3. **Variables SCSS** [el grueso] — Leer [`references/scss-variables.md`](references/scss-variables.md):
   `$o-*`, `$o-color-palettes` + `o-make-palette`, paletas, `$o-theme-font-configs` y headings/body,
   presets `$o-website-values-palettes`/`user_values.scss`, ORDEN DE CARGA primary→bootstrap_overridden→secondary.
4. **Layout y opciones** — Leer [`references/layout-and-options.md`](references/layout-and-options.md):
   `website.layout`, `template_header_*`/`template_footer_*`, `theme.utils` (`_header_templates`,
   `enable_view`, `_<modulo>_post_copy`), `customize_show`.
5. **Revisión** — Leer [`references/gotchas.md`](references/gotchas.md) ANTES de cerrar.
6. **Probar** — instalar `-i theme_xxx`, verificar paleta/fuentes, cambiar header/footer en el editor,
   y comprobar que un `--update` no revierte personalizaciones.

## GOTCHAS

1. **Las variables primarias van SOLO en `web._assets_primary_variables`** — Definir `$o-*`
   en `web.assets_frontend` no funciona: para entonces Bootstrap y los helpers ya consumieron
   las variables. El SCSS de variables se añade a `web._assets_primary_variables`. Ver el patrón
   de `website` en `website/__manifest__.py:238-244`.
2. **Orden de carga es sagrado: primary → bootstrap_overridden → secondary** — `web._assets_helpers`
   incluye primary y secondary en ese orden (`web/__manifest__.py:393-394`); `secondary_variables`
   deriva de las primarias (`web_editor/static/src/scss/secondary_variables.scss:23`, `$o-color-palette` hace `map-merge($o-base-color-palette, $o-color-palette)`; módulo **`web_editor` en v17/v18, `html_editor` en v19** — ver `references/scss-variables.md`). Si rompes el orden, las derivadas usan valores viejos.
3. **`o-make-palette($primary, $secondary, $overrides)` genera la paleta** — No definas las 5
   `o-color-N` a mano salvo necesidad; pasa el color primario y deja que calcule contraste.
   Definición y algoritmo de contraste en `web_editor/static/src/scss/web_editor.variables.scss:452-480`, `o-make-palette` (v17/v18; en **v19** es `html_editor/static/src/scss/html_editor.variables.scss` y esas líneas NO se han re-verificado).
4. **Las paletas se registran con `map-merge($o-color-palettes, (...))`, NO reasignando** —
   Si haces `$o-color-palettes: (...)` borras todas las demás. El patrón correcto (merge) está
   en `website/static/src/scss/primary_variables.scss:55-56`, `$o-color-palettes` (**v17/v18**; en **v19** es `:61-62` —
   el fichero se desplaza 6 líneas, ver `references/scss-variables.md`).
5. **Fuentes: headings derivan de body si no se setean** — En `secondary_variables.scss`
   `$o-theme-headings-font: o-get-font-info('headings') or $o-theme-font` (`website/static/src/scss/secondary_variables.scss:288`, `$o-theme-headings-font`).
   Para una fuente nueva hay que añadirla a `$o-theme-font-configs` (`website/static/src/scss/primary_variables.scss:2264`, `$o-theme-font-configs`)
   con su `url` de Google Fonts; si no, no se importa (`website/static/src/scss/website.wysiwyg.fonts.scss:1-13`).
6. **El preset por defecto (Inter/Inter Tight) está en `$o-website-values-palettes`** —
   Ahí van `font`, `headings-font`, paddings de botón, radios. v18 `website/static/src/scss/primary_variables.scss:2204-2224`, `$o-website-values-palettes`;
   v19 lo separa en `$o-website-default-headings-font` (`v19 website/static/src/scss/primary_variables.scss:2211`, `$o-website-default-headings-font`).
7. **`customize_show="True"` hace la vista una OPCIÓN toggleable** — Es un campo de `ir.ui.view`
   (`portal/models/ir_ui_view.py:10`, `View.customize_show`) y de `theme.ir.ui.view` (`website/models/theme_models.py:73`),
   copiado al instalar (`website/models/theme_models.py:103`, `ThemeView._convert_to_base_model`). Sin él, una vista heredada se aplica siempre y no
   aparece como opción en el editor.
8. **Cambiar header/footer es exclusivo: activar uno desactiva el resto** — `enable_view` recorre
   `_header_templates`/`_footer_templates` y desactiva todos antes de activar el elegido
   (`website/models/theme_models.py:342-353`, `Theme.enable_view`). El default va el ÚLTIMO de la lista (`website/models/theme_models.py:222-235`, `Theme._header_templates`).
9. **Override que se pierde al actualizar: edita variables/SCSS, no escribas en `ir.ui.view` activo** —
   Las personalizaciones runtime del editor se guardan en `user_values.scss`/`user_color_palette.scss`
   vía `make_scss_customization` (`website/models/assets.py:16-42`, `Assets.make_scss_customization`) y `_reset_default_config` las
   resetea al re-aplicar tema (`website/models/theme_models.py:257-291`, `class Theme`). Para cambios permanentes del tema, ponlos
   en el SCSS del módulo (van al bundle), no como dato runtime que un re-apply borra.
10. **`theme.utils` es `AbstractModel` con `_auto = False`** — No es una tabla; sus métodos
    (`enable_view`, `_post_copy`) se llaman con `with_context(website_id=...)`. Verificado v18
    `website/models/theme_models.py:217-220`, `Theme`. El hook por-tema es `_<nombre_modulo>_post_copy`
    (`website/models/theme_models.py:248-255`, `Theme._post_copy`), invocado en `_theme_load` solo con `apply_new_theme`
    (`website/models/ir_module_module.py:227-248`, `IrModuleModule._theme_load`).
11. **`web.assets_frontend` es para reglas CSS/JS del frontend, NO para variables** — Las reglas
    de estilo del tema (que consumen las variables ya resueltas) van a `web.assets_frontend`
    (`web/__manifest__.py:162`); las variables a `_assets_primary_variables`. Mezclarlos es el error
    nº1 (ver gotcha 1).
12. **Los datos del tema se cargan por `_theme_model_names`** — Solo `ir.ui.view`, `ir.asset`,
    `website.page`, `website.menu`, `ir.attachment` (como `theme.*`) se copian por website en
    `_theme_load` (`website/models/ir_module_module.py:227-248`, `IrModuleModule._theme_load`). Otros modelos no se "themifican" automáticamente.
13. **`website.layout` hereda de `portal.frontend_layout`** — No es una plantilla raíz; xpath sobre
    `//header` y `//footer` (`website/views/website_templates.xml:39`, `layout`; `website/views/website_templates.xml:259` y `:281`, `plausible_script`). Para alterar el
    layout, hereda `website.layout` con `inherit_id`, no lo redefinas.

## NEVER

- Poner variables `$o-*` en `web.assets_frontend` (no se resuelven a tiempo) — van a `_assets_primary_variables`
- Reasignar `$o-color-palettes` / `$o-theme-font-configs` (borra las existentes) — usar `map-merge`
- Hardcodear hex en plantillas/CSS en vez de usar las color combinations (`o_cc1..5`) y `o-color()`
- Guardar personalizaciones del tema como registros `ir.ui.view` activos en vez de SCSS del módulo (se pierden al re-aplicar)
- Crear vistas de header/footer sin meterlas en `_header_templates`/`_footer_templates` si deben ser opciones exclusivas
- Olvidar `customize_show="True"` en una vista que debe ser opción activable en el editor
- Importar una fuente con `@import url(...)` suelto en vez de declararla en `$o-theme-font-configs`
- Aportar contenido HTML / snippets desde un theme module sin separar responsabilidades (delegar)

## ALWAYS

- Leer la reference de cada fase antes de generar SCSS/XML
- Confirmar paleta, fuentes y estilo de header/footer con el usuario (Step 1)
- `category: 'Theme'` y `depends: ['website']` en el `__manifest__.py`
- Definir variables en `_assets_primary_variables` y reglas en `assets_frontend`
- Registrar paletas/fuentes con `map-merge` y generar paletas con `o-make-palette`
- Respetar el orden primary → bootstrap_overridden → secondary
- Usar `customize_show="True"` + listas `_header_templates`/`_footer_templates` para opciones
- Poner cambios permanentes en SCSS del módulo (sobreviven al update), no como dato runtime
- Usar las color combinations (`o_cc*`) y `o-color()` en lugar de colores literales
- **Provenance marker (ALWAYS):** every generated file (except `.csv`/`.json` — no comment syntax) ends with `Made with odoo-theme-developer · v<version del frontmatter de esta skill>`, wrapped in the file-type comment (SCSS `//`, XML `<!-- -->`), idempotent. Full rule: `references/shared/skill-provenance-marker.md`.

## Delegates to

| Skill | Suite | Cuándo |
|-------|-------|--------|
| odoo-frontend | common | Contenido HTML/Bootstrap del editor dentro de páginas (no el chrome del tema) |
| odoo-website-snippets | developer | Snippets reutilizables drag-and-drop con sus options |
| odoo-website-frontend-js | developer | JS público / widgets / interacciones del frontend |
| odoo-website-controller | developer | Rutas y render de páginas que el tema viste |
| odoo-version-api-reference | common | Diffs SCSS/variables/bundles entre v17/v18/v19 |
| odoo-module-creator | developer | Scaffolding base del módulo `theme_*` |

## Consumed by

- `odoo-module-creator` — cuando un módulo nuevo es un tema website
- `odoo-module-upgrader` — al migrar variables/bundles SCSS de un tema entre versiones

## REFERENCES

| Fichero | Contenido |
|---------|-----------|
| [`references/theme-module.md`](references/theme-module.md) | Estructura del módulo `theme_*`, `__manifest__.py`, carpetas `static/src/scss`, declaración de SCSS en bundles, `theme.utils`/`_theme_load`, `_theme_model_names` |
| [`references/scss-variables.md`](references/scss-variables.md) | Variables `$o-*`, `$o-color-palettes` + `o-make-palette`, paletas color/gris/tema, fuentes `$o-theme-font-configs`, presets `$o-website-values-palettes`, orden de carga |
| [`references/layout-and-options.md`](references/layout-and-options.md) | `website.layout`, `template_header_*`/`template_footer_*`, modelo `theme.utils` (`enable_view`, `_post_copy`), `customize_show` para opciones |
| [`references/gotchas.md`](references/gotchas.md) | 13 gotchas con file:line: orden de carga, customize_show, override perdido en update, fuentes, bundle equivocado |
