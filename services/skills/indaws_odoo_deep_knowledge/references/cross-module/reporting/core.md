# Reporting / BI (transversal) — Core (Odoo 18.0, + v19 diffs)

<!-- verified: v18.0 @ 2026-06 -->

Dominio transversal de reporting. Cubre CUATRO motores distintos que la gente confunde:

1. **QWeb → PDF/HTML/text** (documentos imprimibles: facturas, albaranes). Motor: wkhtmltopdf.
   Fuente: `odoo/addons/base/models/ir_actions_report.py`, `addons/web/views/report_templates.xml`,
   `addons/web/controllers/report.py`, `odoo/addons/base/models/report_paperformat.py`. → §1-9.
2. **Vistas SQL de análisis** (`sale.report`, `account.invoice.report`, `stock.report.*`): modelos
   `_auto=False` que materializan un SELECT/GROUP BY en una vista PG o subquery. → §10-12.
3. **Agregación pivot/graph** vía `read_group`/`_read_group`/`web_read_group` sobre cualquier modelo.
   Fuente: `odoo/models.py`, `addons/web/static/src/views/pivot|graph`. → §13-16.
4. **Motor account.report (EE)**: informes financieros declarativos (P&L, Balance, 303…) con
   `account.report.expression` y 6 engines. Modelo en CE `account`, lógica en EE `account_reports`. → §17.

Para troubleshooting ver `gotchas.md` (prefijo **RPTG**). Integraciones (account_reports EE,
spreadsheet, dashboards) en `integrations.md`. Diffs de versión en `v19_changes.md`.

---

## PARTE A — QWeb → PDF (wkhtmltopdf)

---

## 1. `ir.actions.report` — el modelo del informe

Modelo `ir.actions.report` (`_table = 'ir_act_report_xml'`), hereda `ir.actions.actions`.
Definición: `odoo/addons/base/models/ir_actions_report.py:139-166`, `IrActionsReport._name`.

### `report_type` (Selection) — `odoo/addons/base/models/ir_actions_report.py:151-159`, `class IrActionsReport`
Tres tipos, cada uno con su propio método de render:
- `qweb-html` — se abre directo en el navegador (`_render_qweb_html`).
- `qweb-pdf` — render con **wkhtmltopdf** y descarga (`_render_qweb_pdf`). **Default.**
- `qweb-text` — texto plano (`_render_qweb_text`).

`default='qweb-pdf'`, `required=True`. El help del campo dice literalmente "PDF means the
report will be rendered using Wkhtmltopdf and downloaded by the user".

### Campos clave
- `report_name` (Char, required) — nombre del template QWeb (ej. `account.report_invoice`).
- `report_file` — path al fichero del template.
- `paperformat_id` (Many2one `report.paperformat`) — `odoo/addons/base/models/ir_actions_report.py:166`, `class IrActionsReport`. **Opcional**: si
  vacío, se hereda de la compañía (ver §4).
- `print_report_name` — expresión Python (`object`, `time`) para el nombre del fichero descargado.
- `attachment` / `attachment_use` — guardar/reutilizar el PDF como adjunto (caché).
- `groups_id` — restricción de acceso al informe.

### Helpers de paperformat — `odoo/addons/base/models/ir_actions_report.py:273-277`, `IrActionsReport.get_paperformat`
```python
def get_paperformat(self):
    return self.paperformat_id or self.env.company.paperformat_id
def get_paperformat_by_xmlid(self, xml_id):
    return self.env.ref(xml_id).get_paperformat() if xml_id else self.env.company.paperformat_id
```
Es la **cadena de fallback**: report.paperformat_id → company.paperformat_id.

---

## 2. Pipeline de render PDF

Entry point: `_render_qweb_pdf(report_ref, res_ids, data)` — `odoo/addons/base/models/ir_actions_report.py:1014`, `IrActionsReport._pre_render_qweb_pdf`.

```
_render_qweb_pdf
  └─ _pre_render_qweb_pdf  (ir_actions_report.py:1000)
       ├─ si report_type != 'pdf' (html en context) → _render_qweb_html  (línea 1009)
       └─ _render_qweb_pdf_prepare_streams  (línea 795)
            ├─ _render_qweb_html → genera el HTML completo  (línea 860)
            ├─ _prepare_html → separa body/header/footer + specific_paperformat_args  (línea 862)
            └─ _run_wkhtmltopdf → subprocess wkhtmltopdf  (línea 872 → def 502)
                 └─ _build_wkhtmltopdf_args → construye los flags CLI  (def 287)
```

