<!-- verified: v18.0 @ 2026-06 -->
# Security Architecture — Odoo 18.0

Odoo access control is a **layered, AND-stacked** system. A user action passes only if **every** layer allows it. Layers, in order of evaluation during an ORM operation:

1. **Model ACL** (`ir.model.access`) — can this group touch this model at all?
2. **Record Rules** (`ir.rule`) — which *rows* of that model can this user see/touch?
3. **Field-level groups** (`field.groups`) — which *columns* can this user see/write?
4. **Menus / actions / views** (`groups` attribute) — UI gating only, NOT a security boundary.

`sudo()` (`env.su = True`) bypasses layers 1, 2 and 3 (NOT 4 — there is no UI in sudo). `with_user(u)` swaps the acting user and re-evaluates all layers as that user.

> **Source map (v18)**: ACL = `addons/base/models/ir_model.py` (class `IrModelAccess`); record rules = `addons/base/models/ir_rule.py`; groups/users = `addons/base/models/res_users.py`; ORM enforcement = `odoo/models.py`; field gating = `odoo/fields.py`; data = `addons/base/security/` (`base_groups.xml`, `base_security.xml`, `ir.model.access.csv`).

---

## 1. The Four Layers — How They Stack

```
User performs an ORM operation (read / write / create / unlink)
  │
  ├─ env.su == True ?  ──YES──►  bypass layers 1-3, proceed
  │                    (models.py:4443 check_access skips when self.env.su)
  ▼ NO
Layer 1: MODEL ACL  (ir.model.access.check)
  → Is model in user's _get_allowed_models(mode)?   ir_model.py:2156
  → NO → AccessError (hard block, "not allowed to <op>")  ir_model.py:2161
  ▼ YES
Layer 2: RECORD RULES  (ir.rule._compute_domain → injected into SQL)
  → On search/read: domain AND-ed into the SQL query   models.py:5595-5597
  → On write/unlink/create: rows re-checked, forbidden subset → AccessError
  → ir_rule.py:203 _make_access_error ("top-secret records")
  ▼ rows that match
Layer 3: FIELD ACCESS  (field.is_accessible / check_field_access_rights)
  → field has groups= and user not in them?            models.py:3793, fields.py:962
  → fields NOT named (read([])/plain view): stripped silently  models.py:3792-3793
  → field EXPLICITLY named (read(['f'])/domain/write): AccessError  models.py:3796-3837
  ▼
Operation proceeds
```

**Key asymmetry**: Layers 1 and 2 always *raise* `AccessError`. Layer 3 is **silent ONLY when the field isn't named** — `read([])` / an ordinary form view just drops inaccessible fields, but explicitly naming one (`read(['restricted'])`, filtering/searching on it, or writing it) **raises AccessError** (`odoo/models.py:3837`, `BaseModel.check_field_access_rights`). The silent case is the #1 source of "my data is missing but no error" bugs (see SECG3).

**Evaluation order within a search**: ACL is checked first (`check_access('read')`), then record rules are *compiled into the WHERE clause* (`_apply_ir_rules`, models.py:5584), so denied rows are never fetched — searching returns fewer records rather than erroring. Direct `browse(id).read()` of a rule-denied id *does* raise.

---

## 2. Groups System (`res.groups`)

### Hierarchy via `implied_ids`

A group that "implies" another grants all of that group's rights. Membership is **materialized**, not computed (see §2.3).

```
base.group_system (Settings / Administration)
  └─ implies → base.group_erp_manager (Access Rights)
        └─ implies → base.group_user (Internal User)
              └─ (base.group_no_one — Technical, debug-only, see SECG5)

base.group_portal  (share=True, no implications)
base.group_public  (share=True, no implications)
```

### Key Fields (res_users.py)

| Field | Line | Purpose |
|-------|------|---------|
| `name` | — | Display name |
| `category_id` | res_users.py:191 | Application category (`ir.module.category`); drives the Settings UI grid. **v19: superseded by `privilege_id`.** |
| `implied_ids` | res_users.py:1463 | Parent groups; members auto-get all implied groups |
| `trans_implied_ids` | res_users.py:1465 | Recursive closure of `implied_ids` (`recursive=True`). **v19: renamed `all_implied_ids`.** |
| `users` | res_users.py:184 | M2M to `res.users` (`res_groups_users_rel`). **v19: renamed `user_ids`.** |
| `share` | res_users.py:194 | True for portal/public groups |
| `color` | res_users.py:192 | Kanban color |

