# Python Code Context Reference

This document describes all variables, functions, and modules available when writing Python code in export templates.

---

## Adding External Libraries

### wrap_module() - Safe Import

Use `wrap_module()` to import external Python libraries with specific methods exposed. This is the **recommended** method as it provides better security.

```python
# Syntax
module = wrap_module('module_name', ['method1', 'method2', ...])

# Examples
requests = wrap_module('requests', ['get', 'post', 'put', 'delete', 'Session'])
response = requests.post('https://api.example.com/orders', json=data)

# Paramiko for SFTP uploads
paramiko = wrap_module('paramiko', ['Transport', 'SFTPClient'])
transport = paramiko.Transport(('sftp.example.com', 22))
transport.connect(username='user', password='pass')
sftp = paramiko.SFTPClient.from_transport(transport)

# CSV generation
csv_module = wrap_module('csv', ['writer', 'DictWriter'])
writer = csv_module.writer(output, delimiter=';')

# XML generation
ET = wrap_module('xml.etree.ElementTree', ['Element', 'SubElement', 'tostring'])
root = ET.Element('orders')

# Base64 encoding
base64_mod = wrap_module('base64', ['b64encode', 'b64decode'])
encoded = base64_mod.b64encode(content.encode())
```

### import_module() - Direct Import

Use `import_module()` for direct imports when you need full module access.

```python
# Syntax
module = import_module('module_name', ['submodule1', 'submodule2'])

# Examples
io_module = import_module('io', ['StringIO', 'BytesIO'])
output = io_module.StringIO()

# UUID generation
uuid_mod = import_module('uuid', ['uuid4'])
unique_id = str(uuid_mod.uuid4())
```

**Note:** `wrap_module()` is preferred for security. Only use `import_module()` when necessary.

---

## Pre-loaded Modules

These modules are already available in the evaluation context:

| Module | Description | Example |
|--------|-------------|---------|
| `time` | Time functions | `time.strftime('%Y%m%d')` |
| `datetime` | Date/time classes | `datetime.datetime.now()` |
| `dateutil` | Date utilities | `dateutil.relativedelta.relativedelta(months=1)` |
| `pytz` | Timezone handling | `pytz.timezone('UTC')` |
| `json` | JSON encoding/decoding | `json.dumps(data, indent=2)` |

### Examples

```python
# Format dates for export
export_date = datetime.datetime.now().strftime('%Y-%m-%d %H:%M:%S')

# Work with timezones
tz = pytz.timezone('Europe/Madrid')
local_time = datetime.datetime.now(tz)
utc_time = local_time.astimezone(pytz.UTC)

# JSON payload
payload = json.dumps({
    'orders': data,
    'timestamp': export_date
}, ensure_ascii=False, indent=2)

# Calculate relative dates
from dateutil.relativedelta import relativedelta
last_month = datetime.date.today() - relativedelta(months=1)
```

---

## Odoo Context Variables

| Variable | Type | Description |
|----------|------|-------------|
| `env` | `Environment` | Odoo environment for database access |
| `model` | `Model` | The model being exported (e.g., `env['sale.order']`) |
| `ctx` | `dict` | Current context dictionary |
| `records` | `recordset` | All records to export (batch mode) |
| `record` | `record` | Single record (when `iter_record=True`) |
| `rec_export` | `record` | Export configuration record |
| `rec_template` | `record` | Template record |
| `rec_registry` | `record` | Current registry (execution log) |

### Examples

```python
# Access export parameters: params are injected FLAT by name into the
# eval context (export.py does eval_ctx.update(params)). Use the bare name
# directly — there is NO `params` dict, so `params.get('api_key')` raises NameError.
headers = {'Authorization': f'Bearer {api_key}'}   # api_key, endpoint_url, ... are bare names
target = endpoint_url

# Read related data
for order in records:
    customer = order.partner_id
    lines = order.order_line

    for line in lines:
        product = line.product_id
        qty = line.product_uom_qty

# Create attachment for exported file
attachment = env['ir.attachment'].create({
    'name': f'export_{datetime.datetime.now():%Y%m%d}.csv',
    'type': 'binary',
    'datas': base64.b64encode(content.encode()),
    'res_model': 'indaws.export',
    'res_id': rec_export.id,
})

# Access company data
company = env.company
company_vat = company.vat
```

---

## Exception Classes

| Exception | Use Case |
|-----------|----------|
| `UserError` | User-friendly error message (stops export) |

### Examples

