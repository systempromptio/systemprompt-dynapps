# Session State Persistence Reference

## Directory Structure

```
<project_path>/.flow-tester/
  sessions/
    FT-20260307-001.json    # Active or completed session
    FT-20260305-002.json    # Previous session
  reports/
    FT-20260307-001-report.md   # Final report (generated on completion)
  cleanup/
    FT-20260307-001-cleanup.sh  # Cleanup script for test data
```

## Session ID Format

`FT-YYYYMMDD-NNN` where NNN is sequential per day.

## Session Lifecycle

```
NEW → IN_PROGRESS → COMPLETED
                  → PAUSED (user exits mid-run)
                  → ABORTED (user cancels)
                  → FAILED (unrecoverable error)
```

## State File Schema

> **`code_sources`: cada raíz nombra los majores distinto.** `$ODOO_SRC/odoo18`,
> `$ODOO_ENTERPRISE_SRC/18`, `$OCA_SRC/v18`, `$PROJECTS_ROOT/indawsodoo/18`. Esta plantilla usaba
> `18.0` para las cuatro y **ninguna existía** (corregido 2026-08-07). La convención es la de
> `references/shared/odoo-source.md` — y ahí manda "resuélvelo, no lo des por hecho": haz `ls`
> de la base antes de escribir la ruta en el estado de sesión.

```json
{
  "session_id": "string (FT-YYYYMMDD-NNN)",
  "created_at": "ISO 8601 datetime",
  "updated_at": "ISO 8601 datetime",
  "status": "new | in_progress | paused | completed | aborted | failed",

  "config": {
    "odoo_url": "string",
    "odoo_db": "string",
    "odoo_version": "string (e.g., 18.0)",
    "odoo_user": "string",
    "protocol": "jsonrpc | json2",
    "test_prefix": "[TEST-FT]",
    "auto_cleanup": false,
    "stop_on_fail": false
  },

  "code_sources": {
    "base": "$ODOO_SRC/odoo18",
    "enterprise": "$ODOO_ENTERPRISE_SRC/18",
    "oca": "$OCA_SRC/v18",
    "indaws": "$PROJECTS_ROOT/indawsodoo/18",
    "custom": "$PROJECTS_ROOT/ProjectName/modules"
  },

  "source_document": {
    "path": "/path/to/original/document",
    "format": "markdown | xlsx | csv | pdf | text",
    "hash": "sha256 of original file for change detection"
  },

  "flows": [
    {
      "id": "FLOW-001",
      "name": "string",
      "description": "string",
      "priority": "critical | high | medium | low",
      "status": "pending | in_progress | passed | failed | skipped",
      "started_at": "ISO 8601 or null",
      "completed_at": "ISO 8601 or null",
      "preconditions": [
        {
          "description": "Partner 'Test Customer' exists",
          "model": "res.partner",
          "domain": [["name", "=", "Test Customer"]],
          "satisfied": true,
          "record_id": 42
        }
      ],
      "steps": [
        {
          "number": 1,
          "description": "Create quotation",
          "action": {
            "type": "create | write | method | search | wizard",
            "model": "sale.order",
            "method": "create",
            "args": {},
            "kwargs": {}
          },
          "status": "pending | passed | failed | skipped | warning",
          "executed_at": "ISO 8601 or null",
          "result": {
            "record_id": 1234,
            "return_value": null,
            "error": null,
            "traceback": null
          },
          "validations": [
            {
              "type": "field_check",
              "model": "sale.order",
              "record_id": 1234,
              "field": "state",
              "operator": "==",
              "expected": "draft",
              "actual": "draft",
              "passed": true
            }
          ],
          "duration_seconds": 1.2,
          "notes": "optional free-text notes"
        }
      ],
      "created_records": [
        {"model": "sale.order", "id": 1234, "ref": "SO-TEST-001"},
        {"model": "sale.order.line", "id": 5678, "ref": null}
      ],
      "error_analysis": {
        "step": 3,
        "error_type": "VALIDATION_ERROR",
        "message": "...",
        "code_reference": "sale/models/sale_order.py:456",
        "suggested_fix": "..."
      }
    }
  ],

  "summary": {
    "total_flows": 0,
    "flows_passed": 0,
    "flows_failed": 0,
    "flows_skipped": 0,
    "flows_pending": 0,
    "total_steps": 0,
    "steps_passed": 0,
    "steps_failed": 0,
    "steps_skipped": 0,
    "steps_warning": 0
  },

  "all_created_records": [
    {"model": "sale.order", "id": 1234, "flow": "FLOW-001"},
    {"model": "account.move", "id": 5678, "flow": "FLOW-001"}
  ]
}
```

## Operations on Session State

### Create New Session

```python
import json, datetime, hashlib

session = {
    "session_id": generate_session_id(),
    "created_at": datetime.datetime.now().isoformat(),
    "updated_at": datetime.datetime.now().isoformat(),
    "status": "new",
    "config": { ... },
    "flows": [],
    "summary": default_summary(),
    "all_created_records": []
}
```

### Update After Step Execution

After every step, update:
1. `step.status`, `step.executed_at`, `step.result`, `step.validations`
2. `flow.status` if all steps done
3. `session.summary` counters
4. `session.updated_at`
5. `session.all_created_records` if new records were created
6. **Write file immediately** — do not batch updates

### Resume Session

1. Read session JSON
2. Find first flow with `status != 'passed' && status != 'skipped'`
3. Within that flow, find first step with `status == 'pending'`
4. Re-authenticate with Odoo
5. Verify previously created records still exist (spot check)
6. Continue from that step

### Cleanup Script Generation

On session completion or abort, generate a cleanup script:

```bash
#!/bin/bash
# Cleanup for session FT-20260307-001
# WARNING: This will DELETE test records. Review before running.

# Delete in reverse dependency order
source auth.sh
# Invoices first (most dependent)
call_method account.move unlink --ids "[5678]"
# Then sale orders
call_method sale.order unlink --ids "[1234]"
```

## Concurrency

- Only ONE session per project should be active (`in_progress`) at a time
- When starting a new session, check for existing `in_progress` sessions
- Offer to: resume existing, pause existing and start new, or abort existing
