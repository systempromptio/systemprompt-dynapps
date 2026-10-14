# Common Pitfalls & Gotchas

Critical issues discovered in real-world implementations. Read this before creating your first import.

---

## Pitfall #0: matching by name uses `display_name`, not `name` — and that duplicates records

**Severity:** CRITICAL - silently creates duplicates in customer data

When a template line has no explicit `domain`, the importer matches the related record with
`FieldModel.name_search(name=value, operator="=")`
(`indawsodoo/18/indaws_data_import/models/indaws_import.py:1317`, `IndawsImport._parse_many2one_value`).

`name_search` does **not** search the `name` column: it searches `display_name`
(`odoo18/odoo/models.py:1897`, `BaseModel.name_search`, unchanged in v19 — `odoo19/odoo/orm/models.py:1542`, `BaseModel.name_search`). For any model whose
display name is built from more than the name, an exact match on the plain name **finds nothing**:

| Model | `name` | `display_name` |
|---|---|---|
| `res.partner` (a contact of a company) | `Ana` | `ACME, Ana` |
| `product.product` (with internal reference) | `Mesa` | `[TAB01] Mesa` |
| `account.account` | `Clientes` | `430000 Clientes` |

**Measured 2026-08-15** (`tools/bench-orm`, `TestCasarPorNombre`): with a contact whose display name
is `ACME, Ana`, `name_search(name="Ana", operator="=")` returns **`[]`** — while `operator="ilike"`
finds it. Nothing errors: the importer concludes the record does not exist and **creates a second
one**. The duplicate surfaces weeks later, in the customer's data.

**What to do**, in order of preference:

1. Give the line an explicit **`domain`** on the real field: `[("name", "=", self)]`,
   `[("default_code", "=", self)]`, `[("code", "=", self)]`. The domain path bypasses
   `name_search` entirely (`indawsodoo/18/indaws_data_import/models/indaws_import.py:1313`, `IndawsImport._parse_many2one_value`).
2. Import the value **as it appears in `display_name`** (`ACME, Ana`, `[TAB01] Mesa`).
3. Best of all when the source allows it: match by **external ID**, which does not go through names.

---

## Pitfall #1: `cell_value` vs `value` in Template Line Code

**Severity:** CRITICAL - causes `NameError` at runtime

Template line field code receives input as `cell_value`, NOT `value`.

```python
# WRONG - NameError: name 'value' is not defined
if value:
    value = int(value)

# CORRECT
if cell_value:
    value = int(cell_value)
else:
    value = False
```

**Rule:** Read from `cell_value`, write to `value`.

**Source:** Method `_parse_code_value` in `indaws_import.py` injects `cell_value` into
eval context and reads result from `eval_ctx.get('value', False)`.

---

## Pitfall #2: Match System Requires BOTH `match=True` AND `match_line_ids`

**Severity:** CRITICAL - causes duplicate records on every import run

Setting `match=True` on template lines is **NOT ENOUGH**. The `_get_match` method uses
`tmpl.match_line_ids` (One2many to `indaws.import.template.match.line`) to build the search
domain, NOT `line_ids.filtered(match=True)`.

**You need:**
1. `match=True` on the template line (populates `match_field_ids` computed field)
2. An `indaws.import.template.match.line` record pointing to the same field

**In XML, write to `match_line_ids` with `Command.create`:**
```xml
<record id="template_species" model="indaws.import.template">
    <field name="match_line_ids" eval="[
        Command.create({
            'sequence': 10,
            'model_id': ref('model_animal_species'),
            'field_id': ref('field_animal_species__provet_id'),
            'domain_operator': '=',
        }),
    ]"/>
</record>
```

**WARNING:** `match_field_ids` is a **COMPUTED** Many2many field (read-only). Writing to it
via `Command.link` does NOT create the required `match_line_ids` records. You MUST write to
`match_line_ids` directly with `Command.create`.

