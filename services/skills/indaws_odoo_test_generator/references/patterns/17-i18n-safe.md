# 17. i18n-Safe Assertion Patterns

**CRITICAL:** Assertions on user-facing strings break when translations are loaded.

### 17.1 Anti-Pattern: Asserting on Translated Strings

```python
# BAD - Breaks when es_ES is loaded
def test_notification_message(self):
    result = self.record.action_execute()
    self.assertIn("Nothing to import", result["params"]["message"])
    # FAILS with: 'Nothing to import' not found in 'Nada que importar.'
```

### 17.2 Correct: Assert on Structure, Not Content

```python
# GOOD - Language-independent
def test_notification_structure(self):
    """Notification returns correct type regardless of language."""
    result = self.record.action_execute()
    self.assertEqual(result["type"], "ir.actions.client")
    self.assertEqual(result["tag"], "display_notification")
    self.assertEqual(result["params"]["type"], "success")
    self.assertTrue(result["params"]["message"])  # Non-empty, any lang
```

### 17.3 Correct: Assert on Technical Values

```python
# GOOD - Assert on state, counts, IDs — never on labels
def test_workflow_result(self):
    self.record.action_confirm()
    self.assertEqual(self.record.state, 'confirmed')  # Technical key
    self.assertEqual(self.record.line_count, 5)         # Numeric
    self.assertTrue(self.record.template_id)             # Relational
```

### 17.4 When You Must Assert on Strings

```python
# If you must check a message, force the language in context
def test_error_message_content(self):
    with self.assertRaises(UserError) as cm:
        self.record.with_context(lang='en_US').action_fail()
    self.assertIn("required field", str(cm.exception))
```
