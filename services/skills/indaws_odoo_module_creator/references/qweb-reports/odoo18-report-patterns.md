# Patrones de Reports en Odoo 18 (Sale, Purchase, Invoice)

Analisis del codigo fuente real de los 3 reports principales de Odoo 18.
Fuente: `sale/report/`, `purchase/report/`, `account/views/report_invoice.xml`.

## 1. Estructura del template

### Cadena de llamadas

```
report_saleorder (wrapper)
  -> report_saleorder_raw (loop docs, set lang)
    -> report_saleorder_document (t-call="web.external_layout")
      -> layout_document_title (titulo)
      -> address (direcciones)
      -> table o_main_table (lineas)
      -> document_tax_totals (totales)
      -> notes
```

### Titulo del documento (v18+)

**NUNCA usar `<h2>` directo**. Usar variable `layout_document_title`:

```xml
<t t-set="layout_document_title">
    <span t-if="doc.state in ['draft','sent']">Quotation # </span>
    <span t-else="">Order # </span>
    <span t-field="doc.name"/>
</t>
```

El layout `external_layout_*` lo renderiza como `<h2 t-out="layout_document_title"/>`.

## 2. Tabla de lineas

### Clases de la tabla

```xml
<table class="o_has_total_table table o_main_table table-borderless">
    <thead style="display: table-row-group">
```

| Clase | Proposito |
|-------|-----------|
| `table` | Bootstrap base |
| `o_main_table` | Marcador Odoo para la tabla principal |
| `table-borderless` | Quita bordes por defecto de Bootstrap |
| `o_has_total_table` | Conecta visualmente con `o_total_table` en tema boxed |

**Los bordes vienen del tema CSS** (`.o_table_standard`, `.o_table_bold`, etc.),
NO de la tabla misma. Por eso usan `table-borderless`.

**CRITICO para reports custom**: `table-borderless` es OBLIGATORIO incluso si usas
inline styles para bordes. Sin ella, BS5 genera `border-bottom-width: var(--border-width)`
que wkhtmltopdf no resuelve → crea rectangulos de altura 0 en el PDF → Chrome los
muestra como lineas finas horizontales fantasma. `table-borderless` pone `border: 0 none`
que los elimina, y los inline styles (`t-att-style`) ganan por especificidad.

### `thead` con repeticion en paginas

```xml
<thead style="display: table-row-group">
```

Inline style obligatorio — hace que el header se repita en cada pagina del PDF.

### Columnas con `name` attribute

```xml
<th name="th_description" class="text-start">Description</th>
<td name="td_name" class="text-start">...</td>
```

El atributo `name` permite herencia via xpath y edicion en Studio.

### Secciones y notas

```xml
<tr t-att-class="'fw-bold o_line_section' if line.display_type == 'line_section'
                 else 'fst-italic o_line_note' if line.display_type == 'line_note'
                 else ''">
    <t t-if="line.display_type == 'line_section'">
        <td colspan="99"><span t-field="line.name"/></td>
    </t>
    <t t-elif="line.display_type == 'line_note'">
        <td colspan="99"><span t-field="line.name"/></td>
    </t>
    <t t-elif="line.display_type == 'product'" name="account_invoice_line_accountable">
        <!-- linea normal -->
    </t>
</tr>
```

Usa `colspan="99"` (no el conteo exacto de columnas) para secciones/notas.

### Descuento condicional

```xml
<!-- Variable global -->
<t t-set="display_discount" t-value="any(l.discount for l in lines_to_report)"/>

<!-- Header -->
<th name="th_discount" t-if="display_discount" class="text-end">Disc.%</th>

<!-- Celda -->
<td t-if="display_discount" class="text-end">
    <span t-field="line.discount"/>
</td>
```

## 3. Valores monetarios — 3 patrones

### Patron A: `t-field` sin opciones (el mas comun)

```xml
<span t-field="line.price_unit">3</span>
<span t-field="line.price_subtotal">27.00</span>
```

Odoo auto-formatea usando el `currency_field` definido en el modelo.
Los numeros de ejemplo (3, 27.00) son placeholders para Studio/previsualizacion.