### Materialized implied membership (CRITICAL)

When you add a user to group A that implies B, Odoo runs a **recursive SQL CTE** that inserts real rows into `res_groups_users_rel` for B (and all transitively implied groups) — these are physical rows, not computed on the fly:

```sql
-- res_users.py:1494-1514 (write override on implied_ids / membership)
WITH RECURSIVE group_imply(gid, hid) AS ( ... )
INSERT INTO res_groups_users_rel (gid, uid)
SELECT ... EXCEPT SELECT ...   -- only missing pairs
```

Consequence (SECG4): removing a user from group A does **not** remove the implied B rows if some other path also implies B — they were inserted as independent rows.

### `_get_group_ids()` — the canonical resolution

```python
# res_users.py:1224  @tools.ormcache('self.id')
def _get_group_ids(self):
    return self.groups_id._ids   # already includes materialized implied groups
```

Both ACL (`odoo/addons/base/models/ir_model.py:2128`, `IrModelAccess`) and record rules (`odoo/addons/base/models/ir_rule.py:131`, `IrRule._get_rules`) call `user._get_group_ids()`, so implied groups are honored everywhere. Cached per user id.

### Group set-algebra registry (`SetDefinitions`) — v18

v18 added a cached registry that models all groups as a set algebra:

```python
# res_users.py:1565  @tools.ormcache(cache='groups')
def _get_group_definitions(self):
    return ...  # a tools.SetDefinitions over every res.groups (with .empty/.universe/.from_ids/.get_id)
```

- `_has_group` resolves an ext-id to a group id via `_get_group_definitions().get_id(ext_id)` (res_users.py:1220).
- ACL's `_get_access_groups(model, mode)` returns a group-expression: `empty` if no ACL row, `universe` if any row has no group (global access), else `from_ids(group_ids)` (ir_model.py:2100-2117).
- This registry uses a **dedicated `'groups'` ormcache bucket**, cleared on `res.groups` create/write/unlink and on `implied_ids` changes (res_users.py:1484, 1531, 1536) — separate from the per-uid ACL/rule caches. Raw `res_groups_users_rel` reads can lag the registry until cleared (SECG34).

### User Types (Mutually Exclusive)

| Type | Group | Method | share |
|------|-------|--------|-------|
| Internal | `base.group_user` | `_is_internal()` res_users.py:1289 | False |
| Portal | `base.group_portal` | `_is_portal()` res_users.py:1293 | True |
| Public | `base.group_public` | `_is_public()` res_users.py:1297 | True |

Enforced by `@api.constrains('groups_id') _check_one_user_type()` (res_users.py:616) — a user may belong to **at most one** group in the `base.module_category_user_type` category. `share` is computed: internal → False, everyone else → True (`_compute_share`, res_users.py:530). **v19** enforces this via a `disjoint_ids` mechanism instead.

### `has_group` / `has_groups`

```python
user.has_group('sale.group_sale_manager')      # bool, single
user.has_groups('sale.group_sale_salesman,!sale.group_sale_manager')
#   → salesman AND NOT manager   (',' = OR-list, '!' = negate one term)
user.has_groups('.')                # always False
user.has_group('base.group_no_one') # True ONLY when request.session.debug
```

- CSV parse + `!` negation: res_users.py:1160-1189. **Negatives checked first** (`:1184-1185`); a spec with only `!`-terms and no positive returns `True` when none match (`return not positives`, `:1189` — SECG35).
- `'.'` → False (`:1172`); `base.group_no_one` debug gate: res_users.py:1207-1208 (`result = result and bool(request and request.session.debug)`)
- **Caller guard, not a result short-circuit** (`:1201-1204`): `has_group` raises `AccessError("You can ony call user.has_group() with your current user.")` if `self` is not the current user, unless `env.su` or the caller is internal (`base.group_user`). The real test is `_has_group` (`:1211-1222`), which resolves the group id via `_get_group_definitions().get_id(ext_id)` (`:1220`) — the v18 group registry, see §2.x — and checks `_get_group_ids()`. To query another user's membership from privileged code use `user.sudo().has_group(...)` (SECG27).

