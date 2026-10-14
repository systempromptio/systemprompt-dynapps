# v19 — `indaws_dataflow` shared layer (export)

**APPLIES TO Odoo 19.0.** In v19, `indaws_data_export` was refactored to build on a new shared
module **`indaws_dataflow`**. The per-flow models from v14–v18 (`indaws.export.variable`,
`indaws.export.log`, `indaws.export.param.mixin`) **no longer exist** — they moved to
`indaws.dataflow.*`. The rest of this skill's references describe the **v18** API; for v19 use
THIS file as the source of truth.

> If you generate v18 XML (`indaws.export.variable`, `variables_ids`) on a v19 DB you get
> `KeyError: 'indaws.export.variable'` at install time. This file prevents that.

---

## 1. Dependency change

| | v14–v18 | v19 |
|---|---|---|
| `__manifest__` depends | `['indaws_base', 'queue_job']` | **`['indaws_dataflow']`** (pulls queue_job + base transitively) |

So a v19 module that uses exports declares `# inDAWS: indaws_dataflow` (not the data modules' old deps).

---

## 2. Model map v18 → v19

| v18 model | v19 model | Notes |
|---|---|---|
| `indaws.export.variable` | **`indaws.dataflow.variable`** | shared by import+export; matched by `model_id` (or global) |
| `indaws.export.log` | **`indaws.dataflow.log`** | adds `flow_type`, `scheduler_id`, `execution_time`, `records_processed/error` |
| `indaws.export.param.mixin` | **`indaws.dataflow.param.mixin`** | adds `json` ttype + `value_json`/`value_json_string` |
| *(none — cron on the export)* | **`indaws.export.scheduler`** (`_inherit indaws.dataflow.scheduler`) | new scheduler with state machine + micro-batch |
| *(none)* | **`indaws.dataflow.connection`** (+ `.connection.param`) | reusable external-system client, shared with import |
| `indaws.export.template` | `indaws.export.template` (`_inherit 'indaws.dataflow.template'`) | still exists; now inherits the base |
| `indaws.export` | `indaws.export` | unchanged name; `log_ids` now → `indaws.dataflow.log` |
| `indaws.export.registry` | `indaws.export.registry` | unchanged name; `state` adds `cancel` |
| `indaws.export.param.template` / `.param.export` | same names (`_inherit indaws.dataflow.param.mixin`) | unchanged names |

**Breaking field rename on the template:** v18 `variables_ids` (typo'd plural) → **v19 `variable_ids`** (Many2many to `indaws.dataflow.variable`, computed, NOT stored).

---

## 3. Variables in v19 (`indaws.dataflow.variable`)

Fields: `name` (required), `description`, `model_id` (optional — empty = **global**), `python_code` (required).
SQL unique: `(name, model_id)`.

**Matching to a template** is automatic and computed (no M2M link record):
```python
# indaws.export.template._compute_variable_ids
variable_ids = env['indaws.dataflow.variable'].search([
    '|', ('model_id', '=', False), ('model_id', '=', tmpl.model_id.id),
])
```
→ A variable is available to a template iff it is **global** (`model_id` empty) or targets the
template's model.

**Setting the result:** the `python_code` must assign `value = <obj>`. The engine pre-seeds
`value=None`, runs `safe_eval(..., mode="exec")`, captures `eval_ctx['value']`.

**Reaching `export_code`:** variables are injected **both** flat (by name) **and** as a `variables`
dict. So `my_var` and `variables['my_var']` both work in `export_code`.

```xml
<!-- Global variable (available to every template) -->
<record id="var_dlg_client" model="indaws.dataflow.variable">
    <field name="name">delegalog_client</field>
    <field name="description">JSON-RPC client builder</field>
    <field name="python_code"><![CDATA[
cfg = env['ir.config_parameter'].sudo()
# requests is preloaded in the base eval context (wrapped) — no wrap_module needed
session = requests
value = {'session': session, 'url': cfg.get_param('my.url')}
]]></field>
</record>

<!-- Model-scoped variable -->
<record id="var_active_partners" model="indaws.dataflow.variable">
    <field name="name">active_partners</field>
    <field name="model_id" ref="base.model_res_partner"/>
    <field name="python_code">value = env['res.partner'].search([('active','=',True)], limit=100)</field>
</record>
```

---

## 4. Params in v19

`indaws.dataflow.param.mixin` ttypes: `text, integer, float, boolean, date, datetime, many2one, json`.
Write the typed value into the matching `value_<ttype>` field (`json` uses `value_json_string`).
`get_value()` returns the typed Python value; `get_params()` returns `{name: value}`.

Params are injected into `export_code` **flat by name** (there is **no `params` dict** key —
access each param by its bare name). This is the same flat-injection behaviour as v18
(`eval_ctx.update(get_params())`); only the param model/typing changed in v19.

```xml
<record id="exp_tmpl_orders" model="indaws.export.template">
    <field name="name">Export Sale Orders</field>
    <field name="model_id" ref="sale.model_sale_order"/>
    <field name="flow_type">export</field>            <!-- new in v19 -->
    <field name="trigger">on_create_or_write</field>
    <field name="filter_domain">[('state','=','sale')]</field>
    <field name="iter_record" eval="True"/>
    <field name="background" eval="True"/>
    <field name="export_code"><![CDATA[
# flat names: api_key (param), delegalog_client (global variable)
dlg = delegalog_client
result = {'sent': record.name}
]]></field>
</record>

<record id="exp_tmpl_orders_p_key" model="indaws.export.param.template">
    <field name="template_id" ref="exp_tmpl_orders"/>
    <field name="name">api_key</field>
    <field name="ttype">text</field>
    <field name="value_text">REPLACE</field>
</record>

<record id="exp_orders" model="indaws.export">
    <field name="name">Export Sale Orders</field>
    <field name="template_id" ref="exp_tmpl_orders"/>
</record>
```

---

## 5. `export_code` eval context (v19)

Built from `indaws.dataflow.template._get_base_eval_context()` + export-specific keys.

**Preloaded (base, no `wrap_module` needed):** `json, open, re, Markup, html2plaintext,
plaintext2html, defaultdict, OrderedDict, Counter, requests, urllib, hashlib, hmac, base64, time,
datetime, dateutil, pytz, format_date, format_amount, _, ValidationError, wrap_module, import_module`.

> Note: `requests`, `base64`, `hashlib`, `re`, etc. are **already wrapped and available** in v19 —
> you usually do NOT need `wrap_module('requests', ...)` anymore (it still works).

**Export-specific:** `env, ctx, model, UserError, record, records, log(level, message, result),
ir_log(message, level), result, action`, plus (in a registry run) `rec_export, rec_template,
rec_registry`, all **param names flat**, all **variable names flat**, and `variables` (dict).

`log` signature is `log(level, message, result_dict)` (writes to `indaws.dataflow.log`).
Set `result = {...}` for the registry; set `action = {...}` to return a UI action (manual runs).

---

## 6. Triggering + scheduler (v19)

- Manual: `env['indaws.export'].browse(id).action_execute()`.
- `on_time` trigger auto-creates an `ir.cron` running `model.browse(<id>).action_execute()`.
  **v19 `ir.cron` has no `numbercall`** — never set it in XML.
- New `indaws.export.scheduler` (`_inherit indaws.dataflow.scheduler`): fields `template_id`
  (→ `indaws.export.template`), `export_id` (required), `flow_type='export'`, `state`
  (`draft/running/paused/error`), `interval_*`/`nextcall`/`cron_active` (related to auto cron),
  `execution_mode` (`single`/`micro_batch`), `max_errors` (auto-pause). Run: `action_execute()`;
  control: `action_start()`, `action_pause()`, `action_reset()`.

---

## 7. v19 gotchas (export)

1. **`indaws.export.variable` is GONE** → use `indaws.dataflow.variable`.
2. **`variables_ids` → `variable_ids`** (singular-ish) on the template; comodel changed.
3. **No `params` dict** in eval context — params arrive flat by name (same as v18; not a v19 change).
4. **`flow_type='export'`** is required on the template (defaulted, but set it in XML to be explicit).
5. **Depend on `indaws_dataflow`**, not on `indaws_data_export`'s old deps.
6. **`ir.cron` without `numbercall`** (removed in Odoo 19).
7. Connections moved to `indaws.dataflow.connection` (shared with import) — see import's v19 ref.