### Patron B: `t-field` con widget monetary explicito

```xml
<span t-field="line.price_subtotal"
      t-options='{"widget": "monetary", "display_currency": o.currency_id}'/>
```

Usado en Purchase Order. Fuerza la moneda del documento.

### Patron C: `t-out` con widget monetary (para valores computados)

```xml
<span t-out="current_subtotal"
      t-options='{"widget": "monetary", "display_currency": doc.currency_id}'/>
```

Usado para subtotales de seccion y valores de `tax_totals` dict.
`t-out` no escapa HTML (a diferencia de `t-esc`).

### Recomendacion para reports custom con wkhtmltopdf

**Si el widget monetary produce artefactos** (EUR, acentos corruptos),
usar un metodo helper que formatea sin simbolo:

```xml
<span t-out="doc.format_monetary(line.price_subtotal)"/>
```

Ver seccion de workarounds en `wkhtmltopdf-gotchas.md`.

## 4. Impuestos en lineas

### Patron estandar (SO, PO, Invoice)

```xml
<t t-set="taxes" t-value="', '.join([(tax.invoice_label or tax.name) for tax in line.tax_id])"/>
<span t-out="taxes"/>
```

- Prioriza `invoice_label` sobre `name`
- Si no hay `invoice_label`, usa `name` (ej: "21% B")
- Se une con coma si hay multiples impuestos
- Renderiza con `t-out` (string plano, no campo ORM)

**Nota**: el nombre del campo varia:
- `sale.order.line`: `tax_id`
- `purchase.order.line`: `taxes_id`
- `account.move.line`: `tax_ids`

## 5. Totales de impuestos (tax_totals)

### SIEMPRE usar el template estandar

```xml
<table class="o_total_table table table-borderless">
    <t t-call="sale.document_tax_totals">
        <t t-set="tax_totals" t-value="doc.tax_totals"/>
        <t t-set="currency" t-value="doc.currency_id"/>
    </t>
</table>
```

Cada modulo tiene su copia `primary`:
- `sale.document_tax_totals`
- `purchase.document_tax_totals`
- `account.document_tax_totals` (el original)

### Estructura del template de totales

```
Subtotal row (o_subtotal) — base imponible
  Tax group row (o_taxes) — nombre IVA + importe
  Tax group row (o_taxes) — si hay mas grupos
[Cash rounding row]
Total row (o_total) — total
```

### Si necesitas formato custom (como Eurolotes)

Iterar manualmente `tax_totals`:

```xml
<t t-set="tax_totals" t-value="doc.tax_totals"/>
<t t-set="all_tax_groups" t-value="[]"/>
<t t-foreach="tax_totals.get('subtotals', [])" t-as="subtotal">
    <t t-foreach="subtotal.get('tax_groups', [])" t-as="tg">
        <t t-esc="all_tax_groups.append(tg) or ''"/>
    </t>
</t>
```

Campos disponibles en cada `tg`:
- `group_name`: "IVA 21%"
- `base_amount_currency`: base imponible
- `display_base_amount_currency`: base para mostrar (puede diferir si `same_tax_base`)
- `tax_amount_currency`: importe del impuesto

## 6. Direcciones

### Widget contact

```xml
<div t-field="doc.partner_id"
     t-options='{"widget": "contact",
                 "fields": ["address", "name", "phone", "vat"],
                 "no_marker": True}'/>
```

Campos disponibles: `address`, `name`, `phone`, `mobile`, `email`, `vat`, `fax`, `website`.

### Doble direccion (facturacion + envio)

```xml
<div class="col-6">
    <strong>Invoice Address:</strong>
    <div t-field="doc.partner_invoice_id" t-options='{...}'/>
</div>
<div class="col-6" t-if="doc.partner_shipping_id != doc.partner_invoice_id">
    <strong>Delivery Address:</strong>
    <div t-field="doc.partner_shipping_id" t-options='{...}'/>
</div>
```

## 7. Bloque informativo

### Patron row con columnas auto

