# Odoo Field Mapping Reference

Common field paths for the most frequently templated Odoo models. Use these as the default mapping when analyzing document images.

## Universal Fields (available on all models)

```
doc.id                          → Record ID
doc.create_date                 → Creation date
doc.write_date                  → Last modified date
doc.display_name                → Computed display name
doc.company_id.name             → Company name
doc.company_id.logo             → Company logo (binary, use Image())
doc.company_id.street            → Company address line 1
doc.company_id.street2           → Company address line 2
doc.company_id.city              → Company city
doc.company_id.state_id.name     → Company state/province
doc.company_id.zip               → Company postal code
doc.company_id.country_id.name   → Company country
doc.company_id.phone             → Company phone
doc.company_id.email             → Company email
doc.company_id.website           → Company website
doc.company_id.vat               → Company tax ID (CIF/NIF)
```

## sale.order (Sales Order / Quotation)

### Header fields
```
doc.name                        → Order reference (S00001)
doc.client_order_ref             → Customer reference
doc.date_order                   → Order date (datetime)
doc.validity_date                → Expiration date
doc.state                        → State: draft/sent/sale/done/cancel
doc.user_id.name                 → Salesperson name
doc.team_id.name                 → Sales team
doc.payment_term_id.name         → Payment terms
doc.pricelist_id.name            → Pricelist
doc.origin                       → Source document
```

### Partner fields
```
doc.partner_id.name              → Customer name
doc.partner_id.street            → Street
doc.partner_id.street2           → Street line 2
doc.partner_id.city              → City
doc.partner_id.state_id.name     → State/Province
doc.partner_id.zip               → Postal code
doc.partner_id.country_id.name   → Country
doc.partner_id.vat               → Tax ID
doc.partner_id.phone             → Phone
doc.partner_id.email             → Email
doc.partner_id.ref               → Internal reference
doc.partner_shipping_id          → Delivery address (same sub-fields)
doc.partner_invoice_id           → Invoice address (same sub-fields)
```

### Line fields (iterate: `doc.order_line`)
```
line.sequence                    → Sequence number
line.name                        → Description (may contain HTML)
line.product_id.name             → Product name
line.product_id.default_code     → Internal reference / SKU
line.product_id.barcode          → Barcode (EAN13)
line.product_id.image_128        → Product image (binary, use Image())
line.product_uom_qty             → Quantity
line.product_uom.name            → Unit of measure
line.price_unit                  → Unit price
line.discount                    → Discount (%)
line.price_subtotal              → Subtotal (excl. tax)
line.price_tax                   → Tax amount
line.price_total                 → Total (incl. tax)
line.tax_id                      → Taxes (recordset, use mapped('name'))
line.display_type                → 'line_section', 'line_note', or False
```

### Totals
```
doc.amount_untaxed               → Subtotal (excl. tax)
doc.amount_tax                   → Total taxes
doc.amount_total                 → Grand total (incl. tax)
doc.currency_id                  → Currency object (for monetary())
doc.currency_id.name             → Currency code (EUR, USD)
doc.currency_id.symbol           → Currency symbol (€, $)
```

### Notes
```
doc.note                         → Internal note (HTML → use Html2Docx)
doc.narration                    → Terms and conditions (varies by version)
```

## account.move (Invoice / Credit Note)

### Header fields
```
doc.name                         → Invoice number (INV/2024/0001)
doc.ref                          → Payment reference
doc.invoice_date                 → Invoice date
doc.invoice_date_due             → Due date
doc.move_type                    → Type: out_invoice/out_refund/in_invoice/in_refund
doc.state                        → State: draft/posted/cancel
doc.payment_reference            → Payment reference (structured)
doc.invoice_origin               → Origin document (SO number)
doc.invoice_user_id.name         → Salesperson
doc.payment_state                → Payment status: not_paid/in_payment/paid/partial
```

### Partner fields
Same as sale.order:
```
doc.partner_id.*                 → Customer/Vendor
doc.partner_shipping_id.*        → Delivery address
```

### Line fields (iterate: `doc.invoice_line_ids`)
```
line.sequence                    → Sequence
line.name                        → Description
line.product_id.name             → Product name
line.product_id.default_code     → SKU
line.quantity                    → Quantity
line.product_uom_id.name         → Unit of measure
line.price_unit                  → Unit price
line.discount                    → Discount (%)
line.price_subtotal              → Subtotal (excl. tax)
line.price_total                 → Total (incl. tax)
line.tax_ids                     → Taxes (recordset)
line.display_type                → 'line_section', 'line_note', or False
```

### Tax summary (iterate: `doc.amount_by_group`)
In Odoo 18, tax groups are accessible via:
```
doc.tax_totals                   → Dict with tax breakdown (JSON-like structure)
```

### Totals
```
doc.amount_untaxed               → Subtotal
doc.amount_tax                   → Tax total
doc.amount_total                 → Grand total
doc.amount_residual              → Amount due (remaining)
doc.currency_id                  → Currency
```

