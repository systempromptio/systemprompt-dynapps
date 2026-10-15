# Batch Processing

## Maximum Batch Size

**RULE**: Never exceed **1000 records per batch**.

**Reasons**:
1. **Timeouts**: Very large requests can exceed time limits
2. **Memory**: Large transactions consume a lot of RAM
3. **Debugging**: Smaller batches make it easier to locate errors

## Batching Implementation

```python
BATCH_SIZE = 1000

def import_with_batching(uid, models, fields, all_data):
    """Imports data in batches of BATCH_SIZE"""
    total_records = len(all_data)
    total_batches = (total_records + BATCH_SIZE - 1) // BATCH_SIZE

    print(f"Total records: {total_records}")
    print(f"Batch size: {BATCH_SIZE}")
    print(f"Total batches: {total_batches}")

    total_imported = 0
    total_errors = 0

    for i in range(total_batches):
        start_idx = i * BATCH_SIZE
        end_idx = min((i + 1) * BATCH_SIZE, total_records)
        batch_data = all_data[start_idx:end_idx]

        print(f"\\nProcessing batch {i+1}/{total_batches} ({len(batch_data)} records)...")

        try:
            result = models.execute_kw(
                DB_NAME, uid, API_KEY,
                'res.partner', 'load',
                [fields, batch_data],
                {}
            )

            ids = result.get('ids', [])
            messages = result.get('messages', [])

            # Analyze result
            errors = [msg for msg in messages if msg.get('type') == 'error']

            if errors:
                print(f"  Errors found:")
                for error in errors:
                    row_num = error.get('rows', {}).get('from', '?')
                    print(f"     Row {row_num}: {error.get('message')}")
                total_errors += len(errors)
            else:
                imported = len(ids) if ids else len(batch_data)
                print(f"  Batch processed: {imported} records")
                total_imported += imported

        except Exception as e:
            print(f"  Error processing batch: {e}")
            total_errors += len(batch_data)

    return total_imported, total_errors
```
