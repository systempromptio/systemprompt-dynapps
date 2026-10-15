# Gotchas — Odoo theme development

Todas con file:line verificado contra `$ODOO_SRC/odoo18/addons/` (notas de versión donde aplica).

## Bundles y orden de carga

1. **Variables `$o-*` SOLO en `web._assets_primary_variables`.** En `web.assets_frontend`
   (`web/__manifest__.py:162`) ya es tarde: Bootstrap y los helpers consumieron las variables.
   Patrón correcto: `website/__manifest__.py:238-244`.

2. **Orden sagrado primary → bootstrap_overridden → secondary.** `web._assets_helpers` incluye
   primary y luego secondary (`web/__manifest__.py:393-394`); `secondary_variables.scss` deriva de
   las primarias vía `map-merge` (`web_editor/static/src/scss/secondary_variables.scss:23`, `$o-color-palette` en v17/v18; **`html_editor/…` en v19**, mismo contenido).
   Romper el orden = derivadas con valores viejos.

3. **`web.assets_frontend` es para reglas, no variables.** Las reglas CSS del tema (que usan las
   variables ya resueltas) van a `web.assets_frontend` (`web/__manifest__.py:162`); las variables a
   `_assets_primary_variables`. Mezclarlos es el error nº1.

4. **`('prepend', ...)` vs orden por defecto.** Añadir un fichero suelto lo pone al final del bundle;
   para inyectar variables ANTES de las de `website`, usa `('prepend', ...)`. Sintaxis real de
   prepend/before/after: `website/__manifest__.py:245-247,297-299` y `web/__manifest__.py:278`.

## Paletas y colores

5. **`map-merge`, nunca reasignar `$o-color-palettes`.** `$o-color-palettes: (...)` borra todas las
   demás paletas. El patrón merge correcto: `website/static/src/scss/primary_variables.scss:55-56`, `$o-color-palettes`
   (**v17/v18**; **v19 `:61-62`**).

6. **`o-make-palette` calcula contraste; no pelees los 5 colores a mano.** Pasa primario (+secundario
   +overrides). Algoritmo: `web_editor/static/src/scss/web_editor.variables.scss:452-480`, `o-make-palette` (v17/v18; en v19 el fichero es `html_editor/static/src/scss/html_editor.variables.scss`).

7. **Hex literal en plantillas/CSS = se ignora el sistema de color combinations.** Usa clases
   `o_cc o_ccN` en QWeb y `o-color('o-color-1')` / `o-apply-colors(N)` en SCSS
   (`web_editor/static/src/scss/web_editor.variables.scss:354-417`, `$o-base-color-combination`). Las paletas mapean `'menu'`/`'footer'`/`'copyright'` a un
   nº de combo.

## Fuentes

8. **Fuente nueva: registrar en `$o-theme-font-configs`, no `@import` suelto.** El import efectivo
   itera ese mapa (`website/static/src/scss/website.wysiwyg.fonts.scss:1-13`). Catálogo:
   `website/static/src/scss/primary_variables.scss:2264-2297`, `$o-theme-font-configs`. Sin entrada con su `url`, la fuente
   no se descarga.

9. **`headings` hereda de `base` si no la defines.** `$o-theme-headings-font: o-get-font-info('headings')
   or $o-theme-font` (`website/static/src/scss/secondary_variables.scss:288`, `$o-theme-headings-font`). Igual para `navbar`/
   `buttons` (`:296-298`). Selecciona la fuente vía el preset `$o-website-values-palettes`
   (`website/static/src/scss/primary_variables.scss:2204-2224`, `$o-website-values-palettes`), no a mano en secondary.

## Override perdido en update

10. **Cambios permanentes del tema → SCSS del módulo; NO datos runtime.** Lo que el editor cambia se
    guarda en `user_values.scss`/`user_color_palette.scss` vía `make_scss_customization`
    (`website/models/assets.py:16-42`, `Assets.make_scss_customization`), y `_reset_default_config` (`website/models/theme_models.py:257-291`, `Theme._reset_default_config`) lo
    RESETEA al re-aplicar tema. Si pones tu personalización como dato runtime, un re-apply la borra.
    Ponla en el `primary_variables.scss` del módulo (va al bundle, sobrevive).

11. **No edites `options/user_values.scss` a mano.** Es el hook que el editor reescribe
    (`website/static/src/scss/options/user_values.scss:5-7`, `$o-user-website-values`, contiene `// -- hook --`). Tus valores
    de tema van en `$o-website-values-palettes` dentro de tu propio SCSS.

## Layout y opciones

12. **`customize_show="True"` o la vista se aplica siempre.** Sin él, una vista heredada no es una
    opción toggleable del editor (`portal/models/ir_ui_view.py:10`, `View.customize_show`; copiado en
    `website/models/theme_models.py:73`, `ThemeView.customize_show`, y `website/models/theme_models.py:103`, `ThemeView._convert_to_base_model`).

13. **Header/footer es exclusivo: el default va el ÚLTIMO de la lista.** `enable_view` apaga todos
    los de su grupo antes de encender uno (`website/models/theme_models.py:342-353`, `Theme.enable_view`); las listas tienen el default
    al final (`website/models/theme_models.py:222-235`, `Theme._header_templates`, y `website/models/theme_models.py:236-246`, `Theme._footer_templates`). Un header propio debe añadirse a `_header_templates`
    (heredando `theme.utils`) o no se gestionará como opción exclusiva.

14. **`theme.utils` es `AbstractModel` (`_auto=False`).** Sin tabla; métodos con
    `with_context(website_id=...)` (`website/models/theme_models.py:217-220`, `class Theme`). El hook por-tema es
    `_<modulo>_post_copy` (`website/models/theme_models.py:248-255`, `Theme._post_copy`), llamado en `_theme_load` solo con
    `apply_new_theme` (`website/models/ir_module_module.py:227-248`, `IrModuleModule._theme_load`).

15. **Solo 5 modelos se themifican.** `_theme_model_names`: `ir.ui.view`, `ir.asset`, `website.page`,
    `website.menu`, `ir.attachment` (`website/models/ir_module_module.py:22-28`, `IrModuleModule._theme_model_names`). Otros modelos no se copian por
    website automáticamente.

16. **`website.layout` hereda `portal.frontend_layout`; no lo redefinas.** Modifícalo con
    `inherit_id="website.layout"` + xpath (`website/views/website_templates.xml:39`, `layout`; `:259` y `:281`, `plausible_script`).
