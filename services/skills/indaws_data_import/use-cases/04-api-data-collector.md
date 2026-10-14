# Use Case 04: API Data Collector

**Scenario:** Fetch data from external REST API and import into Odoo automatically.

**Difficulty:** ⭐⭐⭐ Advanced

**Time:** ~45 minutes

---

## Prerequisites

- [x] `indaws_data_import` module installed
- [x] `queue_job` configured
- [x] External API credentials
- [x] API documentation (endpoints, authentication)
- [x] Understanding of API response structure
- [x] `requests` library installed on Odoo server

---

## Scenario Description

**Business Need:** Daily import of customer data from external CRM API.

**API Details:**
- **Endpoint:** `https://crm-api.example.com/api/v1/customers`
- **Auth:** Bearer token
- **Method:** GET
- **Response format:** JSON

```json
{
  "data": [
    {
      "id": "CRM001",
      "name": "Acme Corp",
      "email": "contact@acme.com",
      "phone": "+1-555-0100",
      "country": "US",
      "vat": "US123456789"
    },
    ...
  ],
  "total": 150
}
```

**Requirements:**
- Fetch customers from API
- Transform JSON to Odoo format
- Match by external ID (CRM ID)
- Create/update partners
- Run daily at 2 AM

---

## Architecture

```
External CRM API
    ↓ (Data Collector fetches data)
JSON Response
    ↓ (Collector transforms to import format)
Import Template
    ↓ (Field mappings)
Odoo Partners
```

---

## Understanding the Code Context

Before writing data collector code, review [Python Code Context Reference](../references/python-code-context.md).

**Key points for data collectors:**
- Collector code runs inside `safe_eval`
- Use `wrap_module()` to import `requests` and other libraries
- Pre-loaded: `datetime`, `time`, `json`, `pytz`, `dateutil`
- Must set `value` variable with list of dicts (import data)
- Use `log()` for debugging and monitoring
- **Connection variables are injected as flat names** (NOT a `params` dict)
- **Collector `variable_ids`** inject additional variables (reusable helpers, functions)

---

## Step 1: Create Connection + Variable (Authentication)

The recommended architecture uses **Connection → Variable → Collector**:

1. **Connection** holds credentials (params: api_url, api_token)
2. **Variable** processes credentials and returns auth dict
3. **Collector** uses the variable's output to make API calls

### Create the Auth Variable

```python
# Create variable that handles authentication
variable_id = models.execute_kw(db, uid, password,
    'indaws.import.variable', 'create',
    [{
        'name': 'crm_auth',  # This name becomes available in collector context
        'description': 'CRM API authentication - returns {base_url, headers}',
        'python_code': '''
# Connection params are injected as flat variables (api_url, api_token)
# They come from the connection's param_ids
req = wrap_module('requests', ['post'])

if not api_token:
    raise UserError('CRM API token is required')

# For simple Bearer auth, just build the headers dict
value = {
    'base_url': api_url,
    'headers': {'Authorization': 'Bearer %s' % api_token},
}
'''
    }]
)
```

### Create the Connection

```python
# Create connection with params
connection_id = models.execute_kw(db, uid, password,
    'indaws.import.connection', 'create',
    [{
        'name': 'CRM API Connection',
        'variable_id': variable_id,  # Links to the auth variable
    }]
)

# Add connection params (injected into variable's eval context)
for param in [
    {'name': 'api_url', 'ttype': 'text', 'value_text': 'https://crm-api.example.com'},
    {'name': 'api_token', 'ttype': 'text', 'value_text': 'your-api-token-here'},
]:
    param['connection_id'] = connection_id
    models.execute_kw(db, uid, password, 'indaws.import.param', 'create', [param])
```

### Create the Data Collector

