# wkhtmltopdf Gotchas y Patrones Practicos

Problemas reales encontrados al generar PDFs con wkhtmltopdf en Odoo 14-19.
Basado en analisis del codigo fuente de Odoo 18 (`web/views/report_templates.xml`,
`ir_actions_report.py`, y los SCSS de `web/static/src/webclient/actions/reports/`).

## 0. Arquitectura del pipeline PDF en Odoo 18

```
QWeb template
  -> HTML completo (con header/footer/page)
    -> _prepare_html() separa:
       - <div class="header"> -> header.html (envuelto en minimal_layout)
       - <div class="footer"> -> footer.html (envuelto en minimal_layout)
       - <div class="article"> -> body_N.html (uno por documento)
    -> _build_wkhtmltopdf_args() genera argumentos CLI
    -> subprocess.Popen(['wkhtmltopdf', ...args, header.html, footer.html, body_0.html, ..., output.pdf])
```

### Ficheros temporales

Cada pieza (header, footer, cada body) se escribe en `/tmp/` como HTML.
Se escriben con `.encode()` (UTF-8 por defecto). El `<meta charset="utf-8"/>` esta
en `minimal_layout` que envuelve cada pieza.

### Argumentos tipicos de wkhtmltopdf

```
--disable-local-file-access --quiet --page-size A4
--margin-top 85 --dpi 90 --zoom 1.0667 --header-spacing 80
--margin-left 7 --margin-bottom 7 --margin-right 7
--orientation Portrait --javascript-delay 1000
--header-html /tmp/header.html --footer-html /tmp/footer.html
/tmp/body_0.html /tmp/body_1.html ...
/tmp/output.pdf
```

Nota: `--zoom = 96 / dpi` (compensacion DPI de wkhtmltopdf >= 0.12.2).

### El mecanismo subst() para multi-empresa

wkhtmltopdf inyecta `?page=X&topage=Y&webpage=Z` en cada pagina.
El `minimal_layout` incluye una funcion JS `subst()` que:
1. Lee `webpage` del query string
2. Selecciona el header/footer correcto (por indice de documento)
3. Reemplaza `{page}` y `{topage}` con los valores reales

Esto permite un solo header.html con multiples `<div>` hijos,
uno por empresa/documento.

## 1. CSS Variables (`var()`) NO funcionan

### El problema fundamental

wkhtmltopdf usa QtWebKit que **no soporta CSS custom properties** (`var(--name)`).
Bootstrap 5 usa `var()` extensivamente:

```css
/* Bootstrap 5 genera esto: */
.table {
    --bs-table-border-color: #212529;
    border-color: var(--bs-table-border-color);  /* wkhtmltopdf IGNORA esto */
}
.table > :not(caption) > * > * {
    border-bottom-width: 1px;
    /* border-color se hereda de .table -> var() -> NADA */
}
```

Resultado: **las tablas `.table` pierden todos los bordes en PDF**.

### Como lo resuelve Odoo internamente

Odoo tiene 3 archivos SCSS dedicados a parchear esto:

1. **`bootstrap_overridden_report.scss`** — Sobreescribe variables SCSS de Bootstrap
   ANTES de compilar (ej: `$border-color: $o-gray-900`)
2. **`bootstrap_review_report.scss`** — Reemplaza `var()` por valores estaticos
   DESPUES de compilar (ej: `.bg-primary { background-color: RGB(113,75,103) }`)
3. **`utilities_custom_report.scss`** — Regenera utilidades de borde con valores
   estaticos (ej: `.border { border: 1px solid $border-color }`)

### Solucion para reports custom

**Opcion A (RECOMENDADA): Inline `<style>` en el template QWeb**

El metodo MAS FIABLE. Declarar un bloque `<style>` directamente en el template:

```xml
<template id="report_saleorder_document">
    <t t-call="web.html_container">
        <style>
            /* Selectores con especificidad >= BS5 */
            .eur_lines > :not(caption) > * > .borderlr {
                border-left: 1px solid black !important;
                border-right: 1px solid black !important;
                border-bottom: none !important;
                border-top: none !important;
            }
        </style>
        <t t-foreach="docs" t-as="doc">
            <!-- ... -->
        </t>
    </t>
</template>
```

