# QWeb PDF Report Migration Guide (v14 → v18)

Referencia de cambios breaking en templates QWeb de reportes PDF entre versiones de Odoo.

## Indice

| Sec | Cambio | Versiones | Severidad |
|-----|--------|-----------|-----------|
| 1 | Bootstrap 4 → 5 | v15→v16 | CRITICAL |
| 2 | t-raw → t-out | v15→v16 | HIGH |
| 3 | Tax Totals Evolution | v14→v18 | CRITICAL |
| 4 | layout_document_title | v17→v18 | CRITICAL |
| 5 | field tag en reports | v16→v17 | BLOCKING |
| 6 | External Layout Changes | v14→v18 | HIGH |
| 7 | Report Template Changes (sale/invoice/purchase) | v14→v18 | HIGH |
| 8 | XPath Breakage Patterns | v14→v18 | HIGH |
| 9 | Report Render API | v14→v18 | CRITICAL |
| 10 | Asset Loading, Layouts, Paper Format | v14→v18 | HIGH |
| 11 | wkhtmltopdf & Company Fields | v14→v18 | MEDIUM |
| 12 | Deteccion Sistematica (script) | Todas | — |

## 1. Bootstrap 4 → 5 (v15→v16)

Afecta layout visual de todos los reportes.

| BS4 (v14-v15) | BS5 (v16+) | | BS4 (v14-v15) | BS5 (v16+) |
|----------------|------------|-|----------------|------------|
| `text-right` | `text-end` | | `text-left` | `text-start` |
| `pull-right` | `float-end` | | `pull-left` | `float-start` |
| `float-right` | `float-end` | | `float-left` | `float-start` |
| `table-condensed` | `table-sm` | | `no-gutters` | `g-0` |
| `ml-*` | `ms-*` | | `mr-*` | `me-*` |
| `pl-*` | `ps-*` | | `pr-*` | `pe-*` |
| `font-weight-bold` | `fw-bold` | | `font-weight-normal` | `fw-normal` |
| `font-italic` | `fst-italic` | | `badge-pill` | `rounded-pill` |
| `border-right` | `border-end` | | `border-left` | `border-start` |
| `rounded-right` | `rounded-end` | | `rounded-left` | `rounded-start` |
| `data-toggle` | `data-bs-toggle` | | `data-target` | `data-bs-target` |
| `data-dismiss` | `data-bs-dismiss` | | `col-xs-*` | `col-*` |
| `form-group` | `mb-3` | | `form-row` | `row g-3` |

**Nota**: `ml-*/mr-*/pl-*/pr-*` requieren revision manual (pueden ser clases custom).

## 2. t-raw → t-out (v15→v16)

Deprecado en v15, puede fallar en v18+.

```xml
<!-- ANTES -->              <!-- DESPUES -->
<t t-raw="0"/>             <t t-out="0"/>
<span t-raw="doc.note"/>   <span t-out="doc.note"/>
```

- `t-out` renderiza HTML si el valor es `markupsafe.Markup`, escapa si es string normal
- Campos HTML (`doc.note`, `doc.comment`) funcionan con `t-out` porque Odoo los marca como `Markup`

## 3. Tax Totals Evolution (v14→v18)

Cambia en cada version major, rompe templates heredados.

### v14: Inline con amount_by_group

```xml
<t t-foreach="doc.amount_by_group" t-as="amount_by_group">
    <tr>
        <td><span t-esc="amount_by_group[0]"/></td>
        <td class="text-right">
            <span t-esc="amount_by_group[1]"
                  t-options='{"widget": "monetary", "display_currency": doc.pricelist_id.currency_id}'/>
        </td>
    </tr>
</t>
```

### v15: JSON string

```xml
<t t-set="tax_totals" t-value="json.loads(doc.tax_totals_json)"/>
<t t-call="account.document_tax_totals"/>
```

### v16-v17: Dict nativo

```xml
<t t-set="tax_totals" t-value="doc.tax_totals"/>
<t t-call="account.document_tax_totals"/>
```

### v18: Templates por modelo + currency obligatorio

