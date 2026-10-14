# 1. TransactionCase Patterns

### 1.1 Basic setUpClass Pattern

```python
from odoo import Command
from odoo.tests import tagged
from odoo.tests.common import TransactionCase


@tagged('post_install', '-at_install')
class TestMyModel(TransactionCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()

        # Disable mail tracking for performance
        cls.env = cls.env(context=dict(cls.env.context, **{
            'no_reset_password': True,
            'mail_create_nosubscribe': True,
            'mail_create_nolog': True,
            'tracking_disable': True,
        }))

        # Reference existing data
        cls.company = cls.env.ref('base.main_company')
        cls.currency = cls.env.ref('base.USD')

        # Create test data (shared across all tests in class)
        cls.partner = cls.env['res.partner'].create({
            'name': 'Test Partner',
            'email': 'test@example.com',
            'country_id': cls.env.ref('base.us').id,
        })
```

### 1.1b Why `@tagged('post_install', '-at_install')` — not cosmetic

`at_install` (the DEFAULT if you don't tag) runs the test **during** the module
load graph, right after your module loads — so the registry only contains modules
loaded *up to your module's position*: your deps and nothing else. Modules that are
**not** in your dependency chain (`account`, `sale`, `stock`, …) load **later** and
their field extensions are **not yet in the registry** when your `at_install` test runs.

Failure mode (real, silent until a big DB):

- Your test creates a `res.users` (→ `res.partner`) or any record that carries a
  `required`/NOT NULL column contributed by a non-dependency module — e.g.
  `account` adds `res.partner.autopost_bills` (`Selection`, `required=True`, `default='ask'`).
- On a **small** DB (few modules) it passes: `account` happens to load before your
  module, so the field + its default are present.
- On a **large real DB** (hundreds of modules) `account` loads *after* your module.
  At `at_install` time the field is absent from `_fields` (so its default never
  applies) **but the DB column exists** (account is installed) → INSERT →
  `psycopg2.errors.NotNullViolation: null value in column "autopost_bills"`.
- It works fine in `odoo shell` (full registry) and often at `-i` (different order),
  which makes it maddening to diagnose. Diagnostic tell: in the failing test,
  `self.env['res.partner']._fields.get('autopost_bills')` is `None`.

Rule: **any test that creates `res.users`/`res.partner` (or records pulling fields
from modules you don't depend on) MUST be `post_install`.** `post_install` runs after
the *entire* graph loads, so all installed modules' fields exist. Cost is negligible.

```python
from odoo.tests.common import TransactionCase, tagged

@tagged('post_install', '-at_install')
class TestX(TransactionCase):
    ...
```

### 1.2 Inherited Common Class Pattern

Used when multiple test files need the same base data (from `account/tests/common.py`):

```python
class AccountTestInvoicingCommon(TransactionCase):

    @classmethod
    def setUpClass(cls, chart_template_ref=None):
        super().setUpClass()
        # Setup chart of accounts, taxes, etc.

    @classmethod
    def setup_company_data(cls, company_name, chart_template=None, **kwargs):
        """Create a complete company with accounting configuration."""
        company = cls.env['res.company'].create({
            'name': company_name, **kwargs
        })
        chart_template.try_loading(company=company, install_demo=False)
        return {
            'company': company,
            'currency': company.currency_id,
            'default_account_revenue': ...,
            'default_account_expense': ...,
            'default_tax_sale': ...,
            'default_tax_purchase': ...,
        }
```

### 1.3 Multi-Inheritance Common Pattern

From `sale/tests/common.py`:

```python
class SaleCommon(ProductCommon, SalesTeamCommon):
    """Combines product and sales team test data."""

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.pricelist = cls.env['product.pricelist'].create({
            'name': 'Test Pricelist',
        })
        cls.sale_order = cls.env['sale.order'].create({
            'partner_id': cls.partner.id,
            'pricelist_id': cls.pricelist.id,
        })
```
