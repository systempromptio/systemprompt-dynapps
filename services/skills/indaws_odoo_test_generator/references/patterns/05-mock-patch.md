# 5. Mock and Patch Patterns

### 5.1 Patching with self.patch()

```python
def test_with_patch(self):
    """Test with mocked method."""
    self.patch(
        type(self.env['res.partner']),
        '_compute_credit',
        lambda self: self.update({'credit': 0})
    )
    # credit always returns 0 now
```

### 5.2 Time Freezing

```python
from freezegun import freeze_time  # v14-v17
# from odoo.tests import freeze_time  # v18+ (built-in)

def test_time_dependent(self):
    """Test time-dependent logic."""
    with freeze_time('2024-06-15'):
        record = self.env['model'].create({'date': fields.Date.today()})
        self.assertEqual(record.date, date(2024, 6, 15))

    with freeze_time('2024-07-01'):
        record._compute_overdue()
        self.assertTrue(record.is_overdue)
```

### 5.3 Muting Loggers

```python
from odoo.tools import mute_logger

@mute_logger('odoo.addons.base.models.ir_cron')
def test_with_muted_logs(self):
    """Test that triggers expected warnings without cluttering output."""
    self.env.ref('module.cron_id').method_direct_trigger()
```

### 5.4 Patching External Services

```python
from unittest.mock import patch, MagicMock

def test_external_api_call(self):
    """Test method that calls external API."""
    mock_response = MagicMock()
    mock_response.status_code = 200
    mock_response.json.return_value = {'status': 'ok'}

    with patch('odoo.addons.my_module.models.my_model.requests.post',
               return_value=mock_response):
        result = self.record.call_external_api()
        self.assertTrue(result)
```
