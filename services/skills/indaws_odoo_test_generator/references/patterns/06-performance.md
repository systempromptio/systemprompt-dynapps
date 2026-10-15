# 6. Performance Testing Patterns

### 6.1 Query Count Assertion

```python
from odoo.tests import tagged, users, warmup

@tagged('post_install', '-at_install')
class TestPerformance(TransactionCase):

    @users('admin')
    @warmup
    def test_search_performance(self):
        """Assert search operation stays within query budget."""
        with self.assertQueryCount(admin=5):
            self.env['sale.order'].search([
                ('state', '=', 'sale'),
                ('partner_id', '=', self.partner.id),
            ])
```

### 6.2 Exact Query Assertion

```python
def test_exact_queries(self):
    """Assert exact SQL queries executed."""
    with self.assertQueries([
        '''SELECT "sale_order"."id" FROM "sale_order" WHERE ...''',
    ]):
        self.env['sale.order'].search([('state', '=', 'draft')])
```
