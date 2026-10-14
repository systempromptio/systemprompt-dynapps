# Python Code Context Reference

This document describes all variables, functions, and modules available when writing Python code in import templates.

---

## Adding External Libraries

### wrap_module() - Safe Import

Use `wrap_module()` to import external Python libraries with specific methods exposed. This is the **recommended** method as it provides better security.

```python
# Syntax
module = wrap_module('module_name', ['method1', 'method2', ...])

# Examples
requests = wrap_module('requests', ['get', 'post', 'put', 'delete', 'Session'])
response = requests.get('https://api.example.com/data')

# Paramiko for SFTP
paramiko = wrap_module('paramiko', ['Transport', 'SFTPClient'])
transport = paramiko.Transport(('sftp.example.com', 22))

# BeautifulSoup for HTML parsing
bs4 = wrap_module('bs4', ['BeautifulSoup'])
soup = bs4.BeautifulSoup(html_content, 'html.parser')

# PIL for image processing
PIL = wrap_module('PIL', ['Image'])
img = PIL.Image.open(io.BytesIO(image_data))

# openpyxl for Excel manipulation
openpyxl = wrap_module('openpyxl', ['load_workbook', 'Workbook'])
wb = openpyxl.load_workbook(file_path)
```

### import_module() - Direct Import

Use `import_module()` for direct imports when you need full module access.

```python
# Syntax
module = import_module('module_name', ['submodule1', 'submodule2'])

# Examples
paramiko = import_module('paramiko', ['Transport', 'SFTPClient'])

# For nested imports
from_list = import_module('xml.etree', ['ElementTree'])
```

**Note:** `wrap_module()` is preferred for security. Only use `import_module()` when necessary.

---

## Pre-loaded Modules

These modules are already available in the evaluation context:

| Module | Description | Example |
|--------|-------------|---------|
| `time` | Time functions | `time.sleep(1)` |
| `datetime` | Date/time classes | `datetime.datetime.now()` |
| `dateutil` | Date parsing | `dateutil.parser.parse('2024-01-15')` |
| `pytz` | Timezone handling | `pytz.timezone('Europe/Madrid')` |
| `json` | JSON encoding/decoding | `json.loads(data)` |
| `hashlib` | MD5 hashing | `hashlib.md5(text.encode()).hexdigest()` |

### Examples

```python
# Parse flexible date formats
from dateutil import parser
date = dateutil.parser.parse('15/01/2024', dayfirst=True)

# Work with timezones
tz = pytz.timezone('Europe/Madrid')
local_time = datetime.datetime.now(tz)

# JSON operations
data = json.loads('{"name": "Product A"}')
json_str = json.dumps(data, ensure_ascii=False)

# MD5 hash for deduplication
hash_value = hashlib.md5(f"{name}|{code}".encode()).hexdigest()
```

---

## Always in scope, and not obvious (verified in source)

These come from `_get_eval_context()`
(`indawsodoo/18/indaws_data_import/models/indaws_import_template.py:259-460`, `IndawsImportTemplate._get_eval_context`) and are available in
**every** Python stage, yet none of them appeared in this documentation until 2026-08-15:

| Name | What it is | Why it matters |
|---|---|---|
| `Command` | Odoo's `Command` namespace | Use `Command.link(id)` / `Command.set(ids)` for x2many instead of legacy tuples `(6, 0, [...])` |
| `odoo` | the `odoo` module itself | Full API surface from inside the sandbox — powerful, and worth knowing before granting template edition to anyone |
| `record` / `records` | the record(s) being processed | Default to `False`; populated depending on stage |
| `ctx` | `self._context` | The context the import runs with |
| `_logger` | logger proxy | Prefer `log(...)`, but this exists |
| `Warning` | alias of `UserError` | **Deprecated in the code itself** (`"TODO: Remove it"`), do not use it in new code |

## Context Variables by Code Location

**CRITICAL:** Different code locations have different variables available. Using the wrong
variable name is the #1 source of runtime errors.

### Collector `python_code` Context

| Variable | Type | Description |
|----------|------|-------------|
| `env` | `Environment` | Odoo environment |
| `wrap_module` | function | Import external libraries safely |
| `log` | function | Logging function |
| `UserError` | exception | Raise to stop execution |
| `value` | any | **Output** - set this to list of dicts (the collected data) |
| `last_execution_date` | datetime/None | Last successful scheduler execution (for incremental sync) |
| `scheduler` | record/None | The scheduler record (has interval_type, interval_number) |
| `yield_data` | function | Streaming: flush buffer to create import + process immediately |
| *(variable names)* | any | Connection variables + collector `variable_ids` injected by name |