**¿Por que?**
- Se renderiza dentro del HTML que wkhtmltopdf recibe → SIEMPRE se aplica
- No depende del asset bundle (que puede fallar silenciosamente)
- `__manifest__.py` assets puede no crear `ir_asset` record → CSS nunca llega al PDF
- Fichero `.css` externo depende de compilacion SCSS → puede no incluirse

**CRITICO: Especificidad BS5.** Bootstrap 5 usa:
```css
.table > :not(caption) > * > * { border-bottom-width: 1px; ... }
```
Esto tiene mayor especificidad que `.borderlr` solo. Para ganar:
```css
/* INCORRECTO: pierde contra BS5 */
.borderlr { border-left: 1px solid black !important; }

/* CORRECTO: misma estructura que BS5, gana por cascade + !important */
.eur_lines > :not(caption) > * > .borderlr {
    border-left: 1px solid black !important;
    border-right: 1px solid black !important;
    border-bottom: none !important;
    border-top: none !important;
}
```

Patron: `.<tabla_clase> > :not(caption) > * > .<mi_clase>`.

**Opcion B: Inline styles directos (alternativa simple)**

Para reportes con pocas variantes, usar `style=""` o variables QWeb:

```xml
<t t-set="S_LR" t-value="'border-left:1px solid black;border-right:1px solid black;border-top:none;border-bottom:none'"/>
<td t-att-style="S_LR">valor</td>
```

**CUIDADO**: No mezclar `style=""` y `t-att-style=""` en el mismo elemento — solo uno se aplica.

> ### OBLIGATORIO: `table-borderless` en tablas con bordes inline
>
> BS5 `.table` genera `border-bottom-width: var(--border-width)` que wkhtmltopdf
> no resuelve. Esto crea **rectangulos de altura 0** en el PDF que Chrome PDFium
> renderiza como lineas finas horizontales (Firefox pdf.js los ignora).
>
> **Solucion**: Añadir `table-borderless` a todas las tablas del body:
> ```xml
> <table class="table table-sm table-borderless eur_lines" style="border-collapse:collapse;">
> ```
>
> `table-borderless` pone `border: 0 none` en todos td/th/tr, eliminando los
> rectangulos fantasma. Los inline styles (`t-att-style="S_LR"`) ganan por
> especificidad y mantienen los bordes custom.
>
> Sin `table-borderless`: 530 ghost lines en PDF.
> Con `table-borderless`: 116 (solo tablas del header fuera de article).
>
> Odoo estandar usa `table-borderless` en TODOS sus reports (sale, purchase, invoice).

> ### `<style>` DEBE estar dentro de `<div class="article">`
>
> `_prepare_html()` extrae SOLO el `<div class="article">` para pasarlo a
> wkhtmltopdf. Un `<style>` fuera del article se pierde en el body.
> Sin embargo, `<style>` dentro del body de un div puede no procesarse
> correctamente en QtWebKit. Preferir clases del asset bundle
> (`table-borderless`) sobre `<style>` inline para resets de borde.

**Opcion C: SCSS en el asset bundle (NO RECOMENDADA)**

Puede funcionar pero es fragil:
- `__manifest__.py` assets puede no crear `ir_asset` → CSS nunca se incluye
- Requiere limpiar cache tras cambios (`DELETE FROM ir_attachment WHERE url LIKE '%/web/assets/%'`)
- Selectores deben igualar especificidad de BS5

```scss
// Si funciona el asset, necesitas alta especificidad:
.eur_lines > :not(caption) > * > .borderlr {
    border-left: 1px solid black !important;
}
```

