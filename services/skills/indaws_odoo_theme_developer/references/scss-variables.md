# SCSS Variables — primarias, paletas, fonts, orden de carga

Variables `$o-*` que un tema define para cambiar el look. Verificado contra
`$ODOO_SRC/odoo18/addons/`. Diffs v17/v19 al final de cada sección.

## 1. Orden de carga (CRÍTICO)

```
web._assets_primary_variables      ← aquí pones TUS variables $o-*
        ↓
bootstrap_overridden.scss           ← Odoo mapea $o-* a variables Bootstrap ($primary, etc.)
        ↓
web._assets_secondary_variables     ← deriva valores de las primarias (no editar)
        ↓
web._assets_bootstrap               ← Bootstrap compila usando lo anterior
        ↓
web.assets_frontend (reglas CSS)    ← aquí pones tus REGLAS, que ya ven todo resuelto
```

> ⚠️ **LAS RUTAS DE ESTE FICHERO SON DE v17/v18. En v19 el módulo `web_editor` NO EXISTE.**
> Verificado 2026-08-07: no hay `addons/web_editor` en v19; su contenido se mudó a `html_editor`, y
> uno de los ficheros **además se renombró**:
>
> | Fichero | v17 / v18 | v19 |
> |---|---|---|
> | `secondary_variables.scss` | `web_editor/static/src/scss/` | **`html_editor/static/src/scss/`** |
> | variables del editor | `web_editor/static/src/scss/web_editor.variables.scss` | **`html_editor/static/src/scss/html_editor.variables.scss`** |
> | (nuevo en v19) | — | `html_builder/static/src/scss/builder.variables.scss` |
>
> El **contenido** de `secondary_variables.scss` es idéntico en v18 y v19 (mismos `map-merge` en las
> líneas 10 y 13), así que los números de línea de esta referencia siguen valiendo; lo que cambia es
> la ruta. Las líneas citadas de `web_editor.variables.scss` **no se han re-verificado** contra el
> `html_editor.variables.scss` de v19: trátalas como v18 y comprueba antes de citar en v19.

> **Y `primary_variables.scss` se desplaza 6 líneas en v19, aunque el fichero no cambie de sitio**
> (medido 2026-08-17). Es el fichero que esta referencia cita más, así que conviene tenerlo delante:
>
> | Ancla | v17 | v18 | v19 |
> |---|---|---|---|
> | `$o-base-color-palette: map-merge(...)` | `:6` | `:6` | `:6` |
> | `$o-color-palettes: map-merge($o-color-palettes,` | **`:55`** | **`:55`** | **`:61`** |
> | `$o-website-values-palettes: (` | — | `:2204` | `:2212` |
>
> O sea que **las citas por encima de `:6` valen para las tres versiones y las de debajo no**: v19
> inserta contenido entre medias. Si copias un número de esta página a un proyecto v19, comprueba el
> ancla —`grep -n '^\$o-color-palettes: map-merge'`— en vez de fiarte del número.

- `web._assets_helpers` incluye primary y luego secondary, en ese orden:
  `web/__manifest__.py:393-394`.
- `secondary_variables.scss` deriva de las primarias: hace
  `map-merge($o-base-color-palette, $o-color-palette)` etc.
  (`web_editor/static/src/scss/secondary_variables.scss:23`, `$o-color-palette`).
- Consecuencia: si defines una variable primaria en el bundle equivocado (frontend), las
  derivadas ya se calcularon con el valor por defecto. **Variables → primary; reglas → frontend.**

## 2. Color palettes

Fichero base: `web_editor/static/src/scss/web_editor.variables.scss`.

- `$o-color-palettes` (mapa de paletas base) → `:286-301`. Cada paleta define `o-color-1..5`:
  ```scss
  $o-color-palettes: (
      'base-1': (
          'o-color-1': $o-enterprise-color,
          'o-color-2': #2D3142,
          'o-color-3': #F3F2F2,
          'o-color-4': #FFFFFF,
          'o-color-5': #111827,
      ),
      ...
  ) !default;
  ```
- `$o-color-palette-name: 'base-1'` → `:302` (paleta activa por defecto).
- `$o-theme-color-palettes` (colores semánticos alpha..epsilon, legacy) → `:305-318`.
- `$o-base-gray-color-palette` (grises 100..900 + white/black) → `:327-339`;
  `$o-transparent-grays` (`black-15`, `white-50`, ...) → `:340-349`.

### Función `o-make-palette`
Definición → `web_editor/static/src/scss/web_editor.variables.scss:452-480`, `o-make-palette`. Genera una paleta de 5 colores
a partir de un primario (y opcional secundario + overrides), calculando contraste automáticamente:

```scss
@function o-make-palette($-primary, $-secondary: null, $-overrides-map: null) { ... }
```

### Registrar una paleta nueva (patrón merge — NO reasignar)
Verificado en `website/static/src/scss/primary_variables.scss:55-56`, `$o-color-palettes` (**v17/v18**; **v19 `:61-62`**):

```scss
$o-color-palettes: map-merge($o-color-palettes, (
    'acme-1': o-make-palette(#1A73E8, #34A853, (
        'menu': 4,
        'footer': 4,
        'copyright-custom': 'black-15',
    )),
    'acme-2': (
        'o-color-1': #B99932,
        'o-color-2': #DED1C1,
        'o-color-3': #F5F5F5,
        'o-color-4': #FFFFFF,
        'o-color-5': #373737,
        'menu': 5,
        'copyright': 4,
    ),
));
```
Las claves `'menu'`, `'footer'`, `'copyright'` apuntan a un nº de color combination (1..5).
Las claves `'o-ccN-headings'`, `'o-ccN-link'`, `'o-ccN-btn-primary'`, etc. afinan cada combo
(ejemplos reales en `website/static/src/scss/primary_variables.scss:184-208`, `$o-color-palettes`; también `:325-347`).