### Notes
```
doc.narration                    → Terms/notes (HTML → use Html2Docx)
doc.fiscal_position_id.name      → Fiscal position
```

## stock.picking (Delivery Slip / Receipt)

### Header fields
```
doc.name                         → Reference (WH/OUT/00001)
doc.origin                       → Source document (SO number)
doc.date_done                    → Effective date (when validated)
doc.scheduled_date               → Scheduled date
doc.state                        → State: draft/waiting/confirmed/assigned/done/cancel
doc.picking_type_id.name         → Operation type (Delivery Orders, Receipts)
doc.carrier_id.name              → Carrier/Shipping method
doc.carrier_tracking_ref         → Tracking number
doc.weight                       → Total weight
```

### Partner fields
```
doc.partner_id.*                 → Delivery partner
doc.company_id.*                 → Company
```

### Line fields (iterate: `doc.move_ids` or `doc.move_line_ids`)
```
# stock.move (aggregated by product)
move.product_id.name             → Product name
move.product_id.default_code     → SKU
move.product_uom_qty             → Demand quantity
move.quantity                    → Done quantity (Odoo 18)
move.product_uom.name            → Unit of measure
move.weight                      → Weight

# stock.move.line (detailed, with lots/serials)
ml.product_id.name               → Product name
ml.lot_id.name                   → Lot/Serial number
ml.qty_done                      → Done quantity
ml.package_id.name               → Package
ml.result_package_id.name        → Destination package
ml.location_id.display_name      → Source location
ml.location_dest_id.display_name → Destination location
```

## purchase.order (Purchase Order)

### Header fields
```
doc.name                         → PO reference (P00001)
doc.partner_ref                  → Vendor reference
doc.date_order                   → Order date
doc.date_planned                 → Expected arrival
doc.state                        → State: draft/sent/purchase/done/cancel
doc.user_id.name                 → Purchase representative
doc.payment_term_id.name         → Payment terms
doc.incoterm_id.name             → Incoterm
doc.origin                       → Source document
```

### Partner (Vendor)
```
doc.partner_id.*                 → Vendor (same structure as sale.order)
```

### Line fields (iterate: `doc.order_line`)
```
line.sequence                    → Sequence
line.name                        → Description
line.product_id.name             → Product name
line.product_id.default_code     → SKU
line.product_qty                 → Quantity
line.product_uom.name            → Unit of measure
line.price_unit                  → Unit price
line.discount                    → Discount (%) — if module installed
line.price_subtotal              → Subtotal
line.taxes_id                    → Taxes (recordset)
line.date_planned                → Scheduled date
line.display_type                → 'line_section', 'line_note', or False
```

### Totals
```
doc.amount_untaxed               → Subtotal
doc.amount_tax                   → Tax total
doc.amount_total                 → Grand total
doc.currency_id                  → Currency
```

## Field Type → Template Pattern

| Odoo Field Type | Template Pattern | Example |
|---|---|---|
| Char, Text | `{{ doc.field }}` | `{{ doc.name }}` |
| Html | `{{ r Html2Docx(doc.field) }}` | `{{ r Html2Docx(doc.note) }}` |
| Integer, Float | `{{ format_lang(doc.field, digits=N) }}` | `{{ format_lang(doc.qty, digits=0) }}` |
| Monetary | `{{ monetary(doc.field) }}` | `{{ monetary(doc.amount_total) }}` |
| Date | `{{ format_date(doc.field) }}` | `{{ format_date(doc.date_order) }}` |
| Datetime | `{{ format_date(doc.field) }}` | `{{ format_date(doc.create_date) }}` |
| Boolean | `{% if doc.field %}Yes{% else %}No{% endif %}` | `{% if doc.active %}` |
| Many2one | `{{ doc.field.name }}` | `{{ doc.partner_id.name }}` |
| One2many | `{%tr for x in doc.field %} ... {%tr endfor %}` | See loop patterns |
| Binary (image) | `{{ Image(doc.field, width=W) }}` | `{{ Image(doc.logo, 30) }}` |
| Selection | `{{ doc.field }}` (raw) or use dict mapping | `{{ doc.state }}` |

## Handling display_type (Section/Note Lines)

Many Odoo line models have a `display_type` field for visual grouping:

```
{%tr for line in doc.order_line %}
{% if line.display_type == 'line_section' %}
  SECTION: {{ line.name }}          → Bold, merged across columns
{% elif line.display_type == 'line_note' %}
  NOTE: {{ line.name }}             → Italic, merged across columns
{% else %}
  PRODUCT ROW: {{ line.product_id.name }} │ {{ line.product_uom_qty }} │ ...
{% endif %}
{%tr endfor %}
```

**Note**: In the actual DOCX template, section and note rows typically merge all cells and apply different formatting. The python-docx generator script handles this with conditional cell merging.