> ### WARNING: CSS `!important` en asset bundle SIEMPRE gana contra inline styles
>
> Este problema se descubrio tras pruebas extensas de trial-and-error. Si tienes CSS antiguo
> en el asset bundle con `!important`, tus inline styles perderan **aunque inline styles
> normalmente tengan prioridad sobre clases CSS**.
>
> **Por que ocurre esto:**
> CSS `!important` en stylesheet tiene mayor precedencia que inline style sin `!important`.
> La cascada es: `inline !important` > `stylesheet !important` > `inline` > `stylesheet`.
> No puedes poner `!important` en inline styles de HTML.
>
> **Escenario tipico:**
> ```css
> /* En tu .css antiguo del asset bundle: */
> .table.table-sm.myclass td { border: none !important; }
> ```
> ```xml
> <!-- En tu template: -->
> <td style="border:1px solid black">  <!-- PIERDE contra el !important del bundle -->
> ```
>
> **El problema de persistencia del cache:**
>
> Aunque elimines el modulo de `assets` en `__manifest__.py`, el CSS compilado
> puede seguir presente en el bundle cacheado (`web.report_assets_common.min.css`).
> Para eliminarlo completamente:
>
> 1. **Vaciar el contenido del fichero CSS** (no solo quitarlo del manifest):
>    Dejar el fichero `.css`/`.scss` en `static/` vacio o borrarlo del disco.
> 2. **Limpiar el cache de assets** (obligatorio):
>    ```sql
>    DELETE FROM ir_attachment WHERE url LIKE '%/web/assets/%';
>    ```
> 3. **Actualizar el modulo** y reiniciar Odoo.
>
> Si solo quitas la referencia del manifest sin vaciar el fichero, el CSS antiguo
> puede seguir siendo servido desde el bundle cacheado.
>
> **Orden recomendado al migrar de CSS-based a inline-based:**
>
> a. Vaciar el fichero `.css`/`.scss` antiguo de `static/` (o borrarlo del disco)
> b. Quitar la entrada del manifest si todavia esta
> c. `DELETE FROM ir_attachment WHERE url LIKE '%/web/assets/%';`
> d. Actualizar el modulo + reiniciar Odoo
> e. Usar inline styles via `t-att-style` con variables QWeb (`S_BF`, `S_LR`, `S_LRB`, `S_NONE`)
> f. Para utilidades (text-align, font-size, padding) que no conflicten con bordes,
>    un bloque `<style>` en el template sigue siendo valido
> g. NUNCA tener reglas de borde en asset bundle CSS Y en inline styles a la vez

**Opcion D: Clase `.o_ignore_layout_styling`**

Odoo 18 soporta esta clase en tablas para desactivar el tema de layout:
```xml
<table class="table table-sm o_ignore_layout_styling">
```
Esto evita que `.o_table_standard` (u otros temas) apliquen bordes.
No resuelve el problema de BS5 `.table` base — solo desactiva el tema Odoo.

### Otros CSS que NO funcionan en wkhtmltopdf

| Feature | Status | Workaround |
|---------|--------|------------|
| `var(--name)` | No funciona | Valores estaticos |
| `calc()` | No funciona en 0.12.6 | Valores pre-calculados |
| `@media queries` | No se procesan | Evitar `col-lg-*` en reports |
| Flexbox | Parcial | Añadir `-webkit-box-*` prefixes |
| `display: grid` | No funciona | Usar tablas o flexbox |
| `position: sticky` | No funciona | Usar `display: table-header-group` en `<thead>` |

## 2. Fuentes y encoding

### Fuente por defecto

Odoo 18 usa `Lato` (cargada desde `web/static/fonts/`). Soporta:
- Acentos (a, e, i, o, u con tildes)
- Euro (EUR), Libra (GBP), Yen (JPY)
- Mayoria de simbolos monetarios occidentales

Para simbolos no cubiertos (Rupia, Won), Odoo usa fallback:
`font-family: Lato, Lucida Grande, Verdana, sans-serif`

### Encoding UTF-8

El pipeline es 100% UTF-8:
- Templates XML: `encoding="utf-8"`
- HTML renderizado: `<meta charset="utf-8"/>`
- Ficheros temp: `.encode()` (UTF-8)
- subprocess: `encoding="utf-8"` en stdout/stderr