```python
# Stop export with error
if not api_key:
    raise UserError('API key is required!')

# Handle API errors
if response.status_code != 200:
    raise UserError(f'API Error {response.status_code}: {response.text}')

# Validation before export
if not records:
    raise UserError('No records to export')
```

---

## Logging Functions

### log() - Export Log
Logs to `indaws.export.log` with structured data.

```python
# Syntax
log(level, message, result_dict)

# Levels: "info", "warning", "error"

# Examples
log('info', 'Starting export', {'record_count': len(records)})

log('warning', 'Some records skipped', {
    'skipped': 5,
    'reason': 'Missing required fields'
})

log('error', 'API connection failed', {
    'endpoint': api_url,
    'status_code': response.status_code
})
```

### ir_log() - System Log
Logs to Odoo's `ir.logging` table.

```python
# Syntax
ir_log(message, level="info")

# Examples
ir_log("Export started")
ir_log("Connection error", level="error")
```

---

## Result Variable

The `result` variable should be set with export results. It's returned to the registry.

```python
# Set export result
result = {
    'success': True,
    'records_exported': len(records),
    'api_response': response.json(),
    'filename': 'export_20240115.csv',
}

# For errors (if not raising exception)
result = {
    'success': False,
    'error': str(e),
    'records_attempted': len(records),
}
```

---

## Action Variable

The `action` variable can return a UI action after manual exports.

```python
# Return action to show attachment
action = {
    'type': 'ir.actions.act_url',
    'url': f'/web/content/{attachment.id}?download=true',
    'target': 'new',
}

# Return action to open records
action = {
    'type': 'ir.actions.act_window',
    'name': 'Exported Records',
    'res_model': 'sale.order',
    'view_mode': 'tree,form',
    'domain': [('id', 'in', exported_ids)],
}

# No UI action needed
action = None
```

---

## Common Export Patterns

### JSON API Export

```python
import requests
import json

requests = wrap_module('requests', ['post', 'get'])

# Build payload
payload = {
    'orders': []
}

for order in records:
    payload['orders'].append({
        'id': order.name,
        'customer': order.partner_id.name,
        'date': order.date_order.strftime('%Y-%m-%d'),
        'total': float(order.amount_total),
        'lines': [{
            'product': line.product_id.name,
            'qty': line.product_uom_qty,
            'price': float(line.price_unit),
        } for line in order.order_line]
    })

# Send to API (params are bare names in the eval context)
response = requests.post(
    f'{api_url}/orders',
    json=payload,
    headers={
        'Authorization': f'Bearer {api_key}',
        'Content-Type': 'application/json',
    },
    timeout=30
)

if response.status_code not in [200, 201]:
    raise UserError(f'API Error: {response.status_code} - {response.text}')

log('info', f'Exported {len(records)} orders', {
    'response': response.json()
})

result = {
    'success': True,
    'records_exported': len(records),
    'api_response': response.json(),
}

action = None
```

### CSV File Generation

```python
import csv
import io
import base64

csv_mod = wrap_module('csv', ['writer'])
io_mod = import_module('io', ['StringIO'])

# Create CSV in memory
output = io_mod.StringIO()
writer = csv_mod.writer(output, delimiter=';', quoting=csv.QUOTE_ALL)

# Write header
writer.writerow(['Order ID', 'Customer', 'Date', 'Total'])

# Write data
for order in records:
    writer.writerow([
        order.name,
        order.partner_id.name,
        order.date_order.strftime('%Y-%m-%d'),
        f"{order.amount_total:.2f}",
    ])

# Get content
csv_content = output.getvalue()
output.close()

# Create attachment
base64_mod = wrap_module('base64', ['b64encode'])
filename = f"orders_{datetime.datetime.now():%Y%m%d_%H%M%S}.csv"

attachment = env['ir.attachment'].create({
    'name': filename,
    'type': 'binary',
    'datas': base64_mod.b64encode(csv_content.encode('utf-8')),
    'res_model': 'indaws.export',
    'res_id': rec_export.id,
})

result = {
    'success': True,
    'filename': filename,
    'attachment_id': attachment.id,
    'records_exported': len(records),
}

action = None
```

### SFTP Upload