---

## 3. Model-Level ACL (`ir.model.access`)

### CSV Format

```csv
id,name,model_id:id,group_id:id,perm_read,perm_write,perm_create,perm_unlink
access_sale_order_user,sale.order user,model_sale_order,sales_team.group_sale_salesman,1,1,1,0
access_sale_order_manager,sale.order mgr,model_sale_order,sales_team.group_sale_manager,1,1,1,1
access_sale_order_portal,sale.order portal,model_sale_order,base.group_portal,1,0,0,0
```

Permission fields: `perm_read/perm_write/perm_create/perm_unlink` (ir_model.py:2073-2076).

### How `check()` works (ir_model.py)

```python
def check(self, model, mode='read', raise_exception=True):   # ir_model.py:2146
    if self.env.su:                 # ir_model.py:2148
        return True                 # root has every access
    has_access = model in self._get_allowed_models(mode)   # ir_model.py:2156
    if not has_access and raise_exception:
        raise self._make_access_error(model, mode)         # ir_model.py:2161

@tools.ormcache('self.env.uid', 'mode')        # ir_model.py:2124
def _get_allowed_models(self, mode='read'):
    group_ids = self.env.user._get_group_ids()             # ir_model.py:2128
    # SQL: SELECT model FROM ir_model_access a JOIN ir_model m ...
    #      WHERE a.perm_<mode> AND a.active
    #        AND (a.group_id IS NULL OR a.group_id IN %s)   # ir_model.py:2130-2141
```

### Rules of the ACL layer

- **No access record for a model → DENIED** (secure by default — SECG2).
- **`group_id IS NULL` (empty) → public/global access** to all users incl. portal/public (deprecated; logs `"Rule %s has no group..."` at ir_model.py:2197 — SECG10).
- **Multiple records for same model+group → OR-ed** (any record granting the perm is enough). The SQL `GROUP BY m.model` collapses them.
- Manager implies user → manager inherits user's ACL rows too (via `_get_group_ids`).
- Cache invalidation: any create/write/unlink on `ir.model.access` calls `call_cache_clearing_methods()`, which does **both** `env.invalidate_all()` and `env.registry.clear_cache()` (ir_model.py:2181-2183), called from create/write/unlink at 2190/2201/2205.
- v18 also exposes `_get_access_groups(model, mode)` (ir_model.py:2100-2117, `@ormcache('model_name','access_mode')`): returns the *group set* allowed for an operation as a `SetDefinitions` expression (`empty` / `universe` / `from_ids`) — the algebraic counterpart of the per-uid `_get_allowed_models`.

> ACL is **model-wide and operation-wide** — it cannot restrict by row or by field. That is the job of layers 2 and 3.

---

## 4. Record-Level Rules (`ir.rule`)

### Structure

```xml
<record model="ir.rule" id="sale_order_comp_rule">
    <field name="name">Sales Order multi-company</field>
    <field name="model_id" ref="model_sale_order"/>
    <field name="domain_force">[('company_id', 'in', company_ids)]</field>
    <!-- NO groups → GLOBAL rule -->
</record>

<record model="ir.rule" id="sale_order_personal_rule">
    <field name="name">Personal Orders</field>
    <field name="model_id" ref="model_sale_order"/>
    <field name="domain_force">['|',('user_id','=',user.id),('user_id','=',False)]</field>
    <field name="groups" eval="[(4, ref('sales_team.group_sale_salesman'))]"/>
</record>
```

`domain_force` is `fields.Text` (ir_rule.py:23). The computed `global` flag is simply `not rule.groups` (`_compute_global`, ir_rule.py:52-55; the field is exposed as `global_` to dodge the Python keyword, ir_rule.py:271).

### Per-operation rules

`ir.rule` has its own `perm_read/perm_write/perm_create/perm_unlink`, all **default True** (ir_rule.py:24-27). A rule with only `perm_read=True` restricts reads but leaves writes unrestricted. A SQL CHECK forces at least one perm true (ir_rule.py:29-33). So a single model can have different visibility for read vs write.