### `_render_qweb_html` → el HTML que ve wkhtmltopdf
Produce el HTML completo del informe. Es el MISMO HTML que devuelve la ruta `/report/html/...`
(ver §6), salvo que en el path PDF el `debug` se fuerza a `False` (ver RPTG12).

### `_prepare_html` — `odoo/addons/base/models/ir_actions_report.py:368-455`, `IrActionsReport._prepare_html`
Parte el HTML de un único documento multi-página en piezas, usando XPath sobre clases CSS:
- `//div[@class~='header']` → `#minimal_layout_report_headers`
- `//div[@class~='footer']` → `#minimal_layout_report_footers`
- `//div[@class~='article']` → cada `body` (uno por registro; lee `data-oe-model`/`data-oe-id`).

Cada body se re-renderiza envuelto en `web.minimal_layout` (línea 415). Header y footer se renderizan
también con `minimal_layout` pero con `subst=True` (líneas 441, 449) → activa el truco JS de
substitución por página (ver §5).

Además extrae los **`data-report-*`** del tag `<html>` raíz → `specific_paperformat_args`
(líneas 433-438). Estos atributos **tienen prioridad** sobre el `report.paperformat` (ver §4).

### `_run_wkhtmltopdf` — `odoo/addons/base/models/ir_actions_report.py:502-637`, `IrActionsReport._run_wkhtmltopdf`
1. Resuelve `paperformat_id` vía `get_paperformat()` (línea 524).
2. `_build_wkhtmltopdf_args(...)` → flags base (línea 527).
3. Si hay `request.db`: crea sesión temporal + **cookie-jar** para que wkhtmltopdf resuelva enlaces
   internos autenticados (`--cookie-jar`, líneas 539-558). Sin esto, imágenes/QR vía URL fallan.
4. Escribe header/footer a ficheros temporales → `--header-html` / `--footer-html` (líneas 559-571).
5. Escribe cada body a fichero temporal. Tablas >4 MiB se parten en bloques de 500 filas
   (`_split_table`) por el coste exponencial de wkhtmltopdf con tablas grandes (líneas 572-595).
6. Ejecuta `subprocess.Popen([bin] + command_args + files_command_args + paths + [out.pdf])`
   (línea 599). **returncode 0 y 1 se aceptan** (1 = warnings); otros → `UserError` (líneas 603-622).
   `-11` (SIGSEGV) → mensaje "Memory limit too low".
7. Lee el PDF, limpia temporales, devuelve bytes.

### Split multi-documento — `odoo/addons/base/models/ir_actions_report.py:891-960`, `IrActionsReport._render_qweb_pdf_prepare_streams`
Tras generar 1 PDF con N documentos, se parte por registro usando los **PDF outlines** (los `<h1>`
del documento). Si nº outlines == nº registros y hay heading en página 1 → split limpio; si no, se
renderiza cada registro por separado (fallback recursivo, línea 956). Relevante para `attachment`
por registro.

---

## 3. `--javascript-delay` y returncode
- `report.print_delay` (ir.config_parameter, default `'1000'` ms) → `--javascript-delay`
  (`odoo/addons/base/models/ir_actions_report.py:360-361`, `IrActionsReport._build_wkhtmltopdf_args`). Tiempo que wkhtmltopdf espera a que el JS pinte antes de
  capturar. Subirlo si contenido dinámico sale incompleto.
- `--disable-smart-shrinking` si `paperformat.disable_shrinking` (línea 357).

---

## 4. `report.paperformat`

Modelo `report.paperformat` — `class report_paperformat` en `odoo/addons/base/models/report_paperformat.py:166`, `class report_paperformat`.
Campos y defaults verificados (`odoo/addons/base/models/report_paperformat.py:172-190`, `report_paperformat.format`):

| Campo | Tipo | Default | Flag wkhtmltopdf | Línea |
|---|---|---|---|---|
| `format` | Selection (A4, Letter, ..., `custom`) | `A4` | `--page-size` (si != custom) | 172 |
| `margin_top` | Float (mm) | `40` | `--margin-top` | 173 |
| `margin_bottom` | Float (mm) | `20` | `--margin-bottom` | 174 |
| `margin_left` | Float (mm) | `7` | `--margin-left` | 175 |
| `margin_right` | Float (mm) | `7` | `--margin-right` | 176 |
| `page_height` / `page_width` | Integer (mm) | `False` | `--page-height`/`--page-width` (solo si `format=='custom'`) | 177-178 |
| `orientation` | Selection (Landscape/Portrait) | — | `--orientation` | 179 |
| `header_line` | Boolean | `False` | `--header-line` | 183 |
| `header_spacing` | Integer (mm) | `35` | `--header-spacing` | 184 |
| `disable_shrinking` | Boolean | — | `--disable-smart-shrinking` | 185 |
| `dpi` | Integer (required) | `90` | `--dpi` (+ `--zoom 96/dpi` si versión soporta) | 186 |
| `css_margins` | Boolean | `False` | (no flag; activa clase `o_css_margins`) | 190 |

