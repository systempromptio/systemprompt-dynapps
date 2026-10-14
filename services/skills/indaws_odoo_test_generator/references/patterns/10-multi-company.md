# 10. Multi-Company Testing Pattern

```python
@classmethod
def setUpClass(cls):
    super().setUpClass()
    cls.company_a = cls.env['res.company'].create({'name': 'Company A'})
    cls.company_b = cls.env['res.company'].create({'name': 'Company B'})

    cls.user_a = new_test_user(
        cls.env, login='user_a',
        company_id=cls.company_a.id,
        company_ids=[cls.company_a.id, cls.company_b.id],
    )

def test_multi_company_record(self):
    """Test record visibility across companies."""
    record = self.env['my.model'].with_company(self.company_a).create({
        'name': 'Test',
        'company_id': self.company_a.id,
    })

    # Switch to company B context
    with_b = self.env['my.model'].with_company(self.company_b)
    visible = with_b.search([('id', '=', record.id)])
    self.assertFalse(visible, "Record from Company A should not be visible in Company B")
```