### Combination Logic (THE core fact)

```
Final domain = AND(all global rules) AND OR(matching group rules)
```

```python
# ir_rule.py:141-168 _compute_domain
for rule in rules:
    if rule.global:        global_domains.append(dom)     # :161
    else (user in groups): group_domains.append(dom)      # :163
if not group_domains:
    return expression.AND(global_domains)                 # :167
return expression.AND(global_domains + [expression.OR(group_domains)])  # :168
```

| Rule kind | No groups (global) | With groups |
|-----------|--------------------|-------------|
| Combination | **AND** — every one must match | **OR** — any one matching grants |
| Applies to | ALL users | Only users in those groups |
| Use for | Mandatory walls (multi-company) | Graduated access (mine vs all) |

**Practical implications (SECG1):**
- Adding a **global** rule is *always restrictive* — it intersects with everything.
- Adding a **group** rule is *permissive within that group* — it widens what those users see (OR with other group rules).
- A global rule **cannot be widened** by a group rule. If a global rule excludes a row, no group rule brings it back. (Common lockout cause.)
- A group rule only applies to a user if the user is in that group; otherwise it's invisible to them (and contributes nothing to their OR set → if they have *no* matching group rule and *no* global rule, they see all rows of that model that pass ACL).

### Delegated (`_inherits`) parents are folded in (SECG36)

Before collecting the model's own rules, `_compute_domain` walks `self.env[model]._inherits` and AND-s each delegated parent's domain into the global set as `[(parent_field, 'any', parent_domain)]` (ir_rule.py:143-146). So `res.users` is also bound by `res.partner` rules, etc. A child model can "lose" rows because of a *parent's* rule that isn't visible in the child's own rule list.

### Domain evaluation context (ir_rule.py:45-50)

```python
{
  'user': self.env.user.with_context({}),   # :46  (empty ctx — avoids recursion)
  'time': time,                              # :47
  'company_ids': self.env.companies.ids,     # :48  switched (allowed) companies
  'company_id': self.env.company.id,         # :49  active company
}
```

Note `company_ids` = the **switched** company set (multi-company selector), not `user.company_ids`. This is why multi-company rules use `company_ids`, not the user field.

### Rule selection (ir_rule.py:112-132)

`_get_rules` selects rules where `model = %s AND active AND perm_<mode> AND (global OR id IN <rules whose group_id IN user's groups>)`, filtering by `self.env.user._get_group_ids()` (ir_rule.py:131) — so implied groups count here too. Note `_get_rules` returns empty under `env.su` (ir_rule.py:119-120). Subtle: the SQL pre-selection uses `_get_group_ids()` (the ormcached materialized set), but the in-Python "does this group rule apply" test inside `_compute_domain` uses `rule.groups & self.env.user.groups_id` (ir_rule.py:154, 162) — two different membership paths that can momentarily disagree right after a programmatic group change (SECG37).

### Caching (ir_rule.py:137)

```python
@tools.ormcache('self.env.uid', 'self.env.su', 'model_name', 'mode',
                'tuple(self._compute_domain_context_values())')
```

- Cache key = **uid + su + model + mode + allowed_company_ids** (the only dynamic context value, ir_rule.py:170-178).
- Any create/write/unlink on `ir.rule` calls `env.registry.clear_cache()` (ir_rule.py:182/190/200).
- In `--dev xml`, the ormcache is conditionally disabled so rules reload each call.
- `_make_access_error` logs and raises the "top-secret records" message (ir_rule.py:203-217).

### Common domain patterns

```python
[('company_id', 'in', company_ids)]                          # multi-company (global)
['|', ('user_id', '=', user.id), ('user_id', '=', False)]    # personal (group)
[(1, '=', 1)]                                                # see-all override (manager group)
[('message_partner_ids', 'child_of', [user.commercial_partner_id.id])]  # portal
[('create_uid', '=', user.id)]                               # creator-only (wizards)
```

---

## 5. Field-Level Access (`field.groups`)