`_build_wkhtmltopdf_args` — `odoo/addons/base/models/ir_actions_report.py:287-366`, `IrActionsReport._build_wkhtmltopdf_args`. Lógica relevante:
- `format` + custom (líneas 312-318): solo se usan `page_width`/`page_height` si `format == 'custom'`.
  Si pones width/height pero dejas `format='A4'`, las dimensiones custom **se ignoran** (RPTG2).
- `dpi`/`--zoom` (líneas 325-337): si la versión wkhtmltopdf >= 0.12.2
  (`wkhtmltopdf_dpi_zoom_ratio=True`, línea 111) añade `--zoom 96/dpi` para compensar.
  En Windows con dpi<=95 fuerza dpi=96.
- **Prioridad de los `data-report-*`** (specific_paperformat_args) sobre el record:
  `data-report-margin-top`, `data-report-header-spacing`, `data-report-dpi`,
  `data-report-landscape` ganan al `report.paperformat` (líneas 320-322, 326-327, 339-340).

### `css_margins` + `o_css_margins`
`minimal_layout` añade la clase `o_css_margins` al `<body>` si
`get_paperformat_by_xmlid(report_xml_id).css_margins` es True (`web/views/report_templates.xml:270`, `minimal_layout`, en el
`<body t-attf-class="o_body_pdf {{...css_margins and 'o_css_margins'}}...">`). Permite controlar
márgenes vía CSS en vez de los flags de wkhtmltopdf.

---

## 5. Layouts: `web.minimal_layout` vs `web.external_layout`

Dos capas distintas, no confundir (ver RPTG5):

### `web.minimal_layout` — `web/views/report_templates.xml:229-274`, `minimal_layout`
Es el **wrapper de PDF**. Lo aplica internamente `_prepare_html` a cada body/header/footer.
`_get_layout()` devuelve `web.minimal_layout` (`odoo/addons/base/models/ir_actions_report.py:279-280`, `IrActionsReport._get_layout`). Contiene:
- `<base t-att-href="base_url"/>` para resolver rutas relativas.
- Los tres `t-call-assets` (ver §7).
- El **script JS `subst()`** (solo cuando `subst=True`): wkhtmltopdf inyecta una query string por
  página (`?page=&topage=&webpage=...`); `subst()` parsea `webpage`, extrae el índice del documento
  y selecciona el header/footer correcto de entre todos los acumulados en
  `#minimal_layout_report_headers/footers`. Así un solo PDF multi-doc muestra el header de cada
  compañía/registro correcto. Si este JS no corre → header/footer mal o repetidos (RPTG3).

### `web.external_layout` — `web/views/report_templates.xml:802-820`, `external_layout`
Es el **marco visual de empresa** (logo, dirección, pie con datos fiscales). Es lo que el usuario
incluye en su template con `<t t-call="web.external_layout">`. Despacha por compañía:
```xml
<t t-if="company.external_report_layout_id" t-call="{{company.external_report_layout_id.sudo().key}}">
<t t-else="else" t-call="web.external_layout_standard">
```
Variantes (en `report_templates.xml`): `external_layout_standard` (496), `external_layout_boxed`
(367), `external_layout_bold` (432), `external_layout_striped` (305), y además `folder` (557),
`wave` (643), `bubble` (716). La variante activa la decide
`res.company.external_report_layout_id` (Configuración → Ajustes → Diseño del documento).

### `web.html_container` / `web.report_layout` — `web/views/report_templates.xml:93`, `html_container` y `web/views/report_templates.xml:6`, `report_layout`
`html_container` envuelve en `report_layout` (el `<!DOCTYPE html>` con `<head>` + assets). Es el
contenedor que se usa en informes "normales" (no en el path de PDF interno, que usa minimal_layout).
`report_layout` carga **solo** `web.report_assets_common` (línea 17), NO `report_assets_pdf`.

