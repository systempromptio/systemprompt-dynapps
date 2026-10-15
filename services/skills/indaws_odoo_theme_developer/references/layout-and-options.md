# Layout & Options — website.layout, header/footer, customize_show

Cómo un tema modifica el chrome del sitio (header/footer) y expone opciones activables.
Verificado contra `$ODOO_SRC/odoo18/addons/website/`.

## 1. `website.layout`

- Definición → `website/views/website_templates.xml:39`, `layout`:
  ```xml
  <template id="layout" name="Main layout" inherit_id="portal.frontend_layout">
  ```
  No es una plantilla raíz: **hereda de `portal.frontend_layout`** y opera con xpath.
- Atributos sobre el `<header>` → `website/views/website_templates.xml:169-171`, `layout` (`data-name="Header"`, aria labels).
- Atributos sobre el `<footer>` → `website/views/website_templates.xml:195-198`, `layout` (`t-attf-class` con `o_colored_level o_cc`,
  `t-cache`).
- Inyección al final del `<body>` → `website/views/website_templates.xml:314`, `custom_code_layout` (`website.custom_code_footer`).

Un controller frontend renderiza páginas con `t-call="website.layout"` para heredar este chrome
(ver skill `odoo-website-controller`). Para alterarlo, **hereda** `website.layout` con `inherit_id`
y xpath; nunca lo redefinas.

## 2. Header / footer templates como opciones exclusivas

Los headers/footers son plantillas que heredan `website.layout` y se activan de una en una.
Las listas viven en el modelo `theme.utils`:

- `_header_templates` → `website/models/theme_models.py:222-235`, `Theme._header_templates`:
  ```python
  _header_templates = [
      'website.template_header_hamburger',
      'website.template_header_vertical',
      'website.template_header_sidebar',
      'website.template_header_boxed',
      'website.template_header_stretch',
      'website.template_header_search',
      'website.template_header_sales_one', ... 'website.template_header_sales_four',
      'website.template_header_default',   # default → SIEMPRE el último
  ]
  ```
- `_footer_templates` → `website/models/theme_models.py:236-246`, `class Theme` (idéntico patrón, `website.footer_custom` último).

Ejemplos de las plantillas:
- `template_header_default` → `website/views/website_templates.xml:493`, `template_header_default` (`inherit_id="website.layout" active="True"`).
- `footer_custom` (default) → `website/views/website_templates.xml:1660`, `footer_custom` (`active="True"`).
- `template_footer_descriptive` → `website/views/website_templates.xml:1716`, `template_footer_descriptive` (`active="False"`).

### Activación exclusiva
`enable_view`/`disable_view` → `website/models/theme_models.py:342-353`, `Theme.enable_view`:
```python
@api.model
def enable_view(self, xml_id):
    if xml_id in self._header_templates:
        for view in self._header_templates:
            self.disable_view(view)        # apaga todos los headers
    elif xml_id in self._footer_templates:
        for view in self._footer_templates:
            self.disable_view(view)        # apaga todos los footers
    self._toggle_view(xml_id, True)        # enciende el elegido
```
`_toggle_view` (`website/models/theme_models.py:312-331`, `Theme._toggle_view`) escribe `active` sobre la vista específica del website.

→ Un tema con header/footer propios los AÑADE a estas listas (heredando `theme.utils`) y los
activa en su `_<modulo>_post_copy`:
```python
class ThemeUtils(models.AbstractModel):
    _inherit = 'theme.utils'
    _header_templates = ['theme_acme.template_header_acme'] + models.AbstractModel._header_templates
    def _theme_acme_post_copy(self, mod):
        self.enable_view('theme_acme.template_header_acme')
```
(Hook `_post_copy` → `website/models/theme_models.py:248-255`, `Theme._post_copy`; invocado en `_theme_load` solo con
`apply_new_theme` → `website/models/ir_module_module.py:227-248`, `IrModuleModule._theme_load`.)

## 3. Opciones toggleables: `customize_show`

`customize_show="True"` convierte una vista heredada en una **opción** que el usuario activa/
desactiva desde el editor (panel "Customize"), en lugar de aplicarse siempre.

- Campo en `ir.ui.view` → `portal/models/ir_ui_view.py:10`, `View.customize_show`:
  ```python
  customize_show = fields.Boolean("Show As Optional Inherit", default=False)
  ```
- Campo en `theme.ir.ui.view` → `website/models/theme_models.py:73`, `ThemeView.customize_show`, copiado a la vista real en
  `_convert_to_base_model` → `website/models/theme_models.py:103`, `ThemeView._convert_to_base_model` (`'customize_show': self.customize_show`).

Ejemplos reales de uso:
- `website_sale/views/templates.xml:1547`, `alternative_products`:
  ```xml
  <template id="product_custom_text" inherit_id="website_sale.product"
            customize_show="True" active="True" name="Terms and Conditions" priority="21">
  ```
- `portal/views/portal_templates.xml:273`, `portal_docs_entry_layout`:
  ```xml
  <template id="portal_docs_entry_layout" inherit_id="portal.portal_docs_entry"
            active="True" customize_show="True" name="Use Pictograms">
  ```

### Patrón para una opción de tema
```xml
<template id="opt_acme_rounded_cards" inherit_id="website.layout"
          customize_show="True" active="False" name="ACME: Rounded Cards">
    <xpath expr="//main" position="attributes">
        <attribute name="class" add="o_acme_rounded" separator=" "/>
    </xpath>
</template>
```
- `customize_show="True"` → aparece como opción en el editor.
- `active="False"` → desactivada por defecto; el usuario la enciende.
- Sin `customize_show`, una vista heredada se aplica siempre y NO aparece como opción.

## 4. Resumen del flujo de un tema con layout propio

1. SCSS de variables → `web._assets_primary_variables` (paleta + fuentes + preset).
2. Reglas de estilo → `web.assets_frontend`.
3. Plantillas header/footer heredando `website.layout`, registradas en `_header_templates`/
   `_footer_templates` (heredando `theme.utils`).
4. Opciones extra como vistas con `customize_show="True"`.
5. `_<modulo>_post_copy` activa el header/footer/paleta por defecto al aplicar el tema.

## 5. Notas de versión

- `theme.utils`, `_header_templates`, `customize_show` y `website.layout` existen y siguen el
  mismo patrón en v17/v18/v19. Líneas concretas pueden variar:
  - v18 `theme.utils` → `website/models/theme_models.py:217-220`, `class Theme`; `customize_show` campo → `:72-73`.
  - v17 `theme.utils` → `website/models/theme_models.py:217`, `class Theme`; `_header_templates` → `:221`.
  - v19 `theme.utils` → `website/models/theme_models.py:214-219`, `ThemePage._convert_to_base_model`; `customize_show` → `:72`.
- Para diffs finos de las plantillas de header/footer entre versiones, delega en
  `odoo-version-api-reference` / `odoo-module-upgrader`.