```xml
<!-- Sale Order -->
<t t-call="sale.document_tax_totals">
    <t t-set="tax_totals" t-value="doc.tax_totals"/>
    <t t-set="currency" t-value="doc.currency_id"/>
</t>

<!-- Invoice -->
<t t-call="account.document_tax_totals">
    <t t-set="tax_totals" t-value="doc.tax_totals"/>
    <t t-set="currency" t-value="doc.currency_id"/>
</t>

<!-- Purchase -->
<t t-call="purchase.document_tax_totals">
    <t t-set="tax_totals" t-value="doc.tax_totals"/>
    <t t-set="currency" t-value="doc.currency_id"/>
</t>
```

Cambios dict v18: keys `formatted_*` → numeros con widget monetary, `currency` param OBLIGATORIO.

### Templates tax totals por modelo y version

| Version | Sale | Invoice | Purchase |
|---------|------|---------|----------|
| v14 | Inline (amount_by_group) | Inline | Inline |
| v15 | `account.document_tax_totals` | `account.document_tax_totals` | `account.document_tax_totals` |
| v16-v17 | `account.document_tax_totals` | `account.document_tax_totals` | `account.document_tax_totals` |
| v18 | `sale.document_tax_totals` | `account.document_tax_totals` | `purchase.document_tax_totals` |

## 4. layout_document_title (v17→v18)

XPaths a `//h2` dejan de funcionar en v18.

En v14-v17, titulo con `<h2>` directo. En v18, via variable que el layout renderiza:

```xml
<!-- v14-v17 -->
<h2>
    <span t-if="doc.state in ['draft','sent']">Quotation</span>
    <span t-else="">Sales Order</span>
    <span t-field="doc.name"/>
</h2>

<!-- v18 -->
<t t-set="layout_document_title">
    <t t-if="doc.state in ['draft','sent']">Quotation</t>
    <t t-else="">Order</t>
    <span t-field="doc.name"/>
</t>
<!-- El layout renderiza: <h2 t-if="layout_document_title" t-out="layout_document_title"/> -->
```

### Impacto en templates heredados

```xml
<!-- ANTES (v14-v17) -->
<xpath expr="//div[hasclass('page')]/h2" position="replace">
    <h2>MY CUSTOM TITLE <span t-field="doc.name"/></h2>
</xpath>

<!-- DESPUES (v18) -->
<xpath expr="//t[@t-set='layout_document_title']" position="replace">
    <t t-set="layout_document_title">
        MY CUSTOM TITLE <span t-field="doc.name"/>
    </t>
</xpath>
```

## 5. field t-field en reports (v16→v17)

BLOCKING: `<field>` solo valido en form/tree. En reportes v17+, usar `<span>` u otro tag HTML.

```xml
<!-- ANTES (v14-v16) -->          <!-- DESPUES (v17+) -->
<field t-field="doc.name"/>       <span t-field="doc.name"/>
<field t-field="doc.partner_id"/> <span t-field="doc.partner_id"/>
<field t-field="line.price_unit"/>  <span t-field="line.price_unit"/>
```

## 6. External Layout Changes (v14→v18)

### Evolucion del campo de seleccion

| Version | Campo | Tipo |
|---------|-------|------|
| v14-v15 | `external_report_layout` | String ('standard', 'clean', 'boxed', 'bold') |
| v16+ | `external_report_layout_id` | Many2one (ir.ui.view) |

### Clases CSS del layout v18

| v14-v17 | v18 |
|---------|-----|
| `o_standard_header` | (eliminado) |
| `o_standard_footer` | `o_footer_content` |
| `o_background_header` | (eliminado) |
| `o_background_footer` | `o_footer_content` |

Estructura v18:
```xml
<div class="header">
    <div class="o_report_layout_standard"><!-- Logo + company info --></div>
</div>
<div class="article" ...>
    <h2 t-if="layout_document_title" t-out="layout_document_title"/>
    <t t-out="0"/>
</div>
<div class="footer">
    <div class="o_footer_content"><!-- Footer --></div>
</div>
```

### web.address_layout — riesgo de duplicacion

En v17+, si el layout custom llama a `web.address_layout` Y el template define un bloque de direccion, los datos se DUPLICAN. Solucion: no llamar `address_layout` si tienes direccion custom.

## 7. Report Template Changes (sale/invoice/purchase)

