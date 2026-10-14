# Use Case 04: Scheduled Export (on_time trigger)

**Scenario:** Daily export of customers to CSV file and upload to SFTP server.

**Difficulty:** ⭐⭐⭐ Advanced

**Time:** ~30 minutes

---

## Prerequisites

- [x] `indaws_data_export` module installed
- [x] `queue_job` configured
- [x] SFTP server credentials (optional)
- [x] Understanding of cron scheduling
- [x] `paramiko` library installed (for SFTP)

---

## Scenario Description

**Business Need:** Export all active customers to CSV file daily at 2 AM and upload to SFTP server for external CRM.

**Trigger:** `on_time` - Scheduled execution

**Schedule:** Daily at 2:00 AM

**Export Format:** CSV file with columns:
- Customer ID
- Name
- Email
- Phone
- Country
- Total Sales

**Delivery Method:** SFTP upload to `/exports/customers/`

---

## Understanding the Code Context

Before writing template code, review [Python Code Context Reference](../references/python-code-context.md).

**Key points:**
- Template code runs inside `safe_eval` - no direct imports allowed
- Use `wrap_module()` to import external libraries like `csv`, `paramiko`
- Pre-loaded: `datetime`, `time`, `json`, `pytz`, `dateutil`
- Use `import_module()` for `io.StringIO`, `io.BytesIO`

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

# Get res.partner model ID
model_id = models.execute_kw(db, uid, password,
    'ir.model', 'search',
    [[('model', '=', 'res.partner')]], {'limit': 1}
)[0]

# Create template
# NOTE: The export_code below runs inside Odoo with safe_eval
template_id = models.execute_kw(db, uid, password,
    'indaws.export.template', 'create',
    [{
        'name': 'Daily Customer Export to CSV',
        'model_id': model_id,
        'trigger': 'on_time',
        'export_code': '''
# ============================================================
# THIS CODE RUNS INSIDE ODOO WITH safe_eval
# No direct imports allowed - use wrap_module() / import_module()
# See: references/python-code-context.md
# ============================================================

# Import required modules using wrap_module/import_module
csv_mod = wrap_module('csv', ['writer', 'QUOTE_ALL'])
io_mod = import_module('io', ['StringIO', 'BytesIO'])
base64_mod = wrap_module('base64', ['b64encode'])

# Get customers (records is provided by the system based on filter_domain)
customers = records

# Generate CSV in memory
output = io_mod.StringIO()
writer = csv_mod.writer(output, delimiter=';', quoting=csv_mod.QUOTE_ALL)

# Write header
writer.writerow([
    'Customer ID',
    'Name',
    'Email',
    'Phone',
    'Country',
    'Total Sales',
])

# Write data rows
for customer in customers:
    # Calculate total sales
    sales = env['sale.order'].search([
        ('partner_id', '=', customer.id),
        ('state', 'in', ['sale', 'done'])
    ])
    total_sales = sum(sales.mapped('amount_total'))

    writer.writerow([
        customer.id,
        customer.name or '',
        customer.email or '',
        customer.phone or '',
        customer.country_id.name if customer.country_id else '',
        f"{total_sales:.2f}",
    ])

# Get CSV content
csv_content = output.getvalue()
output.close()

# Generate filename with date (datetime is pre-loaded)
filename = f"customers_{datetime.datetime.now().strftime('%Y%m%d_%H%M%S')}.csv"

# Upload to SFTP if configured
sftp_host = params.get('sftp_host')
sftp_user = params.get('sftp_user')
sftp_pass = params.get('sftp_pass')
sftp_port = params.get('sftp_port', 22)
sftp_path = params.get('sftp_path', '/exports/customers/')

if sftp_host and sftp_user and sftp_pass:
    try:
        # Import paramiko for SFTP
        paramiko = wrap_module('paramiko', ['Transport', 'SFTPClient'])

        # Connect to SFTP
        transport = paramiko.Transport((sftp_host, int(sftp_port)))
        transport.connect(username=sftp_user, password=sftp_pass)
        sftp = paramiko.SFTPClient.from_transport(transport)

        # Create directory if not exists
        try:
            sftp.chdir(sftp_path)
        except IOError:
            # Directory doesn't exist, create it
            parts = sftp_path.strip('/').split('/')
            current = '/'
            for part in parts:
                current = f"{current}{part}/"
                try:
                    sftp.mkdir(current)
                except IOError:
                    pass  # Already exists

        # Upload file
        file_path = f"{sftp_path}/{filename}"
        sftp.putfo(io_mod.BytesIO(csv_content.encode('utf-8')), file_path)

        # Close connection
        sftp.close()
        transport.close()

        log('info', f'File uploaded to SFTP: {file_path}', {
            'filename': filename,
            'records_count': len(customers),
            'sftp_path': file_path
        })

        result = {
            'success': True,
            'filename': filename,
            'records_exported': len(customers),
            'sftp_uploaded': True,
            'sftp_path': file_path,
        }

    except Exception as e:
        log('error', f'SFTP upload failed: {str(e)}', {
            'filename': filename,
            'records_count': len(customers)
        })

        # Still create attachment in Odoo
        result = {
            'success': False,
            'filename': filename,
            'records_exported': len(customers),
            'sftp_uploaded': False,
            'error': str(e),
        }

else:
    log('warning', 'SFTP not configured, file not uploaded', {
        'filename': filename,
        'records_count': len(customers)
    })

    result = {
        'success': True,
        'filename': filename,
        'records_exported': len(customers),
        'sftp_uploaded': False,
        'note': 'SFTP not configured',
    }

# Also create attachment in Odoo for backup
attachment = env['ir.attachment'].create({
    'name': filename,
    'type': 'binary',
    'datas': base64_mod.b64encode(csv_content.encode('utf-8')),
    'res_model': 'indaws.export',
    'res_id': rec_export.id,
    'description': f'Customer export - {len(customers)} records',
})

result['attachment_id'] = attachment.id

action = None
'''
    }]
)

