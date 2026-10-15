# Example: Create a Valued Delivery Slip Report

## Scenario

Create a new PDF report for `stock.picking` that shows line quantities with their purchase/sale values. This report does NOT exist in standard Odoo — it requires a Python report class for value computation.

## Step 1: Python Report Class

```python
# report/report_valued_delivery_slip.py
from odoo import api, models


class ReportValuedDeliverySlip(models.AbstractModel):
    _name = 'report.my_module.report_valued_delivery_slip'
    _description = 'Valued Delivery Slip'

    @api.model
    def _get_report_values(self, docids, data=None):
        pickings = self.env['stock.picking'].browse(docids)
        return {
            'doc_ids': docids,
            'doc_model': 'stock.picking',
            'docs': pickings,
            'get_line_value': self._get_line_value,
        }

    def _get_line_value(self, move_line):
        """Compute value based on move type (incoming=purchase, outgoing=sale)."""
        move = move_line.move_id
        if move.picking_id.picking_type_code == 'incoming':
            return move_line.qty_done * move.price_unit
        else:
            return move_line.qty_done * move.sale_line_id.price_unit
```

## Step 2: Report Action + Template

```xml
<!-- report/report_valued_delivery_slip.xml -->
<?xml version="1.0" encoding="utf-8"?>
<odoo>
    <record id="action_report_valued_delivery_slip" model="ir.actions.report">
        <field name="name">Valued Delivery Slip</field>
        <field name="model">stock.picking</field>
        <field name="report_type">qweb-pdf</field>
        <field name="report_name">my_module.report_valued_delivery_slip</field>
        <field name="report_file">my_module.report_valued_delivery_slip</field>
        <field name="binding_model_id" ref="stock.model_stock_picking"/>
        <field name="binding_type">report</field>
        <field name="print_report_name">'Valued Slip - %s' % object.name</field>
    </record>

    <template id="report_valued_delivery_slip">
        <t t-call="web.html_container">
            <t t-foreach="docs" t-as="doc">
                <t t-call="web.external_layout">
                    <div class="page">
                        <t t-set="layout_document_title">
                            Valued Delivery Slip <span t-field="doc.name"/>
                        </t>

                        <div class="row mt-4 mb-4">
                            <div class="col-6">
                                <strong>Customer:</strong>
                                <div t-field="doc.partner_id"
                                     t-options='{"widget": "contact", "fields": ["address", "name"], "no_marker": true}'/>
                            </div>
                            <div class="col-3">
                                <strong>Date:</strong>
                                <p t-field="doc.scheduled_date" t-options='{"widget": "date"}'/>
                            </div>
                            <div class="col-3" t-if="doc.origin">
                                <strong>Source:</strong>
                                <p t-field="doc.origin"/>
                            </div>
                        </div>

                        <table class="table table-sm o_main_table mt-4">
                            <thead>
                                <tr>
                                    <th>Product</th>
                                    <th class="text-end">Qty Done</th>
                                    <th class="text-end">UoM</th>
                                    <th class="text-end">Unit Price</th>
                                    <th class="text-end">Value</th>
                                </tr>
                            </thead>
                            <tbody>
                                <t t-set="total_value" t-value="0"/>
                                <tr t-foreach="doc.move_line_ids.filtered(lambda l: l.qty_done > 0)"
                                    t-as="line">
                                    <td><span t-field="line.product_id"/></td>
                                    <td class="text-end">
                                        <span t-field="line.qty_done"/>
                                    </td>
                                    <td class="text-end">
                                        <span t-field="line.product_uom_id"/>
                                    </td>
                                    <td class="text-end">
                                        <span t-out="'%.2f' % (get_line_value(line) / line.qty_done if line.qty_done else 0)"/>
                                    </td>
                                    <td class="text-end">
                                        <strong t-out="'%.2f' % get_line_value(line)"/>
                                        <t t-set="total_value" t-value="total_value + get_line_value(line)"/>
                                    </td>
                                </tr>
                            </tbody>
                            <tfoot>
                                <tr>
                                    <td colspan="4" class="text-end"><strong>Total Value</strong></td>
                                    <td class="text-end">
                                        <strong t-out="'%.2f %s' % (total_value, doc.company_id.currency_id.symbol)"/>
                                    </td>
                                </tr>
                            </tfoot>
                        </table>
                    </div>
                </t>
            </t>
        </t>
    </template>
</odoo>
```

## Step 3: Manifest

```python
'data': [
    'report/report_valued_delivery_slip.xml',
],
```

## Result

The report appears in the Print menu of stock.picking form view. Generates a PDF with product lines, quantities, unit prices, values, and a total.
