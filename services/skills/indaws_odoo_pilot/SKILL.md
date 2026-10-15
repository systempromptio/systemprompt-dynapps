---
name: indaws-odoo-pilot
version: 1.0.4
effort: low
model: sonnet
description: Control and manage Odoo instances through API. Use this skill when you need to interact with Odoo databases, execute operations, manage modules, perform CRUD operations on records, or configure Odoo settings. Supports both Odoo below 19.0 (JSON-RPC) and Odoo 19.0+ (JSON2) protocols. Includes automated scripts for authentication, module management (install/uninstall/upgrade), record operations (search/create/update/delete), and generic method execution.
---

# Odoo Pilot

Control Odoo instances through API calls. Provides scripts for authentication, module management, and CRUD operations via HTTP API.

## Protocol Selection (CRITICAL)

- **Odoo >= 19.0**: ALWAYS use JSON2 protocol
- **Odoo <= 18.0**: ALWAYS use JSON-RPC protocol
- `auth.sh` auto-detects version and selects the correct protocol

## Quick Start

```bash
# 1. Set environment variables
export ODOO_URL="https://your-instance.odoo.com"
export ODOO_DB="your_database"
export ODOO_USER="admin"
export ODOO_KEY="your-api-key-or-password"

# 2. Authenticate (auto-detects protocol)
eval $(./scripts/auth.sh)

# 3. Use any script
./scripts/search_records.sh res.partner '[["is_company","=",true]]' '["name","email"]' 5
./scripts/install_module.sh sale_management
./scripts/create_record.sh res.partner '{"name":"New Company","email":"info@company.com"}'
```

## Available Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `auth.sh [protocol]` | Authenticate (auto-detect, json2, jsonrpc) | `eval $(./scripts/auth.sh)` |
| `detect_version.sh` | Show Odoo version info | `./scripts/detect_version.sh` |
| `install_module.sh` | Install module | `./scripts/install_module.sh sale` |
| `uninstall_module.sh` | Uninstall module | `./scripts/uninstall_module.sh sale` |
| `upgrade_module.sh` | Upgrade module | `./scripts/upgrade_module.sh sale` |
| `search_records.sh` | Search & read records | `./scripts/search_records.sh model domain [fields] [limit] [offset]` |
| `create_record.sh` | Create record | `./scripts/create_record.sh model '{json}'` |
| `update_record.sh` | Update record(s) | `./scripts/update_record.sh model '[ids]' '{json}'` |
| `delete_record.sh` | Delete record(s) | `./scripts/delete_record.sh model '[ids]'` |
| `execute_method.sh` | Call any ORM method | `./scripts/execute_method.sh model method '[args]' '{kwargs}'` |

## Key Rules

- **ALWAYS** authenticate first with `eval $(./scripts/auth.sh)` before any operation
- **ALWAYS** use environment variables for credentials -- NEVER hardcode them
- **ALWAYS** use `eval $()` wrapper for auth scripts (they export env vars)
- **ALWAYS** check exit codes: 0 = success, 1 = error
- **ALWAYS** use HTTPS in production
- **NEVER** commit credentials to version control
- **NEVER** use `auth_json2.sh` or `auth_jsonrpc.sh` directly -- use `auth.sh` instead
- stdout = data (JSON), stderr = status/errors -- parse accordingly with `jq`
- Deletion is permanent -- ensure backups exist before using `delete_record.sh`
- Add delays between bulk operations to avoid rate limiting

## Domain Syntax Quick Reference

```
Single:    [["field","=","value"]]
AND:       [["f1","=","a"],["f2",">",10]]
OR:        ["|",["f1","=","a"],["f2","=","b"]]
NOT:       ["!",["field","=","value"]]
```

## Gotchas

Read `references/gotchas.md` FIRST -- it covers the most common mistakes and edge cases.

See `references/gotchas.md` for the full list. Key ones:
1. **Wrong protocol = silent auth failure** — json2 for v19+, jsonrpc for v18-. Check version first.
2. **search_read returns max 80 records by default** — always set explicit limit or use pagination.
3. **Many2one fields return [id, name] tuples** — extract id with `field[0]` not `field`.
4. **context must include lang:es_ES** — otherwise translated fields return English values.
5. **API calls need sudo for cross-company data** — record rules apply to API calls too.

See `references/shared/odoo-connection.md` for the canonical connection pattern, `references/shared/batch-limits.md` for pagination, and `references/shared/field-mapping-syntax.md` for field types.

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| onboarding | common | When credentials are missing |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| odoo-studio | common | API operations for Studio customizations |
| odoo-importation | common | API connection for bulk imports |
| odoo-copy-staging-master | common | API operations for config migration |
| migration-assistant | common | Gate queries and project data |
| odoo-mentor | common | Project/CRM health data |
| ticket-report | common | Ticket data from Helpdesk |
| timesheet-report | common | Timesheet data queries |
| org-chart | common | HR data for org visualization |
| email-campaign-builder | common | CRM data for campaigns |
| migration-estimator | migration-lab | Create the quotation (sale.order) via API |
| migration-portfolio / -billing-tracker / -closeout | migration-lab | Project, billing milestones and final invoice via API |
| odoo-server-action-creator | common | API validation of server actions |
| 12 consultant skills | consultant | Odoo instance operations |
| 10 sales skills | sales | CRM and pipeline data |
| 4 marketing skills | marketing | Odoo data for campaigns |
| 2 management skills | management | Project/billing data |

## References

| File | Content |
|------|---------|
| [references/gotchas.md](references/gotchas.md) | Common mistakes and edge cases (READ FIRST) |
| [references/scripts-reference.md](references/scripts-reference.md) | Detailed script documentation with examples |
| [references/protocol-details.md](references/protocol-details.md) | JSON-RPC vs JSON2 protocol comparison |
| [references/workflows.md](references/workflows.md) | Common workflow patterns (bulk ops, search+update) |
| [references/error-handling.md](references/error-handling.md) | Error codes, troubleshooting, common models, security |
| [README.md](README.md) | Quick reference guide |
| [STRUCTURE.md](STRUCTURE.md) | File organization & purpose guide |
| [PROTOCOL_GUIDE.md](PROTOCOL_GUIDE.md) | Protocol comparison & migration guide |
| [JSON2_ENDPOINT_FORMAT.md](JSON2_ENDPOINT_FORMAT.md) | JSON2 format reference (Odoo >= 19) |
| [EXAMPLES_JSON2.md](EXAMPLES_JSON2.md) | Practical JSON2 usage examples |
| [CHANGELOG_JSON2.md](CHANGELOG_JSON2.md) | JSON2 implementation details & fixes |
