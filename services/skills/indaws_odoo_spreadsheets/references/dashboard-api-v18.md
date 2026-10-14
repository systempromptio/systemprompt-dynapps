# Dashboard API — v18 (via odoo-pilot)

## Prerequisites

Active odoo-pilot connection. Authenticate first:
```bash
eval $(./scripts/auth.sh)
```

## Models

| Model | Purpose |
|-------|---------|
| `spreadsheet.dashboard` | Dashboard records (name, data, group) |
| `spreadsheet.dashboard.group` | Dashboard categories (Sales, Finance, etc.) |
| `ir.attachment` | Snapshot storage (Enterprise collaborative editing) |

### Dashboard Model Fields (v18)

| Field | Type | Description |
|-------|------|-------------|
| `name` | Char | Dashboard display name |
| `dashboard_group_id` | Many2one | Category group |
| `group_ids` | Many2many | Security groups with access |
| `spreadsheet_data` | Text | JSON string with full dashboard definition |
| `sample_dashboard_file_path` | Char | Path to sample JSON (onboarding) |
| `company_id` | Many2one | Optional company isolation |

## List Groups

```bash
search_records.sh "spreadsheet.dashboard.group" '[]' '["name","id"]'
```

## List Dashboards

```bash
search_records.sh "spreadsheet.dashboard" '[]' '["name","dashboard_group_id","group_ids"]'
```

## Create

```bash
# Build JSON data first (see core-v18.md), then:
create_record.sh "spreadsheet.dashboard" "{
  \"name\": \"My Dashboard\",
  \"dashboard_group_id\": $GROUP_ID,
  \"group_ids\": [[6, 0, [$SECURITY_GROUP_ID]]],
  \"spreadsheet_data\": $(cat dashboard.json | jq -Rs .)
}"
```

**Tip:** Write the JSON to a temp file first, then use `jq -Rs .` to escape it as a string value for `spreadsheet_data`.

## Read

```bash
# Use get_readonly_dashboard to get the real data (not direct read)
execute_method.sh "spreadsheet.dashboard" "get_readonly_dashboard" "[[$DASHBOARD_ID]]" '{}'
```

Returns dict with `snapshot` key. Parse: `echo "$result" | jq -r '.snapshot' | jq .`

**Always use `get_readonly_dashboard()`, not direct `.read()`** — Enterprise may return minimal data otherwise.

## Update

```bash
# 1. Read current data
CURRENT=$(execute_method.sh "spreadsheet.dashboard" "get_readonly_dashboard" "[[$DASHBOARD_ID]]" '{}')

# 2. Modify the snapshot JSON (preserve UUIDs!)

# 3. Write back
update_record.sh "spreadsheet.dashboard" "[$DASHBOARD_ID]" "{
  \"spreadsheet_data\": $(echo "$MODIFIED_JSON" | jq -Rs .)
}"
```

**CRITICAL**: Always read first, preserve existing UUIDs, write full JSON back.

## Delete

```bash
delete_record.sh "spreadsheet.dashboard" "[$DASHBOARD_ID]"
```

## Analyze Model Fields (for pivot/list design)

```bash
# Get fields of a model to design pivots
search_records.sh "ir.model.fields" \
  '[[\"model\",\"=\",\"sale.report\"]]' \
  '["name","field_description","ttype","relation","store"]' 200
```

## Find Security Groups

```bash
# Find group ID by XML ID
search_records.sh "ir.model.data" \
  '[[\"module\",\"=\",\"sales_team\"],[\"name\",\"=\",\"group_sale_salesman\"]]' \
  '["res_id"]' 1
```

## Common Groups (IDs are DB-specific — search by name)

| Name | Typical XML ID |
|------|---------------|
| Finance | `account.group_account_user` |
| Sales | `sales_team.group_sale_salesman` |
| Human Resources | `hr.group_hr_user` |
| Logistics | `stock.group_stock_user` |
| CRM | `sales_team.group_sale_salesman` |

## Snapshot Management

Enterprise stores collaborative data in `ir.attachment`. Writing `spreadsheet_data` invalidates old snapshot; new one created on next load.

```bash
search_records.sh "ir.attachment" \
  '[[\"res_model\",\"=\",\"spreadsheet.dashboard\"],[\"res_id\",\"=\",'$DASHBOARD_ID'],[\"name\",\"ilike\",\"snapshot\"]]' \
  '["id","file_size"]'
```