**Connection variables are injected as flat names, NOT as a `params` dict:**
```python
# WRONG - params dict does NOT exist in collector context
api_url = params.get('api_url')

# CORRECT - connection variable is injected under its variable.name
# If connection has variable_id with name="provet_oauth_token",
# then provet_oauth_token is directly available:
conn = provet_oauth_token  # dict with {base_url, headers, ...}
base_url = conn.get('base_url', '')
```

**Connection params are injected into the variable's eval context, not the collector's.**
The variable processes them and its `value` output becomes the collector context variable.

### Template Line `code` Context (Field Transformation)

| Variable | Type | Description |
|----------|------|-------------|
| `cell_value` | any | **Input** - the value from the row for this field |
| `env` | `Environment` | Odoo environment |
| `row` | dict | The full current row data |
| `tmpl_field` | record | The template line record being processed |
| `rec_import` | record | The import execution record |
| `value` | any | **Output** - set this to the transformed value |
| `parse_continue` | bool | Set to True to indicate you handled parsing |

```python
# WRONG - NameError: name 'value' is not defined (when reading input)
if value:
    value = int(value)

# CORRECT - read from cell_value, write to value
if cell_value:
    value = int(cell_value)
else:
    value = False
```

**How it works internally** (method `_parse_code_value`):
1. Framework sets `eval_ctx['cell_value'] = <input from row>`
2. Your code executes via `safe_eval`
3. Framework reads result from `eval_ctx.get('value', False)`

### Template `before_import_code` / `finish_import_code` Context

| Variable | Type | Description |
|----------|------|-------------|
| `env` | `Environment` | Odoo environment |
| `rec_import` | record | The import execution record (has `.rows` with all row data) |
| `log` | function | Logging function |
| `wrap_module` | function | Import external libraries safely |
| `model` | `Model` | The target model |

### Template `before_create_update_code` Context

| Variable | Type | Description |
|----------|------|-------------|
| `env` | `Environment` | Odoo environment |
| `record_vals` | dict | Values that will be used for create/write (modifiable) |
| `rec_import` | record | The import execution record |

### General Odoo Examples

```python
# Search for existing records
partner = env['res.partner'].search([('email', '=', email)], limit=1)

# Create new record
new_product = env['product.product'].create({
    'name': 'New Product',
    'default_code': 'PROD001',
})

# Access current user
current_user = env.user
company = env.company

# Use sudo for elevated permissions
admin_partner = env['res.partner'].sudo().search([('id', '=', 1)])
```

---

## Variable `python_code` Context

Variables (`indaws.import.variable`) are evaluated before collector code runs.

| Variable | Type | Description |
|----------|------|-------------|
| `env` | `Environment` | Odoo environment |
| *(connection params)* | str/int | Each `indaws.import.param` injected by its `name` |
| `value` | any | **Output** - set this to the variable's result |

```python
# Example: OAuth token variable
# Connection params (client_id, client_secret, base_url, token_url) are available by name
req = wrap_module('requests', ['post'])

resp = req.post(token_url, data={
    'grant_type': 'client_credentials',
    'client_id': client_id,
    'client_secret': client_secret,
})
token = resp.json().get('access_token', '')
value = {'base_url': base_url, 'headers': {'Authorization': 'Bearer %s' % token}}
```

### Variables as Functions

You can assign a **function** to `value`. The function retains access to `env` via closure:

```python
# This variable returns a callable function, not a static value
def _resolve(url, model_name=None):
    if not url:
        return False
    try:
        pid = int(str(url).rstrip('/').split('/')[-1])
    except (ValueError, IndexError):
        return False
    if not model_name:
        return pid
    record = env[model_name].search([('provet_id', '=', pid)], limit=1)
    return record.id if record else False

value = _resolve  # Callable! Used as: url_resolver(url, 'res.partner')
```

**Why this works:** `safe_eval` with `nocopy=True` preserves function closures, so `env`
remains accessible when the function is called later from the collector context.

---

## Incremental Sync Pattern

Collectors can implement incremental sync using `last_execution_date` from the
scheduler context. This avoids re-downloading the entire dataset on each run.

