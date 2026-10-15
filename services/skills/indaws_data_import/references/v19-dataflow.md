# v19 — `indaws_dataflow` shared layer (import)

**APPLIES TO Odoo 19.0.** In v19, `indaws_data_import` was refactored onto a new shared module
**`indaws_dataflow`**. The connection and variable models from v14–v18
(`indaws.import.connection`, `indaws.import.variable`) **no longer exist** — they moved to
`indaws.dataflow.*`. The other references in this skill describe the **v18** API; for v19 use THIS
file as the source of truth.

> Generating v18 XML (`indaws.import.variable`, `indaws.import.connection`) on a v19 DB fails at
> install. The migration `migrations/19.0.1.0.0/pre-migrate.py` moves existing rows to the new tables.

---

## 1. Dependency change

| | v14–v18 | v19 |
|---|---|---|
| `__manifest__` depends | `['indaws_base', 'queue_job']` | **`['indaws_dataflow']`** |
| external python | pandas, dateutil, xlrd, openpyxl, requests | same (`['dateutil','openpyxl','pandas','requests','xlrd']`) |

`pandas` etc. must be installed in the Odoo runtime (the base `indaws/odoo` image may not ship them
— `pip install pandas openpyxl xlrd` if missing).

---

## 2. Model map v18 → v19 (CRITICAL)

| v18 model | v19 model | Notes |
|---|---|---|
| **`indaws.import.connection`** | **`indaws.dataflow.connection`** | model deleted from import; now shared |
| **`indaws.import.variable`** | **`indaws.dataflow.variable`** | model deleted from import; shared with export |
| `indaws.import.param` (connection params) | **`indaws.dataflow.connection.param`** | connection params live here now |
| `indaws.import.scheduler` (standalone) | `indaws.import.scheduler` (`_inherit 'indaws.dataflow.scheduler'`) | same name, now inherits base |
| `indaws.import.template` (standalone) | `indaws.import.template` (`_inherit 'indaws.dataflow.template'`) | same name, inherits base |
| `indaws.import.data.collector` | same | `variable_ids` comodel → `indaws.dataflow.variable` |
| `indaws.import.template.line` / `.match.line` | same | unchanged names |
| `indaws.import` (execution) | same | `defer_compute` field **removed** in 19.0.1.2.0 |

**Other field renames:** template `md5_active`/`md5_field_id` → **`sha1_active`/`sha1_field_id`**.
Dynamic hash fields on target models: `x_import_md5_*` → `x_import_sha1_*`.

---

## 3. Connection + Variable in v19 (the part that breaks v18 XML)

Architecture is the same shape — **Connection → Variable → Collector** — but the models moved:

```
indaws.dataflow.connection
    ├── variable_id    → indaws.dataflow.variable   (REQUIRED — builds the client, sets `value`)
    └── param_ids      → indaws.dataflow.connection.param  (credentials, injected into the variable)

indaws.dataflow.variable
    ├── name           # flat name under which the client is injected into the collector
    ├── model_id       # optional; empty = global (shared by all templates/collectors)
    └── python_code    # must set: value = <client object>
```

- Connection params are injected into the **variable's** eval context as **flat names** (NOT a
  `params` dict). The variable's `value` becomes available to the collector under `variable.name`.
- `connection.get_connection(eval_ctx)` runs the variable and returns the client object.

```xml
<!-- Variable: builds the client -->
<record id="var_api" model="indaws.dataflow.variable">
    <field name="name">api_client</field>
    <field name="python_code"><![CDATA[
# connection params (api_token, api_url) arrive as flat names here
# requests is preloaded (wrapped) in the base eval context
sess = requests
value = {'session': sess, 'token': api_token, 'url': api_url}
]]></field>
</record>

<!-- Connection: variable + params -->
<record id="conn_api" model="indaws.dataflow.connection">
    <field name="name">My REST API</field>
    <field name="company_id" ref="base.main_company"/>
    <field name="variable_id" ref="var_api"/>
    <field name="param_ids">
        <record id="conn_api_token" model="indaws.dataflow.connection.param">
            <field name="name">api_token</field>
            <field name="ttype">text</field>
            <field name="value_text">CHANGE_ME</field>
        </record>
        <record id="conn_api_url" model="indaws.dataflow.connection.param">
            <field name="name">api_url</field>
            <field name="ttype">text</field>
            <field name="value_text">https://api.example.com</field>
        </record>
    </field>
</record>
```

---

## 4. Data collector (v19)

