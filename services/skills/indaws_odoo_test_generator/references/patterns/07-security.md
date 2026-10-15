# 7. Security Testing Patterns

### 7.1 Access Rights Test

```python
from odoo.exceptions import AccessError
from odoo.tests import new_test_user

def test_access_rights(self):
    """Test that restricted user cannot access manager features."""
    user = new_test_user(self.env, login='test_user', groups='base.group_user')

    with self.assertRaises(AccessError):
        self.env['restricted.model'].with_user(user).create({'name': 'test'})
```

### 7.2 Record Rules Test

```python
def test_record_rules(self):
    """Test multi-company record rules."""
    company_a = self.env['res.company'].create({'name': 'Company A'})
    company_b = self.env['res.company'].create({'name': 'Company B'})

    user_a = new_test_user(
        self.env, login='user_a',
        company_id=company_a.id,
        company_ids=[company_a.id],
    )

    record_b = self.env['my.model'].with_company(company_b).create({
        'name': 'Record B',
        'company_id': company_b.id,
    })

    # User A should not see records from Company B
    visible = self.env['my.model'].with_user(user_a).search(
        [('id', '=', record_b.id)]
    )
    self.assertFalse(visible)
```

### 7.3 Sudo Pattern Test

```python
def test_sudo_escalation(self):
    """Test that sudo allows bypassing access rules."""
    restricted_user = new_test_user(self.env, login='restricted')

    with self.assertRaises(AccessError):
        self.env['admin.model'].with_user(restricted_user).search([])

    # With sudo, it should work
    records = self.env['admin.model'].with_user(restricted_user).sudo().search([])
    self.assertTrue(records)
```