```python
collector_id = models.execute_kw(db, uid, password,
    'indaws.import.data.collector', 'create',
    [{
        'name': 'CRM Customers Collector',
        'python_code': '''
# ============================================================
# THIS CODE RUNS INSIDE ODOO WITH safe_eval
# Connection variables are injected as flat names.
# 'crm_auth' comes from the connection's variable (name="crm_auth")
# ============================================================

req = wrap_module('requests', ['get', 'post', 'exceptions'])

# crm_auth is the dict returned by the variable's python_code
conn = crm_auth
base_url = conn.get('base_url', '')
headers = conn.get('headers', {})

if not base_url:
    raise UserError('CRM auth variable returned no base_url')

# Fetch data from API (with pagination)
all_customers = []
page = 1

while True:
    try:
        response = req.get(
            '%s/api/v1/customers' % base_url,
            headers=headers,
            params={'page': page, 'per_page': 100},
            timeout=30
        )
    except Exception as e:
        raise UserError('API connection error: %s' % str(e))

    if response.status_code == 401:
        raise UserError('API authentication failed - check token')
    if response.status_code != 200:
        raise UserError('API Error %s: %s' % (response.status_code, response.text[:300]))

    data = response.json()
    customers = data.get('data', [])

    if not customers:
        break

    all_customers.extend(customers)

    if len(customers) < 100:
        break

    page += 1
    log('Fetched page %d, total so far: %d' % (page, len(all_customers)), level='info')

# Transform to import format (list of dicts with keys matching template line names)
import_data = []
for c in all_customers:
    import_data.append({
        'crm_id': c.get('id', ''),
        'name': c.get('name', ''),
        'email': c.get('email', ''),
        'phone': c.get('phone', ''),
        'country_code': c.get('country', ''),
        'vat': c.get('vat', ''),
    })

log('Fetched %d customers from CRM API' % len(import_data), level='info')

# MUST set value to list of dicts
value = import_data
'''
    }]
)
print(f"Data collector created with ID: {collector_id}")
```

### Large Datasets: yield_data Streaming

For APIs returning >5K records, use `yield_data()` instead of accumulating everything
in `value`. This keeps memory constant regardless of dataset size:

```python
# Inside collector python_code:
YIELD_SIZE = 10000
REQ_DELAY = 0.7  # Rate limit: ~85 req/min
buffer = []
total = 0

while True:
    if page > 1:
        time.sleep(REQ_DELAY)

    resp = req.get('%s/api/v1/customers' % base_url, headers=headers,
                   params={'page': page, 'per_page': 1000}, timeout=60)

    # Retry on 429 (throttled)
    if resp.status_code == 429:
        log('API throttled, waiting 30s...')
        time.sleep(30)
        continue

    if resp.status_code != 200:
        log('API error [%s]' % resp.status_code, level='error')
        break

    data = resp.json()
    for r in data.get('results', []):
        buffer.append({...})  # Transform row

    # Flush when buffer is large enough
    if len(buffer) >= YIELD_SIZE:
        total += len(buffer)
        log('yield_data %d records (total: %d)' % (len(buffer), total))
        yield_data(buffer)
        buffer = []

    if not data.get('next'):
        break
    page += 1

# Flush remaining
if buffer:
    total += len(buffer)
    yield_data(buffer)

log('Total: %d records' % total)
```

Each `yield_data()` call triggers the scheduler to create a new `indaws.import` record
and process it. The collector does NOT set `value` — streaming replaces it.

### Optional: Add Helper Variables to Collector

If your collector needs reusable functions (e.g., URL resolvers), link them via `variable_ids`:

```python
# Create a helper variable (callable function)
helper_var_id = models.execute_kw(db, uid, password,
    'indaws.import.variable', 'create',
    [{
        'name': 'find_country',
        'python_code': '''
def _find(code):
    if not code:
        return False
    country = env['res.country'].search([('code', '=', code)], limit=1)
    return country.id if country else False

value = _find
'''
    }]
)

# Link to collector (in addition to connection variables)
models.execute_kw(db, uid, password,
    'indaws.import.data.collector', 'write',
    [[collector_id], {'variable_ids': [(4, helper_var_id)]}]
)

# Now in collector code, find_country is callable:
# country_id = find_country('ES')  → returns Odoo ID
```

---

## Step 2: Create Import Template