print(f"Template created with ID: {template_id}")
```

---

## Step 2: Add SFTP Parameters

```python
# SFTP parameters
sftp_params = [
    {'name': 'sftp_host', 'value': 'sftp.example.com'},
    {'name': 'sftp_user', 'value': 'odoo_export'},
    {'name': 'sftp_pass', 'value': 'your-password-here'},  # CHANGE THIS!
    {'name': 'sftp_port', 'value': '22'},
    {'name': 'sftp_path', 'value': '/exports/customers/'},
]

for param in sftp_params:
    models.execute_kw(db, uid, password,
        'indaws.export.param.template', 'create',
        [{
            'template_id': template_id,
            'name': param['name'],
            'ttype': 'text',
            'value_text': param['value'],
        }]
    )

print("SFTP parameters created")
```

---

## Step 3: Create Export with Schedule

```python
# Create export
export_id = models.execute_kw(db, uid, password,
    'indaws.export', 'create',
    [{
        'name': 'Daily Customer Export',
        'template_id': template_id,
        'trigger': 'on_time',
        'filter_domain': "[('customer_rank', '>', 0), ('active', '=', True)]",  # Only active customers
        'background': True,
        'iter_record': False,  # Process all in one batch
    }]
)

print(f"Export created with ID: {export_id}")
```

### Configure Schedule

```python
from datetime import datetime, timedelta

# Calculate next execution (tomorrow at 2 AM)
tomorrow_2am = (datetime.now() + timedelta(days=1)).replace(
    hour=2, minute=0, second=0, microsecond=0
)

# Update cron settings
models.execute_kw(db, uid, password,
    'indaws.export', 'write',
    [[export_id], {
        'interval_number': 1,
        'interval_type': 'days',
        'nextcall': tomorrow_2am.strftime('%Y-%m-%d %H:%M:%S'),
        'cron_active': True,  # Activate cron
    }]
)

print(f"Scheduled for daily execution at 2:00 AM (next: {tomorrow_2am})")
```

---

## Step 4: Test Export Manually

Before scheduling, test manually:

```python
# Execute export manually
models.execute_kw(db, uid, password,
    'indaws.export', 'action_execute',
    [[export_id]]
)

