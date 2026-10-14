# Anatomía de un Report QWeb-PDF en Odoo

Un report PDF en Odoo tiene 4 componentes. Todos son necesarios excepto el Python class (opcional).

## 1. ir.actions.report (XML record) — OBLIGATORIO

Define el binding entre modelo y template. Va en `report/` o `views/` del módulo.

```xml
<record id="report_delivery_slip_valued" model="ir.actions.report">
    <field name="name">Valued Delivery Slip</field>
    <field name="model">stock.picking</field>
    <field name="report_type">qweb-pdf</field>
    <field name="report_name">my_module.report_delivery_slip_valued</field>
    <field name="report_file">my_module.report_delivery_slip_valued</field>
    <field name="binding_model_id" ref="stock.model_stock_picking"/>
    <field name="binding_type">report</field>
    <field name="paperformat_id" ref="my_module.paperformat_delivery_slip"/>
    <field name="print_report_name">'Delivery Slip - %s' % object.name</field>
    <field name="attachment">('Delivery-%s' % object.name) + '.pdf'</field>
    <field name="attachment_use" eval="True"/>
</record>
```

### Campos clave

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `name` | Char | Nombre visible en menú "Imprimir" |
| `model` | Char | Modelo de los registros (ej: `sale.order`) |
| `report_type` | Selection | `qweb-pdf` (PDF), `qweb-html` (HTML), `qweb-text` (texto plano) |
| `report_name` | Char | XML ID del template QWeb (module.template_id) |
| `report_file` | Char | Normalmente igual a `report_name` |
| `binding_model_id` | Many2one | Ref al modelo — añade botón "Imprimir" en la vista |
| `binding_type` | Selection | `report` (menú Imprimir), `action` (menú Acción) |
| `paperformat_id` | Many2one | Paper format personalizado (opcional, default A4) |
| `print_report_name` | Char | Expresión Python para nombre del fichero descargado |
| `attachment` | Char | Expresión Python — si se evalúa, guarda PDF como adjunto |
| `attachment_use` | Boolean | Si True, reutiliza adjunto existente (no regenera) |
| `groups_id` | Many2many | Grupos de seguridad que pueden ver el report |

### report_type opciones

| Tipo | Uso | Engine |
|------|-----|--------|
| `qweb-pdf` | Documentos para imprimir/enviar (facturas, albaranes) | QWeb → HTML → wkhtmltopdf → PDF |
| `qweb-html` | Previsualizaciones, dashboards | QWeb → HTML (se muestra en navegador) |
| `qweb-text` | Ficheros de texto (CSV, TXT) | QWeb → texto plano |

## 2. QWeb Template (XML) — OBLIGATORIO

El template HTML/QWeb que define el contenido del report. Va en `report/` del módulo.

### Estructura mínima

```xml
<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <template id="report_delivery_slip_valued">
        <t t-call="web.html_container">
            <t t-foreach="docs" t-as="doc">
                <t t-call="web.external_layout">
                    <div class="page">
                        <!-- Contenido del report -->
                    </div>
                </t>
            </t>
        </t>
    </template>
</odoo>
```

### Variables disponibles en el template

| Variable | Tipo | Descripción |
|----------|------|-------------|
| `docs` | Recordset | Todos los registros seleccionados |
| `doc` | Record | Registro actual (dentro del `t-foreach`) |
| `doc_ids` | List[int] | IDs de los registros |
| `doc_model` | str | Nombre del modelo (`sale.order`) |
| `time` | module | Módulo Python `time` |
| `user` | Record | Usuario actual (`res.users`) |
| `company` | Record | Compañía actual (`res.company`) |
| `data` | dict | Data extra pasada al report (si se invocó con data) |

### Containers y layouts

| Template | Cuándo usar |
|----------|-------------|
| `web.html_container` | **Siempre** — wrapper raíz con CSS y assets |
| `web.external_layout` | Documentos externos (facturas, albaranes) — añade header/footer empresa |
| `web.internal_layout` | Documentos internos — header simplificado |
| `web.minimal_layout` | Sin header/footer — solo el contenido |

## 3. Python Report Class (OPCIONAL)

Solo necesaria si el template necesita datos que no están directamente en el modelo. Hereda de `models.AbstractModel`.

