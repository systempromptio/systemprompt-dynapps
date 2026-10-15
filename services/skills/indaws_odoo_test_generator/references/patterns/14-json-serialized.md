# 14. JSON and Serialized Field Testing

Fields that store JSON (Text fields with JSON content, or `Serialized` fields) need specific patterns.

### 14.1 JSON Text Field Roundtrip

```python
import json

def test_json_field_roundtrip(self):
    """JSON stored in text field survives write/read cycle."""
    data = {"key1": "value1", "key2": [1, 2, 3]}
    self.record.config_json = json.dumps(data)
    result = json.loads(self.record.config_json)
    self.assertEqual(result, data)

def test_json_field_invalid_handled(self):
    """Method handles invalid JSON gracefully."""
    self.record.config_json = "not valid json"
    # Method should not raise, should return fallback
    result = self.record._parse_config()
    self.assertFalse(result)  # or whatever the fallback is
```

### 14.2 Serialized Field

```python
def test_serialized_field_returns_dict(self):
    """Serialized field returns proper Python dict."""
    graph = self.record.graph_data
    self.assertIsInstance(graph, dict)

def test_serialized_field_empty_case(self):
    """Serialized field returns empty dict when no data."""
    record = self.env['my.model'].create({...})
    self.assertEqual(record.graph_data, {})
```

### 14.3 JSON Mapping Operations

```python
def test_json_map_auto_generate(self):
    """Auto-generate creates valid JSON mapping."""
    self.record.action_auto_generate_map()
    self.assertTrue(self.record.map_json)
    parsed = json.loads(self.record.map_json)
    self.assertIsInstance(parsed, dict)
    # Verify all expected keys present
    for key in expected_keys:
        self.assertIn(key, parsed)
```