```python
import io

paramiko = wrap_module('paramiko', ['Transport', 'SFTPClient'])
io_mod = import_module('io', ['BytesIO'])

# Generate file content (CSV, XML, etc.)
content = generate_export_content(records)

# SFTP connection — params are bare names (configured in the template).
# sftp_host, sftp_user, sftp_pass, sftp_port, sftp_path
sftp_port = int(sftp_port)

try:
    transport = paramiko.Transport((sftp_host, sftp_port))
    transport.connect(username=sftp_user, password=sftp_pass)
    sftp = paramiko.SFTPClient.from_transport(transport)

    # Upload file
    filename = f"export_{datetime.datetime.now():%Y%m%d_%H%M%S}.csv"
    file_path = f"{sftp_path}/{filename}"

    sftp.putfo(io_mod.BytesIO(content.encode('utf-8')), file_path)

    sftp.close()
    transport.close()

    log('info', f'File uploaded to SFTP: {file_path}', {
        'filename': filename,
        'records_count': len(records)
    })

    result = {
        'success': True,
        'sftp_uploaded': True,
        'sftp_path': file_path,
        'records_exported': len(records),
    }

except Exception as e:
    log('error', f'SFTP upload failed: {str(e)}', None)
    raise UserError(f'SFTP Error: {str(e)}')

action = None
```

### XML Generation

```python
ET = wrap_module('xml.etree.ElementTree', ['Element', 'SubElement', 'tostring'])

# Build XML
root = ET.Element('Orders')
root.set('timestamp', datetime.datetime.now().isoformat())

for order in records:
    order_elem = ET.SubElement(root, 'Order')
    order_elem.set('id', order.name)

    customer = ET.SubElement(order_elem, 'Customer')
    customer.text = order.partner_id.name

    total = ET.SubElement(order_elem, 'Total')
    total.text = str(order.amount_total)

    lines = ET.SubElement(order_elem, 'Lines')
    for line in order.order_line:
        line_elem = ET.SubElement(lines, 'Line')
        line_elem.set('product', line.product_id.name)
        line_elem.set('qty', str(line.product_uom_qty))

# Convert to string
xml_content = ET.tostring(root, encoding='unicode')

result = {
    'success': True,
    'xml_content': xml_content,
    'records_exported': len(records),
}
```

### Webhook with Retry

```python
requests = wrap_module('requests', ['post'])

# params are bare names: webhook_url, max_retries
max_retries = int(max_retries)

payload = {
    'event': 'order_created',
    'order_id': record.name,
    'customer': record.partner_id.name,
    'total': float(record.amount_total),
}

# Retry logic
for attempt in range(max_retries):
    try:
        response = requests.post(
            webhook_url,
            json=payload,
            headers={'Content-Type': 'application/json'},
            timeout=10
        )

        if response.status_code in [200, 201, 202]:
            log('info', f'Webhook sent successfully', {
                'order': record.name,
                'attempt': attempt + 1
            })
            break
        else:
            log('warning', f'Webhook failed, retrying...', {
                'attempt': attempt + 1,
                'status': response.status_code
            })

    except Exception as e:
        log('warning', f'Webhook error, retrying...', {
            'attempt': attempt + 1,
            'error': str(e)
        })

    if attempt == max_retries - 1:
        raise UserError(f'Webhook failed after {max_retries} attempts')

    time.sleep(2 ** attempt)  # Exponential backoff

result = {
    'success': True,
    'webhook_sent': True,
    'order': record.name,
}

action = None
```

---

## Export Parameters (params)

Template parameters are injected **FLAT, by name** into the eval context
(`export.py` does `eval_ctx.update(self.get_params())`). There is **no `params` dict** —
access each parameter by its bare name. `params.get('api_key')` raises `NameError`.

```python
# Parameters configured in the template are available as bare names:
#   api_url, api_key, timeout, debug
timeout = int(timeout)        # cast as needed
debug_mode = bool(debug)

# Check required parameters
if not api_url:
    raise UserError('API URL parameter is required')
```

---

## Template Variables (variables)

Variables are also injected **FLAT, by name** (`eval_ctx.update(self.template_id.get_variables(...))`).
Access each variable by its bare name — there is **no `variables` dict** in v18
(`variables.get(...)` raises `NameError`).

```python
# Variables defined in the template are available as bare names:
#   format_date, field_mapping
formatted = format_date(record.date)
mapping = field_mapping
```

---

## Related Documentation

- **[Template Structure](template-structure.md)** - How to configure export templates
- **[Trigger Types](trigger-types.md)** - Configure when exports execute
- **[API Integrations](api-integrations.md)** - Common API integration patterns
- **[Troubleshooting](troubleshooting.md)** - Common issues and solutions
