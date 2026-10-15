# Authoring Tools (`mcp.tool`)

The rest of this skill designs the **exposure** (which models/fields/ops the
positive-list opens). This reference is about the other half: writing a good
*function* — a custom `mcp.tool` whose `python_code` runs on the server. Verified
against `dynapps_mcp_server` v15–v19; the authoring contract is identical across
all five branches (only the output layer differs — see the version note).

## First: do you even need a custom tool?

A custom tool is Python you maintain and that runs privileged-adjacent code. Most
asks don't need one — reach for it only after the declarative path can't do it:

1. **Auto-CRUD** (`allow_read/create/write/unlink` on an exposure) — search, read,
   create, write, delete. Schemas + annotations are generated for you.
2. **Core tools** (`whoami`, `describe_model`, `resolve`, `aggregate`,
   `get_selection`, `list_models`, attachments) — orientation, name→id, grouped
   counts/sums, file bytes. Free, ACL-safe, no code.
3. **Method exposure** (`mcp.method`) — call a vetted model method
   (`action_confirm`…) without writing a wrapper.
4. **Custom tool** — *only* when the result needs computed logic, multi-model
   orchestration, or a shaped payload none of the above produce (e.g. "monthly
   sales KPIs by rep as one object"). The GOD server's own tools
   (the `mcp_god_tools.xml` data file in `dynapps_mcp_server`) are the canonical
   worked examples — read them.

If a request is "search orders", it's auto-CRUD, not a tool. Writing a tool there
adds a maintenance liability and a second code path around the same data.

## Method exposure (`mcp.method`) — before you wrap a method in a tool

"Let the agent confirm an order / validate a picking" is a **method exposure**,
not a custom tool. From `models/mcp_method.py`:

- A catalog row: `model_id` (→ `mcp.model`), `method_name`, `description`,
  `input_schema`, `readonly`, `category`. Unique per `(model, method)`. It shows up
  in `tools/list` as `<model_with_underscores>__<method_name>`
  (e.g. `sale_order__action_confirm`), gated by the exposure's `method_ids`.
- **Hard blacklist** (`_PUBLIC_BLACKLIST`) — cannot be exposed, at create, at
  discovery, and re-checked at invocation: `create/write/unlink/copy/copy_data`,
  `action_unarchive/action_cancel/action_draft`, `button_cancel/button_draft`,
  `_register_hook/init/auto_init`, `_check_credentials/signup/_set_password/`
  `reset_password/action_reset_password`, `message_post/message_subscribe/`
  `message_unsubscribe`. Any `_`-prefixed method is rejected outright. Don't try to
  route around this by calling the primitive from a custom tool — that's the same
  hole with the guard removed.
- **`readonly` is a hint only** — it drives the client's `readOnlyHint` (whether to
  ask the user for confirmation); it enforces nothing server-side. Set it honestly
  so destructive methods prompt for confirmation.
- **Input schema convention** (auto-inferred by Discover, `_discover_candidates`):
  `{"ids": [<int>...], <named params as strings>}`. `ids` selects the recordset
  (omitted ⇒ called in a no-record/class context); each non-defaulted parameter of
  the method signature becomes a required arg. Discover also seeds `readonly`
  (`get_`/`read_`/`search_` prefixes) and `category` (`action_` prefix → action).
- **Invocation** (`_call`) runs as the **real user** (`env(user=user.id)`, never
  sudo), requires a routing server (refuses without one — else it bypasses the
  positive-list), and installs a runtime **field guard**: the method's own
  `read()` calls are bounded to the exposure's readable fields (whitelist set even
  when empty ⇒ **fail-closed**) and masked (`mcp_mask_rules`), under a
  `statement_timeout`. So a method you expose can't read more fields than the
  exposure allows, even internally.

Use a custom tool over a method only when you need to *orchestrate* several calls
or *reshape* the result — not to reach a method the blacklist blocks.

## Anatomy (the fields that matter)

From `models/mcp_tool.py`:

