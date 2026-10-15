# 20. End-to-End Flow Test Patterns

End-to-end (E2E) flow tests verify the complete business cycle across multiple models.
Unlike unit tests (one method) or form tests (one onchange), E2E tests exercise the
**full chain**: create document → confirm → process → invoice → verify final state.

---

## 20.1 When to Generate E2E Flow Tests

**Mandatory** when the module touches **2+ models in the same business flow**:

| Module Depends On | Flow to Test | Priority |
|---|---|---|
| `sale` + `stock` | SO → confirm → delivery → verify picking | **MUST HAVE** |
| `sale` + `account` | SO → confirm → invoice → post → verify amounts | **MUST HAVE** |
| `sale` + `stock` + `account` | SO → confirm → deliver → invoice → post → verify all | **MUST HAVE** |
| `purchase` + `stock` | PO → confirm → receive → verify picking | **MUST HAVE** |
| `purchase` + `account` | PO → confirm → invoice → post → verify amounts | **MUST HAVE** |
| `purchase` + `stock` + `account` | PO → confirm → receive → invoice → post → verify all | **MUST HAVE** |
| `mrp` + `stock` | MO → confirm → produce → verify moves | **SHOULD HAVE** |
| `sale` + `mrp` | SO → confirm → MO created → produce → deliver | **SHOULD HAVE** |
| `sale` + `purchase` (dropship/MTO) | SO → PO generated → receive → deliver → invoice | **SHOULD HAVE** |

**Also mandatory** when the module modifies:
- `_prepare_invoice_line` or `_prepare_account_move_line` (invoice creation)
- `_get_stock_move_values` or `_get_moves_to_assign` (stock propagation)
- `_compute_qty_invoiced` or `_compute_qty_delivered` (reconciliation)
- `_compute_amount` on sale/purchase lines (pricing changes)
- Any field that propagates across SO → invoice, PO → invoice, or SO → picking

### 20.1.1 Detection Grep Patterns

```bash
# Methods that propagate data across documents
grep -rn "_prepare_invoice_line\|_prepare_account_move_line" --include="*.py" <module_path>/models/
grep -rn "_get_stock_move_values" --include="*.py" <module_path>/models/
grep -rn "_compute_qty_invoiced\|_compute_qty_delivered" --include="*.py" <module_path>/models/

# Cross-model inheritance (inherits multiple flow models)
grep -rn "_inherit.*sale.order.line\|_inherit.*purchase.order.line\|_inherit.*account.move.line\|_inherit.*stock.move" --include="*.py" <module_path>/models/
```

**Rule:** If the module inherits **2+ models** from {sale.order.line, purchase.order.line,
account.move.line, stock.move, stock.rule} → generate E2E flow tests.

---

## 20.2 E2E Test Structure

### File naming: `tests/test_flow_e2e.py`

```python
from odoo import Command, fields
from odoo.tests import tagged

from .common import <Module>Common


@tagged("post_install", "-at_install")
class TestPurchaseFlowE2E(<Module>Common):
    """End-to-end purchase flow tests."""

    def test_purchase_full_flow(self):
        """PO → confirm → receive → invoice → post → verify."""
        # 1. Create document
        order = self.env["purchase.order"].create({...})
        line = order.order_line
        # Verify initial amounts
        self.assertAlmostEqual(line.price_subtotal, expected, places=2)

        # 2. Confirm
        order.button_confirm()
        self.assertEqual(order.state, "purchase")

        # 3. Receive goods
        picking = order.picking_ids
        self.assertTrue(picking)
        picking.move_ids.quantity = expected_qty
        picking.button_validate()
        self.assertEqual(picking.state, "done")

        # 4. Create invoice
        order.action_create_invoice()
        invoice = order.invoice_ids
        self.assertTrue(invoice)

        # 5. Verify invoice line (THE KEY ASSERTIONS)
        inv_line = invoice.invoice_line_ids.filtered(
            lambda l: l.product_id == self.product
        )
        self.assertAlmostEqual(inv_line.quantity, expected_qty, places=2)
        self.assertAlmostEqual(inv_line.price_subtotal, expected_amount, places=2)
        # Verify custom fields propagated correctly
        self.assertEqual(inv_line.custom_field, expected_value)

        # 6. Post invoice
        invoice.invoice_date = fields.Date.today()
        invoice.action_post()
        self.assertEqual(invoice.state, "posted")

        # 7. Verify reconciliation on original document
        self.assertAlmostEqual(line.qty_invoiced, expected_qty, places=2)


@tagged("post_install", "-at_install")
class TestSaleFlowE2E(<Module>Common):
    """End-to-end sale flow tests."""

    def test_sale_full_flow(self):
        """SO → confirm → deliver → invoice → post → verify."""
        # 1. Create SO
        order = self.env["sale.order"].create({...})
        so_line = order.order_line
        self.assertAlmostEqual(so_line.price_subtotal, expected, places=2)

        # 2. Confirm
        order.action_confirm()
        self.assertEqual(order.state, "sale")

        # 3. Deliver
        picking = order.picking_ids
        self.assertTrue(picking)
        picking.move_ids.quantity = expected_qty
        picking.button_validate()
        self.assertEqual(picking.state, "done")

        # 4. Invoice
        invoice = order._create_invoices()
        self.assertTrue(invoice)

        # 5. Verify invoice line
        inv_line = invoice.invoice_line_ids.filtered(
            lambda l: l.product_id == self.product
        )
        self.assertAlmostEqual(inv_line.quantity, expected_qty, places=2)
        self.assertAlmostEqual(inv_line.price_subtotal, expected_amount, places=2)

        # 6. Post
        invoice.action_post()
        self.assertEqual(invoice.state, "posted")

        # 7. Verify reconciliation
        self.assertAlmostEqual(so_line.qty_invoiced, expected_qty, places=2)
        self.assertAlmostEqual(so_line.qty_to_invoice, 0.0, places=2)
```

