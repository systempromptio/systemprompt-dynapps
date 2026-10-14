# Layout Patterns para Reports QWeb

## External Layout (documentos externos)

El layout más común. Añade header con logo/datos empresa y footer.

```xml
<t t-call="web.external_layout">
    <div class="page">
        <!-- Tu contenido -->
    </div>
</t>
```

### Estructura del external layout (v18)

```
┌──────────────────────────────────────┐
│ HEADER                               │
│ ┌──────────────────────────────────┐ │
│ │ Logo  │  Company Name            │ │
│ │       │  Address, VAT            │ │
│ └──────────────────────────────────┘ │
├──────────────────────────────────────┤
│ BODY (tu contenido)                  │
│ ┌──────────────────────────────────┐ │
│ │ <h2> layout_document_title       │ │
│ │ <t t-out="0"/>  ← tu template   │ │
│ └──────────────────────────────────┘ │
├──────────────────────────────────────┤
│ FOOTER                               │
│ ┌──────────────────────────────────┐ │
│ │ Company details / Page X of Y   │ │
│ └──────────────────────────────────┘ │
└──────────────────────────────────────┘
```

### Personalizar header

```xml
<!-- Heredar el layout para modificar header -->
<template id="external_layout_custom" inherit_id="web.external_layout_standard">
    <xpath expr="//div[hasclass('o_report_layout_standard')]" position="replace">
        <div class="o_report_layout_standard">
            <!-- Tu header custom -->
            <div class="row">
                <div class="col-6">
                    <img t-if="company.logo" t-att-src="image_data_uri(company.logo)"
                         style="max-height: 60px;" alt="Logo"/>
                </div>
                <div class="col-6 text-end">
                    <strong t-field="company.name"/><br/>
                    <span t-field="company.street"/><br/>
                    <span t-field="company.zip"/> <span t-field="company.city"/>
                </div>
            </div>
        </div>
    </xpath>
</template>
```

### Personalizar footer

```xml
<template id="report_footer_custom" inherit_id="web.external_layout_standard">
    <xpath expr="//div[hasclass('o_footer_content')]" position="replace">
        <div class="o_footer_content text-center" style="font-size: 8pt;">
            <span t-field="company.name"/> |
            CIF: <span t-field="company.vat"/> |
            <span t-field="company.phone"/> |
            <span t-field="company.email"/>
            <div class="text-end">
                Page <span class="page"/> of <span class="topage"/>
            </div>
        </div>
    </xpath>
</template>
```

## Document Title (v18+)

En v18+, el título del documento NO se pone con `<h2>` directo. Se usa una variable que el layout renderiza.

```xml
<!-- v18+ -->
<t t-set="layout_document_title">
    <t t-if="doc.state in ('draft', 'sent')">Quotation</t>
    <t t-else="">Sales Order</t>
    <span t-field="doc.name"/>
</t>
```

En v14-v17, se usa `<h2>` directamente dentro de `<div class="page">`.

## Information Block

Bloque estándar para datos del documento (fecha, referencia, cliente, etc.).

```xml
<div id="informations" class="row mt-4 mb-4">
    <div class="col-auto" t-if="doc.date_order" name="date_order">
        <strong>Date:</strong>
        <p t-field="doc.date_order" t-options='{"widget": "date"}'/>
    </div>
    <div class="col-auto" t-if="doc.partner_id" name="partner">
        <strong>Customer:</strong>
        <p t-field="doc.partner_id"/>
    </div>
    <div class="col-auto" t-if="doc.origin" name="origin">
        <strong>Source:</strong>
        <p t-field="doc.origin"/>
    </div>
</div>
```

## Address Block

### Simple (un address)

```xml
<div class="row">
    <div class="col-6">
        <strong>Customer Address:</strong>
        <div t-field="doc.partner_id"
             t-options='{"widget": "contact", "fields": ["address", "name", "phone", "vat"], "no_marker": True}'/>
    </div>
</div>
```

### Doble (invoice + shipping)

