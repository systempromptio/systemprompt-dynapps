# Workflow, Architecture & Trigger Types

## Workflow (Mandatory Sequence)

### STEP 1: Information Gathering [STOP]

Use `AskUserQuestion` to gather:

**Priority 1 — Basic:**
1. What to export? (model, fields)
2. When? manual / on_create / on_write / on_create_or_write / on_unlink / on_time
3. Where? REST API / SFTP / file (CSV/Excel/JSON) / webhook / database
4. Which records? All, domain filter, specific field changes

**Priority 2 — Technical:**
5. Odoo version (14-19)
6. Instance details (URL, DB, credentials or local path)
7. Data format (JSON/XML/CSV/custom)
8. Authentication (API key, OAuth, Basic, Bearer)

**Priority 3 — Advanced:**
9. Error handling (retry, notification, fallback)
10. Performance (background async, iter_record)

**STOP** — Wait for response.

### STEP 2: Select Use Case
| Need | Trigger | Guide |
|---|---|---|
| Send new records to API | on_create | [02-on-create-webhook.md](../use-cases/02-on-create-webhook.md) |
| Daily/hourly export | on_time | [04-scheduled-export.md](../use-cases/04-scheduled-export.md) |

### STEP 3: Follow the use case guide (prerequisites, template, trigger, code, test)

### STEP 4: Validation & Testing
1. Verify structure: trigger, model, template, background setting
2. Test manually: `export.action_execute()` → check registry
3. Test automatic trigger: create/update/delete test record
4. Monitor errors: logs, failed registries

---

## Architecture — Models

```
indaws.export.template          # Reusable config
    ├── export_code             # Python code to export
    ├── records_code            # Python code to select records
    ├── variables_ids           # Python variables
    └── param_ids               # Parameters

indaws.export                   # Export configuration
    ├── template_id             # → template
    ├── trigger                 # When to execute
    ├── filter_domain           # Which records
    ├── trigger_field_ids       # Which field changes (on_write)
    ├── cron_id                 # Scheduled execution
    └── registry_ids            # History

indaws.export.registry          # Execution log
    ├── state                   # pending/queued/process/done/failed
    ├── result                  # Export result
    └── queue_job_ids           # Async jobs

indaws.export.log               # Detailed logs (level + message)
```

### ORM Hooks
Module patches `create()`, `write()`, `_compute_field_value()`, `unlink()` via `_register_hook()`.
**Caution:** Heavy exports on frequently updated models impact perf. Use `filter_domain`, `trigger_field_ids`, `background=True`.

---

## Trigger Types

| Trigger | When | Use For | Key Filters |
|---|---|---|---|
| `manual` | User clicks / `action_execute()` | On-demand, testing | — |
| `on_create` | Record created | Webhooks, initial sync | `filter_domain` |
| `on_write` | Record updated | Change sync | `filter_pre_domain`, `filter_domain`, `trigger_field_ids` |
| `on_create_or_write` | Created or updated | Complete sync | All above |
| `on_unlink` | Record deleted | Deletion notifications | `filter_domain` |
| `on_time` | Cron schedule | Periodic/batch | `interval_number`, `interval_type` |

---

## Python Code Context

Available variables in export_code:
- `env`, `model`, `records`, `record` (if iter_record), `rec_export`, `rec_template`, `rec_registry`
- `log(message)`, `variables` (dict), `params` (dict)

See [python-code-context.md](python-code-context.md) for full reference.

---

## Performance

| Setting | When to use |
|---|---|
| `background=True` | API calls, large transforms, multiple records |
| `iter_record=True` | Independent per-record processing, API per record |
| `trigger_field_ids` set | Only trigger on specific field changes (not ALL writes) |
