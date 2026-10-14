# 13. Module Constants and Configuration Testing

Module-level constants (dicts, sets, lists) are critical configuration that should be validated.

### 13.1 Set/Dict Consistency

```python
def test_no_overlap_between_sets(self):
    """Verify exclusion sets don't overlap with inclusion sets."""
    from ..models.my_model import INCLUDED_MODELS, EXCLUDED_MODELS
    overlap = INCLUDED_MODELS & EXCLUDED_MODELS
    self.assertEqual(
        overlap, set(),
        "Models in both sets: %s" % overlap,
    )

def test_dict_completeness(self):
    """Verify lookup dict covers all values of a related set."""
    from ..models.my_model import TARGET_MODELS, STRATEGY_MAP
    for model in TARGET_MODELS:
        self.assertIn(
            model, STRATEGY_MAP,
            "%s missing from STRATEGY_MAP" % model,
        )
```

### 13.2 Registry Consistency

```python
def test_referenced_models_exist_in_registry(self):
    """All models referenced in constants exist in Odoo registry."""
    from ..models.my_model import SUPPORTED_MODELS
    for model in SUPPORTED_MODELS:
        self.assertIn(
            model, self.env,
            "%s not in registry" % model,
        )
```

### 13.3 Dict Structure Validation

```python
def test_config_dict_structure(self):
    """Config dict has correct nested types."""
    from ..models.my_model import FIELD_CONFIG
    for model, field_map in FIELD_CONFIG.items():
        self.assertIsInstance(field_map, dict, model)
        for field_name, config in field_map.items():
            self.assertIsInstance(config, dict,
                                 "%s.%s" % (model, field_name))
```

### 13.4 Selection Values Validation

```python
def test_selection_values_exist_in_model(self):
    """All selection values in config exist on the actual field."""
    from ..models.my_model import VALID_STATES
    field_def = self.env['my.model'].fields_get(['state'])
    valid_keys = {s[0] for s in field_def['state']['selection']}
    for state in VALID_STATES:
        self.assertIn(state, valid_keys,
                      "State '%s' not in field definition" % state)
```