print("Export triggered manually - processing...")

# Wait for completion
import time
time.sleep(10)

# Check latest registry
registries = models.execute_kw(db, uid, password,
    'indaws.export.registry', 'search_read',
    [[('export_id', '=', export_id)]],
    {
        'fields': ['state', 'result', 'failed_msg', 'create_date'],
        'order': 'id desc',
        'limit': 1
    }
)

if registries:
    reg = registries[0]
    print(f"Registry state: {reg['state']}")
    if reg['result']:
        print(f"Result: {reg['result']}")
    if reg['failed_msg']:
        print(f"Error: {reg['failed_msg']}")
```

---

## Verification

### Check Generated File

```python
# Find attachment
attachments = models.execute_kw(db, uid, password,
    'ir.attachment', 'search_read',
    [[
        ('res_model', '=', 'indaws.export'),
        ('res_id', '=', export_id)
    ]],
    {
        'fields': ['name', 'create_date', 'file_size'],
        'order': 'id desc',
        'limit': 1
    }
)

if attachments:
    att = attachments[0]
    print(f"File: {att['name']} ({att['file_size']} bytes)")
    print(f"Created: {att['create_date']}")
```

### Download and Verify CSV

```python
import base64

# Download attachment content
att_data = models.execute_kw(db, uid, password,
    'ir.attachment', 'read',
    [[attachments[0]['id']]],
    {'fields': ['datas']}
)[0]

csv_content = base64.b64decode(att_data['datas']).decode('utf-8')

# Print first few lines
lines = csv_content.split('\n')[:5]
for line in lines:
    print(line)
```

### Verify SFTP Upload

```bash
# Connect to SFTP server and verify file
sftp username@sftp.example.com
cd /exports/customers/
ls -lh
get customers_20240101_020000.csv
exit

# Check file locally
head customers_20240101_020000.csv
```

---

## Monitoring

### Check Cron Status

```python
# Get cron details
cron_data = models.execute_kw(db, uid, password,
    'indaws.export', 'read',
    [[export_id]],
    {'fields': ['cron_id', 'nextcall', 'cron_active', 'interval_number', 'interval_type']}
)[0]

if cron_data['cron_id']:
    cron = models.execute_kw(db, uid, password,
        'ir.cron', 'read',
        [[cron_data['cron_id'][0]]],
        {'fields': ['name', 'active', 'nextcall', 'numbercall']}
    )[0]

    print(f"Cron: {cron['name']}")
    print(f"Active: {cron['active']}")
    print(f"Next call: {cron['nextcall']}")
    print(f"Executions: {cron['numbercall']}")
```

### View Export History

```python
# Get last 10 exports
registries = models.execute_kw(db, uid, password,
    'indaws.export.registry', 'search_read',
    [[('export_id', '=', export_id)]],
    {
        'fields': ['state', 'create_date', 'done_date', 'result'],
        'order': 'id desc',
        'limit': 10
    }
)

print(f"Export history (last {len(registries)} executions):")
for reg in registries:
    status = reg['state']
    date = reg['create_date']
    print(f"- {date}: {status}")
```

---

## Troubleshooting

### Issue: Cron not executing

**Possible causes:**
1. Cron not active
2. Odoo cron worker not running
3. Wrong timezone

**Solution:**
```python
# Check cron is active
models.execute_kw(db, uid, password,
    'indaws.export', 'write',
    [[export_id], {'cron_active': True}]
)