| Field | Role |
|---|---|
| `name` | Technical id, snake_case, unique per server. `orm_` prefix is **reserved** (rejected at create). |
| `description` | **What the LLM reads.** Say WHAT it does, WHEN to use it, WHAT it returns — plus disambiguation ("prefer this over X for monthly aggregates"). This is the single biggest lever on tool-selection quality. |
| `input_schema` | JSON Schema (Draft 2020-12). Validated at save (`_check_input_schema`) **and** on every call before dispatch (`controllers/mcp.py` `tools/call`). |
| `python_code` | The body. Runs via `safe_eval`; must assign `result`. See the sandbox contract below. |
| `category` | `query`/`report`/`action`/`custom` → drives the client annotations (below). |
| `model_id` | If set, injects `Model` into the eval context. Also makes the tool subject to the server's positive-list check for that model in `tools/list`. |
| `group_ids` | Restrict to res.groups (empty = all authenticated users). |
| `cache_ttl_seconds` | `>0` caches the result per **(tool, user_id, args-hash)** — never shared across ACLs. Use for read-only lookups the LLM repeats. `0` for any `action`. |
| `timeout_seconds` | Postgres `statement_timeout` (per SQL statement). `0` = global default (30s). Bounds a runaway *query*, **not** a pure-CPU loop (that's on the gunicorn timeout). |
| `sudo_execution` | Almost never. Gated: writable only by *MCP Sudo Admin*, and honored only if the routing server has `allow_sudo_tools=True`. Defaulting it on breaks the security floor. |

## The `description` contract

The description is the prompt the model uses to pick the tool. A weak one causes
wrong-tool calls and re-prompts (token cost). Pattern (see every GOD tool):

> `<what it does>. <when to use / disambiguation>. Argument: <key facts>. Returns <shape>.`

Idempotency, two-step confirmation, and "does NOT write" belong here too — the
`propose_*`/`apply_*` tools state their contract in the description so the model
sequences them correctly.

## The safe_eval sandbox contract

`python_code` runs in `_get_eval_context`. This is the exact surface — write to it:

**Available**: `env`, `params` (the validated arguments), `user`, `company`,
`uid`, `result` (assign this), `Model` (only if `model_id` set), `_`,
`UserError`, `ValidationError`, `log`, `callable`, safe `getattr`/`hasattr`
(refuse `_`-prefixed), `mcp_list_methods(Model)` (there is no `dir()`), `json`
(`dumps`/`loads` only), `hashlib` (`sha256` only), and the standard Odoo builtins
from `ir.actions` (`time`, `datetime`, `dateutil`, `timezone`, `Command`, …).

**Forbidden** (propose-time AST scan `_scan_proposal_ast` + safe_eval at runtime):
`import`, `exec`/`eval`/`compile`/`open`/`__import__`/`globals`/`locals`/`vars`/
`breakpoint`/`input`; any `_private`/dunder attribute access (except `_helper_*`).
Referencing `env['x.y']` for a model **not in the MCP catalog** is a propose-time
*warning*, not an error.

**Return**: assign `result`. `None`→`""`; `dict`/`list`→JSON; anything else→`str()`.
Return a **dict** (or list) — it becomes `structuredContent` on v19 and is what
TOON/truncation operate on.

## `env`, ACL and the security floor (never sudo)

The tool's `env` is built as `self.env(user=user.id)` — the **real caller, su=False**
— unless `sudo_execution` is on *and* the server allows it. So `ir.model.access`
and `ir.rule` apply inside your code automatically. Consequences:

- Don't reach for `.sudo()` to "make a read work" — that punches through the floor
  the whole product depends on. Fix the caller's ACL instead.
- If your tool touches a model, it should be in the catalog/exposure (or at least
  the caller's ACL must allow it) — the AST warning is telling you this.
- Destructive model primitives are blacklisted from method exposure
  (`create/write/unlink/copy/message_post/_set_password/…`, see `mcp.method._PUBLIC_BLACKLIST`
  and `list_sensitive_fields`). Don't re-open them by calling them from a tool.

## Result-shape conventions (token ergonomics)

The transport layer already caps scalar lists (`m2m_max_items`), encodes TOON when
configured, emits `structuredContent` (v19), and smart-truncates over
`max_result_size`. Your job is to not fight it:

- **Return lean, structured objects.** Mirror the GOD tools: top-level dict with
  the payload plus `warnings`, `errors`, `next_step`, `hint` keys where useful —
  they steer the model's next call.
- **Never inline binaries.** Bytes never go in `result`. Return a value-ref
  (`{"__ref__": "attachment", "id": <id>}`) or issue an out-of-band link
  (`odoo_attachment_link`); big files never enter the context. Inline args are
  hard-capped too — files come *in* via `odoo_request_upload`.
- **Aggregate, don't dump.** For "how many / total / by X", compute with
  `_read_group` in the tool (or tell the model to use `odoo_aggregate`) instead of
  returning rows.

## PII masking (v19 only)

On **v19** the server's mask rules are propagated into the tool env
(`mcp_mask_rules` in context) and applied on the **read path** via
`base_read_guard` → `apply_mask_rules` (`models/_mask.py`), so `read()` /
`search_read()` inside your tool are masked automatically — no cooperation needed.

**Rule shape** (`{"model", "field", "strategy"}`): `model` is an exact Odoo model
or `"*"` (all-models wildcard; an exact-model rule always wins over `*` for the
same field). `field` is a **plain field name — no dotted paths** (`partner_id.email`
is rejected; to hide a value one hop away, put the rule on the *target* model). A
many2one **label** is masked by a rule on the comodel's synthetic `display_name`
field. `create`/`write` **input is never masked** (the LLM authored those vals) —
masking protects *output*, not what the agent writes.

**Five strategies** (`constants.MASK_STRATEGIES`) — pick by what the consumer needs:

| Strategy | Output | Use when |
|---|---|---|
| `redact` | `[REDACTED]` | Value must not appear at all. |
| `partial` | `j***@acme.com`, `+34 ***5678`, `f***t` | A human needs to recognise/verify it. |
| `hash` | `hash-1a2b3c4d` (stable) | Need referential identity (join/dedup) without the value. |
| `fake_consistent` | `apple.river@zinc.example`, `+34 6…` (stable) | Realistic-looking, stable across rows — demos, analytics. |
| `category` | `(email at acme.com)`, `(number)`, `(string len=20)` | Only the *shape* matters. |

`hash`/`fake_consistent` are stable within a DB (salted by
`DYNAPPS_MCP_MASK_SALT` env → `mcp.mask.salt` ICP → random persisted secret) and
**not brute-forceable** from the masked output alone — so use those, not
`redact`/`partial`, when a tool must group or dedup on a PII value.

**The bypass to remember:** `_read_group`, `name_search`, raw SQL, and any value
you format by hand **skip the read guard** and leak the raw value. If your tool
aggregates or name-searches a field that may carry a rule, mask it yourself with
the single-value helpers `strategy_for(rules, model, field)` + `mask_value(...)`
(`_mask.py`) — copy `_core_tools.py` (`_ct_aggregate` *refuses* to group/measure a
masked field; `_ct_resolve` masks the `display_name` label). Masking a field is not
a substitute for not exposing it when in doubt.

On **v15–v18 there is no output masking at all** — do not rely on it; if a field is
sensitive, keep it off the positive-list.

## Authoring safely via the GOD server (propose → validate → apply)

When the AI builds a tool live (not in a data module), it goes through the GOD
meta-tools, which encode the safe lifecycle — reuse it, don't shortcut it:

1. `propose_custom_tool` — runs `_validate_proposal_code` (safe_eval check + AST
   scan + JSON-Schema check), returns a `fingerprint`. Writes nothing.
2. `apply_custom_tool` — re-validates, checks `confirm_token == fingerprint`
   (any byte changed ⇒ mismatch ⇒ refused), creates the tool **`active=False`**.
3. **A human admin activates it** from the UI after reading the code.

The fingerprint is an *integrity* check (proposal unchanged), not authorization —
authorization is GOD access itself, and privileged flags (`allow_sudo_tools`,
`orm_full_access`) are forced off in apply regardless of the proposal. For
irreversible actions, copy the **two-step confirm_token** pattern from
`delete_server` (first call previews + returns a token derived from the target;
second call with the token applies).

## Copyable skeletons

**Read/report tool** (cache on, no writes):
```python
# category='report', cache_ttl_seconds=300, model_id optional
partner = env['res.partner'].browse(params['partner_id']).exists()
if not partner:
    raise UserError(_('Partner %s not found', params['partner_id']))
orders = env['sale.order'].search([('partner_id', '=', partner.id)])
result = {
    'partner': partner.display_name,
    'order_count': len(orders),
    'total': sum(orders.mapped('amount_total')),
    'hint': 'Amounts are in company currency.',
}
```

**Action tool** (idempotent, structured errors):
```python
# category='action', cache_ttl_seconds=0
errors = []
order = env['sale.order'].browse(params['order_id']).exists()
if not order:
    raise UserError(_('Order not found'))
if order.state == 'sale':
    result = {'already_confirmed': True, 'id': order.id}  # idempotent
else:
    order.action_confirm()  # runs as the caller — ir.rule/ACL apply
    result = {'confirmed': True, 'id': order.id, 'state': order.state}
```

## Version note

Authoring contract (fields, sandbox, JSON-Schema validation at save+call, TOON,
`_validate_proposal_code`) is identical v15–v19. Differences that affect a tool:

- **`structuredContent`**: v19 only. On v15–v18 the client gets `content[text]`
  only — the result shape must be legible from the text/description.
  Verificado 2026-08-07 contra `$INDAWS_MCP/mcp{15..19}`: **0 ficheros** lo mencionan en mcp15-18,
  **2** en mcp19.
- **Output PII masking**: v19 only (see above).
  ⚠️ **No lo compruebes grepeando `mask`**: mcp18 tiene 4 ficheros con esa palabra y **no son
  masking de salida** — son *masking de logs* (`_mask_sensitive` en
  `dynapps_mcp_server/controllers/mcp.py:95`, `SENSITIVE_FIELD_NAMES`, "mask sensitive keys in a dict/list tree", con su
  lista en `dynapps_mcp_server/constants.py:31-32` bajo el epígrafe "Audit / log masking"): tapan los params
  **antes de que entren en el log de la request**. El masking de **salida** es v19 y es otro
  mecanismo: `_mask_m2o_labels(self, result, fnames, mask_rules)` en
  `dynapps_mcp_server/models/base_read_guard.py:93`, `BaseMcpReadGuard._mcp_cap_binaries`, con reglas y grupos (`tests/test_mask_seed.py`).
  Confundirlos hace creer que un v18 ya oculta PII al cliente, y **no**: solo la oculta en su log.
- **Core-tools registry** (`_core_tools.py`): **v16+, verificado 2026-08-07** — no existe en
  `mcp15`; sí en `mcp16`…`mcp19` (`dynapps_mcp_server/models/_core_tools.py`). Verify the v15 surface before
  assuming `aggregate`/`attachment_link` exist there.
- **`_ai_read`** (v19 LLM-friendly serialization): **NOT in core — it is Enterprise.** Corrected
  2026-08-07; this said "available in core". Verified: **0 occurrences in all of CE 19**, and it lives
  in EE 19 in three places — `ai/models/models.py:105`, `BaseModel._ai_read`; `ai/models/ir_attachment.py:279`;
  `ai_fields/models/mail_message.py:9`, `MailMessage`. Still **not wired** into the MCP dispatch in any branch, so it
  remains an opportunity rather than a feature — but planning on it means **planning on Enterprise**,
  which for a client on CE is not an opportunity at all. Check the edition before proposing it.

Verify field/method names in the target version against
`common-skills:odoo-version-api-reference` and the indaws-core-developer v18/v19 fixes —
e.g. `ir.model.modules` is a non-stored computed field and cannot appear in a search domain
(worked around in `list_available_models`). Verified 2026-08-07:
`modules = fields.Char(compute='_in_modules', string='In Apps', …)` —
`odoo19/odoo/addons/base/models/ir_model.py:233`, `IrModel.modules`, **sin `store=` y sin `search=`**. Hay un gemelo para
campos en `:562`. Y **no es exclusivo de v19**: el atributo lleva ahí desde antes, así que el
workaround vale para cualquier rama, no solo la 19.

Contraste útil con el caso de al lado: `all_group_ids` **sí** es computed no almacenado y **sí** se
puede filtrar, porque declara `search='_search_all_group_ids'` (`odoo/addons/base/models/res_users.py:259`, `ResUsers.all_group_ids`, v19). La regla no es
"computed no se puede buscar", es **"sin `search=` no se puede buscar"**.

Made with odoo-mcp-builder · v1.0.0