---

## 5b. Anatomía de un informe estándar (wrapper → `_document`)

Un informe estándar de Odoo NO es una plantilla monolítica: es un **envoltorio fino** que delega
en un **`_document`**. Patrón verificado en `sale.order` (`addons/sale/report/ir_actions_report_templates.xml`):

```xml
<template id="report_saleorder">                       <!-- el ir.actions.report apunta AQUÍ (report_name) -->
  <t t-call="web.html_container">
    <t t-foreach="docs" t-as="doc">
      <t t-call="sale.report_saleorder_document" t-lang="doc.partner_id.lang"/>  <!-- delega -->
    </t>
  </t>
</template>
<template id="report_saleorder_document">             <!-- contenido + herencias cuelgan de AQUÍ -->
  <t t-call="web.external_layout">                     <!-- marco de empresa -->
    <t t-set="lines_to_report" t-value="doc._get_order_lines_to_report()"/>   <!-- ⚠ var computada -->
    <t t-foreach="lines_to_report" t-as="line"> ... </t>                       <!-- NO doc.order_line -->
    <table class="o_total_table ...">
      <t t-call="sale.document_tax_totals"><t t-set="tax_totals" t-value="doc.tax_totals"/></t>  <!-- widget calculado -->
    </table>
  </t>
</template>
```

Implicaciones al **extender/importar/clonar** un informe estándar:
- **`report_X` = envoltorio** (`web.html_container` + `t-foreach docs` + `t-call report_X_document`).
  Todo el contenido y TODAS las herencias cuelgan del **`_document`**, no del envoltorio → los `xpath`
  de herencia apuntan al `_document`.
- **Líneas ≠ `doc.order_line`**: se usa una **variable computada** (`lines_to_report =
  doc._get_order_lines_to_report()`) que ya ordena/filtra secciones, notas y combos. Reconstruirla a
  mano pierde ese orden.
- **Totales = `o_total_table` + widget `tax_totals`** (`doc.tax_totals`): bloque **calculado** con el
  desglose de impuestos por grupo, no campos sueltos. No lo sustituyas por `sum(...)` salvo que
  aceptes perder el desglose.

## 5c. Cómo Odoo ensambla el arch del template — `get_combined_arch()`

Las plantillas QWeb son `ir.ui.view`. El informe apunta a la vista raíz por `report_name == view.key`.
Antes de renderizar, Odoo **combina la vista raíz con TODAS sus extensiones** (verificado
`base/models/ir_ui_view.py`, v18 y v19):

- `get_combined_arch()` (`odoo/addons/base/models/ir_ui_view.py:933`, `View.get_combined_arch`, v19 `:1043`) sube por `inherit_id` a la raíz `primary`
  y combina, vía `apply_inheritance_specs` (`:837`), todas las vistas `mode='extension'` que la heredan.
- El set de extensiones se obtiene con la CTE recursiva `_get_inheriting_views()` (`:621`) — recoge
  las herencias **desde cualquier nodo** del árbol, no solo hijas directas.
- El branding (`data-oe-*` para el editor web/Studio) lo añade `inherit_branding()` (`:775`) **solo si
  `self._context.get('inherit_branding')`** (`:901,920`); el marcado por-nodo ocurre en el render de
  `ir.qweb`, no en `get_combined_arch`.
- `apply_inheritance_specs` acepta xpath con la función custom **`hasclass('page')`** (registrada en
  `odoo/addons/base/models/ir_ui_view.py:136-137`, `xpath_utils`) → anclar por clase (`//div[hasclass('page')]`) es más robusto que
  por posición.

Consecuencia práctica: para modificar un informe estándar **heredas** (vista `extension` con `inherit_id`
al `_document`), y Odoo entrega el arch combinado al motor. Nunca hace falta copiar la plantilla entera.

---

## 6. Controlador de informes — `addons/web/controllers/report.py`

Ruta genérica `report_routes` — `web/controllers/report.py:23-50`, `ReportController.report_routes`:
```
/report/<converter>/<reportname>
/report/<converter>/<reportname>/<docids>
```
- `converter='html'` → `_render_qweb_html` → devuelve el HTML crudo (línea 38-40).
  **Esta es la URL para depurar el render** (`/report/html/account.report_invoice/42`).
- `converter='pdf'` → `_render_qweb_pdf` (línea 41-43).
- `converter='text'` → `_render_qweb_text` (línea 44-46).

