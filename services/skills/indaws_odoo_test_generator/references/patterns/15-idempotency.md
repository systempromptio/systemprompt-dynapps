# 15. Idempotency Testing

Operations that can be run multiple times must produce the same result.

### 15.1 Create Idempotency

```python
def test_create_idempotent(self):
    """Running the creation process twice does not duplicate records."""
    self.record.action_generate()
    count_after_first = self.env['target.model'].search_count(
        [('source_id', '=', self.record.id)]
    )
    self.record.action_generate()
    count_after_second = self.env['target.model'].search_count(
        [('source_id', '=', self.record.id)]
    )
    self.assertEqual(count_after_first, count_after_second)
```

### 15.2 State Idempotency

```python
def test_action_idempotent(self):
    """Calling action twice doesn't change state or raise."""
    self.record.action_confirm()
    self.assertEqual(self.record.state, 'confirmed')
    # Second call should not raise or change anything
    self.record.action_confirm()
    self.assertEqual(self.record.state, 'confirmed')
```

### 15.3 Write Idempotency (Import/Sync)

```python
def test_import_idempotent(self):
    """Re-importing same data updates, not duplicates."""
    self.record.action_import()
    created = self.env['target.model'].search_count([])
    self.record.action_import()
    after_reimport = self.env['target.model'].search_count([])
    self.assertEqual(created, after_reimport)
```