```python
margin       = fields.Float(groups='sale.group_sale_manager')
credit_limit = fields.Float(groups='account.group_account_invoice,account.group_account_readonly')  # OR
technical    = fields.Char(groups='base.group_no_one')   # debug-only
secret       = fields.Char(groups='.')                   # NO ONE (only sudo)
```

```python
# fields.py:962 is_accessible(self, env)
#   no groups       → True (everyone)
#   groups == '.'   → False (nobody, even admin)
#   env.is_superuser→ True
#   else            → env.user.has_groups(self.groups)   # CSV OR
# models.py:3793 check_field_access_rights strips inaccessible fields from the read set
```

**Behavior when not accessible** (`check_field_access_rights`, models.py:3777):
- **fields NOT named** (`read([])`, plain view) → field omitted from result (silent). `sudo().read([...])` reveals it. Write of an unnamed field can't happen, but values for inaccessible keys are dropped.
- **field EXPLICITLY named** (`read(['f'])`, `search([('f',…)])`, `write({'f':…})`) → builds `invalid_fields` and **raises `AccessError`** (models.py:3796-3837; the message lists the allowed groups when in debug/`group_no_one`, :3814-3835).
- So it is **NOT "never raises"** (correcting the old mental model) — silence is conditional on not naming the field (SECG3).

Field groups gate **storage access**, not just the form widget — the value never reaches SQL on write and is never returned on read. `groups='.'` (SECG15) is stricter than `base.group_system`: only `env.su` passes, useful for tokens/hashes.

---

## 6. `sudo()`, `with_user()`, `with_company()`

### `sudo()`

```python
records.sudo()        # env.su = True → bypass ACL + record rules + field groups
records.sudo(False)   # restore normal enforcement
```

- Sets `env.su = True`. **Does NOT change `env.uid`** — `self.env.user` still returns the real user (SECG6). To genuinely act as someone else, use `with_user`.
- Affects the *whole chain* of related records read through the recordset (sudo is sticky on the env).

### `with_user()`

```python
records.with_user(other_user)   # env.uid = other_user, su = False
```

Sets `env(user=user, su=False)` (models.py:6321-6330), so it **always resets su to False** — `recs.sudo().with_user(u)` is no longer sudo. The one exception: `with_user(SUPERUSER_ID)`/`base.user_root` stays in su because `Environment.__new__` forces `su=True` for the superuser uid (api.py:570-573). Re-evaluates every layer as that user. Right tool for impersonation/permission testing (SECG31).

### `with_company()` / `with_context(allowed_company_ids=...)`

Changes `env.company` / `env.companies`, which drives the `company_id`/`company_ids` in record-rule domains (ir_rule.py:48-49). `with_company` does NOT bypass anything — it changes which company-scoped rows are visible.

| Pattern | When |
|---------|------|
| `sudo()` | System automation, cross-company record creation, reading restricted tech models in code |
| `sudo(False)` | Re-enable checks inside a sudo block before user-facing work |
| `with_user(u)` | Impersonation, permission tests |
| `with_company(c)` | Operate scoped to a specific company's rules |
| **Avoid** `sudo()` | Any path driven by untrusted/portal input — it disables all walls |

---

## 7. ORM Enforcement Entry Points (v18 unified API)

v18 consolidated the old v17 pair (`check_access_rights` + `check_access_rule`) into one family:

| Method | Returns | Raises | Source |
|--------|---------|--------|--------|
| `check_access(op)` | None | AccessError | models.py:4429 (skips if `env.su`, :4443) |
| `has_access(op)` | bool | never | models.py:4446 (`env.su or not _check_access`, :4451) |
| `_filtered_access(op)` | recordset subset | never | models.py:4453 (`self - result[0]`, :4461) |
| `_apply_ir_rules(query, mode)` | (mutates query) | — | models.py:5584 (injects rule domain into SQL, :5595-5597) |
| `check_field_access_rights(op, fields)` | filtered field list | — | models.py:3777 |

```python
records.check_access('write')              # hard: raises if denied
if records.has_access('unlink'):           # soft: bool
    records.unlink()
readable = records._filtered_access('read') # subset that passes
```