```python
# In collector python_code:
modified_since = None
if last_execution_date and scheduler:
    # Safety gap: subtract one cron interval to avoid missing records
    _interval_map = {
        'minutes': 'minutes', 'hours': 'hours',
        'days': 'days', 'weeks': 'weeks', 'months': 'days',
    }
    _interval_type = scheduler.interval_type or 'hours'
    _interval_number = scheduler.interval_number or 1
    if _interval_type == 'months':
        _interval_number = _interval_number * 30
    _delta_key = _interval_map.get(_interval_type, 'hours')
    _delta = datetime.timedelta(**{_delta_key: _interval_number})
    _modified_from = last_execution_date - _delta
    modified_since = _modified_from.strftime('%Y-%m-%dT%H:%M:%S+00:00')
    log('Incremental sync: modified_since=%s' % modified_since)
else:
    log('Full sync: no last_execution_date')

# Use in API params:
api_params = {'page': page, 'page_size': 1000}
if modified_since:
    api_params['modified__gte'] = modified_since
```

**How it works:**
- First run: `last_execution_date` is None → full sync
- Subsequent runs: filters by `modified__gte` with safety gap
- To force full sync: clear `last_execution_date` on the scheduler record

---

## Exception Classes

| Exception | Use Case |
|-----------|----------|
| `UserError` | User-friendly error message (stops import) |
| `ValidationError` | Validation failure |
| `SkipRow` | Skip current row without error |

### Examples

```python
# Stop import with error
if not required_field:
    raise UserError('Required field is missing!')

# Validation error
if price < 0:
    raise ValidationError('Price cannot be negative')

# Skip row silently
if should_skip:
    raise SkipRow('Skipping duplicate record')
```

---

## Logging Function

```python
# Syntax
log(message, level="info")

# Levels: "info", "warning", "error", "debug"

# Examples
log("Starting import process")
log("Record not found, creating new", level="warning")
log(f"Error processing row: {str(e)}", level="error")
```

Logs are stored in `ir.logging` table and visible in Odoo logs.

---

## Helper Functions

### Data Transformation Helpers

#### convert_to_dict()
Convert list of dicts to dict indexed by key field.

```python
# Syntax
convert_to_dict(list_of_dicts, key_field, key_value=None, key_cast=str)

# Example
products = [
    {'code': 'A001', 'name': 'Product A', 'price': 10},
    {'code': 'A002', 'name': 'Product B', 'price': 20},
]

# Convert to dict by code
by_code = convert_to_dict(products, 'code')
# Result: {'A001': {'code': 'A001', 'name': 'Product A', 'price': 10}, ...}

# Get only names
names_by_code = convert_to_dict(products, 'code', 'name')
# Result: {'A001': 'Product A', 'A002': 'Product B'}
```

#### group_by_key()
Group list of dicts by key field.

```python
# Syntax
group_by_key(dict_list, group_key)

# Example
orders = [
    {'customer': 'John', 'product': 'A'},
    {'customer': 'John', 'product': 'B'},
    {'customer': 'Jane', 'product': 'C'},
]

grouped = group_by_key(orders, 'customer')
# Result: {'John': [{...}, {...}], 'Jane': [{...}]}
```

#### get_dict_value()
Safely get value from dict with default.

```python
# Syntax
get_dict_value(dict_values, key, default=None, key_cast=str)

# Example
data = {'123': 'value'}
result = get_dict_value(data, 123, default='not found')
# Result: 'value' (key is cast to string)
```

#### ensure_value()
Return None if value is falsy.

```python
# Syntax
ensure_value(value)

# Example
name = ensure_value(row.get('name'))  # Returns None if empty string
```

#### to_bool()
Convert various values to boolean.

```python
# Syntax
to_bool(value)

# Truthy values: True, 'True', 'true', '1', 1, 'yes', 'Yes', 'YES'

# Example
is_active = to_bool(row.get('active'))  # '1' -> True
```

### Tree/Hierarchy Helpers

#### to_tree()
Build tree structure from flat list.

```python
# For hierarchical data like categories
tree = to_tree(categories_dict, parent_id, 'parent_id')
```

#### tree_to_str()
Convert tree to path string.

```python
path = tree_to_str(tree, 'name', separator=' / ')
# Result: "Root / Child / Grandchild"
```

### Text Parsing

#### parse_map_txt()
Parse mapping text to dictionary.

```python
# Syntax
parse_map_txt(map_txt, separator=';')

# Example
map_text = """
OLD001;NEW001
OLD002;NEW002
"""
mapping = parse_map_txt(map_text)
# Result: {'OLD001': ['NEW001'], 'OLD002': ['NEW002']}
```

#### flatten_values()
Flatten nested values.

```python
flat = flatten_values(nested_data)
```

#### flatten_list_of_dicts()
Flatten list of nested dictionaries.

