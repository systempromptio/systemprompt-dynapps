# Response and Error Handling

## Response Structure

```python
result = {
    'ids': [14, 15, 16],  # IDs of created/updated records
    'messages': [         # List of errors/warnings
        {
            'type': 'error',        # 'error' or 'warning'
            'message': 'Error description',
            'rows': {
                'from': 2,          # Row where it occurred (0-indexed)
                'to': 2
            },
            'field': 'email'        # Field that caused the error
        }
    ]
}
```

## Interpreting Results

### Successful Import

```python
if not result.get('messages'):
    ids = result.get('ids', [])
    print(f"Successfully imported {len(ids)} records")
```

### Errors

```python
errors = [m for m in result.get('messages', []) if m.get('type') == 'error']
if errors:
    print(f"{len(errors)} errors:")
    for error in errors:
        row = error.get('rows', {}).get('from', '?')
        field = error.get('field', 'unknown')
        message = error.get('message', 'Unknown error')
        print(f"   Row {row+2} (field '{field}'): {message}")
```

## Common Errors and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `No matching records found for external id 'X'` | Incorrect XML ID in relation | Verify XML ID exists |
| `Field X is required` | Missing required value | Add value for field |
| `Record already exists with this email` | Uniqueness constraint | Change duplicate value |
| `Invalid value for field X` | Incorrect data type | Verify value format |
