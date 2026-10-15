# 18. Testing Without External Dependencies

When code integrates with external systems (APIs, remote servers, file systems), test the logic without the connection.

### 18.1 Test Pre-Conditions and Validation

```python
def test_requires_configuration(self):
    """Action fails gracefully without external config."""
    with self.assertRaises(UserError):
        self.record.action_sync()  # No connection configured

def test_requires_ready_state(self):
    """Cannot execute from draft state."""
    self.assertEqual(self.record.state, 'draft')
    with self.assertRaises(UserError):
        self.record.action_execute()
```

### 18.2 Test Internal Logic by Preparing Data Manually

```python
def test_processing_logic_without_fetch(self):
    """Test data processing without actually fetching from remote."""
    # Manually create the data that would come from external system
    self.record.write({
        'field_ids': [Command.create({
            'remote_field_name': 'name',
            'local_field_name': 'name',
            'remote_field_type': 'char',
            'local_field_type': 'char',
        })],
        'state': 'discovered',
    })
    # Test the generation logic (doesn't need external connection)
    active_fields = self.record.field_ids.filtered(
        lambda f: not f.skip
    )
    code = self.record._generate_code(active_fields)
    self.assertIn("name", code)
```

### 18.3 Test Skip/Guard Logic

```python
def test_skip_when_no_data(self):
    """Lines with no external data are auto-skipped."""
    self.record.remote_count = 0
    self.group.action_execute()
    self.assertEqual(self.line.state, 'skipped')

def test_retry_count_configurable(self):
    """Retry configuration has sensible defaults."""
    self.assertEqual(self.group.max_retries, 3)
```