The v17 trio is **deprecated since 18.0** (all warn + delegate): `check_access_rights(op, raise_exception)` → `check_access`/`has_access` (models.py:4490-4505), `check_access_rule(op)` → `check_access` (:4507-4518), `_filter_access_rules(op)`/`_filter_access_rules_python(op)` → `_filtered_access` (:4520-4533). The single overridable core is `_check_access(op)` (models.py:4464-4487): it calls ACL `check(...)` first, then (only on real ids, `if any(self._ids)`) `ir.rule._compute_domain` + `filtered_domain`, returning `(forbidden_records, error_factory)` or `None`. All public methods honor `env.su` as a short-circuit.

---

## 8. Multi-Company

### User & env fields

```python
user.company_id    # default company (M2O, required)
user.company_ids   # allowed companies (M2M)
env.company        # active company (from context)
env.companies      # switched companies (allowed_company_ids in context)
```

**`allowed_company_ids` is validated — but only outside sudo** (api.py:694-762). `env.company`/`env.companies` read the context key and, when `not env.su`, raise `AccessError("Access to unauthorized or invalid companies.")` if any id is outside `user._get_company_ids()` (api.py:717-720, 748-750). **Under `sudo()` there are NO checks** — any company id is accepted (docstrings: "No sanity checks applied in sudo mode!", :705-713, :735-743) → a sudo block that trusts a context company is an inter-company write vector (SECG32). And when the key is **absent**, `env.companies` defaults to *all* the user's companies (api.py:752-762), `env.company` to `user.company_id` (:722) — so cron/report/`/web/image` paths run multi-company-wide unless you `with_company(...)` (SECG33). Also: `Environment.__new__` forces `su=True` for `uid==SUPERUSER_ID` (api.py:570-573), and `with_user` drops su to False otherwise (api.py:649) — SECG31.

### How it plugs into security

The standard multi-company **global** rule:

```xml
<record model="ir.rule" id="model_comp_rule">
    <field name="model_id" ref="model_my_model"/>
    <field name="domain_force">[('company_id', 'in', company_ids)]</field>
</record>
```

`company_ids` here resolves to `env.companies.ids` (ir_rule.py:48) = the **switched** set, so toggling companies in the top-right selector changes visibility live (and busts the rule cache, since `allowed_company_ids` is the dynamic cache key).

**Having `company_id` on a model does NOT auto-create this rule** (SECG7) — you must ship it, or users see all companies' rows.

### `check_company=True`

```python
partner_id = fields.Many2one('res.partner', check_company=True)
```

Validates that the related record's company is compatible with the record's/user's companies and **raises on mismatch** — but does **not** filter dropdowns (SECG13). For UI filtering you still add a `[('company_id', 'in', allowed_company_ids)]` domain or rely on the comodel's own record rule.

### Branches (v17+)

`company_id` can be a branch (child company). Multi-company rules using `company_id in company_ids` naturally include selected branches; reconciliation/consolidation respect the parent/child tree only (not sibling companies).

---

## 9. Portal & Public

### Portal access pattern

```csv
# ACL: read-only on the model
access_sale_order_portal,sale.order portal,model_sale_order,base.group_portal,1,0,0,0
```

```xml
<!-- Record rule: own documents only -->
<record model="ir.rule" id="sale_order_rule_portal">
    <field name="domain_force">[('message_partner_ids','child_of',[user.commercial_partner_id.id])]</field>
    <field name="groups" eval="[(4, ref('base.group_portal'))]"/>
</record>
```

**Portal visibility is 100% record-rule-driven.** A portal ACL of `perm_read=1` with **no** record rule means the portal user reads **every** record of that model (SECG8). The `child_of [commercial_partner_id]` pattern (SECG14) exposes documents followed by *any* contact under the same company — use `=` for strict per-contact.

| Aspect | Internal | Portal |
|--------|----------|--------|
| Group | `base.group_user` | `base.group_portal` |
| share | False | True |
| Menus | full (per group) | portal menus only |
| ACL | broad RW | read-only on allowed models |
| Record rules | by role/team | by ownership (follower/partner) |
| Field groups | all | many hidden |

Public (`base.group_public`) is the website visitor — even narrower; controllers usually `sudo()` deliberately and re-check via `access_token`.

