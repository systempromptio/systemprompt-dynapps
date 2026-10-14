# Theme Module — estructura, manifest, assets

Cómo se monta un módulo `theme_*` en Odoo v17-v19. Todas las líneas verificadas contra
`$ODOO_SRC/odoo18/addons/`.

## 1. Estructura de carpetas

Un tema CE mínimo (`theme_default`) es muy pequeño porque el "look" lo aporta `website`
mediante SCSS. Un tema real añade su propio SCSS + plantillas de layout:

```
theme_acme/
├── __init__.py
├── __manifest__.py
├── data/                      # opcional: registros theme.* (menus, pages)
├── static/
│   ├── description/
│   │   ├── icon.png
│   │   ├── cover.png
│   │   └── theme_acme_screenshot.jpg
│   └── src/
│       ├── scss/
│       │   ├── primary_variables.scss   # variables $o-* (paletas, fuentes, presets)
│       │   └── theme.scss               # reglas de estilo (usan las variables ya resueltas)
│       └── ...
└── views/
    └── layout.xml             # opcional: header/footer templates, options customize_show
```

`theme_default` real: `theme_default/__manifest__.py:1-19`, `theme_default/data/generate_primary_template.xml`.
Sus únicos assets estáticos son las imágenes de descripción
(`theme_default/static/description/{icon.png,cover.png,*_screenshot.jpg}`).

## 2. `__manifest__.py`

Verificado en `theme_default/__manifest__.py:4-18`:

```python
{
    'name': 'ACME Theme',
    'description': 'ACME website theme',
    'category': 'Theme',                 # obligatorio para que aparezca como tema
    'sequence': 1000,
    'version': '1.0',
    'depends': ['website'],              # un tema SIEMPRE depende de website
    'data': [
        # 'data/...xml',                 # registros theme.* opcionales
    ],
    'images': [                          # screenshots para el selector de temas
        'static/description/cover.png',
        'static/description/theme_acme_screenshot.jpg',
    ],
    'assets': {                          # aquí van los SCSS (ver §3)
        # ...
    },
    'license': 'LGPL-3',
}
```

- `category: 'Theme'` es lo que lo clasifica como tema instalable desde el website.
- `depends: ['website']` da acceso a `web._assets_primary_variables` y `website.layout`.
- Las `images` se muestran en el selector visual de temas.

## 3. Declarar el SCSS del tema en los bundles

El tema NO carga su SCSS con `<template>` ni `<link>`; lo añade a los **bundles** desde la
clave `assets` del manifest. Dos destinos distintos:

```python
'assets': {
    # VARIABLES: van al bundle de variables primarias.
    'web._assets_primary_variables': [
        'theme_acme/static/src/scss/primary_variables.scss',
    ],
    # REGLAS DE ESTILO: consumen las variables ya resueltas → bundle frontend.
    'web.assets_frontend': [
        'theme_acme/static/src/scss/theme.scss',
    ],
},
```

Esto replica exactamente cómo `website` se engancha a esos bundles:
- `website/__manifest__.py:238-244` añade su `primary_variables.scss` + `user_values.scss` +
  los `user_*_palette.scss` a `web._assets_primary_variables`.
- `website/__manifest__.py:245-247` hace `('prepend', '.../secondary_variables.scss')` a
  `web._assets_secondary_variables`.
- `website/__manifest__.py:297-299` hace `('prepend', '.../bootstrap_overridden.scss')` a
  `web._assets_frontend_helpers`.

### Operaciones de orden en bundles
- `('prepend', 'path')` — inserta al inicio del bundle (antes que lo existente).
- `('append', 'path')` o `'path'` suelto — al final.
- `('before', 'ancla', 'nuevo')` / `('after', 'ancla', 'nuevo')` — relativo a un fichero.
- `('remove', 'path')` — quita un fichero del bundle.

Ejemplo `('after', ...)` real: `web/__manifest__.py:278`.

Definición de los bundles base (`web/__manifest__.py`):
- `web._assets_primary_variables` → `:378-381`
- `web._assets_secondary_variables` → `:382-384`
- `web._assets_helpers` (incluye primary+secondary) → `:393-394`
- `web.assets_frontend` → `:162`

## 4. Carga e instalación del tema

Al instalar/aplicar un tema, `website` copia sus registros `theme.*` a registros reales
**por cada website**:

- Modelos themificables (`website/models/ir_module_module.py:22-28`, `IrModuleModule._theme_model_names`):
  ```python
  _theme_model_names = OrderedDict([
      ('ir.ui.view', 'theme.ir.ui.view'),
      ('ir.asset', 'theme.ir.asset'),
      ('website.page', 'theme.website.page'),
      ('website.menu', 'theme.website.menu'),
      ('ir.attachment', 'theme.ir.attachment'),
  ])
  ```
  Solo estos modelos se copian; los demás no se "themifican".

- `_theme_load(website)` (`website/models/ir_module_module.py:227-248`, `IrModuleModule._theme_load`): por cada modelo llama
  `_update_records(...)`; y si el contexto trae `apply_new_theme`, ejecuta
  `theme.utils._post_copy(module)`.

- Hook por-tema: `theme.utils._post_copy` busca `_<nombre_modulo>_post_copy` y lo ejecuta
  (`website/models/theme_models.py:248-255`, `Theme._post_copy`). Ahí un tema activa su header/footer/paleta por defecto.

- `theme.utils` es `AbstractModel` con `_auto = False` (`website/models/theme_models.py:217-220`, `class Theme`): no tiene
  tabla; sus métodos se invocan con `with_context(website_id=...)`.

## 5. `__init__.py`

Si el tema solo aporta SCSS/XML, `__init__.py` puede estar vacío. Si define un
`_<modulo>_post_copy`, ese método va en un `models/theme_utils.py` que hereda `theme.utils`:

```python
from odoo import models

class ThemeUtils(models.AbstractModel):
    _inherit = 'theme.utils'

    def _theme_acme_post_copy(self, mod):
        self.enable_view('theme_acme.template_header_acme')
        self.enable_view('theme_acme.template_footer_acme')
```

(Patrón derivado de `website/models/theme_models.py:248-255`, `Theme._post_copy` + `enable_view` en `:342-353`.)
