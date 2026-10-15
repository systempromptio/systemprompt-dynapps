# Security — Version Changes (v17 → v18 → v19)

All v19 facts verified against `odoo19/`; v17 against `odoo17/`; v18 baseline `odoo18/`.

---

## Part A — v17 → v18 evolution (so you know the v18 baseline)

v18 **consolidated** the old two-method access API into one family:

| v17 method | v17 source | v18 status |
|------------|-----------|-----------|
| `check_access_rights(op, raise_exception)` | `odoo/models.py:4139`, `BaseModel.fetch` | **merged** → `check_access(op)` / `has_access(op)` |
| `check_access_rule(op)` | `odoo/models.py:4150`, `BaseModel.fetch` | **merged** → `check_access(op)` |
| `_filter_access_rules(op)` (SQL) | `odoo/models.py:4186`, `BaseModel._determine_fields_to_fetch` | deprecated alias → `_filtered_access` |
| `_filter_access_rules_python(op)` | `odoo/models.py:4212`, `BaseModel._fetch_query` | folded into `_filtered_access` |

- v17 required calling **both** `check_access_rights` (ACL) *and* `check_access_rule` (rules) in sequence; v18 does both in one `check_access`.
- `field.is_accessible` is **new in v18** (absent in v17 `fields.py`) — field-group gating became a first-class field method.
- Unchanged v17→v18: `ir_rule._compute_domain` AND/OR logic and ormcache keys (`odoo/addons/base/models/ir_rule.py:137-141`, `IrRule._compute_domain`); `IrModelAccess._get_allowed_models` ormcache `('uid','mode')` (`odoo/addons/base/models/ir_model.py:2129-2130`, `IrModelAccess._get_allowed_models`); `has_group` `group_no_one` debug gate.

**Deprecated/removed in v18** (still present as shims, will warn): `check_access_rights`, `check_access_rule`, `_filter_access_rules`. New code uses `check_access` / `has_access` / `_filtered_access`.

---

## Part B — v18 → v19 (the big refactor)

v19 splits `res_users.py` into `res_groups.py`, `res_users.py`, `res_groups_privilege.py` and introduces a **privilege** layer plus consolidated field-access checks.

### Breaking renames (code & XML)

| v18 | v19 | Source (v19) |
|-----|-----|-------------|
| `res.users.groups_id` | `group_ids` | `odoo/addons/base/models/res_users.py:257`, `class ResUsers`, v19 |
| `res.groups.users` | `user_ids` | `odoo/addons/base/models/res_groups.py:17`, `class ResGroups` |
| `res.groups.trans_implied_ids` | `all_implied_ids` (+ `implied_by_ids`, `all_implied_by_ids`) | `odoo/addons/base/models/res_groups.py:71`, `class ResGroups`, v19 (`implied_by_ids` en `:74-75`, `all_implied_by_ids` en `:76-77`) |
| `field.is_accessible(env)` | `model._has_field_access(field, op)` / `_check_field_access(field, op)` | `odoo/orm/models.py:3369, 3384`, `BaseModel` |
| `ir.rule._compute_domain` → list | → **`Domain` object** | `odoo/addons/base/models/ir_rule.py:141`, `class IrRule` (`-> Domain`, `Domain.AND(...).optimize(model)` :154) |
| rule group check `groups_id` | `all_group_ids` | `odoo/addons/base/models/ir_rule.py:158`, `IrRule._compute_domain` (`self.env.user.all_group_ids`) |
| `_sql_constraints` | `models.Constraint(...)`; uniqueness now `(privilege_id, name)` | `odoo/addons/base/models/res_groups.py:39`, `class ResGroups`, v19 |
| `expression.AND/OR` | `Domain.AND/OR` / `&` `|` | `odoo/addons/base/models/ir_rule.py:154`, `IrRule._compute_domain` |
| `auto_join=True` | `bypass_search_access=True` (auto for `_inherits`) | `orm/fields_relational.py:39`, `_Relational.bypass_search_access` y `orm/fields_relational.py:259`, `Many2one._setup_attrs__` |

### New models / fields

