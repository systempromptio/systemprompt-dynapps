# 9. Common Assertion Helpers

### 9.1 assertRecordValues (Most Used)

```python
def test_record_values(self):
    """Compare multiple records against expected dicts."""
    self.assertRecordValues(self.order.order_line.sorted('sequence'), [
        {
            'product_id': self.product_a.id,
            'product_uom_qty': 5.0,
            'price_unit': 100.0,
            'price_subtotal': 500.0,
        },
        {
            'product_id': self.product_b.id,
            'product_uom_qty': 2.0,
            'price_unit': 200.0,
            'price_subtotal': 400.0,
        },
    ])
```

### 9.2 assertXMLEqual

```python
def test_view_arch(self):
    """Compare XML view architectures."""
    self.assertXMLEqual(
        view.get_combined_arch(),
        '''<form>
            <group>
                <field name="name"/>
                <field name="amount"/>
            </group>
        </form>'''
    )
```

### 9.3 Like Pattern (v18+)

```python
def test_sql_like_pattern(self):
    """Use Like for partial SQL matching."""
    self.assertEqual(
        "SELECT id, name, amount FROM sale_order WHERE state = 'draft'",
        Like('SELECT ... FROM sale_order WHERE ...')
    )
```

### 9.4 HTML/Markup Field Comparison (v15+)

From v15+, Odoo wraps HTML field values in `markupsafe.Markup` objects. Direct `assertEqual` against plain strings will fail even if the text content matches.

```python
# WRONG — fails: Markup('<p>Hello</p>') != '<p>Hello</p>'
self.assertEqual(record.description, '<p>Hello</p>')

# OK — convert to string first
self.assertEqual(str(record.description), '<p>Hello</p>')

# OK — check substring presence (ignores Markup wrapper)
self.assertIn('Hello', str(record.description))

# OK — strip HTML tags for content-only comparison
from odoo.tools import html2plaintext
self.assertEqual(html2plaintext(record.description).strip(), 'Hello')

# OK — compare Markup to Markup
from markupsafe import Markup
self.assertEqual(record.description, Markup('<p>Hello</p>'))
```

> **Note:** In v14, HTML fields return plain `str`. This pitfall only applies from v15 onwards where `markupsafe.Markup` was adopted for XSS prevention.