**Via SQL (emergency fix for existing templates):**
```sql
INSERT INTO indaws_import_template_match_line (template_id, field_id, domain_operator, sequence)
SELECT t.id, f.id, '=', 10
FROM indaws_import_template t
JOIN ir_model_fields f ON f.model = 'your.model' AND f.name = 'provet_id'
WHERE t.name = 'Your Template';
```

---

## Pitfall #3: `params.get()` Does Not Exist in Collector Context

**Severity:** HIGH - causes `NameError` at runtime

Connection params are **NOT** available as a `params` dict in the collector. They are injected
into the **variable's** eval context. The variable processes them and its `value` output is
injected into the collector context under `variable.name`.

```python
# WRONG - NameError: name 'params' is not defined (in collector code)
api_url = params.get('api_url')

# CORRECT architecture:
# 1. Connection has params: api_url, api_token
# 2. Connection has variable_id → variable "my_auth"
# 3. Variable code: value = {'base_url': api_url, 'headers': {...}}
# 4. Collector code: conn = my_auth; base_url = conn.get('base_url')
```

---

## Pitfall #4: `noupdate="1"` Prevents XML Updates on Upgrade

**Severity:** HIGH - changes to XML are silently ignored

Records with `noupdate="1"` will NOT be updated when you run `-u module`. This means changes
to `python_code`, `variable_ids`, or any other field in XML are ignored if the record already
exists in the database.

**During development/testing:**
```sql
UPDATE ir_model_data SET noupdate = false
WHERE module = 'your_module' AND noupdate = true;
```

**Or update records directly via API after upgrade.**

**For production:** Keep `noupdate="1"` to protect user customizations. Only initial install
creates the records from XML; subsequent upgrades skip them.

---

## Pitfall #5: API Fields May Differ from Documentation

**Severity:** MEDIUM - causes empty/wrong data

External API field names may differ from what you expect. Always verify the actual API response.

**Real example from Provet Cloud:**
- `/codelist/species/` returns `label` (not `name`)
- `/codelist/breeds/` returns `label` (not `name`) and `species` as URL (not `parent_id` int)
- `/department/` returns `name` (not `label`)

**Solution:** Always log a sample response during development:
```python
log('Sample record: %s' % json.dumps(results[0]) if results else 'empty', level='info')
```

---

## Pitfall #6: URL Fields vs Integer Fields in REST APIs

**Severity:** MEDIUM - resolver may fail silently

REST APIs may return related fields as URLs (`"species": "https://.../codelist/species/5826001/"`)
or as integers (`"parent_id": 5826001`). A URL resolver that does `split('/')[-1]` works for both,
but you must handle edge cases:

```python
# This handles both URLs and plain integers
def _resolve(url_or_id):
    if not url_or_id:
        return False
    try:
        return int(str(url_or_id).rstrip('/').split('/')[-1])
    except (ValueError, IndexError):
        return False
```

---

## Pitfall #7: Collector `variable_ids` Not Updated on Upgrade

**Severity:** MEDIUM - collector runs without expected variables

Many2many fields like `variable_ids` in XML with `noupdate="1"` won't be updated on upgrade.
If you add a new `variable_ids` reference to an existing collector XML, it won't apply.

**Fix via API:**
```python
# Add variable_id=4 to collector_id=23
collector.write({'variable_ids': [(4, variable_id)]})
```

**Fix via SQL:**
```sql
INSERT INTO indaws_import_data_collector_variable_rel (collector_id, variable_id)
VALUES (23, 4) ON CONFLICT DO NOTHING;
```

---

## Pitfall #8: Functions in Variables Need `nocopy=True`

**Severity:** LOW - framework handles this automatically

When a variable's `python_code` assigns a function to `value`, it works because
`indaws_import_variable.get_variables()` calls `safe_eval` with `nocopy=True`.
This preserves the function's closure, keeping `env` accessible.

**You don't need to do anything special** - just be aware that this is why it works.
If you see closure-related issues, verify `nocopy=True` is used in the `safe_eval` call.

---

## Pitfall #9: N+1 Query Problem with Many2one Lookups