Si los acentos/simbolos salen corruptos (`Â â,¬`, `â‚¬`), las causas posibles son:
1. **CAUSA MAS COMUN: Falta `<div class="article">`** — `_prepare_html()` busca
   `<div class="article">` para envolver el body en `minimal_layout` (que tiene
   `<meta charset="utf-8"/>`). Si usas `<div class="page">` el body se pasa a
   wkhtmltopdf SIN charset → interpreta UTF-8 como Latin-1 → € corrupto.
   **FIX**: Cambiar `<div class="page">` por `<div class="article">`.
2. **wkhtmltopdf version antigua** (< 0.12.5) sin soporte UTF-8 completo
3. **Fuente sin glyph** — el font no tiene el caracter
4. **Doble `web.html_container`** — nunca anidar dos html_container
5. **Asset bundle no carga** — sin el CSS de fonts, wkhtmltopdf usa fuente del sistema

### Solucion practica para simbolo de moneda

Crear un metodo helper que formatea sin simbolo (solo numero):

```python
def format_monetary(self, value):
    """Formatea valor monetario: 1.234,56 (sin simbolo)."""
    currency = self.currency_id
    rounded = float_round(value, precision_digits=currency.decimal_places)
    formatted = float_repr(rounded, currency.decimal_places)
    # Formato español: punto miles, coma decimal
    formatted = formatted.replace('.', ',')
    parts = formatted.split(',')
    integer_part = parts[0]
    decimal_part = parts[1] if len(parts) > 1 else ''
    # Separador de miles
    negative = integer_part.startswith('-')
    if negative:
        integer_part = integer_part[1:]
    groups = []
    while integer_part:
        groups.append(integer_part[-3:])
        integer_part = integer_part[:-3]
    integer_part = '.'.join(reversed(groups))
    if negative:
        integer_part = '-' + integer_part
    return '%s,%s' % (integer_part, decimal_part) if decimal_part else integer_part
```

Y en la columna del template poner el nombre con simbolo en el header:
```xml
<th>EUROS</th>  <!-- El header ya indica la moneda -->
<td><span t-out="doc.format_monetary(line.price_subtotal)"/></td>
```

## 3. Custom Header y Footer (sin external_layout)

### Como funciona el mecanismo

`_prepare_html()` busca `<div class="header">` y `<div class="footer">`
en el HTML renderizado y los **extrae** para pasarlos como ficheros separados
a wkhtmltopdf via `--header-html` y `--footer-html`.

El `<div class="article">` queda como body. **NUNCA usar `<div class="page">`** — ver seccion 2.

### Patron sin external_layout

**IMPORTANTE**: Dos templates separados — wrapper + documento.
El wrapper tiene `html_container`, el documento NO.

```xml
<!-- WRAPPER: tiene html_container y el loop de docs -->
<template id="my_report">
    <t t-call="web.html_container">
        <t t-foreach="docs" t-as="doc">
            <t t-call="my_module.my_report_document" t-lang="doc.partner_id.lang"/>
        </t>
    </t>
</template>

<!-- DOCUMENTO: SIN html_container, define header/article/footer -->
<template id="my_report_document">
    <!-- Variables de borde (en scope del documento, NO en t-call) -->
    <t t-set="S_BF" t-value="'border:1px solid black'"/>

    <!-- HEADER: extraido por _prepare_html, envuelto en minimal_layout -->
    <div class="header">
        <!-- contenido que se repite en cada pagina -->
    </div>

    <!-- BODY: OBLIGATORIO usar class="article", NO "page" -->
    <div class="article" t-att-data-oe-model="doc._name" t-att-data-oe-id="doc.id">
        <!-- contenido principal -->
    </div>

    <!-- FOOTER: extraido por _prepare_html, envuelto en minimal_layout -->
    <div class="footer">
        <!-- pie de pagina -->
    </div>
</template>
```

> **REGLAS CRITICAS:**
> 1. El documento NUNCA debe llamar a `web.html_container` (doble nesting rompe charset)
> 2. El body DEBE usar `class="article"` (no `class="page"`) para que `_prepare_html`
>    lo envuelva en `minimal_layout` con `<meta charset="utf-8"/>`
> 3. Variables QWeb (`t-set`) definidas dentro de un `t-call` NO se propagan al caller.
>    Definir S_BF, S_LR, etc. directamente en el template del documento.

