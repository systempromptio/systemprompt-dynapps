# Batch Limits & Pagination

Rules for batch operations across all skills that interact with Odoo API.

## Search/Read Limits

| Operation | Default limit | Recommended | Max safe |
|-----------|--------------|-------------|----------|
| `search_read` | 80 | 200 | 1000 |
| `search` (IDs only) | 80 | 500 | 5000 |
| `read` (by IDs) | unlimited | 200 per call | 500 |
| `load` (import) | N/A | 100 per batch | 200 |
| `create` (batch) | N/A | 50 per call | 100 |
| `write` (batch) | N/A | 100 per call | 500 |
| `unlink` (batch) | N/A | 100 per call | 500 |

## Pagination Pattern

```python
offset = 0
batch_size = 200
all_records = []
while True:
    batch = model.search_read(domain, fields=fields, limit=batch_size, offset=offset)
    if not batch:
        break
    all_records.extend(batch)
    offset += batch_size
```

## Import Batching (load method)

```python
BATCH_SIZE = 100  # Optimal for load()
for i in range(0, len(rows), BATCH_SIZE):
    batch = rows[i:i + BATCH_SIZE]
    result = model.load(fields, batch)
    # Check result['messages'] for errors
```

**Why 100?** Larger batches risk timeout on complex models (sale.order with lines). Smaller batches are inefficient.

## Timeout Rules

| Operation | Expected time | Timeout |
|-----------|--------------|---------|
| Single record CRUD | < 1s | 30s |
| Batch 100 records | 2-10s | 60s |
| Module install | 10-120s | 300s |
| Full search (10K records) | 5-30s | 120s |

## From Your Skill

Reference this document instead of defining your own limits:

```markdown
## Batch Operations
Follow limits in `references/shared/batch-limits.md`. Use pagination for any search that may return >200 records.
```