---

## 20.3 Mixed Product Tests

When the module adds conditional behavior (e.g., some products are by_weight, some are
subscription, some have special tax handling), **always test a mixed order**:

```python
def test_sale_full_flow_mixed(self):
    """SO with mixed product types processes correctly."""
    order = self.env["sale.order"].create({
        "partner_id": self.partner.id,
        "order_line": [
            Command.create({
                "product_id": self.special_product.id,
                "product_uom_qty": 5,
                # custom fields for special behavior
            }),
            Command.create({
                "product_id": self.normal_product.id,
                "product_uom_qty": 10,
            }),
        ],
    })
    order.action_confirm()

    # Deliver all
    picking = order.picking_ids
    for move in picking.move_ids:
        move.quantity = move.product_uom_qty
    picking.button_validate()

    # Invoice
    invoice = order._create_invoices()
    invoice.action_post()

    # Verify each line independently
    special_line = invoice.invoice_line_ids.filtered(
        lambda l: l.product_id == self.special_product
    )
    normal_line = invoice.invoice_line_ids.filtered(
        lambda l: l.product_id == self.normal_product
    )

    # Special product: custom behavior applied
    self.assertAlmostEqual(special_line.quantity, expected_special, places=2)
    self.assertAlmostEqual(special_line.price_subtotal, expected_amount_special, places=2)

    # Normal product: standard behavior preserved
    self.assertAlmostEqual(normal_line.quantity, 10.0, places=2)
    self.assertAlmostEqual(normal_line.price_subtotal, expected_amount_normal, places=2)

    # Total invoice
    self.assertAlmostEqual(
        invoice.amount_untaxed,
        expected_amount_special + expected_amount_normal,
        places=2,
    )

    # All lines fully invoiced
    for line in order.order_line:
        self.assertAlmostEqual(line.qty_to_invoice, 0.0, places=2)
```

---

## 20.4 Key Assertions for E2E Tests

| Checkpoint | What to Assert | Why |
|---|---|---|
| After create | `line.price_subtotal` | Custom pricing computed correctly |
| After confirm | `order.state` | Confirmation didn't break |
| After picking | `picking.state == 'done'` | Delivery completed |
| After picking | `move.custom_field` | Custom fields propagated to stock.move |
| Invoice line | `inv_line.quantity` | Correct qty (may differ from SO qty) |
| Invoice line | `inv_line.price_subtotal` | Amount matches expectation |
| Invoice line | `inv_line.custom_field` | Custom fields propagated to invoice |
| After post | `invoice.state == 'posted'` | No accounting errors |
| Reconciliation | `so_line.qty_invoiced` | Units reconciled correctly |
| Reconciliation | `so_line.qty_to_invoice == 0` | Fully invoiced |

---

## 20.5 Common Pitfalls in E2E Tests

### invoice_date required for posting
Purchase invoices need `invoice_date` before `action_post()`:
```python
invoice.invoice_date = fields.Date.today()
invoice.action_post()
```

### purchase_method affects qty_to_invoice
If product has `purchase_method='receive'` (default for storable), you must receive
goods before invoicing. Use `purchase_method='purchase'` in test data or receive first.

### Sale invoice: _create_invoices vs action_create_invoice
- `order._create_invoices()` returns the invoice recordset directly
- `order.action_create_invoice()` returns an action dict; get invoice from `order.invoice_ids`

### Picking validation may trigger backorder wizard
For partial deliveries, `button_validate()` may open a wizard. For full delivery,
set all `move.quantity = move.product_uom_qty` before validating.

### Import fields.Date for invoice_date
```python
from odoo import Command, fields  # Don't forget fields!
```

---

## 20.6 When NOT to Generate E2E Tests

- Module only adds **view changes** (no model logic)
- Module only adds **computed fields** that don't propagate across documents
- Module only adds **constraints** or **onchanges** (unit/form tests suffice)
- Module only touches **one model** (no cross-document flow)
