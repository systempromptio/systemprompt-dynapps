# Report Structure — Odoo QWeb PDF

## 1. Report Action (`ir.actions.report`)

```xml
<record id="action_report_sale_order" model="ir.actions.report">
    <field name="name">Sale Order</field>
    <field name="model">sale.order</field>
    <field name="report_type">qweb-pdf</field>
    <field name="report_name">sale_custom.report_sale_order</field>
    <field name="report_file">sale_custom.report_sale_order</field>
    <field name="binding_model_id" ref="sale.model_sale_order"/>
    <field name="binding_type">report</field>
    <field name="paperformat_id" ref="paperformat_euro_custom"/>
</record>
```

Campos críticos:
| Campo | Descripción |
|-------|-------------|
| `report_type` | SIEMPRE `qweb-pdf` para PDF. `qweb-html` para previsualización HTML |
| `report_name` | Debe coincidir EXACTAMENTE con el `id` de la plantilla QWeb (`module.template_id`) |
| `report_file` | Ruta al fichero XML de la plantilla (sin extensión). Generalmente igual a `report_name` |
| `binding_model_id` | Modelo al que se adjunta el botón "Imprimir" en la UI |
| `binding_type` | `report` para reportes; `action` para acciones de servidor |
| `paperformat_id` | Referencia al paper format; si se omite, usa el de la empresa |

## 2. Paper Format (`report.paperformat`)

```xml
<record id="paperformat_euro_custom" model="report.paperformat">
    <field name="name">Custom A4 Portrait</field>
    <field name="default" eval="False"/>
    <field name="format">A4</field>
    <field name="page_height">0</field>
    <field name="page_width">0</field>
    <field name="orientation">Portrait</field>
    <field name="margin_top">40</field>
    <field name="margin_bottom">23</field>
    <field name="margin_left">7</field>
    <field name="margin_right">7</field>
    <field name="header_line" eval="False"/>
    <field name="header_spacing">35</field>
    <field name="dpi">90</field>
</record>
```

Valores de `format`: `A4`, `A3`, `A5`, `Letter`, `Legal`, `custom`.
Para `custom` especificar `page_height` y `page_width` en mm.
`header_spacing`: espacio en mm para el header de wkhtmltopdf. Aumentar si
el header se superpone al cuerpo.

## 3. Estructura de ficheros en el módulo

```
my_module/
├── __manifest__.py
├── models/
│   └── report_sale_order.py      # AbstractModel (opcional, solo si hay lógica Python)
└── report/
    ├── report_sale_order.xml               # ir.actions.report + paperformat
    └── report_sale_order_template.xml      # plantilla QWeb
```

Separar el record action/paperformat de la plantilla QWeb facilita la herencia.

## 4. Declaración en `__manifest__.py`

```python
'data': [
    # ANTES que vistas que añaden botones de impresión
    'report/report_sale_order.xml',
    'report/report_sale_order_template.xml',
],
```

## 5. AbstractModel Python (solo si se necesita lógica personalizada)

```python
# models/report_sale_order.py  — v16 y anterior
class ReportSaleOrder(models.AbstractModel):
    _name = 'report.sale_custom.report_sale_order'  # DEBE ser report.<report_name>
    _description = 'Sale Order Report'

    @api.model
    def _get_report_values(self, docids, data=None):
        docs = self.env['sale.order'].browse(docids)
        return {'doc_ids': docids, 'doc_model': 'sale.order', 'docs': docs}
```

El nombre del método es `_get_report_values(self, docids, data=None)` en
v16, v17, v18 y v19 (no cambia). `_get_report_data` NO existe.

Sin AbstractModel, Odoo inyecta automáticamente `doc` (registro) y `docs`
(recordset) en el contexto QWeb.

## 6. Variables disponibles en el contexto QWeb

| Variable | Descripción |
|----------|-------------|
| `doc` | Registro actual (mono-registro) |
| `docs` | Recordset completo seleccionado |
| `doc_ids` | Lista de IDs |
| `user` | Usuario que genera el informe |
| `company` | Empresa activa |
| `time` | Módulo Python `time` |
| `datetime` | `safe_eval.datetime` (global de `ir.qweb`, `odoo/addons/base/models/ir_qweb.py:954`, `IrQWeb._prepare_environment`) |
| `context_timestamp` | **SOLO en informes** — inyectado por `ir.actions.report` (`odoo/addons/base/models/ir_actions_report.py:768`, `IrActionsReport._render_template`); convierte a la tz del usuario. `context_timestamp(datetime.datetime.now()).strftime('…')`. NO está en vistas QWeb normales |
| `image_data_uri` | Global de `ir.qweb` (`odoo/addons/base/models/ir_qweb.py:956`, `IrQWeb._prepare_environment`); convierte binario base64 → `data:` URI para `<img t-att-src>` (imprescindible en wkhtml) |