`/report/check_wkhtmltopdf` — `web/controllers/report.py:153-155`, `ReportController.check_wkhtmltopdf`: devuelve el estado del binario (la UI llama esto
antes de imprimir; ver §8).

`/report/barcode/<type>/<value>` — `web/controllers/report.py:55-58`, `ReportController.report_barcode`: render de códigos de barras vía reportlab,
usado dentro de los templates con `<img t-att-src="'/report/barcode/QR/...'"/>`.

---

## 7. `t-call-assets` y los bundles de assets

El CSS/JS de un informe NO está inline: se inyecta vía `t-call-assets` en el `<head>` del layout.
Si falta el `t-call-assets`, el CSS no aplica y el PDF sale sin estilo (RPTG6).

### `web.minimal_layout` carga 3 bundles — `web/views/report_templates.xml:234-236`, `minimal_layout`
```xml
<t t-call-assets="web.report_assets_pdf" t-js="false"/>    <!-- CSS reset pdf -->
<t t-call-assets="web.report_assets_common" t-js="false"/> <!-- CSS común -->
<t t-call-assets="web.report_assets_pdf" t-css="false"/>   <!-- JS pdf -->
```

### `web.report_assets_common` — `addons/web/__manifest__.py:259-318`
El grande: variables SCSS, Bootstrap backend, FontAwesome, odoo_ui_icons, fonts, y los SCSS de
report (`report.scss`, `report_tables.scss`, `layout_*.scss`, `asset_styles_company_report.scss`).
Es lo que da estilo real a las tablas/layout. Se carga tanto en `report_layout` (HTML) como en
`minimal_layout` (PDF).

### `web.report_assets_pdf` — `addons/web/__manifest__.py:320-322`
Mínimo: solo `reset.min.css`. Es el reset específico para que WebKit/wkhtmltopdf pinte consistente.

> Módulos como `account`, `event`, `hr_skills` **extienden** `web.report_assets_common` desde su
> `__manifest__.py` para añadir su propio CSS de informe. Si tu CSS custom no aparece en el PDF,
> probablemente lo metiste en `web.assets_backend` en vez de en `web.report_assets_common` (RPTG6).

---

## 8. Estado de wkhtmltopdf — `wkhtmltopdf_state`

Se calcula UNA VEZ al arrancar Odoo (módulo-level) — `odoo/addons/base/models/ir_actions_report.py:91-117`, `wkhtmltopdf_state`. Estados:
- `install` — binario no encontrado (`OSError`). `_render_qweb_pdf` lanza UserError (línea 844-849).
- `upgrade` — versión < 0.12.0.
- `ok` — versión >= 0.12.0.
- `workers` — Odoo arrancado con `workers=1`. **wkhtmltopdf necesita >=2 workers** porque hace una
  request HTTP de vuelta a Odoo para renderizar el HTML; con 1 worker hay deadlock (línea 113-115).
- `broken` — binario no devuelve versión parseable.

`wkhtmltopdf_dpi_zoom_ratio=True` si versión >= 0.12.2 (línea 111). Expuesto vía
`get_wkhtmltopdf_state()` (línea 261-271).

---

## 9. v19 — diffs verificados

**El motor sigue siendo wkhtmltopdf en v19. NO hay Chromium/Playwright nativo.**
Fuente: `odoo19/odoo/addons/base/models/ir_actions_report.py`.

- `report_type` idéntico: `qweb-html` / `qweb-pdf` / `qweb-text`, default `qweb-pdf` (v19 línea 170-174).
- `report.paperformat`: mismos campos y defaults (margin_top=40, header_spacing=35, dpi=90,
  css_margins) — `odoo/addons/base/models/report_paperformat.py:171-189`, `report_paperformat.default`.
- Misma cadena de estados `install/ok/upgrade/workers/broken` y misma comprobación >=0.12.0 /
  >=0.12.2 (v19 líneas 111-121).
- Mismo truco "disable debug en render PDF" para no romper el lazy-load de assets, con el mismo
  comentario al issue #2083 (`additional_context = {'debug': False}` en v19 línea 876; en v18 es
  línea 856).

**Cambio estructural v19**: `_run_wkhtmltopdf` ahora tiene un helper **a nivel de módulo**
`_run_wkhtmltopdf(args)` que ejecuta el subprocess con `subprocess.run(..., check=False)`
(v19 línea 41-54), separado del método del modelo. El binario se resuelve vía `_wkhtml().bin`.
Además v19 detecta QT no parcheado al convertir múltiples documentos y lanza
`UserError("Tried to convert multiple documents in wkhtmltopdf using unpatched QT")` /
`unittest.SkipTest` en tests (v19 líneas 627-628). En v18 esa rama no existe.

