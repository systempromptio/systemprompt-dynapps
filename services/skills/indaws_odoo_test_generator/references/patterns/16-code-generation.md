# 16. Dynamic Code Generation Testing

When methods generate Python code as strings (for safe_eval, exec, or templates).

### 16.1 Generated Code Contains Expected Fragments

```python
def test_generated_code_contains_model_ref(self):
    """Generated code references the correct model."""
    code = self.record._generate_match_code()
    self.assertIn("env['res.partner']", code)
    self.assertIn("value = None", code)

def test_generated_code_contains_field(self):
    """Generated code includes all mapped fields."""
    code = self.record._generate_collector_code()
    for field in self.record.field_ids.filtered(lambda f: not f.skip):
        self.assertIn(field.remote_field_name, code)
```

### 16.2 Generated Code Passes safe_eval Validation

```python
def test_generated_code_safe_eval_compatible(self):
    """Generated code passes safe_eval validation."""
    from odoo.tools.safe_eval import test_python_expr
    code = self.record._generate_code()
    err = test_python_expr(expr=code.strip(), mode="exec")
    self.assertFalse(
        err,
        "Generated code has forbidden operations: %s" % err,
    )
```

### 16.3 Generated Code Naming Convention

```python
def test_generated_names_follow_convention(self):
    """Generated object names follow expected pattern."""
    name = self.record._get_generated_name()
    self.assertTrue(name.startswith("Connector:"))
    self.assertIn(self.record.model_name, name)

def test_different_types_generate_different_names(self):
    """Full and second_pass generate distinct names."""
    name_full = self.sync_full._get_generated_name()
    name_2nd = self.sync_second._get_generated_name()
    self.assertNotEqual(name_full, name_2nd)
```
