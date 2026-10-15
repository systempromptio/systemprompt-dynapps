# load Method API Reference

## Data Format for load

The `load` method expects two parameters:

```python
result = models.execute_kw(
    DB_NAME, uid, API_KEY,
    'odoo.model', 'load',
    [fields, data],  # ← These two parameters
    {}
)
```

### 1. fields: List of field names

```python
fields = [
    'id',              # External ID (mandatory)
    'name',            # Simple field
    'email',           # Simple field
    'country_id/id',   # Many2one relation (by XML ID)
    'category_id/id',  # Many2many relation (by XML ID)
]
```

### Field Syntax:

| Field Type | Syntax | Example |
|------------|--------|---------|
| Simple field | `'field_name'` | `'name'`, `'email'` |
| Many2one | `'field_name/id'` | `'country_id/id'` |
| Many2many | `'field_name/id'` | `'category_id/id'` |
| External ID | `'id'` | `'id'` (always include) |

### 2. data: List of lists with values

Each row is a list of values in the same order as `fields`.

```python
data = [
    # Row 1
    ['__import__.contact_1', 'John Doe', 'john@email.com', 'base.us', ''],
    # Row 2
    ['__import__.contact_2', 'Jane Smith', 'jane@email.com', 'base.uk', ''],
]
```

## Why Use the load Method?

### Advantages

1. **Optimal Performance**:
   - Processes multiple records in a single transaction
   - 10-100x faster than individual operations
   - Internally optimized by Odoo

2. **Automatic Create/Update**:
   - Automatically detects if the record exists (by external ID)
   - Creates new records or updates existing ones
   - No need to program duplicate detection logic

3. **Integrated Validations**:
   - Applies all model validations
   - Verifies constraints
   - Executes onchange methods
   - Validates relationships automatically

4. **Batch Error Handling**:
   - Returns specific errors per row
   - Doesn't cancel entire import due to one error
   - Allows precise debugging

### Comparison: load vs create/write

| Aspect | `load` (Recommended) | `create`/`write` (Don't use) |
|---------|------------------------|----------------------------------|
| Speed | Very fast (batch) | Slow (1 by 1) |
| Duplicate detection | Automatic (external ID) | Manual (search + create/write) |
| Errors | Per row with detail | Per individual record |
| Code complexity | Low (simple format) | High (manual logic) |
| Relations | Automatic XML IDs | Manual numeric IDs |
