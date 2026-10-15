# 11. Advanced Constraint Testing Patterns

### 11.1 Conditional Constraints

Test constraints that only apply when certain conditions are met (e.g., state-dependent validation).

```python
def test_conditional_constraint(self):
    """Test constraint that only applies in 'confirmed' state."""
    record = self.env['my.model'].create({
        'name': 'Test',
        'state': 'draft',
        'manager_id': False,  # No manager — allowed in draft
    })

    # In draft state, no manager is fine
    self.assertFalse(record.manager_id)

    # Confirming without a manager should raise
    with self.assertRaises(ValidationError):
        record.action_confirm()

    # Assign manager, then confirm — should work
    record.manager_id = self.env.ref('base.user_admin').id
    record.action_confirm()
    self.assertEqual(record.state, 'confirmed')
```

### 11.2 Dependent Field Constraints

Test constraints that validate relationships between multiple fields.

```python
def test_date_range_constraint(self):
    """Test that end_date must be after start_date."""
    with self.assertRaises(ValidationError), self.cr.savepoint():
        self.env['my.model'].create({
            'name': 'Invalid Range',
            'start_date': '2024-12-31',
            'end_date': '2024-01-01',  # Before start
        })

    # Valid range should work
    record = self.env['my.model'].create({
        'name': 'Valid Range',
        'start_date': '2024-01-01',
        'end_date': '2024-12-31',
    })
    self.assertTrue(record.id)
```

### 11.3 Cascading Constraints on Related Records

Test constraints that enforce rules between parent and child records.

```python
def test_cascading_constraint(self):
    """Test that parent cannot be deleted while children exist."""
    parent = self.env['my.model'].create({'name': 'Parent'})
    child = self.env['my.model.line'].create({
        'name': 'Child',
        'parent_id': parent.id,
    })

    # Deleting parent with active children should fail
    with self.assertRaises((ValidationError, IntegrityError)):
        with self.cr.savepoint():
            parent.unlink()

    # After removing children, parent can be deleted
    child.unlink()
    parent.unlink()
    self.assertFalse(parent.exists())
```

### 11.4 Constraint Bypass with Context

Test that constraints can be intentionally bypassed via context flags (common pattern for imports, migrations, or admin overrides).

```python
def test_constraint_bypass_with_context(self):
    """Test that constraint is bypassed with skip_validation context."""
    # Normal creation — constraint enforced
    with self.assertRaises(ValidationError), self.cr.savepoint():
        self.env['my.model'].create({
            'name': 'Test',
            'amount': -100.0,  # Negative not allowed
        })

    # With context flag — constraint bypassed
    record = self.env['my.model'].with_context(skip_validation=True).create({
        'name': 'Test',
        'amount': -100.0,
    })
    self.assertEqual(record.amount, -100.0)
```
