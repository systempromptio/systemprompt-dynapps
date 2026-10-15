# Field Mapping Syntax

Standard patterns for reading and writing Odoo fields via API. Reference this instead of documenting field handling in each skill.

## Field Types in API Responses

| Odoo Field | API Read Returns | API Write Expects |
|------------|-----------------|-------------------|
| Char, Text, Html | `"string value"` | `"string value"` |
| Integer | `42` | `42` |
| Float, Monetary | `3.14` | `3.14` |
| Boolean | `true` / `false` | `true` / `false` |
| Date | `"2026-04-07"` | `"2026-04-07"` |
| Datetime | `"2026-04-07 14:30:00"` | `"2026-04-07 14:30:00"` |
| Selection | `"draft"` | `"draft"` |
| Many2one | `[42, "Partner Name"]` | `42` (ID only) |
| One2many | `[1, 2, 3]` (IDs) | Command tuples (see below) |
| Many2many | `[1, 2, 3]` (IDs) | Command tuples (see below) |
| Binary | `"base64string..."` | `"base64string..."` |

## Many2one Gotcha

```python
# READ returns [id, name] tuple
partner = record['partner_id']  # [42, "Acme Corp"]
partner_id = partner[0]          # 42
partner_name = partner[1]        # "Acme Corp"

# WRITE expects ID only
model.write(record_id, {'partner_id': 42})
```

**Common mistake:** Passing `[42, "Acme Corp"]` to write — causes error.

## Command Tuples (O2M / M2M)

| Command | Syntax | Effect |
|---------|--------|--------|
| CREATE | `[0, 0, {values}]` | Create new linked record |
| UPDATE | `[1, id, {values}]` | Update existing linked record |
| DELETE | `[2, id, 0]` | Delete linked record |
| UNLINK | `[3, id, 0]` | Remove link (M2M only, doesn't delete) |
| LINK | `[4, id, 0]` | Add existing record (M2M only) |
| UNLINK ALL | `[5, 0, 0]` | Remove all links (M2M only) |
| REPLACE | `[6, 0, [ids]]` | Replace all with given IDs (M2M only) |

## External IDs

```python
# Read external ID for a record
ext_id = env['ir.model.data'].search_read(
    [('model', '=', 'res.partner'), ('res_id', '=', 42)],
    ['complete_name']
)
# Returns: "base.partner_admin"

# Resolve external ID to record ID
record = env.ref('base.partner_admin')  # Returns res.partner(42)

# For imports via load(): use "id" column with external ID format
# __import__.model_name_unique_key
```

## From Your Skill

```markdown
## Field Handling
See `references/shared/field-mapping-syntax.md` for API field types, Many2one gotchas, and command tuples.
```