### Gotcha: `binding_model_id` persiste en BD

Al quitar `binding_model_id` de un `<record>` XML y hacer upgrade, Odoo NO lo
elimina de la BD. El report seguira apareciendo en el menu Imprimir.
Solucion: limpiar manualmente via `write({'binding_model_id': False})` o SQL.

### Paper format para header custom grande

```xml
<record id="my_paperformat" model="report.paperformat">
    <field name="margin_top">85</field>      <!-- Espacio total arriba -->
    <field name="margin_bottom">7</field>
    <field name="margin_left">7</field>
    <field name="margin_right">7</field>
    <field name="header_spacing">80</field>   <!-- Espacio header->body -->
    <field name="dpi">90</field>
</record>
```

Relacion: `margin_top` >= `header_spacing` + altura real del header.

### Page break entre documentos

Automatico: cada iteracion de `t-foreach docs` genera un `<div class="article">`
separado que wkhtmltopdf renderiza en pagina nueva.

Para forzar page break dentro de un documento:
```xml
<div style="page-break-before: always;"/>
```

## 4. Tax Totals en v18 (reemplazo de amount_by_group)

| Version | Campo | Tipo |
|---------|-------|------|
| v14-v16 | `amount_by_group` | List of tuples |
| v17 | `tax_totals_json` | JSON string |
| v18+ | `tax_totals` | Binary (computed, JSON dict) |

### Estructura de tax_totals

```python
{
    'same_tax_base': True/False,
    'subtotals': [{
        'name': 'Untaxed Amount',
        'base_amount_currency': 33000.0,
        'tax_groups': [{
            'group_name': 'IVA 21%',
            'base_amount_currency': 16000.0,
            'display_base_amount_currency': 16000.0,
            'tax_amount_currency': 3360.0,
        }]
    }]
}
```

### Patron de iteracion

```xml
<t t-set="tax_totals" t-value="doc.tax_totals"/>
<t t-set="all_tax_groups" t-value="[]"/>
<t t-foreach="tax_totals.get('subtotals', [])" t-as="subtotal">
    <t t-foreach="subtotal.get('tax_groups', [])" t-as="tg">
        <t t-esc="all_tax_groups.append(tg) or ''"/>
    </t>
</t>
<t t-foreach="all_tax_groups" t-as="tg">
    <tr>
        <td><span t-out="doc.format_monetary(tg.get('base_amount_currency', 0))"/></td>
        <td><span t-esc="tg.get('group_name', '')"/></td>
        <td><span t-out="doc.format_monetary(tg.get('tax_amount_currency', 0))"/></td>
    </tr>
</t>
```

### Template estandar de Odoo

```xml
<t t-call="sale.document_tax_totals">
    <t t-set="tax_totals" t-value="doc.tax_totals"/>
    <t t-set="currency" t-value="doc.currency_id"/>
</t>
```

## 5. Bootstrap v13 (BS4) vs v18 (BS5) en reports

| BS4 (v13-v15) | BS5 (v16+) | En inline style |
|---------------|------------|-----------------|
| `text-right` | `text-end` | `text-align:right` |
| `text-left` | `text-start` | `text-align:left` |
| `float-right` | `float-end` | `float:right` |
| `ml-*` / `mr-*` | `ms-*` / `me-*` | `margin-left` / `margin-right` |

**Recomendacion**: Para text-align, redefinir las clases BS4 en el
bloque `<style>` del template (BS5 `text-end` no funciona en wkhtmltopdf):
```css
.text-right, .table td.text-right, .table th.text-right {
    text-align: right !important;
}
```

## 6. Tablas con bordes — dos enfoques

### Enfoque A: Clases CSS en `<style>` inline (RECOMENDADO en v18)