`indaws.import.data.collector`: `name`, `python_code` (required), `param_ids`
(`indaws.import.param`), `variable_ids` (Many2many → **`indaws.dataflow.variable`**, additional
helpers beyond the connection's).

`get_data(params, variables, additional_context={})`:
- connection variables are injected **flat** (e.g. `api_client`), collector `variable_ids` merged on top.
- collector must set **`value`** (list of dicts) OR call **`yield_data(batch)`** (streaming —
  takes priority over `value`, keeps memory flat for big datasets).
- `additional_context` may carry `last_execution_date`, `scheduler`, and (micro-batch) a
  `checkpoint` proxy (`.read()/.write(data)/.progress(processed,total,errors,page)/.clear()`),
  `batch_page_limit`, `batch_yield_size`.

```xml
<record id="collector_products" model="indaws.import.data.collector">
    <field name="name">Products API Collector</field>
    <field name="variable_ids" eval="[(4, ref('var_api'))]"/>
    <field name="python_code"><![CDATA[
client = api_client          # flat name from the connection variable
page = 0
while True:
    resp = client['session'].get('%s/products?page=%d' % (client['url'], page))
    rows = resp.json().get('results', [])
    if not rows:
        break
    yield_data(rows)         # streaming; or accumulate and set value=[...]
    page += 1
]]></field>
</record>
```

---

## 5. Scheduler (v19)

`indaws.import.scheduler` (`_inherit 'indaws.dataflow.scheduler'`):
- `template_id` (→ `indaws.import.template`, required), `data_collector_id` (required),
  **`connections_ids`** (Many2many → **`indaws.dataflow.connection`**, required — note plural),
  `param_ids` (auto-synced from collector), `chunk_size`, `action_import`
  (`no_action`/`import`/`background`).
- inherited: `state` (`draft/running/paused/error`), `flow_type='import'`, `interval_*`/`nextcall`/
  `cron_active` (related to auto-created cron), `execution_mode` (`single`/`micro_batch`),
  `max_errors` (auto-pause), `last_execution_date`.
- Run manually: **`action_execute()`** (sync, with error counting/auto-pause) or
  `action_trigger_now()` (fires the cron). Control: `action_start/action_pause/action_reset`.

```xml
<record id="sched_products" model="indaws.import.scheduler">
    <field name="name">Products Import</field>
    <field name="flow_type">import</field>
    <field name="company_id" ref="base.main_company"/>
    <field name="template_id" ref="tmpl_products"/>
    <field name="data_collector_id" ref="collector_products"/>
    <field name="connections_ids" eval="[(4, ref('conn_api'))]"/>
    <field name="action_import">background</field>
</record>
```

---

## 6. Template + lines (v19)

`indaws.import.template` (`_inherit 'indaws.dataflow.template'`): adds `flow_type` (='import'),
`connection_id`, `group_ids`, `user_ids` from the base. Import fields unchanged in spirit:
`model_id`, `file_type` (`csv`/`txt_separated`/`txt_carrage_return` — **no `xlsx` value**; Excel
detected by extension), `line_ids`, `match_line_ids`, `chunksize` (500), `commit_every`,
`continue_error`, `context_code`, `before_import_code`, `before_create_update_code`,
`finish_import_code`, `code` (after create/update). Lines: `names`, `field_id`, `sequence`,
`match`, `code` (`cell_value` → `value`). Match needs **both** `match=True` on a line AND a
`match_line_ids` record.

---

## 7. Eval context (v19, collector + line code)

Base preloaded (no `wrap_module` needed): `json, open, re, Markup, html2plaintext, plaintext2html,
defaultdict, OrderedDict, Counter, requests, urllib, hashlib, hmac, base64, time, datetime,
dateutil, pytz, format_date, format_amount, _, ValidationError, wrap_module, import_module`.

Import-specific: `env, ctx, UserError, Warning, SkipRow, Command, odoo, log(message, level="info"),
model, record, records`, helper fns (`find_or_create_record`, `to_bool`, `group_by_key`,
`convert_to_dict`, `module_installed`, `url_to_base64`, …).

- **Collector**: `value` (init `[]`), `yield_data(data)`, `last_execution_date`, `scheduler`,
  param names flat, connection variable names flat, (micro-batch) `checkpoint`.
- **Line `code`**: `cell_value` (read), `value` (write), `row`, `rec_import`, `parse_continue`,
  variable names flat, param names flat.
- **`before_import_code`/`finish_import_code`**: `rec_import`, `rec_group`, `value`.

---

## 8. v19 gotchas (import)

1. **`indaws.import.connection` / `indaws.import.variable` are GONE** → use `indaws.dataflow.connection` / `indaws.dataflow.variable`.
2. **`connections_ids` comodel** is now `indaws.dataflow.connection`.
3. **`safe_eval` lost `nocopy`** → never pass `nocopy=True` (TypeError in v19).
4. **`md5_*` → `sha1_*`** on templates and dynamic hash fields.
5. **`defer_compute` removed** from template and import.
6. **Depend on `indaws_dataflow`** (transitively brings queue_job + base).
7. `cell_value` (read) vs `value` (write) in line code — still the #1 bug.
8. Match still needs BOTH `match=True` on the line AND a `match_line_ids` record.
9. Connection params reach the **variable** flat (no `params` dict); the variable's `value` reaches the collector under its `name`.