### Color combinations (`o_cc1`..`o_cc5`)
`$o-base-color-combination` y `$o-color-combinations-presets` →
`web_editor/static/src/scss/web_editor.variables.scss:354-395`, `$o-base-color-combination`. En las plantillas usa clases `o_cc o_ccN` y, en SCSS,
`o-color('o-color-1')` / `o-apply-colors(N)` (mixin en `:407-417`). Nunca hex literal.

## 3. Fuentes (fonts)

- `$o-system-fonts` (stack del sistema) → `web/static/src/scss/primary_variables.scss:30`, `$o-system-fonts`.
- `$o-font-aliases-to-keys` (mapea `headings`/`h2`/`navbar`/`buttons` a claves) →
  `website/static/src/scss/primary_variables.scss:2189-2203`, `$o-font-aliases-to-keys` (v18).
- `$o-base-theme-font-configs` (incluye `SYSTEM_FONTS`) → `website/static/src/scss/primary_variables.scss:2256-2263`, `$o-base-theme-font-configs`.
- `$o-theme-font-configs` (catálogo de fuentes: Roboto, Open Sans, Inter, ...) →
  `website/static/src/scss/primary_variables.scss:2264-2297`, `$o-theme-font-configs`. Cada entrada:
  ```scss
  'Roboto': (
      'family': ('Roboto', sans-serif),
      'url': 'Roboto:300,300i,400,400i,700,700i',   // se importa de Google Fonts
  ),
  ```
- Import efectivo de las fuentes (Google Fonts o local) →
  `website/static/src/scss/website.wysiwyg.fonts.scss:1-13` (itera `$o-theme-font-configs`).
- Derivación headings↔body en secondary → `website/static/src/scss/secondary_variables.scss:284-298`, `$o-theme-font`:
  ```scss
  $o-theme-font: o-get-font-info('base') or ...;
  $o-theme-headings-font: o-get-font-info('headings') or $o-theme-font !default;
  $o-theme-navbar-font:   o-get-font-info('navbar')   or $o-theme-font !default;
  $o-theme-buttons-font:  o-get-font-info('buttons')  or $o-theme-font !default;
  ```
  → Si no defines `headings`, hereda de `base`. El merge base+tema:
  `website/static/src/scss/secondary_variables.scss:225`, `$o-theme-font-configs`.

### Añadir/seleccionar fuente en un tema
```scss
// 1) registrar (merge, no reasignar) si es una fuente nueva
$o-theme-font-configs: map-merge($o-theme-font-configs, (
    'Poppins': ('family': ('Poppins', sans-serif), 'url': 'Poppins:400,500,700'),
));
// 2) seleccionarla como base/headings vía el preset (ver §4)
```

## 4. Presets (`$o-website-values-palettes` / user_values)

- `$o-website-values-palettes` (preset por defecto: fuentes + paddings/radios de botón) →
  `website/static/src/scss/primary_variables.scss:2204-2224`, `$o-website-values-palettes` (v18):
  ```scss
  $o-website-values-palettes: (
      (
          'headings-font': 'Inter Tight',
          'font': 'Inter',
          'btn-padding-x': 1rem,
          'btn-padding-y-lg': 1rem,
          'btn-border-radius-lg': 2rem,
          ...
      ),
  ) !default;
  ```
- `user_values.scss` (hook que el EDITOR reescribe en runtime; parte de
  `web._assets_primary_variables`) → `website/static/src/scss/options/user_values.scss:5-7`, `$o-user-website-values`.
  Contiene `$o-user-website-values: map-merge(...(// -- hook --))`. NO lo edites a mano para
  un tema: lo gestiona el editor (se reescribe). Pon los valores del tema en
  `$o-website-values-palettes` dentro de tu `primary_variables.scss`.

Un tema fija su preset así:
```scss
$o-website-values-palettes: (
    map-merge(nth($o-website-values-palettes, 1), (
        'font': 'Poppins',
        'headings-font': 'Poppins',
        'btn-border-radius': .25rem,
    )),
);
```

## 5. Diffs de versión

- **Bundle `web._assets_primary_variables` (website)**: existe en las 3 versiones, distinta
  línea de manifest: v17 `website/__manifest__.py:179`, v18 `:238`, v19 `:290`.
- **Preset Inter/Inter Tight**:
  - v17: `headings-font: null` por defecto (`v17 website/static/src/scss/primary_variables.scss:2034`, `$o-base-website-values-palette`); el preset Inter
    aún no está fijado igual (`$o-website-values-palettes` en `:2076`).
  - v18: `'headings-font': 'Inter Tight'` literal dentro de `$o-website-values-palettes`
    (`v18 website/static/src/scss/primary_variables.scss:2204-2224`, `$o-website-values-palettes`).
  - v19: extrae `$o-website-default-headings-font: 'Inter Tight'` (`v19 website/static/src/scss/primary_variables.scss:2211`, `$o-website-default-headings-font`)
    y lo referencia en **`:2218-2219`** (remedido 2026-08-17; esta nota decía `:2219-2220`, una
    línea tarde: `:2218` es `'default-headings-font'` y `:2219` `'headings-font'`, las dos apuntando
    a la misma variable). Añade la font `'Inter Tight'` al catálogo (`:2298-2300`, verificado).
- **`$o-color-palettes` / `o-make-palette`**: los NOMBRES son estables en las 3 versiones, pero el
  módulo que los define NO: `web_editor` en v17/v18, **`html_editor` en v19**. Decir "estables en
  web_editor en las 3 versiones" era falso para v19 (corregido 2026-08-07).
- Para cualquier otro diff SCSS entre versiones, delega en `odoo-version-api-reference`.
