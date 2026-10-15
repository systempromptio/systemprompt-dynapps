# 12. Inheritance Testing Patterns

### 12.1 Backward Compatibility Test

When extending a model via `_inherit`, verify that base functionality remains intact.

```python
def test_backward_compatibility(self):
    """Verify base model operations still work after inheritance."""
    # Base CRUD should still work
    record = self.env['inherited.model'].create({
        'name': 'Test Record',
    })
    self.assertTrue(record.id)

    record.write({'name': 'Updated'})
    self.assertEqual(record.name, 'Updated')

    # Base methods should still be callable and produce expected results
    result = record.action_confirm()
    self.assertEqual(record.state, 'confirmed')

    # Base computed fields should still recompute correctly
    record.write({'quantity': 10, 'price_unit': 5.0})
    self.assertEqual(record.amount_total, 50.0)  # Base compute still works
```

### 12.2 Super Call Preservation

Test that overridden methods properly call `super()` and preserve the chain.

```python
def test_super_call_preserved(self):
    """Test overridden write() calls super and preserves base behavior."""
    record = self.env['inherited.model'].create({
        'name': 'Test',
        'state': 'draft',
    })

    # write() is overridden to add custom_log, but base write must still work
    record.write({'name': 'Updated', 'description': 'New desc'})
    self.assertEqual(record.name, 'Updated')          # Base behavior
    self.assertEqual(record.description, 'New desc')   # Base behavior
    self.assertTrue(record.custom_log)                  # Added by override
```

### 12.3 Blocking Base Operations

Test that inherited modules can prevent base operations under certain conditions.

```python
def test_prevent_deletion_in_confirmed_state(self):
    """Test that confirmed records cannot be deleted (added by inheritance)."""
    record = self.env['inherited.model'].create({
        'name': 'Test',
        'state': 'draft',
    })

    # Draft records can be deleted (base behavior preserved)
    draft_id = record.id
    record.unlink()
    self.assertFalse(self.env['inherited.model'].browse(draft_id).exists())

    # Confirmed records cannot be deleted (new restriction from inheritance)
    record2 = self.env['inherited.model'].create({
        'name': 'Test 2',
        'state': 'draft',
    })
    record2.action_confirm()

    with self.assertRaises(UserError):
        record2.unlink()
```
