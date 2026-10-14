# Template Performance Settings

For large-volume imports (>1K records), configure these template fields:

| Field | Default | Recommended | Purpose |
|-------|---------|-------------|---------|
| `chunksize` | 500 | 500 | Records per async job |
| `commit_every` | 0 | 500 | Commit every N records (0 = end of chunk) |
| `defer_compute` | False | True | Defer field recomputation until chunk ends |
| `continue_error` | False | True | Don't stop on single row error |
| `search_in_archived` | False | * | Search matches in archived records |

\* Enable `search_in_archived` when the model has an `active` field AND the external
system can archive/unarchive records. Without it, `_get_match()` uses
`active_test=True` and won't find archived records, causing duplicates.

## context_code for Bypassing Validations

```xml
<field name="context_code">
value = {'avoid_send': True, 'no_vat_validation': True}
</field>
```

Common context keys:
- `avoid_send`: Skip email/notification sending on create/write
- `no_vat_validation`: Skip VAT/NIF format validation
- `tracking_disable`: Skip mail tracking on field changes
- `mail_create_nolog`: Don't log creation in chatter