```xml
<div class="row">
    <div class="col-6">
        <strong>Invoice Address:</strong>
        <div t-field="doc.partner_invoice_id"
             t-options='{"widget": "contact", "fields": ["address", "name", "phone"], "no_marker": True}'/>
    </div>
    <div class="col-6" t-if="doc.partner_shipping_id != doc.partner_invoice_id">
        <strong>Delivery Address:</strong>
        <div t-field="doc.partner_shipping_id"
             t-options='{"widget": "contact", "fields": ["address", "name", "phone"], "no_marker": True}'/>
    </div>
</div>
```

## Line Table Pattern

```xml
<table class="table table-sm o_main_table mt-4">
    <thead>
        <tr>
            <th class="text-start">Description</th>
            <th class="text-end">Quantity</th>
            <th class="text-end">Unit Price</th>
            <th class="text-end" t-if="display_discount">Disc.%</th>
            <th class="text-end">Amount</th>
        </tr>
    </thead>
    <tbody>
        <t t-foreach="doc.order_line" t-as="line">
            <!-- Section headers -->
            <t t-if="line.display_type == 'line_section'">
                <tr>
                    <td colspan="99"><strong t-field="line.name"/></td>
                </tr>
            </t>
            <!-- Note lines -->
            <t t-elif="line.display_type == 'line_note'">
                <tr>
                    <td colspan="99"><em t-field="line.name"/></td>
                </tr>
            </t>
            <!-- Regular lines -->
            <t t-else="">
                <tr>
                    <td><span t-field="line.name"/></td>
                    <td class="text-end">
                        <span t-field="line.product_uom_qty"/>
                        <span t-field="line.product_uom"/>
                    </td>
                    <td class="text-end">
                        <span t-field="line.price_unit"/>
                    </td>
                    <td class="text-end" t-if="display_discount">
                        <span t-field="line.discount"/>
                    </td>
                    <td class="text-end">
                        <span t-field="line.price_subtotal"
                              t-options='{"widget": "monetary", "display_currency": doc.currency_id}'/>
                    </td>
                </tr>
            </t>
        </t>
    </tbody>
</table>
```

## Totals Block

```xml
<div name="so_total_summary" class="row justify-content-end">
    <div class="col-4">
        <table class="table table-sm">
            <tr>
                <td><strong>Subtotal</strong></td>
                <td class="text-end">
                    <span t-field="doc.amount_untaxed"
                          t-options='{"widget": "monetary", "display_currency": doc.currency_id}'/>
                </td>
            </tr>
            <tr>
                <td>Taxes</td>
                <td class="text-end">
                    <span t-field="doc.amount_tax"
                          t-options='{"widget": "monetary", "display_currency": doc.currency_id}'/>
                </td>
            </tr>
            <tr>
                <td><strong>Total</strong></td>
                <td class="text-end">
                    <strong>
                        <span t-field="doc.amount_total"
                              t-options='{"widget": "monetary", "display_currency": doc.currency_id}'/>
                    </strong>
                </td>
            </tr>
        </table>
    </div>
</div>
```

## Notes & Terms

```xml
<div t-if="doc.note" name="order_note" class="mt-4">
    <strong>Notes:</strong>
    <span t-field="doc.note"/>
</div>

<div t-if="doc.payment_term_id" class="mt-4">
    <strong>Payment Terms:</strong>
    <span t-field="doc.payment_term_id.note"/>
</div>
```

## Multi-Language Report

Para que el report se genere en el idioma del partner:

```xml
<t t-call="web.html_container">
    <t t-foreach="docs" t-as="doc">
        <t t-call="web.external_layout" t-lang="doc.partner_id.lang">
            <div class="page">
                <!-- Content renders in partner's language -->
            </div>
        </t>
    </t>
</t>
```

## Page Break Between Documents

Automático con la estructura estándar: cada iteración de `t-foreach` en docs genera una nueva página. Para forzar page break dentro de un documento:

```xml
<div style="page-break-before: always;"/>
```