```python
# Get model ID for res.partner
model_id = models.execute_kw(db, uid, password,
    'ir.model', 'search',
    [[('model', '=', 'res.partner')]], {'limit': 1}
)[0]

# Create template
template_id = models.execute_kw(db, uid, password,
    'indaws.import.template', 'create',
    [{
        'name': 'Import CRM Customers',
        'model_id': model_id,
        'file_type': 'csv',  # Data collector returns dict, treated as CSV
        'chunksize': 100,
        'continue_error': True,
    }]
)

print(f"Template created with ID: {template_id}")
```

### Add Field Mappings

```python
# Get field IDs
fields = models.execute_kw(db, uid, password,
    'ir.model.fields', 'search_read',
    [[
        ('model', '=', 'res.partner'),
        ('name', 'in', ['name', 'email', 'phone', 'country_id', 'vat'])
    ]],
    {'fields': ['name', 'id']}
)

field_map = {f['name']: f['id'] for f in fields}

# Field mappings
mappings = [
    {
        'template_id': template_id,
        'names': 'Name',
        'field_id': field_map['name'],
        'sequence': 10,
    },
    {
        'template_id': template_id,
        'names': 'Email',
        'field_id': field_map['email'],
        'sequence': 20,
    },
    {
        'template_id': template_id,
        'names': 'Phone',
        'field_id': field_map['phone'],
        'sequence': 30,
    },
    {
        'template_id': template_id,
        'names': 'Country Code',
        'field_id': field_map['country_id'],
        'sequence': 40,
        # Custom code to find country by code
        # NOTE: This code runs inside safe_eval
        'code': '''
# Search country by ISO code (cell_value = "US", "ES", etc.)
country = env['res.country'].search([('code', '=', cell_value)], limit=1)
if country:
    value = country.id
    parse_continue = True
else:
    # Country not found - skip field or raise error
    log(f'Country not found: {cell_value}', level='warning')
    value = False
    parse_continue = True
'''
    },
    {
        'template_id': template_id,
        'names': 'VAT',
        'field_id': field_map['vat'],
        'sequence': 50,
    },
]

for mapping in mappings:
    models.execute_kw(db, uid, password,
        'indaws.import.template.line', 'create', [mapping]
    )

print("Field mappings created")
```

### Configure Matching (by External ID)

```python
# Since data collector creates External ID column, matching is automatic
# The import system handles 'External ID' column specially
# But we can also add explicit matching on other fields:

# Match by email as fallback
email_field_id = models.execute_kw(db, uid, password,
    'ir.model.fields', 'search',
    [[('model', '=', 'res.partner'), ('name', '=', 'email')]], {'limit': 1}
)[0]

models.execute_kw(db, uid, password,
    'indaws.import.template.match.line', 'create',
    [{
        'template_id': template_id,
        'field_id': email_field_id,
        'domain_operator': '=',
        'sequence': 10,
    }]
)

print("Matching configured via External ID column + email fallback")
```

---

## Step 3: Create Scheduler (for Daily Execution)

```python
# Create scheduler
scheduler_id = models.execute_kw(db, uid, password,
    'indaws.import.scheduler', 'create',
    [{
        'name': 'Daily CRM Customer Import',
        'template_id': template_id,
        'active': True,
        'interval_number': 1,
        'interval_type': 'days',
        'nextcall': '2024-01-01 02:00:00',  # Daily at 2 AM
    }]
)

print(f"Scheduler created with ID: {scheduler_id}")

# Link data collector to scheduler
models.execute_kw(db, uid, password,
    'indaws.import.scheduler', 'write',
    [[scheduler_id], {
        'data_collector_ids': [(6, 0, [collector_id])]
    }]
)

print("Data collector linked to scheduler")
```

---

## Step 4: Test Data Collector Manually

Before scheduling, test the data collector:

```python
# Execute scheduler manually to test
import_id = models.execute_kw(db, uid, password,
    'indaws.import.scheduler', 'action_execute_import',
    [[scheduler_id]]
)

print(f"Import created and launched: {import_id}")
```

---

## Step 5: Monitor Execution

