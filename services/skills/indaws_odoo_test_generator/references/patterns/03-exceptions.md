# 3. Exception Testing Patterns

### 3.1 Basic Exception Test

```python
def test_raises_user_error(self):
    """Test that invalid operation raises UserError."""
    with self.assertRaises(UserError):
        self.record.action_that_should_fail()
```

### 3.2 Exception with Savepoint (Continue After)

```python
def test_constraint_with_savepoint(self):
    """Test constraint violation without breaking transaction."""
    with self.assertRaises(ValidationError), self.cr.savepoint():
        self.record.write({'amount': -100})
    # Test continues - savepoint rolled back
    self.assertEqual(self.record.amount, 0)  # Original value preserved
```

### 3.3 Exception Message Matching

```python
def test_specific_error_message(self):
    """Test that error message matches expected pattern."""
    with self.assertRaisesRegex(UserError, "cannot be negative"):
        self.record.write({'quantity': -1})
```

### 3.4 SQL Constraint Testing with IntegrityError

For `_sql_constraints` violations, use `psycopg2.IntegrityError` with `cr.savepoint()` to avoid breaking the transaction. Works in all versions (14-19).

```python
from psycopg2 import IntegrityError

def test_sql_unique_constraint(self):
    """Test SQL unique constraint without breaking transaction."""
    self.env['my.model'].create({
        'name': 'Record A',
        'code': 'UNIQUE001',
    })

    # Savepoint allows the test to continue after the DB error
    with self.assertRaises(IntegrityError), self.cr.savepoint():
        self.env['my.model'].create({
            'name': 'Record B',
            'code': 'UNIQUE001',  # Duplicate — triggers SQL constraint
        })

    # Transaction is intact — test continues
    count = self.env['my.model'].search_count([('code', '=', 'UNIQUE001')])
    self.assertEqual(count, 1)
```

> **Important:** Always wrap `IntegrityError` assertions inside `self.cr.savepoint()`. Without it, the failed SQL statement poisons the current transaction and all subsequent ORM calls will fail with `InternalError: current transaction is aborted`.