No hay cambios en la API pública (`_render_qweb_pdf`, `_render_qweb_html`, `_render_qweb_text`,
`minimal_layout`, `external_layout`, `t-call-assets`).

---

## PARTE B — Vistas SQL de análisis (modelos `_auto=False`)

Los informes de análisis (menú *Reporting* de Ventas, Compras, Inventario, Contabilidad) NO son
QWeb: son modelos ORM normales pero **sin tabla propia** (`_auto = False`). Su "tabla" es el
resultado de un `SELECT ... GROUP BY` sobre las tablas reales. Esto permite agregar por SQL
(rápido) y exponerlo a pivot/graph/list como si fuera un modelo cualquiera.

### 10. Dos mecanismos de materialización (¡coexisten en v18!)

**Mecanismo 1 — `_table_query` (subquery inlineada, el moderno)**
`sale.report` y `account.invoice.report` definen un **property `_table_query`** que devuelve un
objeto `SQL`. Ejemplo `account/report/account_invoice_report.py:73-75`, `AccountInvoiceReport._table_query`:
```python
@property
def _table_query(self) -> SQL:
    return SQL('%s %s %s', self._select(), self._from(), self._where())
```
NO se crea ninguna vista PG. En `_table_sql` (`odoo/models.py:871-899`, `BaseModel._table_sql`) Odoo envuelve la query en
`FROM (<subquery>)` **en cada consulta**. Ventaja: nunca hay vista obsoleta; se re-evalúa siempre
(p.ej. `sale.report._from_sale()` recalcula la currency_table por compañía en cada lectura,
`sale/report/sale_report.py:183-195`, `SaleReport._from_sale`). Coste: la subquery se ejecuta entera en cada read_group.

**Mecanismo 2 — `init()` + `CREATE OR REPLACE VIEW` (el clásico)**
`report.stock.quantity`, `fleet.vehicle.cost.report`, etc. sobreescriben `init()`
(`odoo/models.py:3496`, `BaseModel.init`, hook llamado tras `_auto_init`) para crear una **vista PG real**. Ejemplo
`stock/report/report_stock_quantity.py:33-46`, `ReportStockQuantity.init`:
```python
def init(self):
    tools.drop_view_if_exists(self._cr, 'report_stock_quantity')
    query = """CREATE or REPLACE VIEW report_stock_quantity AS (...)"""
    self.env.cr.execute(query, params)
```
La vista se materializa en BD al instalar/actualizar el módulo. Más rápido en lectura (PG la
planifica/cachea) pero **si cambias el SQL hay que `-u` el módulo** para que `init()` re-cree la
vista (RPTG-SQL).

> Regla para saber cuál usa un report: ¿tiene `def init(self)` con `CREATE VIEW`? → mecanismo 2.
> ¿tiene `@property _table_query`/`_query()`? → mecanismo 1. Los reports nuevos de Odoo migran a
> `_table_query` con objetos `SQL` (inyección-safe).

### 11. Anatomía de un report SQL (patrón `sale.report`)

```python
class SaleReport(models.Model):
    _name = "sale.report"
    _description = "Sales Analysis Report"
    _auto = False                 # NO crea tabla
    _rec_name = 'date'
    _order = 'date desc'
    # Cada campo es readonly=True y mapea a una columna del SELECT
    product_uom_qty = fields.Float(string="Qty Ordered", readonly=True)   # → SUM(...)
    price_unit = fields.Float(aggregator='avg', readonly=True)            # → AVG(...)
    nbr = fields.Integer(string="# of Lines", readonly=True)              # → COUNT(*)
```
Fuente `sale/report/sale_report.py:8-94`, `SaleReport`. Partes del SQL, cada una un método-hook sobreescribible:
- `_select_sale()` (`sale/report/sale_report.py:98-170`, `SaleReport._select_sale`): columnas. Cada `fields.X` del modelo DEBE tener su
  `... AS <fname>` aquí. **El nombre de la columna debe coincidir EXACTO con el nombre del campo.**
- `_from_sale()` (`:183-195`): los JOINs (sale_order_line l LEFT JOIN sale_order s …).
- `_where_sale()` (`:197-199`): filtro (`l.display_type IS NULL`).
- `_group_by_sale()` (`:201-230`): el GROUP BY — **toda columna no agregada del SELECT debe estar
  aquí** o PG lanza "column must appear in GROUP BY" (RPTG-SQL).
