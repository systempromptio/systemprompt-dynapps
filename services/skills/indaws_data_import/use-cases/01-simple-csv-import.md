# Use Case 01: Simple CSV Import

**Scenario:** Import products from a simple CSV file with basic field mapping.

**Difficulty:** ⭐ Beginner

**Time:** ~15 minutes

---

## Prerequisites

- [x] `indaws_data_import` module installed
- [x] `queue_job` module installed and configured
- [x] Python libraries: `pandas`, `dateutil`
- [x] Access to Odoo instance (URL + credentials)
- [x] Sample CSV file

---

## Scenario Description

**Business Need:** Import 500 products from supplier CSV file.

**CSV Structure:**
```csv
Code;Name;Price;Category
PROD001;Laptop Dell XPS 13;1299.00;Computers
PROD002;Mouse Logitech MX Master;89.99;Accessories
PROD003;Keyboard Mechanical RGB;149.50;Accessories
```

**Requirements:**
- Match existing products by `default_code` (Code)
- Create new products if not exists
- Update price if product already exists
- Map category by name (Many2one)

---

## Understanding the Code Context

Before writing template code, review [Python Code Context Reference](../references/python-code-context.md).

**Key points for import templates:**
- Field mapping code runs inside `safe_eval`
- Use `wrap_module()` for external libraries
- Pre-loaded: `datetime`, `time`, `json`, `pytz`, `dateutil`, `hashlib`
- Available helpers: `convert_to_dict()`, `find_or_create_record()`, etc.
- Use `SkipRow` exception to skip invalid rows

---

## Step-by-Step Implementation

### Step 1: Prepare CSV File

Ensure CSV has proper structure:
- Headers in first row
- Consistent separator (`;` or `,`)
- UTF-8 encoding (no special characters issues)

### Step 2: Create Import Template

Connect to Odoo and create template:

```python
# This script runs EXTERNALLY via XML-RPC
import xmlrpc.client

url = 'https://your-odoo.com'
db = 'your_database'
username = 'admin'
password = 'your_api_key'

common = xmlrpc.client.ServerProxy(f'{url}/xmlrpc/2/common')
uid = common.authenticate(db, username, password, {})

models = xmlrpc.client.ServerProxy(f'{url}/xmlrpc/2/object')

# Create import template
template_id = models.execute_kw(db, uid, password,
    'indaws.import.template', 'create',
    [{
        'name': 'Import Products from CSV',
        'model_id': models.execute_kw(db, uid, password,
            'ir.model', 'search',
            [[('model', '=', 'product.product')]], {'limit': 1}
        )[0],
        'file_type': 'csv',
        'csv_separator': ';',
        'csv_quotechar': '"',
        'header_row': 1,
        'chunksize': 500,  # Process all 500 in one chunk
        'continue_error': True,  # Continue if one row fails
    }]
)

print(f"Template created with ID: {template_id}")
```

### Step 3: Add Field Mappings

Map CSV columns to Odoo fields:

```python
# Get field IDs
fields = models.execute_kw(db, uid, password,
    'ir.model.fields', 'search_read',
    [[
        ('model', '=', 'product.product'),
        ('name', 'in', ['default_code', 'name', 'list_price', 'categ_id'])
    ]],
    {'fields': ['name', 'id']}
)

field_map = {f['name']: f['id'] for f in fields}

# Create field mapping lines
mappings = [
    {
        'template_id': template_id,
        'names': 'Code',  # CSV column name
        'field_id': field_map['default_code'],
        'sequence': 10,
    },
    {
        'template_id': template_id,
        'names': 'Name',
        'field_id': field_map['name'],
        'sequence': 20,
    },
    {
        'template_id': template_id,
        'names': 'Price',
        'field_id': field_map['list_price'],
        'sequence': 30,
    },
    {
        'template_id': template_id,
        'names': 'Category',
        'field_id': field_map['categ_id'],
        'sequence': 40,
        # For Many2one fields, search by name
        # Leave domain empty to use default name_search
    },
]

for mapping in mappings:
    models.execute_kw(db, uid, password,
        'indaws.import.template.line', 'create', [mapping]
    )

print("Field mappings created successfully")
```

### Step 4: Configure Matching Strategy

Tell the system how to detect existing products:

```python
# Match by default_code (product reference)
models.execute_kw(db, uid, password,
    'indaws.import.template.match.line', 'create',
    [{
        'template_id': template_id,
        'field_id': field_map['default_code'],
        'domain_operator': '=',  # Exact match
        'sequence': 10,
    }]
)

print("Matching rule configured: match by default_code")
```

### Step 5: Create Import Execution

```python
import base64

# Read CSV file
with open('products.csv', 'rb') as f:
    file_content = base64.b64encode(f.read()).decode('utf-8')

# Create import record
import_id = models.execute_kw(db, uid, password,
    'indaws.import', 'create',
    [{
        'template_id': template_id,
        'file': file_content,
        'filename': 'products.csv',
    }]
)

print(f"Import record created with ID: {import_id}")
```

### Step 6: Execute Import

```python
# Launch import (will run in background via queue_job)
models.execute_kw(db, uid, password,
    'indaws.import', 'action_import',
    [[import_id]]
)

print("Import launched! Processing in background...")
```

### Step 7: Monitor Progress

```python
import time

while True:
    # Get import state
    import_data = models.execute_kw(db, uid, password,
        'indaws.import', 'read',
        [[import_id]],
        {'fields': ['state', 'created_record_count', 'updated_record_count', 'error_count']}
    )[0]

    state = import_data['state']
    created = import_data['created_record_count']
    updated = import_data['updated_record_count']
    errors = import_data['error_count']

    print(f"State: {state} | Created: {created} | Updated: {updated} | Errors: {errors}")

    if state in ['done', 'cancel']:
        break

    time.sleep(5)  # Check every 5 seconds

print("Import completed!")
```

