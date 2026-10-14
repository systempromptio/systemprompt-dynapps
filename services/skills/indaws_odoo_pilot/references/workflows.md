# Common Workflows

## Workflow 1: Install Multiple Modules

```bash
# Set credentials
export ODOO_URL="https://demo.odoo.com"
export ODOO_DB="demo"
export ODOO_KEY="your-api-key"
export ODOO_USER="admin"  # Only needed for JSON-RPC

# Authenticate (auto-detects protocol)
eval $(./scripts/auth.sh)

# Install modules
./scripts/install_module.sh sale_management
./scripts/install_module.sh purchase
./scripts/install_module.sh stock
```

## Workflow 2: Bulk Create Partners

```bash
# Authenticate
eval $(./scripts/auth.sh)

# Create multiple partners
for name in "Company A" "Company B" "Company C"; do
    ./scripts/create_record.sh res.partner "{\"name\":\"${name}\",\"is_company\":true}"
done
```

## Workflow 3: Search and Update

```bash
# Find all products with price < 50
RESULTS=$(./scripts/search_records.sh product.product '[["list_price","<",50]]' '["id"]' 1000)

# Extract IDs (requires jq)
IDS=$(echo "$RESULTS" | jq '[.[].id]')

# Update all prices by 10%
./scripts/update_record.sh product.product "$IDS" '{"list_price":55.00}'
```

## Workflow 4: Module Lifecycle Management

```bash
# Check current modules
./scripts/search_records.sh ir.module.module '[["state","=","installed"]]' '["name","state"]'

# Upgrade a module
./scripts/upgrade_module.sh sale

# Uninstall if needed
./scripts/uninstall_module.sh sale
```
