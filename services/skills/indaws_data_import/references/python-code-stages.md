# Python Code Integration

## Code Execution Stages

The import process allows Python code at multiple stages:

```
┌─────────────────────────────────────────────┐
│ 1. before_import_code                       │
│    Runs once before starting                │
│    Use: Setup, validation, preparation      │
└─────────────────────────────────────────────┘
            ↓
┌─────────────────────────────────────────────┐
│ 2. context_code (per row)                   │
│    Runs for each row to set context         │
│    Use: Dynamic context based on row data   │
└─────────────────────────────────────────────┘
            ↓
┌─────────────────────────────────────────────┐
│ 3. Field code (per field, per row)          │
│    Runs for each field to transform value   │
│    Input: cell_value | Output: value        │
│    Use: Complex field transformations       │
└─────────────────────────────────────────────┘
            ↓
┌─────────────────────────────────────────────┐
│ 4. before_create_update_code (per row)      │
│    Runs before creating/updating record     │
│    Use: Final validation, modification      │
└─────────────────────────────────────────────┘
            ↓
┌─────────────────────────────────────────────┐
│ 5. code (per row, after create/update)      │
│    Runs after record is created/updated     │
│    Use: Post-processing, related records    │
└─────────────────────────────────────────────┘
            ↓
┌─────────────────────────────────────────────┐
│ 6. finish_import_code                       │
│    Runs once after all rows processed       │
│    Use: Cleanup, notifications, summary     │
└─────────────────────────────────────────────┘
```

## CRITICAL: Template Line Field Code (`cell_value` vs `value`)

The field code in template lines (`indaws.import.template.line.code`) uses a **different variable
naming convention** than collectors. This is the #1 source of bugs:

```python
# WRONG - will raise NameError: name 'value' is not defined
if value:
    value = int(value)

# CORRECT - input is 'cell_value', output is 'value'
if cell_value:
    value = int(cell_value)
```

**How it works internally** (`_parse_code_value` method):
1. The framework injects the input as `cell_value` into the eval context
2. Your code reads from `cell_value` and writes to `value`
3. The framework reads the result from `eval_ctx.get('value', False)`

## Available variables in field code context

| Variable | Type | Description |
|----------|------|-------------|
| `cell_value` | any | The input value from the row for this field |
| `env` | Environment | Odoo environment |
| `row` | dict | The full current row data |
| `tmpl_field` | record | The template line record |
| `rec_import` | record | The import execution record |
| `value` | any | **Output** - set this to the transformed value |
| `parse_continue` | bool | Set to True if you handled the parsing completely |

**See also:** [python-code-context.md](python-code-context.md) for full eval context details.