```python
from odoo import api, models


class ReportDeliverySlipValued(models.AbstractModel):
    _name = 'report.my_module.report_delivery_slip_valued'
    _description = 'Valued Delivery Slip Report'

    @api.model
    def _get_report_values(self, docids, data=None):
        docs = self.env['stock.picking'].browse(docids)
        return {
            'doc_ids': docids,
            'doc_model': 'stock.picking',
            'docs': docs,
            'data': data,
            # Custom data:
            'total_value': sum(doc.valued_amount for doc in docs),
            'company_vat': docs[0].company_id.vat,
        }
```

### Cuándo crear Python class

| Situación | ¿Necesita Python class? |
|-----------|------------------------|
| Campos directos del modelo | No — usa `doc.field_name` |
| Campos related/computed | No — ya disponibles via ORM |
| Agregaciones, totales custom | **Sí** — calcula en `_get_report_values` |
| Datos de otros modelos no relacionados | **Sí** |
| Lógica de agrupación/ordenamiento compleja | **Sí** |
| Solo mostrar campos del registro | No |

### Naming convention

El `_name` del AbstractModel DEBE coincidir con `report_name` del action:
- Action: `report_name="my_module.report_delivery_slip_valued"`
- Model: `_name = 'report.my_module.report_delivery_slip_valued'`

Si no coinciden, Odoo usa el pipeline genérico (sin datos custom).

## 4. Paper Format (OPCIONAL)

Solo necesario si A4 portrait con márgenes estándar no es suficiente.

```xml
<record id="paperformat_delivery_slip" model="report.paperformat">
    <field name="name">Delivery Slip Format</field>
    <field name="default" eval="False"/>
    <field name="format">A4</field>
    <field name="orientation">Portrait</field>
    <field name="margin_top">25</field>
    <field name="margin_bottom">25</field>
    <field name="margin_left">7</field>
    <field name="margin_right">7</field>
    <field name="header_line" eval="False"/>
    <field name="header_spacing">20</field>
    <field name="dpi">90</field>
    <field name="disable_shrinking" eval="True"/>
</record>
```

### Campos de paper format

| Campo | Tipo | Default | Descripción |
|-------|------|---------|-------------|
| `format` | Selection | A4 | A0-A9, B0-B10, Letter, Legal, Tabloid, custom |
| `orientation` | Selection | Portrait | Portrait / Landscape |
| `margin_top` | Integer | 40 | Margen superior en mm |
| `margin_bottom` | Integer | 20 | Margen inferior en mm |
| `margin_left` | Integer | 7 | Margen izquierdo en mm |
| `margin_right` | Integer | 7 | Margen derecho en mm |
| `header_spacing` | Integer | 35 | Espacio entre header y contenido en mm |
| `header_line` | Boolean | False | Línea separadora bajo el header |
| `dpi` | Integer | 90 | Resolución (afecta tamaño de imágenes) |
| `disable_shrinking` | Boolean | False | Desactiva auto-shrink de wkhtmltopdf |
| `page_height` | Integer | 0 | Altura custom en mm (solo si format=custom) |
| `page_width` | Integer | 0 | Anchura custom en mm (solo si format=custom) |

### v18+: CSS Margins

Si `css_margins=True` en el paper format, los márgenes se controlan via CSS:
```scss
// Variable SCSS (default: 11mm)
$o-default-report-margins: 11mm;

// Clase aplicada al body
.o_css_margins {
    margin: $o-default-report-margins;
}
```

En este modo, los campos `margin_*` del paper format se ignoran.

## Estructura de ficheros en el módulo

```
my_module/
├── __manifest__.py      # data: ['report/report_actions.xml', 'report/report_templates.xml']
├── models/
│   └── ...
├── report/
│   ├── __init__.py      # (si hay Python class)
│   ├── report_actions.xml        # ir.actions.report + report.paperformat
│   ├── report_templates.xml      # QWeb templates
│   └── report_delivery_slip.py   # (opcional) Python report class
└── static/
    └── src/
        └── scss/
            └── report.scss       # (opcional) CSS custom
```

En `__manifest__.py`:
```python
'data': [
    'report/report_actions.xml',
    'report/report_templates.xml',
],
'assets': {
    'web.report_assets_common': [
        'my_module/static/src/scss/report.scss',
    ],
},
```
