# XPath Patterns para Modificar Reports Existentes

## Cómo heredar un report

```xml
<template id="report_saleorder_document_custom"
          inherit_id="sale.report_saleorder_document">
    <xpath expr="SELECTOR" position="POSITION">
        <!-- Tu modificación -->
    </xpath>
</template>
```

### Positions

| Position | Efecto |
|----------|--------|
| `inside` | Añade contenido DENTRO del elemento (al final) |
| `before` | Añade contenido ANTES del elemento |
| `after` | Añade contenido DESPUÉS del elemento |
| `replace` | Reemplaza el elemento COMPLETO |
| `attributes` | Modifica atributos del elemento |

## XPath Targets SEGUROS (estables v16-v19)

Estos selectores son estables porque usan `id` o `name` attributes que Odoo mantiene:

```xml
<!-- Bloque de información del documento -->
<xpath expr="//div[@id='informations']" position="..."/>

<!-- Tabla principal de líneas -->
<xpath expr="//table[hasclass('o_main_table')]" position="..."/>

<!-- Bloque de totales (v16+) -->
<xpath expr="//div[@name='so_total_summary']" position="..."/>

<!-- Notas del pedido -->
<xpath expr="//span[@name='order_note']" position="..."/>

<!-- Cualquier elemento con name= -->
<xpath expr="//div[@name='date_order']" position="..."/>
<xpath expr="//div[@name='partner']" position="..."/>
```

## XPath Targets PELIGROSOS (cambian entre versiones)

| Target | Rompe en | Motivo | Alternativa segura |
|--------|----------|--------|--------------------|
| `//div[hasclass('page')]/h2` | v18 | H2 movido a `layout_document_title` | `//t[@t-set='layout_document_title']` |
| `//t[@t-set='address']` | v16 | Eliminado | `//div[@id='informations']` |
| `//div[@name='total']` | v16 | Renombrado | `//div[@name='so_total_summary']` |
| `//t[@t-foreach='doc.order_line']` | v16 | Usa método en vez de campo directo | Target por `//table[hasclass('o_main_table')]` |
| `//span[@t-field='doc.amount_total']` | v16 | Tax totals reestructurados | `//div[@name='so_total_summary']` |
| `//p[@t-field='doc.note']` | v16 | Cambiado a span | `//span[@name='order_note']` |

## Patrones comunes de modificación

### Añadir columna a la tabla de líneas

```xml
<!-- Añadir header -->
<xpath expr="//table[hasclass('o_main_table')]//thead//tr/th[last()]" position="after">
    <th class="text-end">Weight</th>
</xpath>

<!-- Añadir celda en líneas normales (no section/note) -->
<xpath expr="//table[hasclass('o_main_table')]//tbody//t[@t-else='']//tr/td[last()]" position="after">
    <td class="text-end">
        <span t-field="line.product_id.weight"/>
    </td>
</xpath>

<!-- Ajustar colspan de section/note lines -->
<xpath expr="//table[hasclass('o_main_table')]//tbody//t[@t-if='line.display_type == &quot;line_section&quot;']//td" position="attributes">
    <attribute name="colspan">99</attribute>
</xpath>
```

### Añadir campo en bloque de información

```xml
<xpath expr="//div[@id='informations']" position="inside">
    <div class="col-auto" t-if="doc.client_order_ref" name="client_ref">
        <strong>Your Reference:</strong>
        <p t-field="doc.client_order_ref"/>
    </div>
</xpath>
```

### Reemplazar título del documento (v18+)

```xml
<xpath expr="//t[@t-set='layout_document_title']" position="replace">
    <t t-set="layout_document_title">
        CUSTOM TITLE <span t-field="doc.name"/>
    </t>
</xpath>
```

### Añadir bloque antes de la tabla

```xml
<xpath expr="//table[hasclass('o_main_table')]" position="before">
    <div class="row mb-3">
        <div class="col-12">
            <strong>Special Instructions:</strong>
            <p t-field="doc.note"/>
        </div>
    </div>
</xpath>
```

### Añadir contenido después de totales

```xml
<xpath expr="//div[@name='so_total_summary']" position="after">
    <div class="row mt-4">
        <div class="col-12">
            <p><strong>Bank Details:</strong></p>
            <p t-field="doc.company_id.partner_id.bank_ids[:1].acc_number"/>
        </div>
    </div>
</xpath>
```

### Ocultar un elemento

```xml
<xpath expr="//div[@name='date_order']" position="attributes">
    <attribute name="t-if">False</attribute>
</xpath>
```

### Cambiar atributo de estilo

```xml
<xpath expr="//table[hasclass('o_main_table')]" position="attributes">
    <attribute name="class">table table-sm table-bordered o_main_table mt-4</attribute>
</xpath>
```

## Cómo encontrar el template correcto

```bash
# 1. Buscar el report action
grep -r "report_name.*sale.report_saleorder" $ODOO_SRC/odoo19/addons/sale/

# 2. Buscar el template
grep -r "t-name.*report_saleorder_document\|id=\"report_saleorder_document\"" $ODOO_SRC/odoo19/addons/sale/

# 3. Leer el template original COMPLETO antes de escribir XPath
cat $ODOO_SRC/odoo19/addons/sale/report/sale_report_templates.xml
```

**Regla de oro**: SIEMPRE leer el template original de la versión target antes de escribir XPath. La estructura cambia entre versiones.

## Reports estándar más heredados

| Report | XML ID template | Modelo |
|--------|----------------|--------|
| Sale Order | `sale.report_saleorder_document` | `sale.order` |
| Invoice | `account.report_invoice_document` | `account.move` |
| Delivery Slip | `stock.report_deliveryslip` | `stock.picking` |
| Purchase Order | `purchase.report_purchaseorder_document` | `purchase.order` |
| Picking Operations | `stock.report_picking` | `stock.picking` |

## Errores comunes

1. **XPath no encuentra nada → template custom no aparece**: el XPath está mal o la estructura cambió. Verificar con `grep` en el source de Odoo.
2. **`position="replace"` rompe otros módulos**: si otro módulo hereda el mismo elemento y tú lo reemplazas, su XPath falla silenciosamente. Usar `position="inside"` o `position="after"` cuando sea posible.
3. **Colspan incorrecto**: al añadir columnas, hay que actualizar el `colspan` de las filas section/note. Usar `colspan="99"` para evitar contar.
4. **Herencia circular**: si heredas de un template que a su vez hereda de otro, Odoo los aplica en orden de secuencia del módulo. Verificar con `--log-level=debug` si el XPath se aplica.