```python
import time

# Wait for import to complete
while True:
    import_data = models.execute_kw(db, uid, password,
        'indaws.import', 'search_read',
        [[('scheduler_id', '=', scheduler_id)]],
        {
            'fields': ['state', 'created_record_count', 'updated_record_count', 'error_count'],
            'order': 'id desc',
            'limit': 1
        }
    )

    if not import_data:
        print("No import found yet...")
        time.sleep(5)
        continue

    imp = import_data[0]
    print(f"State: {imp['state']} | Created: {imp['created_record_count']} | Updated: {imp['updated_record_count']} | Errors: {imp['error_count']}")

    if imp['state'] in ['done', 'cancel']:
        break

    time.sleep(10)

print("Import completed!")
```

---

## Testing & Validation

### Test API Connection

Test the API independently first:

```python
# Run this EXTERNALLY to test API
import requests

response = requests.get(
    'https://crm-api.example.com/api/v1/customers',
    headers={'Authorization': 'Bearer your-api-token'},
    params={'page': 1, 'per_page': 5},
    timeout=30
)

print(f"API Status: {response.status_code}")
print(f"Sample data: {response.json()}")
```

### Verify Imported Partners

```python
# Check partners with CRM external IDs
external_ids = models.execute_kw(db, uid, password,
    'ir.model.data', 'search_read',
    [[('module', '=', '__import__'), ('model', '=', 'res.partner'), ('name', 'like', 'crm_')]],
    {'fields': ['name', 'res_id'], 'limit': 10}
)

for ext_id in external_ids:
    partner = models.execute_kw(db, uid, password,
        'res.partner', 'read',
        [[ext_id['res_id']]],
        {'fields': ['name', 'email', 'vat', 'country_id']}
    )[0]
    print(f"Partner: {partner['name']} - {partner['email']}")
```

---

## Performance: Cache Dict for Lookups

When the collector needs to resolve Many2one fields (e.g., country by code), avoid
N+1 queries by building a cache dict at the start:

```python
# One query at start → zero queries per row
_country_cache = {c['code']: c['id'] for c in env['res.country'].search_read([], ['code'])}
_species_cache = {s['name'].strip().upper(): s['id'] for s in env['animal.species'].search_read([], ['name'])}
log('Cache: %d countries, %d species' % (len(_country_cache), len(_species_cache)))

# In the row loop:
country_code = r.get('country', {}).get('code', '')
buffer.append({
    'country_id': _country_cache.get(country_code, False),
    'specie_id': _species_cache.get(raw_species.upper(), False),
})
```

The template line receives an integer ID directly. No `env['model'].search()` per row.

---

## Troubleshooting

### Issue: "API Error 401: Unauthorized"
**Solution:** Verify API token is correct and not expired

### Issue: "API Error 429: Too Many Requests"
**Solution:** Add rate limiting AND retry logic:
```python
# Fixed delay between pages
REQ_DELAY = 0.7  # ~85 req/min (safe for 100 req/min limit)

# In the pagination loop:
if page > 1:
    time.sleep(REQ_DELAY)

resp = req.get(url, headers=headers, params=params, timeout=60)

# Auto-retry on 429
if resp.status_code == 429:
    wait = 30
    log('API throttled on page %d, waiting %ds...' % (page, wait))
    time.sleep(wait)
    resp = req.get(url, headers=headers, params=params, timeout=60)
```

### Issue: "KeyError: 'data'"
**Solution:** API response structure changed, verify JSON structure

### Issue: Data collector returns empty list
**Solution:**
- Check API has data
- Verify pagination logic
- Add debug logging:
```python
log(f'API response: {response.text[:500]}', level='info')
```

### Issue: "No module named 'requests'"
**Solution:**
```bash
pip install requests
# Restart Odoo
```

### Issue: "wrap_module" not recognized
**Solution:** Update indaws_data_import to latest version

---

## Advanced: Add Error Notifications

