# Dashboard API — v19 Delta (via odoo-pilot)

Only v19-specific changes. For base CRUD operations, see `dashboard-api-v18.md`.

## Field Changes

| v18 | v19 |
|-----|-----|
| `company_id` (Many2one) | `company_ids` (Many2many): `[[6, 0, [company_id]]]` |
| — | `favorite_user_ids` (Many2many) — users who favorited |
| — | `is_favorite` (computed Boolean) |
| — | `action_toggle_favorite()` method |

## Read Method

```bash
# v18: get_readonly_dashboard → returns dict with 'snapshot' key
execute_method.sh "spreadsheet.dashboard" "get_readonly_dashboard" "[[$ID]]" '{}'

# v19: _get_serialized_readonly_dashboard → returns JSON string directly
execute_method.sh "spreadsheet.dashboard" "_get_serialized_readonly_dashboard" "[[$ID]]" '{}'
```

## New HTTP Endpoint

```
GET /spreadsheet/dashboard/data/<int:dashboard>
```

Serves dashboard data directly — no RPC overhead.

## Create (v19)

```bash
create_record.sh "spreadsheet.dashboard" "{
  \"name\": \"Dashboard\",
  \"dashboard_group_id\": $GROUP_ID,
  \"group_ids\": [[6, 0, [$SECURITY_GROUP_ID]]],
  \"company_ids\": [[6, 0, [$COMPANY_ID]]],
  \"spreadsheet_data\": $(cat dashboard.json | jq -Rs .)
}"
```

Note: `company_ids` is Many2many (not Many2one like v18's `company_id`).

## Data Storage

- `_get_serialized_readonly_dashboard([id])` returns JSON **string** (not dict)
- File extension: `.osheet.json` for v19 dashboard files
