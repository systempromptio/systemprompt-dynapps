# Test Code Templates

Ready-to-use templates for each test type. Copy and adapt.

---

## 1. common.py — Shared Test Data

```python
from odoo.tests import tagged
from odoo.tests.common import TransactionCase
from odoo import Command


class <Module>Common(TransactionCase):
    """Shared test data for <module_name> tests."""

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        ctx = {'tracking_disable': True, 'mail_create_nolog': True,
               'mail_create_nosubscribe': True, 'no_reset_password': True}
        cls.env = cls.env(context=dict(cls.env.context, **ctx))

        cls.partner = cls.env['res.partner'].create({'name': 'Test Partner'})
        cls.product = cls.env['product.product'].create({
            'name': 'Test Product',
            'is_storable': True,  # v18+ (use type='product' for v14-v17)
        })
```

---

## 2. Unit Tests (TransactionCase)

```python
from odoo import Command
from odoo.exceptions import UserError, ValidationError
from odoo.tests import tagged
from .common import <Module>Common


@tagged('post_install', '-at_install')
class Test<ModelName>(<Module>Common):

    def test_create_basic(self):
        record = self.env['<model.name>'].create({'field1': 'value1'})
        self.assertTrue(record)

    def test_compute_<field>(self):
        self.record.write({'dependency_field': value})
        self.assertEqual(self.record.<field>, expected)

    def test_check_<constraint>(self):
        with self.assertRaises(ValidationError), self.cr.savepoint():
            self.record.write({'field': 'invalid'})

    def test_sql_<constraint>(self):
        from psycopg2 import IntegrityError
        with self.assertRaises(IntegrityError), self.cr.savepoint():
            self.env['model'].create({'field': 'duplicate'})

    def test_action_<workflow>(self):
        self.assertEqual(self.record.state, 'draft')
        self.record.action_confirm()
        self.assertEqual(self.record.state, 'confirmed')
```

---

## 3. Form/Onchange Tests

```python
from odoo import Command
from odoo.tests import Form, tagged
from .common import <Module>Common


@tagged('post_install', '-at_install')
class Test<ModelName>Form(<Module>Common):

    def test_default_values(self):
        form = Form(self.env['<model.name>'])
        self.assertEqual(form.<field>, expected_default)

    def test_onchange_<field>(self):
        with Form(self.env['<model.name>']) as f:
            f.<trigger_field> = self.<value>
            self.assertEqual(f.<dependent_field>, expected)

    def test_one2many_lines(self):
        with Form(self.record) as f:
            with f.<line_ids>.new() as line:
                line.<field> = value
        self.assertEqual(len(self.record.<line_ids>), expected)
```

---

## 4. HTTP/Controller Tests

```python
from odoo.tests import tagged, HttpCase


@tagged('post_install', '-at_install')
class Test<Module>Controllers(HttpCase):

    def test_authenticated_access(self):
        self.authenticate('admin', 'admin')
        response = self.url_open('/my/route')
        self.assertEqual(response.status_code, 200)

    def test_unauthenticated_redirects(self):
        self.authenticate(None, None)
        response = self.url_open('/my/route', allow_redirects=False)
        self.assertIn(response.status_code, (302, 303))

    def test_json_controller(self):
        import json
        self.authenticate('admin', 'admin')
        response = self.url_open('/my/api', data=json.dumps({
            'jsonrpc': '2.0', 'params': {}
        }), headers={'Content-Type': 'application/json'})
        self.assertEqual(response.json().get('result', {}).get('status'), 'ok')
```

---

## 5. Tour Tests (v17-18)

**Python launcher:**
```python
from odoo.tests import tagged, HttpCase


@tagged('post_install', '-at_install')
class Test<Module>Tour(HttpCase):

    def test_<flow>_tour(self):
        self.env['<model>'].create({...})
        self.start_tour('/web', '<module>_<flow>_tour', login='admin', timeout=180)
```

**JS file** (`static/tests/tours/<module>_tour.js`):
```javascript
/** @odoo-module **/
import { registry } from "@web/core/registry";

registry.category("web_tour.tours").add("<module>_<flow>_tour", {
    test: true, url: "/web",
    steps: () => [
        {trigger: '.o_menu_apps .dropdown-toggle', run: 'click'},
    ],
});
```

**Manifest:** Add `'web.assets_tests': ['<module>/static/tests/tours/**/*']`

---

## 6. Security Tests

```python
from odoo.exceptions import AccessError
from odoo.tests import tagged, new_test_user
from .common import <Module>Common


@tagged('post_install', '-at_install')
class Test<Module>Security(<Module>Common):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.user_basic = new_test_user(cls.env, login='test_basic',
                                        groups='base.group_user')
        cls.user_manager = new_test_user(cls.env, login='test_manager',
                                          groups='base.group_user,<module>.group_manager')

    def test_basic_user_read(self):
        self.env['<model>'].with_user(self.user_basic).search([])

    def test_basic_user_cannot_create(self):
        with self.assertRaises(AccessError):
            self.env['<model>'].with_user(self.user_basic).create({...})
```

---

## 7. E2E Flow Tests

See [patterns/20-e2e-flow.md](patterns/20-e2e-flow.md) for complete patterns.

```python
from odoo import Command, fields
from odoo.tests import tagged
from .common import <Module>Common


@tagged("post_install", "-at_install")
class TestSaleFlowE2E(<Module>Common):

    def test_sale_full_flow(self):
        order = self.env["sale.order"].create({
            "partner_id": self.partner.id,
            "order_line": [Command.create({
                "product_id": self.product.id,
                "product_uom_qty": 10, "price_unit": 15.0,
            })],
        })
        order.action_confirm()

        picking = order.picking_ids
        picking.move_ids.quantity = 10
        picking.button_validate()

        invoice = order._create_invoices()
        invoice.action_post()

        self.assertAlmostEqual(order.order_line.qty_invoiced, 10.0, places=2)
```

---

## 8. Performance Tests

```python
from odoo.tests import tagged, users, warmup
from .common import <Module>Common


@tagged('post_install', '-at_install')
class Test<Module>Performance(<Module>Common):

    @users('admin')
    @warmup
    def test_<op>_query_count(self):
        with self.assertQueryCount(admin=<N>):
            self.env['<model>'].<method>()
```

## 9. Report Render Tests (QWeb)

Render sin excepción + asertar el dato que aporta el módulo. API `_render_qweb_html(report_ref, docids)`
→ `(bytes, 'html')`, estable v16→v19. Patrón completo: `patterns/23-qweb-report.md`.

```python
from odoo.tests import TransactionCase, tagged


@tagged('post_install', '-at_install')
class Test<Module>Report(TransactionCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.doc = cls.env['<model>'].create({...})

    def test_report_renders(self):
        html, rtype = self.env['ir.actions.report']._render_qweb_html(
            '<module>.<report_xmlid>', self.doc.ids)
        self.assertEqual(rtype, 'html')
        content = html.decode() if isinstance(html, bytes) else str(html)
        self.assertIn('<dato que aporta el módulo>', content)   # no basta con que renderice
```