### Tabla comparativa por modelo y version

| Elemento | v14 | v15 | v16 | v17 | v18 |
|----------|-----|-----|-----|-----|-----|
| **Titulo** | `<h2>` | `<h2>` | `<h2>` | `<h2>` | `layout_document_title` |
| **Sale address** | `t-set='address'` | `t-set='address'` | `address_layout` | `address_layout` | `#informations` div |
| **Sale lines** | `doc.order_line` | `doc.order_line` | `_get_order_lines_to_report()` | `_get_order_lines_to_report()` | `_get_order_lines_to_report()` |
| **Table class** | `table table-sm o_main_table` | idem | idem | idem | + `mt-4` |
| **Tax labels** | `tax.description` | `tax.description` | `tax.invoice_label or tax.name` | idem | idem |
| **Currency** | `doc.pricelist_id.currency_id` | idem | `doc.currency_id` | idem | idem |
| **Sale discount** | `group_discount_per_so_line` | idem | Eliminado | — | — |
| **Proforma var** | `is_pro_forma` | idem | idem | `is_proforma` | `is_proforma` |
| **Sale notes** | `<p t-field="doc.note">` | idem | `<span name="order_note">` | idem | idem |
| **Invoice payment** | Separado | Separado | `_get_reconciled_info_JSON_values` | idem | `_get_reconciled_vals` |
| **Invoice QR** | No | No | `display_qr_code` | idem | idem |
| **Purchase totals** | — | `json.loads(tax_totals_json)` | `doc.tax_totals` | idem | `purchase.document_tax_totals` + currency |

### XPath targets sale report

```xml
<!-- Funciona en todas las versiones -->
<xpath expr="//div[@id='informations']" position="replace"/>
<xpath expr="//table[hasclass('o_main_table')]" position="replace"/>

<!-- v16+ solamente -->
<xpath expr="//div[@name='so_total_summary']" position="replace"/>
```

### is_proforma vs is_pro_forma

v14-v16: `is_pro_forma` (con underscore). v17+: `is_proforma` (sin underscore, nombre oficial).

### Clases CSS eliminadas en v18

| Clase | Alternativa v18 |
|-------|----------------|
| `border-black` | `border border-dark` o inline style |
| `bg-200` | `bg-light` o color custom |

## 8. XPath Breakage Patterns

### XPaths que ROMPEN entre versiones

| XPath | Rompe en | Causa | Fix |
|-------|----------|-------|-----|
| `//div[hasclass('page')]/h2` | v18 | H2 movido al layout | `//t[@t-set='layout_document_title']` |
| `//t[@t-set='address']` | v16 | Eliminado, usa address_layout | `//div[@id='informations']` |
| `//div[@name='total']` | v16 | Renombrado | `//div[@name='so_total_summary']` |
| `//t[@t-foreach='doc.order_line']` | v16 | Usa `_get_order_lines_to_report()` | Target por clase de tabla |
| `//span[@t-field='doc.amount_total']` | v16 | Tax totals reestructurados | `//div[@name='so_total_summary']` |
| `//p[@t-field='doc.note']` | v16 | Cambiado a span | `//span[@name='order_note']` |

### XPaths SEGUROS (estables v14-v18)

```xml
<xpath expr="//div[@id='informations']" position="replace"/>
<xpath expr="//table[hasclass('o_main_table')]" position="replace"/>
<xpath expr="//div[@name='so_total_summary']" position="replace"/>  <!-- v16+ -->
```

## 9. Report Render API (ir.actions.report)

Firmas de metodos cambian, rompe overrides Python.

### v14/v15 → v16: report_ref como primer parametro (CRITICAL)

```python
# v14/v15:
report = self.env.ref('module.report_action')
pdf_content, report_type = report._render_qweb_pdf(res_ids=[1, 2, 3])

# v16+:
report = self.env['ir.actions.report']
pdf_content, report_type = report._render_qweb_pdf('module.report_action', res_ids=[1, 2, 3])
```

### Firmas por version