## 7. Resumen de diferencias v17/v18/v19

| Aspecto | v16- | v17+ |
|---------|------|------|
| Salida HTML | `t-raw` (eliminado) / `t-esc` | `t-out` |
| Método Python | `_get_report_values` | `_get_report_values` (sin cambios) |
| `group_operator` | soportado | `aggregator` en v18+ |

Confirmar siempre con `odoo-version-api-reference`.

## 8. Anatomía de un informe estándar de Odoo (verificado: `sale.order`, v18)

Un informe estándar NO es una plantilla monolítica. El patrón real (`addons/sale/report/ir_actions_report_templates.xml`):

```xml
<!-- Envoltorio fino: solo pagina y delega -->
<template id="report_saleorder">
  <t t-call="web.html_container">
    <t t-foreach="docs" t-as="doc">
      <t t-call="sale.report_saleorder_document" t-lang="doc.partner_id.lang"/>
    </t>
  </t>
</template>

<!-- El CONTENIDO vive en el _document (aquí es donde se hereda) -->
<template id="report_saleorder_document">
  <t t-call="web.external_layout">            <!-- membrete empresa -->
    <t t-set="doc" t-value="doc.with_context(lang=lang)"/>
    <t t-set="lines_to_report" t-value="doc._get_order_lines_to_report()"/>  <!-- ⚠ var computada -->
    ...
    <t t-foreach="lines_to_report" t-as="line"> ... </t>   <!-- NO doc.order_line directo -->
    ...
    <table class="o_total_table ...">
      <t t-call="sale.document_tax_totals">
        <t t-set="tax_totals" t-value="doc.tax_totals"/>   <!-- ⚠ widget calculado -->
      </t>
    </table>
  </t>
</template>
```

Consecuencias al **extender o clonar** un informe estándar:
- **`report_X` = envoltorio** (`web.html_container` + `t-foreach docs` + `t-call report_X_document`). Todo el contenido y las herencias cuelgan del **`_document`**, no del envoltorio.
- **Líneas ≠ `doc.order_line`**: se usa una **variable computada** (`lines_to_report = doc._get_order_lines_to_report()`) que ordena/filtra secciones, notas y combos. Si copias y pones `doc.order_line` a pelo pierdes ese orden.
- **Totales = `o_total_table` + widget `tax_totals`**: es un bloque **calculado** (`doc.tax_totals`), no campos sueltos. No lo reconstruyas con `sum(...)` a mano salvo que quieras perder el desglose de impuestos por grupo.
- Para **heredar**, apunta el `xpath` al `_document` (donde está `<div class="page">`), no al envoltorio.

## 9. Modificar un informe estándar = HERENCIA, nunca copia

Copiar un informe estándar **degrada la fidelidad** (te pierdes `_document`, `lines_to_report`, `tax_totals`, futuras mejoras del core). La vía correcta:

1. **Extender el arch** con una vista `mode="extension"` (`inherit_id` → la vista `_document`), xpath contra `//div[hasclass('page')]` o un ancla `name=`/clase única.
2. **Cambiar qué informe se imprime** desde el menú "Imprimir" = gestionar el **binding**, no tocar la plantilla:
   - El botón Imprimir sale de `ir.actions.report.binding_model_id` + `binding_type='report'` (verificado `odoo/addons/base/models/ir_actions.py:68`, `IrActions.binding_model_id`, y `:70`, `IrActions.binding_type`).
   - "Sustituir" el estándar = **desvincular** el report estándar (poner su `binding_model_id`/`binding_type` a vacío) y bindear el tuyo. Es **reversible** (restauras el binding original), y deja el report estándar intacto.
3. Nunca dupliques la plantilla entera para un cambio pequeño: una extensión xpath es reversible y sobrevive a las actualizaciones del core.

Confirmar siempre con `odoo-version-api-reference`.
