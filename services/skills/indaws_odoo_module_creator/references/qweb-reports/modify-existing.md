# Example: Modify an Existing Sale Order Report

## Scenario

Add a "Weight" column to the standard sale order report and add bank details after totals.

## Step 1: Find the template to inherit

```bash
# The sale order report template
grep -r "report_saleorder_document" $ODOO_SRC/odoo19/addons/sale/report/
# → sale/report/sale_report_templates.xml → template id="report_saleorder_document"
```

## Step 2: Read the original template

Always read the FULL original template before writing XPath. Structure changes between versions.

## Step 3: Create the inherited template

```xml
<!-- views/report_saleorder_custom.xml -->
<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <!-- Add Weight column header -->
    <template id="sale_order_add_weight_header"
              inherit_id="sale.report_saleorder_document">
        <xpath expr="//table[hasclass('o_main_table')]//thead//tr/th[@name='th_subtotal']"
               position="before">
            <th name="th_weight" class="text-end">Weight (kg)</th>
        </xpath>
    </template>

    <!-- Add Weight column cell -->
    <template id="sale_order_add_weight_cell"
              inherit_id="sale.report_saleorder_document">
        <xpath expr="//table[hasclass('o_main_table')]//tbody//t[@t-else='']//tr/td[@name='td_subtotal']"
               position="before">
            <td name="td_weight" class="text-end">
                <span t-out="'%.2f' % (line.product_id.weight * line.product_uom_qty)"/>
            </td>
        </xpath>
    </template>

    <!-- Add bank details after totals -->
    <template id="sale_order_add_bank_details"
              inherit_id="sale.report_saleorder_document">
        <xpath expr="//div[@name='so_total_summary']" position="after">
            <div class="row mt-4" t-if="doc.company_id.partner_id.bank_ids">
                <div class="col-12">
                    <strong>Bank Details:</strong>
                    <t t-foreach="doc.company_id.partner_id.bank_ids[:1]" t-as="bank">
                        <br/>Bank: <span t-field="bank.bank_id.name"/>
                        <br/>IBAN: <span t-field="bank.acc_number"/>
                        <br/>BIC: <span t-field="bank.bank_id.bic"/>
                    </t>
                </div>
            </div>
        </xpath>
    </template>
</odoo>
```

## Step 4: Register in manifest

```python
'data': [
    'views/report_saleorder_custom.xml',
],
```

## Key decisions explained

1. **Separate templates per modification** — easier to debug and disable individually
2. **Used `name=` attributes as targets** — stable across versions
3. **`position="before"` instead of complex sibling selectors** — simpler and safer
4. **`colspan="99"` not needed** — we only added, didn't restructure section/note rows
5. **Bank details use `t-foreach[:1]`** — only show first bank account, not all
