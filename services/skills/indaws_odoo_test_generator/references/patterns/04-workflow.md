# 4. Workflow Testing Patterns

### 4.1 State Machine Transitions

```python
def test_sale_order_workflow(self):
    """Test complete sale order lifecycle."""
    order = self.sale_order

    # Draft -> Sent
    self.assertEqual(order.state, 'draft')
    order.action_quotation_send()
    self.assertEqual(order.state, 'sent')

    # Sent -> Sale
    order.action_confirm()
    self.assertEqual(order.state, 'sale')
    self.assertTrue(order.invoice_status == 'to invoice')

    # Create invoice
    invoice = order._create_invoices()
    self.assertTrue(invoice)
    self.assertEqual(invoice.move_type, 'out_invoice')

    # Post invoice
    invoice.action_post()
    self.assertEqual(invoice.state, 'posted')
```

### 4.2 Stock Movement Workflow

```python
def test_stock_move_workflow(self):
    """Test complete stock move lifecycle."""
    move = self.env['stock.move'].create({
        'name': 'test_move',
        'location_id': self.supplier_location.id,
        'location_dest_id': self.stock_location.id,
        'product_id': self.product.id,
        'product_uom_qty': 100.0,
    })
    self.assertEqual(move.state, 'draft')

    move._action_confirm()
    self.assertEqual(move.state, 'assigned')
    self.assertEqual(len(move.move_line_ids), 1)

    move.move_line_ids.qty_done = 100.0
    move._action_done()
    self.assertEqual(move.state, 'done')
```
