<!-- verified: v18.0 @ 2026-06 -->
# QWeb Patterns para Informes PDF Odoo

## 1. Estructura base de la plantilla

### Con cabecera de empresa (web.external_layout) — el más común
```xml
<template id="report_sale_order" inherit_id="False">
    <t t-call="web.external_layout">
        <t t-set="o" t-value="doc"/>
        <div class="page">
            <!-- cuerpo del informe -->
        </div>
    </t>
</template>
```

`web.external_layout` añade: logo empresa, nombre, dirección, número de página.
`web.basic_layout` añade solo el DOCTYPE y estilos base de Odoo, sin cabecera empresa.

## 2. t-out vs t-esc (v17+)

**Las dos hacen lo mismo — MEDIDO en v18 y v19** (`tools/bench-orm::TestReportQweb`, mismo
documento renderizado con las cuatro combinaciones):

| | `t-out` | `t-esc` |
|---|---|---|
| campo `Html` (`<b>en negrita</b>`) | pinta el `<b>` | **pinta el `<b>` también** |
| campo `Char` con el mismo texto | **escapa** (`&lt;b&gt;`) | escapa |

Lo que decide si se escapa **no es la directiva: es el valor**. Si viene marcado como seguro
(`markupsafe.Markup`) se pinta; si es un `str` pelado, se escapa. Y quien lo marca es el **tipo de
campo**: un `Html` devuelve `Markup`, un `Char` devuelve `str`.

Dos consecuencias prácticas:

1. **Cambiar el tipo del campo cambia lo que se imprime, sin tocar la plantilla.** Pasar una nota de
   `Html` a `Char` —o al revés— hace que el mismo `t-out` escape o pinte. No hay error; el informe
   sale distinto.
2. **Elegir `t-esc` no protege de nada** si el valor ya es `Markup`. Para forzar texto plano hay que
   actuar sobre el valor (p. ej. pasar por `str()` o limpiar el HTML), no sobre la directiva.

Es el mismo mecanismo que en `message_post`, donde un `str` con HTML se escapa y un `Markup` no
(`odoo-mail-integration`). Regla común: **el escape es propiedad del valor, no de quien lo pinta.**

Guía de uso, entonces, por convención y legibilidad (no por comportamiento):

| Directiva | Usar para |
|-----------|-----------|
| `t-out` | todo — es la forma canónica desde v16 |
| `t-esc` | nada nuevo; sobrevive por compatibilidad |

`t-raw` está deprecado (lanza warning) pero **sigue funcionando en v18 y tambien en v19** — usar
`t-out`. El compilador lo trata igual en las dos (`odoo/addons/base/models/ir_qweb.py:2001-2002`, `IrQWeb._compile_directive_out` v18, `:2372-2373` v19) y el
aviso es el mismo: *"Found deprecated directive @t-raw=%r in template %r. Replace by ..."*
(v18 `:2118`, v19 `:2499`). O sea: **no lo quites por miedo a que v19 lo rompa** — quitalo porque
ensucia el log y porque `t-out` es lo correcto.

```xml
<span t-out="o.name"/>                    <!-- string simple -->
<div t-out="o.note"/>                     <!-- campo html -->
<span t-out="len(o.order_line)"/>         <!-- expresión Python -->
```

## 3. Loops y condicionales

```xml
<!-- Loop básico -->
<tbody>
    <tr t-foreach="o.order_line" t-as="line">
        <td t-out="line.name"/>
        <td t-out="line.product_uom_qty"/>
    </tr>
</tbody>
```

Variables de loop disponibles: `line_index` (0-based), `line_first`, `line_last`,
`line_odd`, `line_even` (para colorear filas alternas).

```xml
<!-- Condicionales -->
<t t-if="o.payment_term_id">
    <p t-out="o.payment_term_id.name"/>
</t>
<t t-elif="o.note">
    <p t-out="o.note"/>
</t>
<t t-else="">
    <p>Sin condiciones de pago</p>
</t>
```

## 4. Widgets de campo (t-options)

```xml
<!-- Moneda / precio -->
<span t-out="o.amount_total"
      t-options='{"widget": "monetary", "display_currency": o.currency_id}'/>

<!-- Fecha -->
<span t-out="o.date_order" t-options='{"widget": "date"}'/>

<!-- Fecha y hora -->
<span t-out="o.write_date" t-options='{"widget": "datetime"}'/>

<!-- Contacto (dirección formateada) -->
<div t-field="o.partner_id"
     t-options='{"widget": "contact",
                 "fields": ["address", "name", "phone", "email"]}'/>
```

## 5. Totales y subtotales

```xml
<!-- Suma inline con t-set -->
<t t-set="total_discount"
   t-value="sum(l.price_subtotal for l in o.order_line if l.discount)"/>

<tfoot>
    <tr>
        <td colspan="3">Subtotal con descuento</td>
        <td>
            <span t-out="total_discount"
                  t-options='{"widget": "monetary",
                              "display_currency": o.currency_id}'/>
        </td>
    </tr>
</tfoot>
```

Agrupación por categoría:
```xml
<t t-set="categories" t-value="o.order_line.mapped('product_id.categ_id')"/>
<t t-foreach="categories" t-as="categ">
    <tr class="group-header">
        <td colspan="4" t-out="categ.name"/>
    </tr>
    <t t-foreach="o.order_line.filtered(lambda l: l.product_id.categ_id == categ)"
       t-as="line">
        <tr><td t-out="line.name"/></tr>
    </t>
</t>
```

## 6. Dirección personalizada (t-set="address" → web.address_layout)

