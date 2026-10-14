# External IDs and XML IDs

## External IDs (Mandatory)

### What is an External ID?

An **External ID** (XML ID) is a unique, human-readable identifier that allows stable reference to records between imports.

### Why are they mandatory?

1. **Prevent duplicates**: If you reimport, it will update instead of duplicate
2. **Traceability**: You can track which records came from imports
3. **Idempotence**: You can run the same import multiple times without issues
4. **Updates**: Modify the file and reimport to update records

### External ID Format

For imports, the format is:

```
__import__.{unique_identifier}
```

**Examples**:
```
__import__.contact_john_doe_001
__import__.product_laptop_dell_x123
__import__.invoice_2024_01_001
```

### External ID Generation Strategies

#### Option 1: Based on record data (Recommended)

```python
def generate_external_id(row, index):
    """Generates external ID based on data"""
    # Normalize name: remove accents, spaces, lowercase
    name = normalize_text(row['Name'])
    name = name.replace(' ', '_')

    # Add index to guarantee uniqueness
    return f"__import__.contact_{name}_{index}"
```

#### Option 2: Based on source system ID

```python
def generate_external_id(row):
    """Generates external ID from original system"""
    original_id = row['source_system_id']
    return f"__import__.contact_legacy_{original_id}"
```

#### Option 3: UUID (if no unique data)

```python
import uuid

def generate_external_id():
    """Generates unique external ID with UUID"""
    unique_id = str(uuid.uuid4())[:8]
    return f"__import__.contact_{unique_id}"
```

## XML IDs for Relations

### What are XML IDs?

**XML IDs** are unique identifiers that Odoo assigns to master data records (countries, currencies, categories, etc.).

### Why use them instead of numeric IDs?

| Aspect | XML ID | Numeric ID |
|--------|--------|------------|
| **Stability** | Identical in all databases | Different per database |
| **Portability** | Works in any environment | Only in specific database |
| **Readability** | `base.es` (Spain) | `68` (what is it?) |

### Common XML IDs in Odoo

#### Countries (res.country)
```
base.es  → Spain
base.mx  → Mexico
base.us  → United States
base.fr  → France
base.uk  → United Kingdom
```

#### States/Provinces (res.country.state)
```
base.state_es_m   → Madrid (Spain)
base.state_us_ca  → California (USA)
base.state_us_ny  → New York (USA)
```

#### Currencies (res.currency)
```
base.EUR  → Euro
base.USD  → US Dollar
base.GBP  → British Pound
```

### Getting XML IDs from Records

```python
def get_xmlid_for_record(uid, models, model_name, record_id):
    """Gets the XML ID of a record given its numeric ID"""
    ext_id = models.execute_kw(
        DB_NAME, uid, API_KEY,
        'ir.model.data', 'search_read',
        [[['model', '=', model_name], ['res_id', '=', int(record_id)]]],
        {'fields': ['name', 'module'], 'limit': 1}
    )

    if ext_id:
        return f"{ext_id[0]['module']}.{ext_id[0]['name']}"
    else:
        return str(int(record_id))  # Fallback to numeric ID
```