# Verify Odoo is running with workers
# Check odoo.conf: workers = 4 (minimum 2)
```

### Issue: SFTP connection fails

**Solution:**
- Verify SFTP credentials
- Check firewall allows outbound port 22
- Test connection manually:
  ```bash
  sftp username@sftp.example.com
  ```

### Issue: "No module named 'paramiko'"

**Solution:**
```bash
pip install paramiko
# Restart Odoo
```

### Issue: "wrap_module" not recognized

**Cause:** Using old version of indaws_data_export

**Solution:** Update module to latest version

### Issue: CSV encoding problems

**Solution:** The code already uses UTF-8 encoding:
```python
csv_content.encode('utf-8')
```

---

## Advanced: Email Report

Send email with export summary. Add to template's `finish_export_code`:

```python
# Update template finish_export_code
finish_code = '''
# Get export result from registry
registry = rec_registry

if registry.state == 'done':
    # Send success email
    rec_export.message_post(
        body=f"""
        <p>Daily customer export completed successfully.</p>
        <ul>
            <li>Records exported: {len(records)}</li>
            <li>File: {registry.result.get('filename', 'N/A')}</li>
            <li>SFTP uploaded: {registry.result.get('sftp_uploaded', False)}</li>
        </ul>
        """,
        subject='[Success] Daily Customer Export',
        partner_ids=env.ref('base.user_admin').partner_id.ids
    )
'''

models.execute_kw(db, uid, password,
    'indaws.export.template', 'write',
    [[template_id], {'finish_export_code': finish_code}]
)
```

---

## Advanced: Compress File

Compress CSV before upload to save bandwidth. Modify template code:

```python
# Inside export_code, after generating CSV:

# Import gzip
gzip_mod = wrap_module('gzip', ['GzipFile'])

# Compress CSV
compressed = io_mod.BytesIO()
with gzip_mod.GzipFile(fileobj=compressed, mode='wb') as gz:
    gz.write(csv_content.encode('utf-8'))

compressed.seek(0)
filename = f"customers_{datetime.datetime.now().strftime('%Y%m%d_%H%M%S')}.csv.gz"

# Upload compressed file to SFTP
sftp.putfo(compressed, f"{sftp_path}/{filename}")
```

---

## Advanced: Excel Format

Generate Excel instead of CSV:

```python
# Inside export_code:

# Import openpyxl (must be installed on server)
openpyxl = wrap_module('openpyxl', ['Workbook'])

# Create workbook
wb = openpyxl.Workbook()
ws = wb.active
ws.title = 'Customers'

# Write header
headers = ['Customer ID', 'Name', 'Email', 'Phone', 'Country', 'Total Sales']
for col, header in enumerate(headers, 1):
    ws.cell(row=1, column=col, value=header)

# Write data
for row, customer in enumerate(customers, 2):
    ws.cell(row=row, column=1, value=customer.id)
    ws.cell(row=row, column=2, value=customer.name or '')
    ws.cell(row=row, column=3, value=customer.email or '')
    ws.cell(row=row, column=4, value=customer.phone or '')
    ws.cell(row=row, column=5, value=customer.country_id.name if customer.country_id else '')
    # Calculate total sales...

# Save to BytesIO
output = io_mod.BytesIO()
wb.save(output)
output.seek(0)

filename = f"customers_{datetime.datetime.now().strftime('%Y%m%d_%H%M%S')}.xlsx"
```

---

## Related Documentation

- **[Python Code Context](../references/python-code-context.md)** - Available functions and modules
- **[01-manual-export-api.md](01-manual-export-api.md)** - Manual trigger alternative
- **[06-csv-generation.md](06-csv-generation.md)** - More CSV generation techniques
- **[07-integration-erp.md](07-integration-erp.md)** - Complex integrations

---

## Summary

✅ **What we achieved:**
- Scheduled daily export at specific time
- CSV file generation using `wrap_module('csv', ...)`
- SFTP upload using `wrap_module('paramiko', ...)`
- Attachment backup in Odoo
- Error handling and logging
- Email notifications

🎯 **Key learnings:**
- Template code uses `safe_eval` - use `wrap_module()` for imports
- `on_time` trigger uses Odoo cron
- `filter_domain` selects which records to export
- SFTP requires `paramiko` library installed on server
- Always create attachment backup
- Monitor cron execution history

**Duration:** ~30 minutes setup, runs automatically daily

**Production tips:**
- Monitor export logs weekly
- Set up email alerts for failures
- Rotate SFTP files monthly
- Test SFTP credentials periodically
