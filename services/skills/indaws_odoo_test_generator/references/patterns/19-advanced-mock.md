# 19. Advanced Mock and Integration Test Patterns

### 19.1 Patching Odoo Model Methods via `type(record)`

When mocking methods on Odoo model instances, use `patch.object(type(record), ...)` — not the string path. This is safer because Odoo models are dynamically generated classes and the module path may not match the runtime class hierarchy.

```python
from unittest.mock import patch, MagicMock

def test_mock_model_method(self):
    """Mock a model method using type(record) pattern."""
    mock_conn = MagicMock()
    mock_conn.logout.return_value = None

    with patch.object(
        type(self.connector), "_get_connection",
        return_value=mock_conn,
    ):
        self.record.action_that_uses_connection()

    # Verify the mock was called
    mock_conn.logout.assert_called_once()
```

> **When to use `type(record)` vs string path:**
> - `patch.object(type(record), "method")` — for Odoo model methods (always works)
> - `patch("odoo.addons.module.models.file.requests.post")` — for imported third-party libs only

### 19.2 Mock Factory Helpers

Create reusable mock builders in `setUpClass` or as helper methods. This avoids duplicating complex mock setup across tests.

```python
class TestWithMockConnection(TransactionCase):

    @classmethod
    def setUpClass(cls):
        super().setUpClass()
        cls.connector = cls.env["my.connector"].create({...})

    def _make_mock_connection(self, fields_get_result, record_count=5):
        """Build a mock external connection object.

        Args:
            fields_get_result: dict simulating fields_get response.
            record_count: int simulating search_count response.
        """
        conn = MagicMock()
        conn.execute.return_value = fields_get_result
        conn.execute_kw.side_effect = self._make_execute_kw(record_count)
        conn.logout.return_value = None
        return conn

    @staticmethod
    def _make_execute_kw(record_count):
        """Return a side_effect callable with conditional logic."""
        def _execute_kw(model, method, args, kwargs=None):
            if method == "search_count":
                return record_count
            if method == "search_read":
                # Return fake records
                limit = (kwargs or {}).get("limit", 500)
                offset = (kwargs or {}).get("offset", 0)
                if offset >= record_count:
                    return []
                batch = min(limit, record_count - offset)
                return [{"id": offset + i + 1} for i in range(batch)]
            return None
        return _execute_kw
```

### 19.3 `side_effect` with Conditional Logic

When a mocked method must return different values depending on arguments, use `side_effect` with a function instead of `return_value`.

```python
def test_mock_with_conditional_side_effect(self):
    """Mock returns different values based on arguments."""
    def mock_remote_call(model, method, *args, **kwargs):
        if model == "res.partner":
            return [{"id": 1, "name": "Partner"}]
        elif model == "res.country":
            return [{"id": 10, "name": "Spain"}]
        return []

    with patch.object(
        type(self.connector), "_remote_execute",
        side_effect=mock_remote_call,
    ):
        result = self.record.action_fetch_data()

    self.assertEqual(len(result), 2)
```

### 19.4 Multi-Phase Integration Test Structure

For modules with multi-step workflows (discover → configure → generate → execute), structure integration tests in phases. Each phase builds on the previous one.

```python
def test_full_flow_discover_generate_execute(self):
    """Complete flow: discover → generate → execute."""
    mock_conn = self._make_mock_connection(
        fields_get_result={
            "name": {"type": "char", "string": "Name", "store": True},
        },
        record_count=10,
    )

    # Phase 1: Discover (mocked external connection)
    with patch.object(
        type(self.connector), "_get_connection",
        return_value=mock_conn,
    ):
        self.sync.action_discover()

    self.assertTrue(self.sync.field_ids)

    # Phase 2: Generate (pure local, no mock needed)
    for field in self.sync.field_ids:
        field.skip = False
    self.sync.action_generate()
    self.assertEqual(self.sync.state, "ready")
    self.assertTrue(self.sync.template_id)

    # Phase 3: Execute (mock the execution engine)
    with patch.object(
        type(self.sync.scheduler_id), "_execute",
        return_value=None,
    ):
        result = self.group.action_execute()

    self.assertEqual(self.group.state, "done")
```

### 19.5 Anti-Pattern: `cr.rollback()` in TransactionCase

**CRITICAL:** `self.env.cr.rollback()` is **forbidden** in `TransactionCase` (Odoo wraps each test in a savepoint and raises if you try to rollback). If the code under test has retry logic with `cr.rollback()`, disable retries in the test:

```python
# WRONG — will raise "Cannot rollback" error in TransactionCase
def test_error_with_retry(self):
    group = self.env["my.group"].create({"max_retries": 3})
    with patch.object(type(scheduler), "_execute", side_effect=Exception("fail")):
        group.action_execute()  # cr.rollback() called internally → BOOM

# CORRECT — disable retries to avoid cr.rollback()
def test_error_handling(self):
    group = self.env["my.group"].create({"max_retries": 0})
    with patch.object(type(scheduler), "_execute", side_effect=Exception("fail")):
        group.action_execute()  # No retry → no rollback
    self.assertEqual(group.state, "error")
```

> **Note:** If you need to test the retry mechanism itself, use `SingleTransactionCase` or mock `self.env.cr.rollback` as a no-op.

### 19.6 Verifying Execution Order Without Tracking

When testing that operations execute in a specific order, avoid `side_effect` functions that take `self` (class-level patching passes the instance as first arg, causing argument mismatches). Instead, verify order via timestamps or state:

```python
def test_execution_order(self):
    """Full sync executes before second_pass."""
    with patch.object(type(self.scheduler), "_execute", return_value=None):
        self.group.action_execute()

    line_full = self.group.line_ids.filtered(lambda l: l.line_type == "full")
    line_2nd = self.group.line_ids.filtered(lambda l: l.line_type == "second_pass")
    self.assertEqual(line_full.state, "done")
    self.assertEqual(line_2nd.state, "done")
    # Verify order via timestamps
    self.assertLessEqual(line_full.last_execute_date, line_2nd.last_execute_date)
```

### 19.7 Idempotent Generate/Update Tests

Test that running a generation step twice updates existing records instead of creating duplicates:

```python
def test_generate_idempotent(self):
    """Running generate twice updates, not duplicates."""
    self.sync.action_generate()
    id_first = self.sync.template_id.id

    self.sync.action_generate()
    id_second = self.sync.template_id.id

    self.assertEqual(id_first, id_second, "Template should be updated, not duplicated")
```