`web.external_layout` no tiene slots `address_header`/`address_footer`. Para
sustituir el bloque de dirección, define `t-set="address"`; Odoo lo renderiza
mediante el subtemplate `web.address_layout`.

```xml
<t t-call="web.external_layout">
    <t t-set="address">
        <div class="custom-address">
            <span t-out="o.partner_id.name"/>
        </div>
    </t>
    <div class="page">
        <!-- cuerpo -->
    </div>
</t>
```

## 7. Saltos de página

```xml
<!-- Salto ANTES del elemento -->
<div style="page-break-before: always;"><!-- nueva página --></div>

<!-- Salto DESPUÉS del elemento -->
<div style="page-break-after: always;"><!-- contenido --></div>

<!-- Intentar mantener unido (no garantizado en wkhtmltopdf) -->
<tr style="page-break-inside: avoid;"><td>...</td></tr>
```

## 8. Códigos de barras

```xml
<!-- Ruta del controlador de barcodes de Odoo — SÍ funciona en reports QWeb -->
<img t-att-src="'/report/barcode/Code128/' + o.name"
     style="width:200px; height:50px;"/>
```

Rutas disponibles: `/report/barcode/Code128/<val>`, `/report/barcode/QR/<val>`,
`/report/barcode/EAN13/<val>`.

El controlador `/report/barcode/...` lo sirve Odoo y es válido en reports (el
docstring oficial de `web/controllers/report.py` lo documenta así). El consejo
de usar base64 aplica a `/web/image/...`, no a este controlador de barcode.

## 9. Imágenes

```xml
<!-- Logo empresa (siempre base64 — no URL) -->
<img t-if="company.logo"
     t-att-src="'data:image/png;base64,' + company.logo.decode('utf-8')"
     style="max-height:80px;"/>

<!-- Imagen de campo Binary en el modelo -->
<img t-if="o.product_id.image_128"
     t-att-src="'data:image/png;base64,' + o.product_id.image_128.decode('utf-8')"
     style="width:50px; height:50px; object-fit:contain;"/>
```

NUNCA usar URLs relativas (`/web/image/...`) en un report QWeb — wkhtmltopdf
no puede resolverlas. Ver `wkhtmltopdf-gotchas.md` §4.

**Preferible al `'data:...;base64,' + campo.decode()`: el helper `image_data_uri`**
(global de QWeb, `odoo/addons/base/models/ir_qweb.py:956`, `IrQWeb._prepare_environment`, verificado v18). Detecta el mimetype por magic
number y devuelve el `data:` URI completo — evita hardcodear `image/png` cuando el
binario es jpg/webp:
```xml
<img t-if="o.product_id.image_128"
     t-att-src="image_data_uri(o.product_id.image_128)"
     style="width:50px; height:50px; object-fit:contain;"/>
```
El `t-if` de guarda es **obligatorio**: si el binario es falsy, `image_data_uri` peta y
tumba el PDF entero (ver §11 y SKILL.md GOTCHA 11).

## 10. Herencia de plantillas existentes

```xml
<template id="report_sale_order_custom"
          inherit_id="sale.report_saleorder">
    <xpath expr="//th[@name='th_description']" position="after">
        <th>Referencia Cliente</th>
    </xpath>
    <xpath expr="//td[@name='td_description']" position="after">
        <td t-out="line.purchase_price"/>
    </xpath>
</template>
```

Para patrones xpath detallados ver
`odoo-module-creator/references/qweb-reports/xpath-patterns.md`.

## 11. Expresiones dinámicas blindadas (fail-safe)

QWeb renderiza **todo-o-nada**: un error runtime en un solo `t-out`/`t-field`/`t-att`
(división por cero, método sobre recordset vacío, atributo inexistente) lanza
`QWebException` (v17/v18) — **`QWebError` en v19**, renombrada sin alias — y **tumba el PDF
entero**, no degrada solo ese nodo. La validación
estática del arch no protege del fallo en render.

Si el informe emite **expresiones que vienen de datos/usuario** (informes configurables,
motores de plantillas, chips), NUNCA las metas crudas en el template. Enrútalas por un
método envoltorio que atrape el error:

```python
class ReportHelper(models.AbstractModel):
    _name = 'my.report.eval'

    @api.model
    def safe_out(self, expr, ctx):
        """Evalúa expr; si peta en render, devuelve '' en vez de tumbar el PDF."""
        try:
            return safe_eval(expr, dict(ctx), mode="eval") or ''
        except Exception:
            return ''
```
```xml
<span t-out="env['my.report.eval'].safe_out(expr_str, {'doc': doc, 'line': line})"/>
```

Regla: **`eval`/`safe_eval` de expresiones dinámicas SIEMPRE dentro de `try/except`**
(ver también `odoo-security-designer` y `odoo-server-action-creator`: la whitelist AST
estática NO garantiza que la expresión no pete en runtime).

## 12. Añadir un tipo/motor de informe (`report_type`)

Para registrar un motor de render nuevo (patrón que usa el propio core para
`qweb-pdf`/`qweb-html`/`qweb-text` y el que introducirá Paper Muncher en v20):

```python
class IrActionsReport(models.Model):
    _inherit = 'ir.actions.report'

    report_type = fields.Selection(
        selection_add=[('my-engine', 'My Engine')],
        ondelete={'my-engine': 'set default'},
    )
```

`selection_add` **extiende** la selección sin pisar los valores del core; `ondelete`
es **obligatorio** al extender una `Selection` con `selection_add` (si no, error al
instalar). El render se engancha sobreescribiendo el método de render correspondiente.