- `_query()` (`:232+`): ensambla WITH/SELECT/FROM/WHERE/GROUP BY.

`id` se sintetiza con `MIN(l.id) AS id` (`sale/report/sale_report.py:100`, `SaleReport._select_sale`): un report SQL **necesita una
columna `id` única** porque el ORM la usa como PK virtual. Si dos filas agregadas comparten el
mismo `MIN(id)` hay colisión silenciosa.

### 12. `_depends` — invalidación de caché del report SQL

Como la "tabla" es virtual, el ORM no sabe qué campos reales la afectan. `_depends` lo declara
para invalidar caché y forzar flush antes de leer. Ej. `account/report/account_invoice_report.py:56-71`, `AccountInvoiceReport._depends`:
```python
_depends = {
    'account.move': ['name','state','move_type','partner_id', ...],
    'account.move.line': ['quantity','price_subtotal', ...],
    'product.product': ['product_tmpl_id','standard_price'],
    ...
}
```
`_table_sql` (`odoo/models.py:884-899`, `BaseModel._table_sql`) recorre el `_depends` (cierre transitivo) y añade
`to_flush` por cada campo → fuerza que los valores pendientes en caché ORM se escriban a BD antes
de ejecutar la subquery. **Sin `_depends`, un report `_table_query` puede leer datos stale** dentro
de la misma transacción (RPTG-SQL).

---

## PARTE C — Agregación pivot / graph (`read_group` family)

Las vistas pivot, graph y el agrupado de la list view NO ejecutan SQL a mano: llaman a la familia
`read_group`. Saber qué método se usa y sus límites es clave para depurar "el pivot da números
raros" o "no me deja agrupar por ese campo".

### 13. La familia de métodos (v18)

| Método | Capa | Uso | Fuente |
|---|---|---|---|
| `_read_group(domain, groupby, aggregates, having, ...)` | ORM bajo nivel | El motor real. Devuelve tuplas. | `odoo/models.py:1965`, `BaseModel._read_group` |
| `read_group(domain, fields, groupby, lazy=True)` | ORM compat | API "vieja" con dicts `__domain`/`__context`. | `odoo/models.py:2806`, `BaseModel.read_group` |
| `web_read_group(...)` | web | Lo que llama el cliente JS (pivot/graph/list). | `addons/web/models/models.py:230`, `Base.web_read` |

**v19**: `read_group` está **deprecada** → usar `_read_group` (backend) o `formatted_read_group`
(resultado formateado). Ver `v19_changes.md`.

### 14. `aggregator` — qué campos se pueden agregar (y cómo)

El atributo del campo que decide la función SQL de agregación. **Renombrado en v18**:
`group_operator` → `aggregator` (`odoo/fields.py:482-484`, `Field._get_attrs`):
```python
if 'group_operator' in attrs:
    warnings.warn("Since Odoo 18, 'group_operator' is deprecated, use 'aggregator' instead", ...)
    attrs['aggregator'] = attrs.pop('group_operator')
```
- Numéricos (`Integer`/`Float`/`Monetary`): `aggregator='sum'` por defecto (`odoo/fields.py:1534`, `Integer._get_attrs` y `odoo/fields.py:1629`, `Float.__init__` y `odoo/fields.py:1721`, `Monetary.__init__`).
- `sequence` → `aggregator=None` por defecto (no tiene sentido sumar) (`odoo/fields.py:1538-1540`, `Integer._get_attrs`).
- Para que un campo sea una **medida (measure)** en pivot/graph debe tener `aggregator` != None.
  `aggregator='avg'` para medias (ej. `price_unit` en sale.report, `sale/report/sale_report.py:83`, `class SaleReport`).
- `_read_group_select` (`odoo/models.py:2047-2073`, `BaseModel._read_group_select`) valida el aggregate: rechaza dot-notation
  (`:2057`), campo inexistente (`:2061`), método no en `READ_GROUP_AGGREGATE` (`:2064`), y `recordset`
  solo sobre relacional/id (`:2069-2070`).
- `READ_GROUP_AGGREGATE` (`odoo/models.py:365-377`, `READ_GROUP_AGGREGATE`) es el dict de las 11 funciones SQL admitidas:
  `sum/avg/max/min/bool_and/bool_or/array_agg/recordset/count/count_distinct`. No hay percentil/mediana.