```python
flat_list = flatten_list_of_dicts(nested_list)
```

---

## Record Creation Helpers

### find_or_create_record()
Find existing record or create new one.

```python
# Syntax
find_or_create_record(model, domain, values, create=True)

# Example
partner = find_or_create_record(
    'res.partner',
    [('email', '=', 'john@example.com')],
    {'name': 'John Doe', 'email': 'john@example.com'},
    create=True
)
```

### find_or_create_record_tree()
Find or create hierarchical records (e.g., categories).

```python
# Syntax
find_or_create_record_tree(model, names, name_field='name', parent_field='parent_id')

# Example - Create category path "Electronics / Computers / Laptops"
category = find_or_create_record_tree(
    'product.category',
    ['Electronics', 'Computers', 'Laptops']
)
```

### url_to_base64()
Download URL and convert to base64.

```python
# Syntax
url_to_base64(url)

# Example
image_data = url_to_base64('https://example.com/product.jpg')
record_vals['image_1920'] = image_data
```

---

## Module Check

### module_installed()
Check if Odoo module is installed.

```python
# Syntax
module_installed(module_name)

# Example
if module_installed('sale'):
    # Sale module specific logic
    pass
```

---

## Date/Time Format Constants

| Constant | Value | Example |
|----------|-------|---------|
| `DEFAULT_SERVER_DATE_FORMAT` | `'%Y-%m-%d'` | `'2024-01-15'` |
| `DEFAULT_SERVER_DATETIME_FORMAT` | `'%Y-%m-%d %H:%M:%S'` | `'2024-01-15 14:30:00'` |
| `DEFAULT_SERVER_TIME_FORMAT` | `'%H:%M:%S'` | `'14:30:00'` |

```python
# Example
from datetime import datetime
date_str = datetime.now().strftime(DEFAULT_SERVER_DATE_FORMAT)
```

---

## Command Object

For ORM operations in field mappings:

```python
# Available commands (Odoo ORM)
Command.create(values)      # (0, 0, values)
Command.update(id, values)  # (1, id, values)
Command.delete(id)          # (2, id, 0)
Command.unlink(id)          # (3, id, 0)
Command.link(id)            # (4, id, 0)
Command.clear()             # (5, 0, 0)
Command.set(ids)            # (6, 0, ids)

# Example: Set Many2many field
record_vals['tag_ids'] = [Command.set([1, 2, 3])]

# Example: Create related records
record_vals['order_line'] = [
    Command.create({'product_id': 1, 'quantity': 5}),
    Command.create({'product_id': 2, 'quantity': 3}),
]
```

---

## Complete Example

```python
# Collector python_code example (correct pattern)
req = wrap_module('requests', ['get', 'post'])

# Connection variable is injected as flat name (NOT params.get!)
conn = my_auth_variable  # dict from connection's variable python_code
base_url = conn.get('base_url', '')
headers = conn.get('headers', {})

if not base_url:
    raise UserError('Auth variable returned no base_url')

# Cache for Many2one resolution (one query, zero per row)
_country_cache = {c['code']: c['id'] for c in env['res.country'].search_read([], ['code'])}

# Paginate API with rate limiting
buffer = []
total = 0
page = 1
YIELD_SIZE = 10000
REQ_DELAY = 0.7

while True:
    if page > 1:
        time.sleep(REQ_DELAY)

    resp = req.get(
        '%s/api/v1/customers' % base_url,
        headers=headers,
        params={'page': page, 'page_size': 1000},
        timeout=60,
    )

    if resp.status_code == 429:
        time.sleep(30)
        continue

    if resp.status_code != 200:
        log('API error [%s]: %s' % (resp.status_code, resp.text[:300]), level='error')
        break

    data = resp.json()
    results = data.get('results', [])

    for r in results:
        country_code = r.get('country', {}).get('code', '')
        buffer.append({
            'external_id': r.get('id', 0),
            'name': r.get('name', ''),
            'email': r.get('email', ''),
            'country_id': _country_cache.get(country_code, False),
        })

    if len(buffer) >= YIELD_SIZE:
        total += len(buffer)
        yield_data(buffer)
        buffer = []

    if not data.get('next'):
        break
    page += 1

if buffer:
    total += len(buffer)
    yield_data(buffer)

log('Fetched %d records total' % total)
```

---

## Related Documentation

- **[Template Structure](template-structure.md)** - How to configure import templates
- **[Field Mappings](field-mappings.md)** - Configure field mappings with Python code
- **[Data Collectors](data-collectors-guide.md)** - Fetch data from external sources