- **`res.groups.privilege`** (new model) — groups organize under a privilege via `privilege_id` Many2one (`odoo/addons/base/models/res_groups.py:36`, `class ResGroups`). Privilege fields: `name` (`odoo/addons/base/models/res_groups_privilege.py:9`, `class ResGroupsPrivilege`), `sequence` (`:12`), `category_id` (`:13`). This **replaces `category_id` as the grouping axis** in the Settings UI (category_id still exists on the privilege, one level up).
- **`all_group_ids`** — computed M2M on `res.users` = explicit groups + all transitively implied; used everywhere for access checks (`odoo/addons/base/models/res_users.py:258`, `Groups._search`, `compute_sudo=True`).
- **`role`** — Selection on `res.users`: `('group_user','User'), ('group_system','Administrator')`, UI-friendly, auto-manages `group_ids` (`odoo/addons/base/models/res_users.py:272`, `ResUsers.role`, v19).
- **`disjoint_ids`** — computed M2M on `res.groups` enforcing mutual exclusion (replaces the old `_check_one_user_type` constraint mechanics); built from the user-type groups (`odoo/addons/base/models/res_groups.py:78`, `ResGroups.disjoint_ids` y `odoo/addons/base/models/res_groups.py:100-106`, `ResGroups._check_user_disjoint_groups`).
- **`_has_field_access(field, op)`** (bool, soft) + **`_check_field_access(field, op)`** (raises, hard) — operation-aware (read vs write), replacing the single `is_accessible` (`odoo/orm/models.py:3369, 3384`, `BaseModel`).
- **`bypass_search_access`** field param — read/search related records without the comodel's access check; auto-`True` for `_inherits` fields (`orm/fields_relational.py:39`, `_Relational.bypass_search_access` y `orm/fields_relational.py:259`, `Many2one._setup_attrs__`).

### Unchanged in v19 (don't assume it moved)

- **`ir.model.access` still uses a single `group_id` Many2one** (`odoo19/odoo/addons/base/models/ir_model.py::group_id`) — there is no `group_ids` on ACL. Privilege is only joined for **display** in access reports (`odoo/addons/base/models/ir_model.py:2101`, `IrModelAccess.group_names_with_access`, v19); the access check still resolves via `group_id` (`odoo/addons/base/models/ir_model.py:2126`, `IrModelAccess._get_access_groups`, v19).
- ACL `check()` / `_get_allowed_models` ormcache approach is conceptually the same.
- The global=AND / group=OR rule combination logic survives (now in `Domain` form).

### Group display-name changes (v19)

`"Settings"` → `"Role / Administrator"`, `"Internal User"` → `"Role / User"`, etc. — cosmetic but affects scripts/tests that match group names as strings.

---

## v19 Migration Checklist

- [ ] `user.groups_id` → `user.group_ids`
- [ ] For "effective groups incl. implied" use `user.all_group_ids` (not `group_ids`)
- [ ] `groups.users` → `groups.user_ids` (Python + XML `<field name="users">`)
- [ ] `trans_implied_ids` → `all_implied_ids`
- [ ] `field.is_accessible(env)` → `model._has_field_access(field, op)` / `_check_field_access`
- [ ] `ir.rule._compute_domain()` return type: list → `Domain` (and any code wrapping it)
- [ ] `expression.AND/OR` in rule code → `Domain.AND/OR` / `&` `|`
- [ ] `_sql_constraints` → `models.Constraint(...)`
- [ ] group uniqueness: category-based → `(privilege_id, name)`
- [ ] rule group checks: `groups_id` → `all_group_ids`
- [ ] handle the `disjoint_ids` user-type enforcement (no longer `_check_one_user_type`)
- [ ] review `auto_join` → `bypass_search_access`; check `_inherits` fields' auto-bypass
- [ ] add/assign `privilege_id` on custom groups (replaces `category_id` grouping)
- [ ] consider `role` Selection for simple user-type assignment in data files
- [ ] tests matching group display names: update for `"Role / ..."` renames
- [ ] **No change needed** for `ir.model.access` `group_id` — still single M2O
