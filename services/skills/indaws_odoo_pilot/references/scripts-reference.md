# Scripts Reference

## Authentication & Detection

### `auth.sh [protocol]`
**Unified authentication script** that automatically detects and uses the best protocol.

```bash
# Auto-detect protocol (tries JSON2 first, falls back to JSON-RPC)
eval $(./scripts/auth.sh)

# Force specific protocol
eval $(./scripts/auth.sh json2)      # Force JSON2 (Odoo >= 19)
eval $(./scripts/auth.sh jsonrpc)    # Force JSON-RPC (Odoo < 19)
```

**Sets**:
- JSON2: `ODOO_UID`, `ODOO_VERSION`, `ODOO_PROTOCOL='json2'`, `ODOO_AUTH_HEADER`
- JSON-RPC: `ODOO_SESSION_ID`, `ODOO_UID`, `ODOO_VERSION`, `ODOO_PROTOCOL='jsonrpc'`

**Requirements**:
- Always: `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`
- JSON-RPC only: `ODOO_USER`

### `detect_version.sh`
Auto-detect Odoo version and protocol support (optional, for information only).

```bash
./scripts/detect_version.sh
```

**Output**: Displays detected Odoo version, protocol type, and endpoints.

### Legacy Authentication Scripts (Deprecated)

`auth_json2.sh` and `auth_jsonrpc.sh` are still available for compatibility but **`auth.sh` is now recommended** as it handles both protocols automatically.

## Module Management

### `install_module.sh <module_name>`
Install an Odoo module.

```bash
./scripts/install_module.sh sale_management
./scripts/install_module.sh stock
```

### `uninstall_module.sh <module_name>`
Uninstall an Odoo module.

```bash
./scripts/uninstall_module.sh sale_management
```

### `upgrade_module.sh <module_name>`
Upgrade an existing module.

```bash
./scripts/upgrade_module.sh account
```

## CRUD Operations

### `search_records.sh <model> <domain> [fields] [limit] [offset]`
Search and read records from any model.

```bash
# Search companies with specific fields
./scripts/search_records.sh res.partner '[["is_company","=",true]]' '["name","email","phone"]' 10

# Search all sale orders
./scripts/search_records.sh sale.order '[]' '["name","state","amount_total"]' 50

# Search with offset (pagination)
./scripts/search_records.sh product.product '[["sale_ok","=",true]]' '["name","list_price"]' 20 40
```

**Domain Syntax**: Odoo uses Polish notation for domains:
- Single condition: `[["field","=","value"]]`
- Multiple AND: `[["field1","=","a"],["field2",">",10]]`
- OR condition: `["|",["field1","=","a"],["field2","=","b"]]`
- NOT condition: `["!",["field","=","value"]]`

### `create_record.sh <model> <values_json>`
Create a new record in any model.

```bash
# Create a partner
./scripts/create_record.sh res.partner '{"name":"John Doe","email":"john@example.com","phone":"+34600000000"}'

# Create a product
./scripts/create_record.sh product.product '{"name":"New Product","list_price":99.99,"type":"consu"}'
```

**Returns**: ID of the newly created record.

### `update_record.sh <model> <record_ids> <values_json>`
Update existing record(s).

```bash
# Update single record
./scripts/update_record.sh res.partner '[123]' '{"phone":"+34612345678","mobile":"+34699999999"}'

# Update multiple records
./scripts/update_record.sh product.product '[1,2,3]' '{"list_price":150.00}'
```

### `delete_record.sh <model> <record_ids>`
Delete record(s) from a model.

```bash
# Delete single record
./scripts/delete_record.sh res.partner '[123]'

# Delete multiple records
./scripts/delete_record.sh product.product '[1,2,3]'
```

**Warning**: Deletion is permanent. Ensure you have backups.

## Generic Method Execution

### `execute_method.sh <model> <method> <args> [kwargs]`
Execute any method on any Odoo model.

```bash
# Get name representation of records
./scripts/execute_method.sh res.partner name_get '[[1,2,3]]' '{}'

# Confirm a sale order
./scripts/execute_method.sh sale.order action_confirm '[[5]]' '{}'

# Get system parameter
./scripts/execute_method.sh ir.config_parameter get_param '["web.base.url"]' '{}'

# Send email
./scripts/execute_method.sh mail.mail send '[[123]]' '{}'
```
