# STEP 4.5: SYNC DATA XML & STEP 4.6: HARDCODED ID REMAPPING

## STEP 4.5: SYNC DATA XML MODIFICATIONS

**Purpose**: Data loaded from module XML files that was modified by the user in staging gets OVERWRITTEN when running `-u all`. This step detects and re-syncs those values.

### What to sync:

1. **Server action code** (`ir.actions.server.code`): Compare code field between staging and production
2. **Automated action domains** (`base.automation.filter_domain`, `filter_pre_domain`)
3. **Cron intervals** (`ir.cron.interval_number`, `nextcall`)
4. **Custom pricing/discount rules**: Any model with `python_code`, `domain`, or `code` text fields
5. **Carrier configuration**: `delivery.carrier` fields like `fixed_price`, `prod_environment`
6. **M2M relationships configured via UI**
7. **Duplicate records from XML + migration**: Identify and remove XML duplicates

### Detection algorithm:

```python
for model, code_fields in MODELS_WITH_XML_DATA.items():
    stg_records = stg.search_read(model, [], ['name'] + code_fields)
    prod_records = prod.search_read(model, [], ['name'] + code_fields)

    stg_by_name = {r['name']: r for r in stg_records}
    prod_by_name = {r['name']: r for r in prod_records}

    for name, sr in stg_by_name.items():
        pr = prod_by_name.get(name)
        if not pr: continue
        diffs = {}
        for field in code_fields:
            if sr.get(field) != pr.get(field) and sr.get(field) is not False:
                diffs[field] = sr[field]
        if diffs:
            prod.write(model, [pr['id']], diffs)
```

### Detect and fix duplicates:

```python
# Example: records with XML duplicates (no short_code) + migrated ones (with short_code)
xml_dupes = prod.search(model, [('short_code', '=', False)])
if xml_dupes:
    prod.unlink(model, xml_dupes)
```

Log: `log_step(4.5, "XML_SYNC", "completed", details="N records synced, M duplicates removed")`

---

## STEP 4.6: HARDCODED ID DETECTION AND REMAPPING IN SERVER ACTION CODE

**Purpose**: Server actions, automated actions, and crons often contain Python code with hardcoded IDs like `browse(5)`, `search([('id','=',17)])`, or m2m commands `[4, 408]`. These IDs refer to **staging record IDs** and are WRONG in production.

### Detection patterns

```python
PATTERNS = [
    (r'\.browse\(\s*(\d+)\s*\)', 'browse'),            # .browse(5)
    (r'\.browse\(\s*\[([^\]]+)\]\s*\)', 'browse_list'), # .browse([5, 6])
    (r"'id'\s*,\s*'='\s*,\s*(\d+)", 'id_equals'),      # ('id', '=', 5)
    (r"'id'\s*,\s*'in'\s*,\s*\[([^\]]+)\]", 'id_in'),  # ('id', 'in', [5, 6])
    (r'\(4,\s*(\d+)\)', 'm2m_add'),                      # (4, 408)
    (r'\(3,\s*(\d+)\)', 'm2m_remove'),                   # (3, 408)
    (r"env\.ref\(\s*['\"]([^'\"]+)['\"]\s*\)", 'env_ref'), # env.ref('module.xmlid')
]

FIELDS_TO_SCAN = {
    'ir.actions.server': ['code'],
    'base.automation': ['filter_domain', 'filter_pre_domain'],
    'ir.cron': ['code'],
}
```

### Identification and Remapping

For each detected hardcoded ID:
1. Determine the model from code context (e.g., `env['stock.warehouse'].browse(5)`)
2. Read the staging record to get its natural key
3. Find the equivalent record in production by natural key
4. Replace in the code string
5. For `env.ref()` calls: verify the xml_id exists in production (these are usually SAFE)
6. For m2m commands `(4, 408)`: check ir.model.data in staging to find what record 408 is
7. If id_mapping.json already has the mapping, use it directly

### Execution sequence

```python
for model, fields in FIELDS_TO_SCAN.items():
    records = stg.search_read(model, [], ['name'] + fields)
    for rec in records:
        for field in fields:
            code = rec.get(field, '') or ''
            remapped_code = code
            unmapped = []
            for pattern, ptype in PATTERNS:
                for match in re.finditer(pattern, code):
                    if ptype == 'env_ref':
                        continue  # Verify xml_id exists, but don't remap
                    staging_ids = extract_ids(match, ptype)
                    for sid in staging_ids:
                        prod_id = resolve_staging_id_to_prod(sid, model, code_context=code)
                        if prod_id:
                            remapped_code = remapped_code.replace(str(sid), str(prod_id))
                        else:
                            unmapped.append({'record': rec['name'], 'field': field,
                                             'staging_id': sid, 'pattern': ptype})
            if remapped_code != code:
                prod_rec = prod.search(model, [('name', '=', rec['name'])])
                if prod_rec:
                    prod.write(model, prod_rec, {field: remapped_code})
            if unmapped:
                log.warning(f"UNMAPPED IDs in {model} '{rec['name']}': {unmapped}")
```

### Report

- **Remapped**: `"ir.actions.server 'Update Stock': browse(5) -> browse(12)"`
- **UNMAPPED (manual fix needed)**: `"base.automation 'Auto Assign': (4, 408) - could not find staging record 408"`

Log: `log_step(4.6, "ID_REMAP", "completed", details="N IDs remapped, M unmapped (manual fix)")`
