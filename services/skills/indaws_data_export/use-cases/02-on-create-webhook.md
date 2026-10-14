# Use Case 02: Webhook on Create (on_create trigger)

**Scenario:** Send new sales orders to external shipping system via webhook.

**Difficulty:** ⭐⭐ Intermediate

**Time:** ~20 minutes

---

## Prerequisites

- [x] `indaws_data_export` module installed
- [x] `queue_job` configured (workers > 1)
- [x] Webhook endpoint URL and authentication
- [x] Understanding of webhook API requirements
- [x] `requests` library installed on Odoo server

---

## Scenario Description

**Business Need:** Automatically send new confirmed sales orders to shipping provider API.

**Trigger:** `on_create` - When new sales order is created

**Webhook Details:**
- **Endpoint:** `https://shipping-api.example.com/v1/orders`
- **Method:** POST
- **Auth:** API Key in header
- **Format:** JSON

**Expected Payload:**
```json
{
  "order_id": "SO001",
  "customer": {
    "name": "John Doe",
    "address": "123 Main St",
    "city": "New York",
    "zip": "10001",
    "country": "US"
  },
  "items": [
    {"product": "Laptop", "qty": 1, "price": 999.00}
  ],
  "total": 999.00
}
```

**Requirements:**
- Only send confirmed orders (`state='sale'`)
- Send in background (don't block user)
- Log response for tracking
- Retry on failure

---

## Understanding the Code Context

Before writing template code, review [Python Code Context Reference](../references/python-code-context.md).

**Key points:**
- Template code runs inside `safe_eval` - no direct imports allowed
- Use `wrap_module()` to import external libraries
- Pre-loaded: `datetime`, `time`, `json`, `pytz`, `dateutil`
- Available: `env`, `model`, `record`, `records`, `params`, `log()`

---

## Step 1: Create Export Template

```python
# This script runs EXTERNALLY via XML-RPC to create the template
import xmlrpc.client

url = 'https://your-odoo.com'
db = 'your_database'
username = 'admin'
password = 'your_api_key'

common = xmlrpc.client.ServerProxy(f'{url}/xmlrpc/2/common')
uid = common.authenticate(db, username, password, {})
models = xmlrpc.client.ServerProxy(f'{url}/xmlrpc/2/object')

# Get sale.order model ID
model_id = models.execute_kw(db, uid, password,
    'ir.model', 'search',
    [[('model', '=', 'sale.order')]], {'limit': 1}
)[0]

# Create template
# NOTE: The export_code below runs inside Odoo with safe_eval
template_id = models.execute_kw(db, uid, password,
    'indaws.export.template', 'create',
    [{
        'name': 'Export Order to Shipping API',
        'model_id': model_id,
        'trigger': 'on_create',
        'export_code': '''
# ============================================================
# THIS CODE RUNS INSIDE ODOO WITH safe_eval
# No direct imports allowed - use wrap_module()
# See: references/python-code-context.md
# ============================================================

# Import requests library using wrap_module
requests = wrap_module('requests', ['post', 'get', 'exceptions'])

# Get the single record (on_create provides one record)
order = record

# Build shipping API payload
payload = {
    "order_id": order.name,
    "customer": {
        "name": order.partner_id.name,
        "address": order.partner_shipping_id.street or '',
        "city": order.partner_shipping_id.city or '',
        "zip": order.partner_shipping_id.zip or '',
        "country": order.partner_shipping_id.country_id.code or '',
    },
    "items": [],
    "total": float(order.amount_total),
}

# Add order lines
for line in order.order_line:
    if line.product_id.type == 'product':  # Only stockable products
        payload["items"].append({
            "product": line.product_id.name,
            "sku": line.product_id.default_code or '',
            "qty": line.product_uom_qty,
            "price": float(line.price_unit),
        })

# Get API configuration from parameters
api_url = params.get('api_url')
api_key = params.get('api_key')

if not api_url or not api_key:
    raise UserError('API URL and API Key are required')

# Send to shipping API
try:
    response = requests.post(
        f'{api_url}/v1/orders',
        json=payload,  # json module is pre-loaded
        headers={
            'Authorization': f'Bearer {api_key}',
            'Content-Type': 'application/json',
        },
        timeout=30
    )

    # Check response
    if response.status_code in [200, 201]:
        response_data = json.loads(response.text)

        result = {
            'success': True,
            'order_id': order.name,
            'shipping_api_response': response_data,
            'status_code': response.status_code,
        }

        # Log success
        log('info', f'Order {order.name} sent successfully to shipping API', result)

        # Store shipping ID in order (optional)
        if 'shipping_id' in response_data:
            order.write({
                'client_order_ref': response_data['shipping_id']
            })

    else:
        # API returned error
        error_msg = f'Shipping API Error {response.status_code}: {response.text}'
        log('error', error_msg, {'order_id': order.name})
        raise UserError(error_msg)

except Exception as e:
    if 'Timeout' in str(type(e)):
        error_msg = 'Shipping API timeout'
    else:
        error_msg = f'Connection error: {str(e)}'

    log('error', error_msg, {'order_id': order.name})
    raise UserError(error_msg)

# No UI action needed for background export
action = None
'''
    }]
)

print(f"Template created with ID: {template_id}")
```

---

## Step 2: Add Template Parameters

```python
# API URL parameter
models.execute_kw(db, uid, password,
    'indaws.export.param.template', 'create',
    [{
        'template_id': template_id,
        'name': 'api_url',
        'ttype': 'text',
        'value_text': 'https://shipping-api.example.com',
        'description': 'Shipping API Base URL',
    }]
)

# API Key parameter
models.execute_kw(db, uid, password,
    'indaws.export.param.template', 'create',
    [{
        'template_id': template_id,
        'name': 'api_key',
        'ttype': 'text',
        'value_text': 'your-api-key-here',  # CHANGE THIS!
        'description': 'Shipping API Authentication Key',
    }]
)

print("Parameters created")
```

---

## Step 3: Create Export Configuration

```python
# Create export
export_id = models.execute_kw(db, uid, password,
    'indaws.export', 'create',
    [{
        'name': 'Auto Export Orders to Shipping',
        'template_id': template_id,
        'trigger': 'on_create',
        'filter_domain': "[('state', '=', 'sale')]",  # Only confirmed orders
        'background': True,  # Process asynchronously via queue_job
        'iter_record': True,  # Process one order at a time
    }]
)

print(f"Export created with ID: {export_id}")
print("Export is now ACTIVE and will trigger automatically!")
```

---

## Step 4: Test the Export

### Create Test Order

```python
# Create test sales order
partner_id = models.execute_kw(db, uid, password,
    'res.partner', 'search',
    [[]],  {'limit': 1}
)[0]

product_id = models.execute_kw(db, uid, password,
    'product.product', 'search',
    [[('type', '=', 'product')]],  {'limit': 1}
)[0]

# Create order
order_id = models.execute_kw(db, uid, password,
    'sale.order', 'create',
    [{
        'partner_id': partner_id,
        'order_line': [(0, 0, {
            'product_id': product_id,
            'product_uom_qty': 1,
        })],
    }]
)

print(f"Test order created: {order_id}")

# Confirm order (this will trigger the export!)
models.execute_kw(db, uid, password,
    'sale.order', 'action_confirm',
    [[order_id]]
)

print("Order confirmed - export should trigger now!")
```

### Monitor Export

```python
import time

# Wait a moment for export to trigger
time.sleep(5)

# Check registries
registries = models.execute_kw(db, uid, password,
    'indaws.export.registry', 'search_read',
    [[('export_id', '=', export_id)]],
    {
        'fields': ['state', 'result', 'failed_msg'],
        'order': 'id desc',
        'limit': 5
    }
)

for reg in registries:
    print(f"Registry {reg['id']}: State={reg['state']}")
    if reg['result']:
        print(f"  Result: {reg['result']}")
    if reg['failed_msg']:
        print(f"  Error: {reg['failed_msg']}")
```

### Check Queue Jobs

```python
# View background jobs
jobs = models.execute_kw(db, uid, password,
    'queue.job', 'search_read',
    [[
        ('model_name', '=', 'indaws.export.registry'),
        ('state', 'in', ['pending', 'enqueued', 'started', 'done', 'failed'])
    ]],
    {
        'fields': ['name', 'state', 'date_created', 'exc_info'],
        'order': 'id desc',
        'limit': 5
    }
)

for job in jobs:
    print(f"Job: {job['name']} - State: {job['state']}")
    if job['exc_info']:
        print(f"  Error: {job['exc_info'][:200]}...")
```

---

## Verification

### Check External System

Verify order was received in shipping system:
- Login to shipping provider dashboard
- Check for new order with ID matching Odoo order name
- Verify order details (items, quantities, address)

### Check Odoo Logs

```python
# Get export logs
logs = models.execute_kw(db, uid, password,
    'indaws.export.log', 'search_read',
    [[('export_id', '=', export_id)]],
    {
        'fields': ['level', 'message', 'result', 'create_date'],
        'order': 'id desc',
        'limit': 10
    }
)

for log_entry in logs:
    print(f"[{log_entry['level']}] {log_entry['message']}")
```

---

## Troubleshooting

### Issue: Export not triggering

**Possible causes:**
1. Export is not active
2. Filter domain doesn't match order
3. queue_job not configured

**Solution:**
```python
# Check export is active
export_data = models.execute_kw(db, uid, password,
    'indaws.export', 'read',
    [[export_id]],
    {'fields': ['active', 'trigger', 'filter_domain']}
)[0]

print(f"Active: {export_data['active']}")
print(f"Trigger: {export_data['trigger']}")
print(f"Filter: {export_data['filter_domain']}")
```

### Issue: "No module named 'requests'"

**Cause:** requests library not installed on Odoo server

**Solution:**
```bash
pip install requests
# Restart Odoo
```

### Issue: "wrap_module" not recognized

**Cause:** Using old version of indaws_data_export

**Solution:** Update module or use fallback:
```python
# Alternative if wrap_module not available
import requests  # Only works if module allows direct imports
```

### Issue: "Connection refused" or timeout

**Solution:**
- Verify API URL is correct
- Check firewall allows outbound HTTPS
- Test API directly with curl:
  ```bash
  curl -X POST https://shipping-api.example.com/v1/orders \
    -H "Authorization: Bearer YOUR_API_KEY" \
    -H "Content-Type: application/json" \
    -d '{"test": true}'
  ```

### Issue: API returns 401 Unauthorized

**Solution:** Verify API key is correct in parameters

### Issue: Some orders not exported

**Solution:** Check filter domain matches all intended orders

---

## Advanced: Add Retry Logic

Exports automatically retry via queue_job, but you can customize:

```python
models.execute_kw(db, uid, password,
    'indaws.export', 'write',
    [[export_id], {
        'max_retries': 3,  # Retry up to 3 times
    }]
)
```

---

## Advanced: Conditional Export

Only export orders above $100:

```python
models.execute_kw(db, uid, password,
    'indaws.export', 'write',
    [[export_id], {
        'filter_domain': "[('state', '=', 'sale'), ('amount_total', '>=', 100)]"
    }]
)
```

---

## Advanced: Custom Webhook Headers

Modify template to add custom headers:

```python
# Inside export_code, modify the requests.post call:
response = requests.post(
    f'{api_url}/v1/orders',
    json=payload,
    headers={
        'Authorization': f'Bearer {api_key}',
        'Content-Type': 'application/json',
        'X-Client-ID': 'odoo-integration',
        'X-Correlation-ID': str(rec_registry.id),  # Use registry ID for tracking
    },
    timeout=30
)
```

---

## Related Documentation

- **[Python Code Context](../references/python-code-context.md)** - Available functions and modules
- **[03-on-write-sync.md](03-on-write-sync.md)** - Sync updates too
- **[05-on-unlink-notification.md](05-on-unlink-notification.md)** - Notify on cancellation
- **[07-integration-erp.md](07-integration-erp.md)** - Complex integrations

---

## Summary

✅ **What we achieved:**
- Automatic webhook on order creation
- Background processing (doesn't block UI)
- Error handling and logging
- Retry mechanism via queue_job
- Audit trail via registries

🎯 **Key learnings:**
- Template code uses `safe_eval` - no direct imports
- Use `wrap_module()` to import external libraries
- `on_create` trigger fires automatically
- `filter_domain` controls which records trigger
- `background=True` uses queue_job for async processing
- `iter_record=True` processes records individually

**Duration:** ~20 minutes setup, triggers automatically forever

**Production tip:** Monitor export logs daily and set up alerts for failures!
