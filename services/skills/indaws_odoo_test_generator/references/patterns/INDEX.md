# Pattern File Index — On-Demand Loading

This index maps detected code components to the specific pattern files needed.
**Load only the files required by the analysis phase — never load all 22.**

---

## Routing Table

After running the detection scans (Phase 1-2 of detection-matrix.md), use this table to determine which pattern files to load.

| Detected Component | Pattern File(s) to Load | Priority |
|--------------------|------------------------|----------|
| Any model (always needed) | [01-transaction-case.md](01-transaction-case.md) | ALWAYS |
| `@api.onchange` / Form class needed | [02-form-class.md](02-form-class.md) | HIGH |
| `raise UserError` / `raise ValidationError` / `@api.constrains` | [03-exceptions.md](03-exceptions.md) | HIGH |
| `def action_*` / `state` field / workflow transitions | [04-workflow.md](04-workflow.md) | HIGH |
| `requests.get/post` / external API / `_remote_*` / `_get_connection` | [05-mock-patch.md](05-mock-patch.md) + [18-external-deps.md](18-external-deps.md) + [19-advanced-mock.md](19-advanced-mock.md) | HIGH |
| Bulk operations / `_cron_*` / performance-critical paths | [06-performance.md](06-performance.md) | MEDIUM |
| Custom groups / `ir.rule` / `ir.model.access.csv` | [07-security.md](07-security.md) | HIGH |
| `@http.route` / controllers | [08-http-case.md](08-http-case.md) | HIGH |
| `assertRecordValues` / `Markup` / HTML fields / multi-record assertions | [09-assertion-helpers.md](09-assertion-helpers.md) | MEDIUM |
| `company_id` / multi-company rules | [10-multi-company.md](10-multi-company.md) | HIGH |
| `_sql_constraints` / conditional constraints / cascading constraints | [11-advanced-constraints.md](11-advanced-constraints.md) | HIGH |
| `_inherit` / `_inherits` / method overrides with `super()` | [12-inheritance.md](12-inheritance.md) | MEDIUM |
| Module-level `UPPER_CASE` dicts/sets/lists | [13-constants.md](13-constants.md) | MEDIUM |
| `Serialized` / `fields.Json` / `json.dumps` / `json.loads` | [14-json-serialized.md](14-json-serialized.md) | MEDIUM |
| Generators / importers / sync operations (idempotent ops) | [15-idempotency.md](15-idempotency.md) | MEDIUM |
| `_generate_*` / `_build_*` / `safe_eval` / `test_python_expr` | [16-code-generation.md](16-code-generation.md) | HIGH |
| User-facing messages / notifications / translated strings | [17-i18n-safe.md](17-i18n-safe.md) | MEDIUM |
| Multi-step workflows (discover/generate/execute) with external deps | [19-advanced-mock.md](19-advanced-mock.md) | HIGH |
| Cross-document flows (SO→invoice, PO→receive→invoice) / inherits 2+ flow models | [20-e2e-flow.md](20-e2e-flow.md) | HIGH |
| Target version is v19 | [21-odoo-v19-gotchas.md](21-odoo-v19-gotchas.md) | ALWAYS (v19) |
| `mcp.server.config` / generic exposure / runtime-chosen model/domain / JSON-RPC/OpenAPI bridge | [22-mcp-multi-principal.md](22-mcp-multi-principal.md) | HIGH (exposure modules) |
| Añade/override informe QWeb (`ir.actions.report`) / modelo `report.*` | [23-qweb-report.md](23-qweb-report.md) | HIGH (módulos con informe) |
| Flujo termina en hardware/websocket: impresión IoT, `bus.bus._sendone`, drivers, webhooks | [24-hardware-websocket-flows.md](24-hardware-websocket-flows.md) | HIGH (módulos IoT/POS/bus) |

---

## Quick Detection → File Mapping

For the grep scan results from detection-matrix.md Phase 5, map directly:

```
Grep: compute=              → 01 (always loaded)
Grep: @api.constrains       → 03, 11
Grep: _sql_constraints      → 11
Grep: @api.onchange         → 02
Grep: def action_           → 04
Grep: def create/write      → 01 (always loaded)
Grep: raise UserError       → 03
Grep: @http.route           → 08
Grep: group_/ir.rule        → 07
Grep: company_id            → 10
Grep: requests.get/post     → 05, 18, 19
Grep: @staticmethod         → 01 (always loaded)
Grep: _generate_/_build_    → 16
Grep: ^[A-Z_]+=             → 13
Grep: Serialized/Json       → 14
Grep: safe_eval             → 16
Grep: mcp.server.config / _readable_exposures / list_models → 22
Grep: _inherit              → 12
Grep: _prepare_invoice_line → 20
Grep: _prepare_account_move_line → 20
Grep: _get_stock_move_values → 20
Grep: _compute_qty_invoiced (override) → 20
Grep: create res.users / res.partner in test → 01 (§1.1b: MUST be post_install)
Grep: render_and_send / _send_websocket / device_ids → 24
Grep: bus.bus._sendone / _sendmany → 24
Grep: iot.device / iot.box / iot_handlers → 24
```

---

## Typical Load Profiles

| Module Type | Files Loaded | Estimated Tokens |
|-------------|-------------|-----------------|
| Simple CRUD model | 01, 03 | ~1,200 |
| Model + onchanges + workflow | 01, 02, 03, 04 | ~2,400 |
| Model + security + multi-company | 01, 03, 07, 10 | ~2,000 |
| External integration (API/connector) | 01, 03, 04, 05, 15, 17, 18, 19 | ~5,500 |
| Full complex module | 01-05, 07, 09, 11 | ~5,000 |
| Cross-document flow module (sale+stock+account) | 01, 02, 12, 20 | ~3,500 |
| Any v19 module | +21 (add to above) | +~1,500 |