---

## 10. Menus, Actions & Views (UI gating — NOT security)

`<menuitem groups="...">`, `<record model="ir.actions.act_window"><field name="groups_id">`, and `<field name="x" groups="..."/>` in views hide UI elements but are **not a security boundary** — a user can still hit the data via RPC/`/web/dataset/call_kw`. Real protection always lives in layers 1-3. View `groups=` on a field is a convenience that mirrors (but does not replace) `field.groups` on the Python field.

---

## 11. Developer Checklist — Security for a New Module

```
1. GROUPS  (security/{module}_security.xml)
   - User group   → implies base.group_user
   - Manager group→ implies the user group
   - Add admin to manager group; set category_id

2. ACL  (security/ir.model.access.csv)
   - User:    read+write+create (no unlink)
   - Manager: read+write+create+unlink
   - Portal:  read only (if applicable)
   - Related models your model reads: at least read for the relevant groups

3. RECORD RULES  (security/{module}_security.xml, inside <data noupdate="1">)
   - Global multi-company:  [('company_id','in',company_ids)]
   - Personal (user group): ['|',('user_id','=',user.id),('user_id','=',False)]
   - See-all (manager group): [(1,'=',1)]
   - Portal (if applicable): partner-based domain

4. FIELD GROUPS  (models/*.py)
   - Sensitive:  groups='module.group_manager'
   - Technical:  groups='base.group_no_one'
   - Secret:     groups='.'   (only sudo)

5. MANIFEST  ('data': [...])
   - Security CSV/XML listed BEFORE views (groups must exist first)

6. TEST  (use with_user, not sudo)
   - user → can RW/create, not unlink
   - manager → can unlink
   - portal → only own docs
   - switch company → only that company's rows
```

---

## 12. Debugging Access Problems — Decision Guide

When something is wrong, identify **which layer** first; the symptom tells you:

| Symptom | Likely layer | Confirm with | Typical fix |
|---------|--------------|--------------|-------------|
| `AccessError: not allowed to <op>` | Layer 1 (ACL) | `env['ir.model.access'].check(model, op, False)` | add/adjust `ir.model.access.csv` row |
| `AccessError: top-secret records` | Layer 2 (rules) | `recs._filtered_access(op)` vs `recs` | fix `domain_force`; check global vs group |
| Field missing in form/RPC, **no error** | Layer 3 (field) | `rec.sudo().read([f])` returns it | adjust Python `field.groups` |
| `search()` returns too few rows | Layer 2 (rules) | `model.sudo().search(d)` returns more | rule too tight, or no see-all rule for group |
| User sees rows from other companies | missing global rule / stray sudo | `model.search([])` count per company | add multi-company rule; remove `sudo()` |
| Rule edit "doesn't apply" | cache | `env.registry.clear_cache()` | clear cache / restart; `--dev xml` to iterate |
| Button hidden but RPC works | Layer 4 (UI only) | call `call_kw` directly | add real ACL/rule — view `groups=` is not a wall |

### Quick diagnostic snippet (run as the affected user)

```python
u = env['res.users'].browse(USER_ID)
recs = env['my.model'].with_user(u)
recs.has_access('read')                 # ACL+rule pass for the set?
recs.browse(ID)._filtered_access('read') # is this row rule-allowed?
env['ir.model.access'].with_user(u).check('my.model', 'read', False)  # ACL only
env['ir.rule']._compute_domain('my.model', 'read')   # the effective row domain
u._get_group_ids()                       # effective groups incl. implied
```

---

## 13. Mental Model — One-Liners to Remember

- **ACL** = *can this group touch this model at all?* (model+operation, no rows, no fields).
- **ir.rule** = *which rows?* (global = AND wall, group = OR widening).
- **field.groups** = *which columns?* (silent when the field isn't named; **raises** if you name/search/write it).
- **menus/views** = *cosmetics* (never trust for security).
- **sudo** = *turn off layers 1-3* (keeps uid). **with_user** = *become someone* (keeps layers).
- **No rule for a model** → user sees all rows that pass ACL. **No ACL** → user sees nothing.
- **Global rule wins** — a group rule can never re-grant what a global rule denies.