```xml
<style>
    /* Reset BS5 borders en tablas del report */
    .table.table-sm.eur_lines > :not(caption) > * > * {
        border-bottom-width: 0 !important;
        border-top-width: 0 !important;
        box-shadow: none !important;
        border-color: black !important;
    }
    /* Clases de borde con especificidad >= BS5 */
    .eur_lines > :not(caption) > * > .border_full { border: 1px solid black !important; }
    .eur_lines > :not(caption) > * > .borderlr {
        border-left: 1px solid black !important;
        border-right: 1px solid black !important;
        border-bottom: none !important;
        border-top: none !important;
    }
    .eur_lines > :not(caption) > * > .borderlrb {
        border-left: 1px solid black !important;
        border-right: 1px solid black !important;
        border-bottom: 1px solid black !important;
        border-top: none !important;
    }
</style>

<table class="table table-sm eur_lines">
    <thead><tr class="border_full"><th class="border_none">COL</th></tr></thead>
    <tbody><tr><td class="borderlr">valor</td></tr></tbody>
</table>
```

### Enfoque B: Variables QWeb para inline styles (alternativa)

```xml
<t t-set="S_BF" t-value="'border:1px solid black'"/>
<t t-set="S_LR" t-value="'border-left:1px solid black;border-right:1px solid black;border-top:none;border-bottom:none'"/>
<t t-set="S_LRB" t-value="'border-left:1px solid black;border-right:1px solid black;border-bottom:1px solid black;border-top:none'"/>
```

### Tabla de lineas con border-collapse

```xml
<table class="table table-sm" style="border-collapse:collapse;min-height:170mm">
    <thead>
        <tr style="border:1px solid black">
            <th style="border:none;text-align:center">COLUMNA</th>
        </tr>
    </thead>
    <tbody>
        <tr style="height:10px">
            <td t-att-style="S_LR" style="text-align:right">valor</td>
        </tr>
        <!-- Fila de cierre -->
        <tr>
            <td t-att-style="S_LRB"/>
        </tr>
    </tbody>
</table>
```

### Nota sobre `border-collapse`

Añadir `style="border-collapse:collapse"` a la tabla es critico.
Sin esto, `table table-sm` de Bootstrap puede añadir spacing
entre celdas que rompe los bordes laterales continuos.

## 7. Cache de assets

Tras modificar CSS/SCSS en un modulo de reports:

```sql
DELETE FROM ir_attachment WHERE url LIKE '%/web/assets/%';
```

Y reiniciar Odoo. Los assets se regeneran en la siguiente peticion.

En desarrollo, usar `--dev xml,assets` para evitar cacheo.

## 8. Debug de reports

1. **HTML preview**: cambiar URL `/report/pdf/` por `/report/html/`
2. **Modo dev**: `--dev xml` evita cache de templates QWeb
3. **Forzar HTML**: temporalmente usar `report_type = qweb-html` en la accion
4. **Ver HTML generado**: en Python, llamar `report._render_qweb_html()` y guardar resultado
5. **Modo debug desactivado**: Odoo fuerza `debug=False` para PDF rendering
   (con debug on, los assets se sirven como ficheros separados y wkhtmltopdf
   no los carga todos a tiempo → CSS faltante)

## 9. Tablas grandes (>500 filas)

Si un body HTML > 4MB, Odoo automaticamente llama `_split_table(tree, 500)`
que divide `<table>` en multiples tablas hermanas de max 500 filas.
Esto evita el crecimiento exponencial de tiempo de wkhtmltopdf con tablas grandes.

## 10. Referencia rapida de selectores seguros

```scss
// Estos FUNCIONAN en wkhtmltopdf:
.my-class { color: #333; }
.my-class { font-size: 10pt; }
.my-class { border: 1px solid black; }
.my-class { background-color: #f0f0f0; }
.my-class { text-align: right; }
.my-class { padding: 2px 5px; }
.my-class { font-weight: bold; }
thead { display: table-header-group; }  // Repite header en cada pagina
tr { page-break-inside: avoid; }

// Estos NO FUNCIONAN:
.my-class { color: var(--my-color); }         // var() ignorado
.my-class { width: calc(100% - 20px); }       // calc() falla
.my-class { display: grid; }                   // grid no soportado
.my-class { position: sticky; }                // sticky no soportado
@media (min-width: 768px) { ... }              // media queries ignoradas
```