```python
# Add code to send email on errors
models.execute_kw(db, uid, password,
    'indaws.import.template', 'write',
    [[template_id], {
        'finish_import_code': '''
# ============================================================
# THIS CODE RUNS INSIDE ODOO WITH safe_eval
# rec_import contains the import record
# ============================================================

# Send email if errors
if rec_import.error_count > 0:
    rec_import.message_post(
        body=f'Import completed with {rec_import.error_count} errors',
        subject='CRM Import Errors',
        partner_ids=env.ref('base.user_admin').partner_id.ids
    )

# Log summary
log(f'Import finished: {rec_import.created_record_count} created, {rec_import.updated_record_count} updated, {rec_import.error_count} errors', level='info')
'''
    }]
)
```

---

## Advanced: OAuth 2.0 Authentication

For APIs requiring OAuth, handle it in the **variable** (not the collector):

```python
# Variable python_code (connection params: client_id, client_secret, base_url, token_url)
req = wrap_module('requests', ['post'])

# Connection params are available as flat variables
resp = req.post(token_url, data={
    'grant_type': 'client_credentials',
    'client_id': client_id,
    'client_secret': client_secret,
}, timeout=30)

if resp.status_code != 200:
    raise UserError('OAuth failed [%s]: %s' % (resp.status_code, resp.text[:300]))

token_data = resp.json()
access_token = token_data.get('access_token', '')

if not access_token:
    raise UserError('OAuth response missing access_token')

# Return auth dict - collector receives this under variable.name
value = {
    'base_url': base_url,
    'headers': {
        'Authorization': 'Bearer %s' % access_token,
        'Accept': 'application/json',
    },
}
```

**Why in the variable?** The variable is evaluated once per scheduler run. All collectors
sharing the same connection reuse the same token without re-authenticating.

---

## Advanced: Incremental Import

The scheduler provides `last_execution_date` in the collector's eval context. Use it
with a safety gap (subtract one cron interval) to avoid missing records near the boundary:

```python
# Inside collector python_code:

# Incremental sync: filter by modification date if available
modified_since = None
if last_execution_date and scheduler:
    # Safety gap: subtract one full cron interval
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
    log('Incremental: modified_since=%s (last_exec=%s, gap=%s)' % (
        modified_since, last_execution_date.strftime('%Y-%m-%dT%H:%M:%S'), str(_delta)))
else:
    log('Full sync: no last_execution_date, downloading all records')

# Add to API params
api_params = {'page': page, 'per_page': 100}
if modified_since:
    api_params['modified_since'] = modified_since

response = req.get('%s/api/v1/customers' % base_url, headers=headers, params=api_params)
```

**How it works:**
- First run: `last_execution_date` is None → full download
- Subsequent runs: only records modified since last run (with overlap gap)
- To force full re-download: clear `last_execution_date` on the scheduler record

---

## Related Documentation

- **[Python Code Context](../references/python-code-context.md)** - Available functions and modules
- **[03-scheduled-import.md](03-scheduled-import.md)** - More scheduling options
- **[05-sftp-automated-import.md](05-sftp-automated-import.md)** - SFTP data collection
- **[07-complex-transformations.md](07-complex-transformations.md)** - Advanced data transformations

---

## Summary

✅ **What we achieved:**
- Created data collector to fetch from API using `wrap_module('requests', ...)`
- Handled authentication and pagination
- Transformed JSON to import format
- Configured automatic daily execution
- Set up error monitoring

🎯 **Key components:**
- **Data Collector**: Fetches external data (code runs in `safe_eval`)
- **Scheduler**: Automates execution
- **Template**: Defines transformation and field mappings
- **queue_job**: Processes asynchronously

🎯 **Key learnings:**
- Collector code uses `safe_eval` - use `wrap_module()` for imports
- Must set `value` variable with list of dicts
- **Connection params go into variable context, NOT collector context** (no `params.get()`)
- Connection variables are injected as flat names in collector context
- Collector `variable_ids` allows injecting reusable helpers (including callable functions)
- Variables can return functions via `value = _my_func` (closures preserve `env`)
- Use `log()` for debugging
- `requests` library must be installed on Odoo server

**Duration:** ~45 minutes setup, runs automatically daily