`_description_aggregator` (`odoo/fields.py:930-940`, `Field._description_aggregator`): un campo **non-stored / computed sin `store=True`
NO es agregable** salvo que su SQL sea materializable; por eso no aparece como medida ni se puede
agrupar por él (RPTG-BI).

### 15. `lazy=True` — el gotcha clásico de read_group

`read_group(..., lazy=True)` (default) **solo agrupa por el PRIMER groupby**; el resto se mete en
`__context` para una segunda llamada (lazy loading de niveles). `odoo/models.py:2831-2856`, `BaseModel.read_group`:
```python
lazy_groupby = groupby[:1] if lazy else groupby   # ← solo el primero si lazy
```
Implicaciones:
- Con `lazy=True` y `groupby=['a','b']` obtienes grupos por `a` y un `__context` con `b` pendiente,
  NO el producto cartesiano. Para datos completos en una llamada → `lazy=False`.
- El nombre de la clave de conteo cambia: con lazy y 1 groupby es `<field>_count`; si no, `__count`
  (`odoo/models.py:2855-2856`, `BaseModel.read_group`).
- La vista pivot/graph hace múltiples read_group (uno por nivel expandido), por eso "expandir" un
  grupo dispara una request nueva (`web/static/src/views/pivot/pivot_model.js:870`, `_expandGroup`).

### 16. Arch parsers pivot/graph — qué lee del XML

**Graph** (`addons/web/static/src/views/graph/graph_arch_parser.js`):
- `<graph stacked="...">` → `archInfo.stacked` (`:19-20`); `type` (bar/line/pie) en `<field>` o root.
- `<field name="x" type="measure"/>` → measure; `type="row"`/sin type → groupBy.
- `invisible="1"`/`"True"` excluye el campo (`:64`).

**Pivot** (`addons/web/static/src/views/pivot/pivot_arch_parser.js`):
- `<field name="x" type="measure"/>` **o** `operator="..."` → medida activa (`activeMeasures`, `:57`).
- `invisible` excluye (`:44-45`). `type="row"`/`"col"` define groupBy inicial de fila/columna.
- `__count` siempre disponible como medida implícita.

`pivot_model.js` documenta el modelo: cada celda = un read_group; los grupos expandidos
(`expandedRowGroupBys`/`expandedColGroupBys`, `:313-314`) disparan read_groups adicionales.

---

## PARTE D — Motor account.report (Enterprise)

Los informes financieros (P&L, Balance, declaraciones de impuestos como 303/347) NO son ni QWeb ni
SQL views: usan un **motor declarativo** propio. El **modelo** `account.report` /
`account.report.line` / `account.report.expression` vive en **CE `account`**
(`addons/account/models/account_report.py`), pero la **lógica de cálculo/render** vive en **EE
`account_reports`** (`odoo_enterprise/18/account_reports/models/account_report.py`).

### 17. Los 6 engines de `account.report.expression`

Definición `account/models/account_report.py:545-557`, `AccountReportExpression.engine`:
```python
engine = fields.Selection([
    ('domain',        "Odoo Domain"),          # cuenta/suma AML que matchean un domain
    ('tax_tags',      "Tax Tags"),             # +tag/-tag sobre account.account.tag
    ('aggregation',   "Aggregate Other Formulas"),  # combina otras expresiones (A + B - C)
    ('account_codes', "Prefix of Account Codes"),    # "4%" suma cuentas que empiezan por 4
    ('external',      "External Value"),       # valor manual / importado
    ('custom',        "Custom Python Function"),     # método Python a medida
])
formula = fields.Char(required=True)   # la fórmula según el engine
subformula = fields.Char()             # modificador (ej. 'sum_if_pos', 'editable')
```
Constraint clave: `domain_engine_subformula_required` (`:586-588`) — engine `domain` SIEMPRE
requiere subformula. Los engines `tax_tags`/`account_codes` reciben `next_groupby=None` por
optimización (`NO_NEXT_GROUPBY_ENGINES`, EE `account_reports/models/account_report.py:45`, `NO_NEXT_GROUPBY_ENGINES`).

`domain_formula`/`account_codes_formula`/`aggregation_formula`/etc. (`:350-355`) son campos
**shortcut store=False** con `inverse=` para crear las expresiones desde el XML de forma compacta.

> Detalle profundo de los engines, subformula sign-policy (`sum_if_pos`), tax_tags y multi-company
> está en `references/cross-module/account_reports/` (prefijo ARG). Aquí solo el mapa de entrada.
