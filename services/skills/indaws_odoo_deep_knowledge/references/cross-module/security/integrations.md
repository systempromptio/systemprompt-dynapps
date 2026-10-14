# Security — Integrations

How the access-control layer (ACL / `ir.rule` / field groups / groups) interacts with the rest of Odoo. Security is **transversal**: it is not a module you install but a substrate every model rides on. This file maps the seams where security meets web/portal, RPC/MCP, multi-company, mail/followers, automation, and the website public user.

> Source paths under `odoo18/odoo/` unless noted.

---

## 1. The security substrate — who calls what

```
                         ┌────────────────────────────┐
   RPC / web / MCP ─────►│   BaseModel ORM (models.py) │
   (call_kw)             │  check_access / has_access  │
                         │  _apply_ir_rules            │
                         └───────┬─────────┬───────────┘
                                 │         │
            ir.model.access      │         │   ir.rule
            (model+op walls) ◄───┘         └──►(row domains, multi-company)
                                 │         │
                         field.groups (column gating, silent)
                                 │
                         res.groups / res.users (_get_group_ids)
```

Everything funnels through `BaseModel`. There is **no** parallel "API security" — XML-RPC, JSON-RPC, JSON2 (v19), MCP, and the web client all reach the same `check_access`. The only differentiator is *which user* the session authenticates as (and whether code does `sudo()`).

---

## 2. Web client & RPC

- The web client calls `/web/dataset/call_kw` → `BaseModel` methods → full layer stack. **View-level `groups=` is not enforced server-side for data**; a crafted `call_kw` reaches any field the user's `field.groups` allow (SECG-arch §10). Treat menus/views/buttons as cosmetic.
- `read()`/`search_read()` strip inaccessible fields silently (`odoo/models.py:3793`, `BaseModel.check_field_access_rights`). A field hidden in the form via view `groups=` but *not* via Python `field.groups` is still returned by RPC.
- `name_search`/`name_get` honor record rules (they `search` internally), so M2O dropdowns are rule-filtered — but **not** `check_company`-filtered (SECG13).
- `bypass_search_access` (v19, `auto_join`/related in v18): related/`_inherits` reads can skip the comodel's ACL — see §6.

## 3. Portal & website (public user)

- **Portal** = authenticated `base.group_portal`; visibility is 100% record-rule-driven (SECG8). The canonical pattern lives in `portal.mixin`:
  - `_document_check_access(model, document_id, access_token)` validates ownership **or** a signed token *before* any sudo read.
  - Controllers then `sudo()` to render fields the portal user's ACL forbids — safe only because the check ran first (SECG28).
- **Public** = `base.group_public` (website visitor). Website controllers (`http.request.env`) run as this near-zero-rights user; almost every data render is `sudo()` + explicit re-check. A missing token check on a sudo render = customer-data leak by URL guessing.
- `access_token` is a `portal.mixin` field; its presence on a record is what lets an unauthenticated link work. Sharing the URL shares the access (by design) — there is no per-recipient revocation short of regenerating the token.
- Portal record rules commonly use `message_partner_ids child_of [commercial_partner_id]` (mail-follower coupling, see §5) — exposes sibling contacts (SECG14).

## 4. API / MCP / external integrators

- An MCP server or external integration authenticates as a **specific Odoo user**; that user's groups/rules/field-groups define the entire blast radius. Least-privilege = give the integration user a narrow group + tight record rule, **not** admin.
- If integration code runs `sudo()` (common in custom connectors), it bypasses ACL + rules + field groups + multi-company (SECG22) — the integration user's restrictions become decorative. Audit connectors for stray `sudo()`.
- JSON2 (v19) and JSON-RPC (≤v18) hit the same `check_access`; protocol choice does not change security.
- `ir.model.access` and `ir.rule` are themselves models — an integration user with write on them can self-escalate. Never grant `base.group_erp_manager`/`base.group_system` to a service account.

## 5. Mail / followers / chatter

- The **follower-based ownership** pattern couples security to `mail.thread`: portal record rules filter on `message_partner_ids` (the followers M2M). Adding someone as a follower *grants them visibility* through the portal rule — a subtle privilege grant via the chatter "Add followers" button.
- `mail.message` itself has record rules; but message *headers* (model + res_id) can leak object identity (cross-ref mail gotchas). A user who can read a message can infer the existence of the linked record even without record access.
- `message_post` runs with the current user's rights; automated posts often `sudo()` to notify followers the actor can't see — fine for notification, but don't let the posted body include sudo-read restricted data.
- Activities (`mail.activity.mixin`) are gated by the underlying record's rules — you can't see activities on records you can't read.

## 6. Multi-company

