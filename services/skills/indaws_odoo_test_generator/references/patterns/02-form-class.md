# 2. Form Class Patterns

### 2.1 Creating a New Record via Form

```python
from odoo.tests import Form

def test_create_via_form(self):
    """Test form defaults and onchanges during creation."""
    form = Form(self.env['sale.order'])
    form.partner_id = self.partner
    # Onchange fires: pricelist, payment_term, fiscal_position auto-filled
    self.assertEqual(form.pricelist_id, self.partner.property_product_pricelist)

    order = form.save()
    self.assertTrue(order.id)
```

### 2.2 Editing an Existing Record via Form

```python
def test_edit_via_form(self):
    """Test editing triggers correct onchanges."""
    with Form(self.existing_order) as f:
        f.partner_id = self.other_partner
        # Auto-save at context manager exit
    self.assertEqual(self.existing_order.partner_id, self.other_partner)
```

### 2.3 One2many Subforms

```python
def test_add_order_lines(self):
    """Test adding/editing/removing order lines."""
    with Form(self.order) as f:
        # Add new line
        with f.order_line.new() as line:
            line.product_id = self.product
            line.product_uom_qty = 5
            # Onchange: price_unit, taxes auto-filled

        # Edit existing line (index 0)
        with f.order_line.edit(0) as line:
            line.product_uom_qty = 10

        # Remove line by index
        f.order_line.remove(index=1)

    self.assertEqual(len(self.order.order_line), 1)
    self.assertEqual(self.order.order_line.product_uom_qty, 10)
```

### 2.4 Many2many Operations in Form

```python
def test_m2m_operations(self):
    """Test many2many add/remove in form."""
    with Form(self.user) as f:
        f.groups_id.add(self.group_manager)          # v17/v18 — see note
        f.groups_id.remove(id=self.group_basic.id)
```

> **v19**: on `res.users` this field is `group_ids`, not `groups_id`
> (`odoo19/odoo/addons/base/models/res_users.py:257`, `ResUsers.group_ids`). There is no alias — a test that builds users with the old name
> fails with `ValueError: Invalid field 'groups_id' in 'res.users'` (measured), which is *not* the
> `AttributeError` most migration notes quote. If the test suite must run on both, resolve the name
> once: `"group_ids" if "group_ids" in env["res.users"]._fields else "groups_id"`.

### 2.5 Form with Context

```python
def test_form_with_context(self):
    """Test form with specific context (e.g., default_move_type)."""
    form = Form(
        self.env['account.move'].with_context(default_move_type='out_invoice')
    )
    form.partner_id = self.partner
    form.invoice_date = fields.Date.from_string('2024-01-01')

    with form.invoice_line_ids.new() as line:
        line.product_id = self.product
        line.tax_ids.clear()
        for tax in self.taxes:
            line.tax_ids.add(tax)

    invoice = form.save()
```
