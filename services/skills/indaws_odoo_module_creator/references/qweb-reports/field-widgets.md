# Field Widgets para Reports QWeb

## Sintaxis base

```xml
<span t-field="doc.field_name"/>
<span t-field="doc.field_name" t-options='{"widget": "widget_name"}'/>
```

Siempre usar `<span>` (o `<div>`, `<td>`, etc.), NUNCA `<field>` en reports (v17+ rompe).

## Widgets disponibles

### Monetary (importes con moneda)

```xml
<span t-field="doc.amount_total"
      t-options='{"widget": "monetary", "display_currency": doc.currency_id}'/>
```

**Siempre** incluir `display_currency`. Sin él, renderiza el float sin formato ni símbolo.

### Date / Datetime

```xml
<!-- Date: "15/03/2026" (formato según locale) -->
<span t-field="doc.date_order" t-options='{"widget": "date"}'/>

<!-- Datetime con formato custom -->
<span t-field="doc.create_date" t-options='{"format": "dd/MM/yyyy HH:mm"}'/>

<!-- Solo fecha de un datetime -->
<span t-field="doc.write_date" t-options='{"widget": "date"}'/>
```

### Float con precisión

```xml
<!-- Float con 2 decimales -->
<span t-field="line.product_uom_qty" t-options='{"precision": 2}'/>

<!-- Float time (horas:minutos) -->
<span t-field="doc.duration" t-options='{"widget": "float_time"}'/>
```

### Contact (dirección completa)

```xml
<div t-field="doc.partner_id"
     t-options='{
         "widget": "contact",
         "fields": ["address", "name", "phone", "vat"],
         "no_marker": true,
         "separator": ", "
     }'/>
```

Campos disponibles en widget contact: `address`, `name`, `phone`, `mobile`, `email`, `vat`, `fax`, `website`.

### Image (campo binary)

```xml
<!-- Desde campo binary del modelo -->
<img t-att-src="image_data_uri(doc.image_1920)"
     style="max-height: 100px;"
     alt="Product Image"/>

<!-- Logo de la empresa -->
<img t-if="company.logo"
     t-att-src="image_data_uri(company.logo)"
     style="max-height: 60px;"
     alt="Company Logo"/>
```

### Barcode

```xml
<!-- Código de barras del campo -->
<img t-att-src="'/report/barcode/?barcode_type=Code128&amp;value=%s&amp;width=300&amp;height=50' % doc.name"/>

<!-- QR code -->
<img t-att-src="'/report/barcode/?barcode_type=QR&amp;value=%s&amp;width=100&amp;height=100' % doc.name"/>
```

Tipos de barcode disponibles: `Code128`, `Code39`, `EAN13`, `EAN8`, `UPCA`, `QR`, `DataMatrix`.

### Selection (valor legible)

```xml
<!-- Muestra el label, no el value técnico -->
<span t-field="doc.state"/>
```

`t-field` en Selection renderiza automáticamente el label traducido.

### Many2one

```xml
<!-- Display name -->
<span t-field="doc.partner_id"/>

<!-- Campo específico del related -->
<span t-field="doc.partner_id.name"/>
<span t-field="doc.partner_id.email"/>
```

### HTML (campos rich text)

```xml
<!-- Renderiza HTML safe -->
<div t-field="doc.note"/>

<!-- Si viene de un compute que retorna HTML -->
<div t-out="doc.computed_html_field"/>
```

### Text (campos multiline)

```xml
<!-- Preserva saltos de línea -->
<span t-field="doc.description" style="white-space: pre-wrap;"/>
```

## Expresiones con t-out (sin widget)

Cuando necesitas formatear manualmente:

```xml
<!-- Valor raw -->
<span t-out="doc.amount_total"/>

<!-- Con formato Python -->
<span t-out="'%.2f' % doc.amount_total"/>

<!-- Condicional -->
<span t-out="doc.name or 'N/A'"/>

<!-- HTML safe (si el valor es Markup) -->
<span t-out="doc.note"/>
```

## Formateo de números sin widget

```xml
<!-- Usando format_amount del env -->
<span t-out="doc.currency_id.format(doc.amount_total)"/>

<!-- Usando format de Python -->
<span t-out="'{:,.2f}'.format(doc.amount_total)"/>
```

## Tabla de referencia rápida

| Tipo campo | Widget | Ejemplo |
|-----------|--------|---------|
| Float (dinero) | `monetary` + `display_currency` | `1,234.56 €` |
| Float (cantidad) | (sin widget) o `precision` | `10.00` |
| Float (tiempo) | `float_time` | `2:30` |
| Date | `date` | `15/03/2026` |
| Datetime | (default) o `format` | `15/03/2026 14:30` |
| Many2one | (default) | `John Doe` |
| Selection | (default) | `Confirmed` |
| HTML | (default via `t-field`) | Renderiza HTML |
| Binary (image) | `image_data_uri()` | `<img>` tag |
| Char (barcode) | URL `/report/barcode/` | `<img>` tag |
| Many2one (address) | `contact` | Dirección formateada |