- The multi-company wall is **just a global `ir.rule`** on `company_id` (`odoo/addons/base/models/ir_rule.py:48`, `IrRule._eval_context` binds `company_ids = env.companies.ids`). It is therefore subject to every rule gotcha:
  - `sudo()` bypasses it (SECG22) → cross-company leak in custom code.
  - Not auto-created (SECG7).
  - Cache keyed on `allowed_company_ids` (SECG9/SECG20).
- `with_company(c)` / `with_context(allowed_company_ids=[...])` changes which company's rows pass the rule, and busts the rule cache.
- `check_company=True` (`fields.py`) cross-validates related-record company but doesn't filter (SECG13).
- **Branches** (parent/child companies): rules using `company_id in company_ids` include selected branches; reconciliation/consolidation respect only the parent/child tree, not siblings (cross-ref account multi-company gotchas).
- `bypass_search_access` / `auto_join` on M2O can let a user read a related record's *fields* (via the join) even without read access on the comodel — a documented escape hatch; use deliberately (v19 `odoo/orm/fields_relational.py:39`, `class _Relational`).

## 7. Automation & cron

- **Server actions / automations / crons** run as a configured user (`ir.cron.user_id`, action's executing user) — their groups/rules apply. A cron owned by admin runs with admin rights (effectively sudo-like reach).
- `ir.actions.server` Python code frequently uses `env` of the trigger user but may `sudo()` — same leak risk (SECG22).
- Studio automations created by admins inherit admin reach; a low-priv user triggering an admin-owned automation can cause privileged writes (privilege boundary crossing). Audit `user_id` on crons/automations.
- ORM cache (ACL + rule domains) is per-worker; a cron changing groups/rules in one worker won't refresh others until `registry.clear_cache()` propagates (SECG16/SECG9).

## 8. Inheritance & related fields

- `related=` and `_inherits` (delegation) fields read the *target* record. In v18 they may use `auto_join`; in v19 `bypass_search_access=True` is auto-set for `_inherits` fields (`odoo19/odoo/orm/fields_relational.py::_setup_attrs__`), letting the delegated fields read without the parent's ACL. *Effect:* a user can read delegated columns of a comodel they have no ACL on.
- `compute_sudo=True` (default for stored computes, `odoo/fields.py:255`, `Field`) computes across data the user can't see → can surface restricted aggregates in a visible field (SECG23).

## 9. Settings UI, Studio & data files

- The **Settings → Users & Companies → Groups** grid is driven by `category_id` (v18) / `privilege_id` (v19). A group with a "user type" category appears as a radio (one-of); other categories appear as selection/boolean widgets. Mis-categorizing a custom group makes it appear in the wrong widget or not at all.
- **Studio** lets admins toggle ACLs and add field `groups=` from the UI; these become real `ir.model.access` / view records. A Studio change by an admin can silently widen access — review Studio-generated ACLs in audits (cross-ref skill `odoo-studio`).
- Security **data files load order matters**: groups must be defined before ACLs/rules/views that reference them (manifest `'data'` order; security XML/CSV before views). A forward reference to a not-yet-loaded group `ref()` raises at install.
- `ir.model.access.csv` rows and `ir.rule` records are **upgradable data**; rules typically ship in `<data noupdate="1">` to protect admin edits (SECG12 trade-off).

## 10. Common safe patterns (copy-paste sanity)

```python
# Privileged read that MUST stay company-scoped (avoid SECG22):
recs = self.env['my.model'].sudo().with_company(self.env.company).search(
    [('company_id', 'in', self.env.companies.ids), ('state', '=', 'done')])

# Portal controller: check BEFORE sudo (avoid SECG28):
order_sudo = self._document_check_access('sale.order', order_id, access_token)
# ... now render order_sudo fields safely

# Real impersonation / permission test (not sudo):
allowed = self.env['my.model'].with_user(target_user).has_access('write')

# Granting visibility through the chatter is a privilege grant — be intentional:
record.message_subscribe(partner_ids=[partner.id])  # portal rule may now expose it
```

## 11. Cross-references

| Topic | Where |
|-------|-------|
| Portal page/controller security patterns | skill `odoo-portal-developer` (`portal.mixin`, `access_token`, `_document_check_access`) |
| `mail.message` header leakage, follower grants | `functional/mail/gotchas.md`, `functional/mail/integrations.md` |
| Multi-company reconciliation (branches only) | `functional/account/integrations.md` (ACMC1) |
| Spreadsheet/dashboard ACL on source model | `functional/spreadsheet/gotchas.md` (SPRG15, SPRG27) |
| Lock dates / bypass objects (account) | `functional/account/integrations.md` (ACLO1/ACLO4) |
| v19 security renames & consolidation | `cross-module/security/v19_changes.md` |
| Layer mechanics & evaluation order | `cross-module/security/architecture.md` |
