---
name: indaws-data-export
description: Advanced skill for creating and managing automated data exports from Odoo using the indaws_data_export module. Use when the user needs to export data to external systems via APIs, webhooks, or files; set up automatic exports triggered by record changes (create/update/delete); schedule periodic exports; or implement complex data synchronization. This skill covers template creation, trigger configuration, Python code integration, and troubleshooting.
version: 1.0.2
allowed-tools: [Bash, Read, Write, Glob, Grep, AskUserQuestion, EnterPlanMode]
effort: medium
model: opus
---

# inDAWS Data Export Skill

Enterprise-grade automated exports from Odoo using `indaws_data_export` (v14-v19).
Automatic triggers on create/write/delete, scheduled crons, custom Python code, async with queue_job.

## ⚠️ Version awareness (v18 vs v19) — READ FIRST

The reference files below describe the **v14–v18** API. In **v19** the module was refactored onto a
new shared **`indaws_dataflow`** layer and several models were **renamed/moved**:

- `indaws.export.variable` → **`indaws.dataflow.variable`** (the v18 model no longer exists)
- `indaws.export.log` → **`indaws.dataflow.log`**; `indaws.export.param.mixin` → **`indaws.dataflow.param.mixin`**
- template field **`variables_ids` → `variable_ids`**; params arrive **flat** (no `params` dict)
- new `indaws.export.scheduler` + shared `indaws.dataflow.connection`; depend on **`indaws_dataflow`**

**For any v19 work, read [references/v19-dataflow.md](references/v19-dataflow.md) FIRST** — generating
v18 XML on a v19 DB fails with `KeyError: 'indaws.export.variable'`.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Analyze export requirements | sonnet | Read config and specs |
| Design export pipeline | **opus** | Architecture decisions |
| Generate code | **opus** | Python code for collectors/dispatchers |

## References (read on demand)

| Ref | When to read |
|-----|-------------|
| [references/v19-dataflow.md](references/v19-dataflow.md) | **FIRST, if target is Odoo 19** — v18→v19 model renames (`indaws.dataflow.*`), eval context, XML examples |
| [references/workflow-and-architecture.md](references/workflow-and-architecture.md) | **Always** — 4-step workflow, models, triggers, ORM hooks, performance (v18 model names) |
| [references/python-code-context.md](references/python-code-context.md) | When writing export Python code — all available variables |
| [use-cases/](use-cases/) | After selecting trigger type — step-by-step guides per scenario |

## Dependencies

```python
'depends': ['indaws_base', 'queue_job']  # queue_job: workers > 1
```

## Use Cases

| Trigger | Guide | Example |
|---------|-------|---------|
| manual | [01-manual-export-api.md](use-cases/01-manual-export-api.md) | Export button in form |
| on_create | [02-on-create-webhook.md](use-cases/02-on-create-webhook.md) | Webhook on new record |
| on_write | [03-on-write-sync.md](use-cases/03-on-write-sync.md) | Sync field changes |
| on_time | [04-scheduled-export.md](use-cases/04-scheduled-export.md) | Daily/hourly cron export |
| on_unlink | [05-on-unlink-notification.md](use-cases/05-on-unlink-notification.md) | Deletion notifications |
| file gen | [06-csv-generation.md](use-cases/06-csv-generation.md) | CSV/Excel generation |
| complex | [07-integration-erp.md](use-cases/07-integration-erp.md) | ERP sync with retry |

## Quick Start

```python
# 1. Create template
template = env['indaws.export.template'].create({
    'name': 'Export Orders',
    'model_id': env.ref('sale.model_sale_order').id,
    'export_code': '''
import requests
data = {'order': record.name, 'total': record.amount_total}
response = requests.post('https://api.example.com/orders',
    json=data, headers={'API-Key': params['api_key']})
result = {'status': response.status_code}
''',
})

# 2. Create export with trigger
export = env['indaws.export'].create({
    'name': 'Auto Export Orders',
    'template_id': template.id,
    'trigger': 'on_create',
    'filter_domain': "[('state', '=', 'sale')]",
    'background': True,
})
```

## Workflow Summary

1. **Gather info** → model, trigger, destination, auth [STOP]
2. **Select use case** → match trigger type to guide
3. **Follow guide** → template + trigger + code + test
4. **Validate** → manual test → auto trigger → monitor errors

Read [workflow-and-architecture.md](references/workflow-and-architecture.md) for full details.

## Prohibitions

- NEVER skip queue_job verification (workers > 1)
- NEVER use empty `trigger_field_ids` on high-frequency models (triggers on ALL writes)
- NEVER hardcode credentials in export_code (use `params`)
- NEVER forget `background=True` for API calls

See references/shared/odoo-connection.md for connection pattern and references/shared/batch-limits.md for pagination.

See references/shared/field-mapping-syntax.md for field types and command tuples.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | API connection for export operations |
| odoo-integration-builder | developer | Custom-code integrations (outbound clients / inbound webhooks) beyond configurable export templates |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| odoo-configurator | consultant | Configure automated exports |
| odoo-integration-builder | developer | When a configurable export by record changes suffices (no custom code) |

## GOTCHAS
0. **Python code stages in English** — the Python you write for export transforms (variables, comments) always in English regardless of client locale. See `references/shared/coding-standards.md`.
1. **Export triggers fire on write** — a bulk update triggers exports for every record. Use batch mode.
2. **API endpoints may rate-limit** — implement retry with exponential backoff.
3. **Datetime fields export in UTC** — consumer systems may expect local timezone.
4. **Binary fields (attachments) are base64** — large files can timeout the export.
5. **Webhook retries don't deduplicate** — consumer must handle idempotency.

## Related Skills

- **odoo-pilot** — Test exports via API
- **odoo-code-auditor** — Audit Python code quality
- **indaws-data-import** — Import counterpart for bidirectional sync
- **odoo-integration-builder** — Custom integration code vs configurable export templates