---

## Adding Custom Field Code

For complex field transformations, you can add Python code to field mappings.
**Note:** This code runs inside `safe_eval` with limited imports.

### Example: Custom Category Lookup

```python
# Update the Category field mapping with custom code
models.execute_kw(db, uid, password,
    'indaws.import.template.line', 'write',
    [[category_mapping_id], {
        'code': '''
# ============================================================
# THIS CODE RUNS INSIDE ODOO WITH safe_eval
# See: references/python-code-context.md
# ============================================================

# cell_value contains the CSV value (e.g., "Computers")
category_name = cell_value

# Search for existing category
category = env['product.category'].search([
    ('name', '=', category_name)
], limit=1)

if category:
    value = category.id
else:
    # Use helper to create if not exists
    category = find_or_create_record(
        'product.category',
        [('name', '=', category_name)],
        {'name': category_name},
        create=True
    )
    value = category.id if category else False

# Set parse_continue to True to indicate we handled the parsing
parse_continue = True
'''
    }]
)
```

### Example: Price Validation

```python
# Add validation code to Price field mapping
models.execute_kw(db, uid, password,
    'indaws.import.template.line', 'write',
    [[price_mapping_id], {
        'code': '''
# Validate and convert price
try:
    price = float(cell_value.replace(',', '.'))
    if price < 0:
        raise UserError('Price cannot be negative')
    value = price
    parse_continue = True
except ValueError:
    raise UserError(f'Invalid price format: {cell_value}')
'''
    }]
)
```

---

## Testing

### Test with Sample Data

Create a test CSV with 3-5 products:

```csv
Code;Name;Price;Category
TEST001;Test Product 1;10.00;All
TEST002;Test Product 2;20.00;All
TEST003;Test Product 3;30.00;All
```

### Verify Results

```python
# Check created products
products = models.execute_kw(db, uid, password,
    'product.product', 'search_read',
    [[('default_code', 'in', ['TEST001', 'TEST002', 'TEST003'])]],
    {'fields': ['default_code', 'name', 'list_price', 'categ_id']}
)

for product in products:
    print(f"Product: {product['default_code']} - {product['name']} - ${product['list_price']}")
```

### Check for Errors

```python
# Get import errors
errors = models.execute_kw(db, uid, password,
    'indaws.import.error', 'search_read',
    [[('import_id', '=', import_id)]],
    {'fields': ['name', 'error_type'], 'limit': 10}
)

if errors:
    print("Errors found:")
    for error in errors:
        print(f"- [{error['error_type']}] {error['name']}")
else:
    print("No errors!")
```

---

## Common Issues & Solutions

### Issue 1: "Extension file not supported"
**Cause:** File extension not in allowed list
**Solution:** Ensure file is `.csv`, `.xls`, `.xlsx`, `.txt`, or `.json`

### Issue 2: "Failed to read the file"
**Cause:** Wrong separator or encoding
**Solution:**
- Verify CSV separator matches `csv_separator` field
- Ensure file is UTF-8 encoded
- Check for special characters

### Issue 3: "Category not found"
**Cause:** Category name doesn't match existing category
**Solution:**
- Use `create_if_not_exist=True` in field mapping
- Or pre-create categories
- Or use custom code with `find_or_create_record()`

### Issue 4: Products duplicated
**Cause:** Matching not configured correctly
**Solution:**
- Verify match field is set (Step 4)
- Check field has unique values in CSV
- Review matching logic

### Issue 5: "wrap_module" not available
**Cause:** Using old version of indaws_data_import or code runs in wrong context
**Solution:**
- Update module to latest version
- Ensure code is in correct field (e.g., `code` field of template line)

---

## Next Steps

### Make it Reusable
Save template for future imports:
- Template is already saved and reusable
- Next time, just create new `indaws.import` record with same template

### Add Validations
Enhance template with Python code:

```python
# Add validation code to template's before_create_update_code
models.execute_kw(db, uid, password,
    'indaws.import.template', 'write',
    [[template_id], {
        'before_create_update_code': '''
# ============================================================
# THIS CODE RUNS INSIDE ODOO WITH safe_eval
# record_vals contains the values to be created/updated
# ============================================================

# Validate price is positive
if record_vals.get('list_price', 0) <= 0:
    raise UserError('Price must be positive!')

# Validate code format
code = record_vals.get('default_code', '')
if code and not code.startswith('PROD'):
    log(f'Warning: Product code {code} does not follow PROD format', level='warning')

# You can modify record_vals here
# record_vals['some_field'] = computed_value
'''
    }]
)
```

### Schedule it
See [03-scheduled-import.md](03-scheduled-import.md) to automate this import daily.

---

## Related Documentation

- **[Python Code Context](../references/python-code-context.md)** - Available functions and modules
- **[02-excel-with-relations.md](02-excel-with-relations.md)** - Import with Many2one/Many2many
- **[03-scheduled-import.md](03-scheduled-import.md)** - Schedule this import daily
- **[07-complex-transformations.md](07-complex-transformations.md)** - Add data transformations

---

## Summary

✅ **What we achieved:**
- Created reusable import template
- Mapped CSV columns to Odoo fields
- Configured matching by product code
- Executed import with monitoring
- Handled errors gracefully

🎯 **Key learnings:**
- Templates are reusable
- Matching prevents duplicates
- Imports run in background via queue_job
- Field code uses `safe_eval` - use `wrap_module()` for external imports
- Use helpers like `find_or_create_record()` for related records
- Error handling is crucial for production

**Duration:** ~15 minutes for setup, instant for subsequent imports