| Metodo | v14/v15 | v16+ |
|--------|---------|------|
| `_render_qweb_pdf` | `(self, res_ids=None, data=None)` | `(self, report_ref, res_ids=None, data=None)` |
| `_render_qweb_html` | `(self, docids, data=None)` | `(self, report_ref, docids, data=None)` |
| `_render_qweb_text` | `(self, docids, data=None)` | `(self, report_ref, docids, data=None)` |
| `_render` | `(self, res_ids, data=None)` | `@api.model (self, report_ref, res_ids, data=None)` |

### v16: _post_pdf eliminado

```python
# v14/v15:
def _post_pdf(self, save_in_attachment, pdf_content=None, res_ids=None)

# v16+ reemplazo:
def _render_qweb_pdf_prepare_streams(self, report_ref, data, res_ids=None)
def _prepare_pdf_report_attachment_vals_list(self, report, streams)
```

Cualquier override de `_post_pdf` debe reescribirse completamente para v16+.

### v17: PyPDF2 → odoo.tools.pdf

```python
# v14-v16:                                    # v17+:
from PyPDF2 import PdfFileWriter, ...         from odoo.tools.pdf import PdfFileWriter, PdfFileReader, PdfReadError
```

### v18: cambios en _run_wkhtmltopdf

```python
# v14-v17: def _run_wkhtmltopdf(self, bodies, header=None, footer=None, ...)
# v18:     def _run_wkhtmltopdf(self, bodies, report_ref=False, header=None, footer=None, ...)

# v18 metodos nuevos:
def _pre_render_qweb_pdf(self, report_ref, res_ids=None, data=None)
def _run_wkhtmltoimage(self, bodies, width, height, image_format="jpg")
```

## 10. Asset Loading, Layouts & Paper Format

### Asset loading (v14 → v15+)

```xml
<!-- v14: via XML inheritance -->
<template id="report_assets_inherit" inherit_id="web.report_assets_common">
    <xpath expr="link[last()]" position="after">
        <link rel="stylesheet" type="text/scss" href="/my_module/static/src/scss/report.scss"/>
    </xpath>
</template>
```

```python
# v15+: via __manifest__.py
'assets': {
    'web.report_assets_common': [
        'my_module/static/src/scss/report.scss',
    ],
}
```

Bundles: `web.report_assets_common` (CSS/JS principal), `web.report_assets_pdf` (CSS PDF).
v17+: JS eliminado de `report_assets_common`, solo CSS.

### Layouts renombrados (v14→v15)

| v14 XML ID | v15+ XML ID |
|------------|-------------|
| `web.report_layout_background` | `web.report_layout_striped` |
| `web.report_layout_clean` | `web.report_layout_bold` |
| `web.external_layout_background` | `web.external_layout_striped` |
| `web.external_layout_clean` | `web.external_layout_bold` |

v18 layouts nuevos: `web.report_layout_bubble`, `web.report_layout_wave`, `web.report_layout_folder`.
v18 SCSS eliminados: `layout_background.scss`, `layout_boxed.scss`, `layout_clean.scss`.

### Paper format (report.paperformat)

| Campo | Version | Efecto |
|-------|---------|--------|
| `disable_shrinking` | v15+ | `--disable-smart-shrinking` en wkhtmltopdf |
| `css_margins` | v18+ | Margenes via CSS (`o_css_margins` class) en vez de CLI args |

v18 `css_margins=True`: body recibe clase `o_css_margins`, margenes controlados por `$o-default-report-margins: 11mm`.

## 11. wkhtmltopdf & Company Fields

### wkhtmltopdf — cambios actionables

v18 fix margin-top=0: usa `'key' in dict` en vez de `.get()` — ahora margin-top=0 funciona correctamente.
v18 nuevo: soporte para `data-report-margin-bottom` override.

### Company report fields

| Campo | v15+ | v18 |
|-------|------|-----|
| `company_details` | Html | Html(translate=True) |
| `layout_background` | Selection(Blank/Geometric/Custom) | Selection(Blank/**Demo logo**/Custom) |

**BREAKING**: Registros con valor `'Geometric'` se vuelven invalidos en v18 — debe migrarse a `'Demo logo'`.

## 12. Deteccion Sistematica

Script completo para detectar todos los problemas de migracion QWeb en un modulo:

```bash
#!/bin/bash
# Uso: ./detect_qweb_issues.sh /path/to/module TARGET_VERSION
MODULE=$1
TARGET=${2:-18}

echo "=== QWeb Report Migration Detection ==="
echo "Module: $MODULE | Target: v$TARGET"
echo ""

# 1. Bootstrap 4 classes (v16+)
if [ "$TARGET" -ge 16 ]; then
    echo "--- [1] Bootstrap 4→5 classes ---"
    grep -rn "text-right\|text-left\|pull-right\|pull-left\|table-condensed" \
        --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
    grep -rn "\bml-[0-9]\|\bmr-[0-9]\|\bpl-[0-9]\|\bpr-[0-9]" \
        --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 2. t-raw deprecated (v16+)
if [ "$TARGET" -ge 16 ]; then
    echo "--- [2] t-raw → t-out ---"
    grep -rn "t-raw" --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 3. Tax totals
echo "--- [3] Tax totals ---"
grep -rn "tax_totals_json\|amount_by_group" \
    --include="*.xml" --include="*.py" "$MODULE" 2>/dev/null || echo "  OK"
if [ "$TARGET" -ge 18 ]; then
    grep -rn "document_tax_totals" --include="*.xml" "$MODULE" \
        | grep -v "currency" 2>/dev/null && echo "  WARNING: Missing currency param in v18"
fi

# 4. layout_document_title (v18+)
if [ "$TARGET" -ge 18 ]; then
    echo "--- [4] layout_document_title ---"
    grep -rn 'expr.*//h2\|expr.*page.*h2' --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 5. <field t-field> in reports (v17+)
if [ "$TARGET" -ge 17 ]; then
    echo "--- [5] <field t-field> in reports ---"
    grep -rn "<field t-field\|<field t-esc\|<field t-out" \
        --include="*.xml" "$MODULE/report/" 2>/dev/null || echo "  OK"
fi

# 6. External layout classes (v18+)
if [ "$TARGET" -ge 18 ]; then
    echo "--- [6] External layout old classes ---"
    grep -rn "o_standard_footer\|o_background_footer\|o_standard_header" \
        --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 7. address_layout duplication risk
echo "--- [7] address_layout usage ---"
grep -rn "address_layout" --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"

# 8. is_pro_forma vs is_proforma (v17+)
if [ "$TARGET" -ge 17 ]; then
    echo "--- [8] is_pro_forma → is_proforma ---"
    grep -rn "is_pro_forma" --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 9. Currency reference
echo "--- [9] Currency reference ---"
grep -rn "pricelist_id.currency_id" --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"

# 10. tax.description (v16+)
if [ "$TARGET" -ge 16 ]; then
    echo "--- [10] tax.description → tax.invoice_label ---"
    grep -rn "tax\.description" --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 11. Locale strings sin comillas (v17+)
if [ "$TARGET" -ge 17 ]; then
    echo "--- [11] Locale strings sin comillas ---"
    grep -rn "== en_US\|== es_ES\|== fr_FR" --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 12. website_id.name (v16+)
if [ "$TARGET" -ge 16 ]; then
    echo "--- [12] website_id.name → website ---"
    grep -rn "website_id\.name" --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 13. Report render API (v16+)
if [ "$TARGET" -ge 16 ]; then
    echo "--- [13] Report render API ---"
    grep -rn "_post_pdf\|def _render_qweb_pdf" --include="*.py" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 14. PyPDF2 import (v17+)
if [ "$TARGET" -ge 17 ]; then
    echo "--- [14] PyPDF2 → odoo.tools.pdf ---"
    grep -rn "from PyPDF2\|import PyPDF2" --include="*.py" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 15. Assets via XML template (v15+)
if [ "$TARGET" -ge 15 ]; then
    echo "--- [15] Assets via XML ---"
    grep -rn "inherit_id.*report_assets_common\|inherit_id.*report_assets_pdf" \
        --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

# 16. Layout XML IDs renombrados (v15+)
if [ "$TARGET" -ge 15 ]; then
    echo "--- [16] Layout IDs renombrados ---"
    grep -rn "external_layout_background\|external_layout_clean" \
        --include="*.xml" "$MODULE" 2>/dev/null || echo "  OK"
fi

echo ""
echo "=== Detection complete ==="
```