```xml
<div class="row mb-4" id="informations">
    <div t-if="doc.client_order_ref" class="col" name="info_reference">
        <strong>Your Reference</strong>
        <div t-field="doc.client_order_ref"/>
    </div>
    <div t-if="doc.date_order" class="col" name="info_date">
        <strong>Order Date</strong>
        <div t-field="doc.date_order" t-options='{"widget": "date"}'/>
    </div>
    <div t-if="doc.user_id" class="col" name="info_salesperson">
        <strong>Salesperson</strong>
        <div t-field="doc.user_id"/>
    </div>
</div>
```

Usa `class="col"` (auto-width) — las columnas se distribuyen equitativamente.

## 8. Inline styles — uso minimo en Odoo estandar

Solo 2 usos de inline styles en los 3 reports:

1. `<thead style="display: table-row-group">` — repeticion header en paginas
2. `<span t-attf-style="text-align:justify" ...>` — en notas condicional

**Todo lo demas es CSS via clases**. Los temas de layout
(`.o_table_standard`, `.o_table_bold`, etc.) controlan los bordes.

## 9. Temas de tabla disponibles

Clase en `<div class="article">`:

| Tema | Clase | Bordes |
|------|-------|--------|
| Standard | `o_table_standard` | Solo borde inferior en thead |
| Bold | `o_table_bold` | Bordes gruesos arriba/abajo thead, abajo ultima fila |
| Striped | `o_table_striped` | Separadores ligeros, fondo alterno en filas |
| Boxed | `o_table_boxed` | Caja completa con separadores internos |
| Boxed-rounded | `o_table_boxed-rounded` | Como boxed con esquinas redondeadas |

Para excluir una tabla del tema: `class="o_ignore_layout_styling"`.

## 10. `t-esc` vs `t-out` vs `t-field`

| Directiva | Uso | Escapa HTML | Ejemplo |
|-----------|-----|-------------|---------|
| `t-field` | Campos ORM del registro | No (usa widget) | `t-field="line.price_unit"` |
| `t-out` | Valores computados, HTML markup | No | `t-out="subtotal"` con `t-options` monetary |
| `t-esc` | Strings planos | Si | `t-esc="company.vat"` |

**Para valores monetarios** de dicts (como `tax_totals`): usar `t-out` + `t-options='{"widget": "monetary"}'`.

**Para campos ORM**: usar `t-field` sin opciones (el campo sabe su moneda).

**Para strings**: usar `t-esc` (seguro contra XSS).

## 11. Reports multi-company

### Patron: report custom por compania sin binding

Los reports custom NO deben tener `binding_model_id` — se inyectan via override
de `get_bindings` cuando la compania los tiene configurados.

```python
# En res.company:
report_sale_id = fields.Many2one('ir.actions.report',
    domain=[('model', '=', 'sale.order'), ('report_type', '=', 'qweb-pdf')])

# En ir.actions.actions:
_CUSTOM_REPORT_FIELDS = {
    'sale.order': 'report_sale_id',
    'account.move': 'report_invoice_id',
    'stock.picking': 'report_picking_id',
}

@api.model
def get_bindings(self, model_name):
    result = super().get_bindings(model_name)
    field_name = self._CUSTOM_REPORT_FIELDS.get(model_name)
    if not field_name:
        return result
    custom_report = self.env.company[field_name]
    if not custom_report:
        return result
    # Inject custom report into Print menu
    action_data = custom_report.sudo().read(['name', 'binding_view_types', 'type'])[0]
    result.setdefault('report', []).append(action_data)
    return result
```

Ventajas:
- Reports estandar siempre visibles (no se tocan)
- Custom aparece SOLO en la compania configurada
- Configuracion en Ajustes Generales (res.config.settings)
- Portal: override controller para usar `company.report_sale_id`

### Portal multi-company

```python
class CustomerPortalReport(CustomerPortal):
    def _get_sale_report_ref(self, order):
        custom = order.company_id.report_sale_id
        return custom.report_name if custom else 'sale.action_report_saleorder'
```