**Severity:** HIGH - causes extreme slowness on large imports

Resolving Many2one fields by searching per row (e.g., country by code) creates
N+1 queries. With 40K records, that's 40K `search()` calls.

```python
# WRONG - N+1 queries (one search per row)
# In template line code:
country = env['res.country'].search([('code', '=', cell_value)], limit=1)
value = country.id if country else False

# CORRECT - Cache dict in collector (one query total, zero per row)
# In collector python_code, at the top:
_country_cache = {c['code']: c['id'] for c in env['res.country'].search_read([], ['code'])}

# In the row loop:
buffer.append({
    'country_id': _country_cache.get(country_code, False),  # Direct ID, no query
})
```

**Pattern:** Build `{lookup_key: odoo_id}` dict once at collector start. Emit the
resolved ID directly. Template line receives an integer and just does
`value = int(cell_value) if cell_value else False`.

Works for: countries, species, breeds, departments, categories - any small/medium
reference table.

---

## Pitfall #10: `archived → active` Boolean Inversion

**Severity:** MEDIUM - all records imported as active even if archived in source

Many external APIs return an `archived` boolean field. Odoo uses `active` (inverse).
Forgetting to invert causes archived records to appear as active in Odoo.

```python
# WRONG - imports archived field directly (Odoo doesn't have 'archived')
buffer.append({'archived': r.get('archived', False)})

# CORRECT - invert to 'active'
archived = r.get('archived', False)
active = not archived if archived is not None else True
buffer.append({'active': active})
```

**Also requires:** Enable `search_in_archived` on the template so `_get_match()`
finds archived records when updating them. Without it, archived records won't match
and will be duplicated.

---

## Pitfall #11: Selection Fields with Translated Values

**Severity:** MEDIUM - selection values silently rejected

External APIs may return selection values in a different language than what Odoo
expects. For example, Provet Cloud returns gender in Spanish ("Hembra"/"Macho")
but Odoo selection expects English ("female"/"male").

```python
# WRONG - Spanish value rejected by selection field
sex = r.get('gender', '')  # "Hembra" - not a valid selection value

# CORRECT - Map to expected selection values
_gender_map = {
    'hembra': 'female', 'macho': 'male',
    'female': 'female', 'male': 'male',
}
raw_gender = (r.get('gender', '') or '').strip().lower()
# Handle composite values like "Hembra, esterilizada"
sex = _gender_map.get(raw_gender.split(',')[0].strip(), '')
```

**Rule:** Always check what values the Odoo selection field expects and map
accordingly in the collector.

---

## Pitfall #12: `partner_firstname` Module Constraint

**Severity:** HIGH - blocks record creation with cryptic error

When importing `res.partner` with `partner_firstname` module installed, the
constraint `_check_name` requires at least `lastname` to have a value. If
firstname, lastname, and lastname2 are all empty/falsy, creation fails with
`EmptyNamesError`.

```python
# WRONG - if API returns empty names, constraint fails
buffer.append({
    'firstname': r.get('firstname', ''),
    'lastname': r.get('lastname', ''),
})

# CORRECT - ensure at least lastname has a value
name = prepare_name(firstname, lastname)
clean_lastname = lastname if name != 'No establecido' else 'No establecido'
buffer.append({
    'name': name,
    'firstname': firstname if firstname != '-' else '',
    'lastname': clean_lastname,
})
```

---

## Quick Reference: Variable Context by Location

| Code Location | Input Variable | Output Variable | Has `env`? | Has `params`? |
|---------------|---------------|-----------------|:----------:|:-------------:|
| Variable `python_code` | *(connection params by name)* | `value` | Yes | No (flat vars) |
| Collector `python_code` | *(variables by name)* | `value` | Yes | No |
| Template line `code` | `cell_value` | `value` | Yes | No |
| `before_import_code` | - | - | Yes | No |
| `before_create_update_code` | `record_vals` | (modify in place) | Yes | No |
| `finish_import_code` | `rec_import` | - | Yes | No |
