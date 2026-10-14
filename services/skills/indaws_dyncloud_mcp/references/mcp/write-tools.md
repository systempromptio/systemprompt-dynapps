# DynCloud MCP — write tools

Four tools that change a customer's live data. Category `data_write`, which `production`
gates behind a JIT grant. Shared conventions: `README.md`.

**Everything here is somebody's business.** Before calling any of them: confirm the
environment is the one they meant, know how many records you are touching
(`orm_search_count`), and prefer the reversible option.

---

## orm_create

```json
{ "company_id": 3, "project_id": 900, "environment_id": 4021,
  "db": "odoo", "target_user_login": "admin",
  "model": "res.partner",
  "vals": { "name": "Aurora Belmonte", "email": "aurora@example.com", "city": "Bilbao" } }
```
```json
{ "ok": true, "data": { "ok": true, "result": { "ids": [9] } } }
```

Notes:

- `vals` is a **single dict**; the tool wraps it for the target. The new id comes back in
  `result.ids`.
- A **binary field takes a value-ref instead of inline base64** —
  `{"__ref__": "attachment", "id": <id>}`, from `file_upload_request`. Use it for anything
  beyond a trivial size, and add `"raw": true` **only** for a field that stores raw bytes
  (in practice `base_import.import.file`). Details and the reasoning: `file-tools.md`.
- Relational fields follow Odoo's own conventions: a `many2one` takes an id, an `x2many`
  takes command tuples. Check with `orm_fields_get` if unsure.

---

## orm_write

```json
{ "model": "res.partner", "ids": [15, 16],
  "vals": { "city": "Bilbao" } }
```

Notes:

- **Count first.** `orm_search` gives you the ids and `orm_search_count` the size; writing
  a domain you have not counted is how a "small fix" becomes 4.000 records.
- Same value-ref support as `orm_create` for binary fields.
- **Archiving beats deleting.** `{"active": false}` is reversible; `orm_unlink` is not.

---

## orm_unlink

```json
{ "model": "res.partner", "ids": [9] }
```

**Destructive and irreversible.** Odoo will refuse when something references the record,
and that refusal is usually right — an error here is more often protection than an
obstacle. Prefer `orm_write {"active": false}` unless the user explicitly asked for a
permanent delete, and say what you are about to remove before you do it.

---

## orm_call_method

Any public business method that has no dedicated tool — `action_confirm`, `button_validate`,
`execute`, a custom method of the customer's own module.

```json
{ "model": "sale.order", "method_name": "action_confirm", "args": [[42]] }
```

Notes:

- `args` is positional, `kwargs` keyword — the same shape the method has in Python. Most
  record methods take the ids as `args: [[id, ...]]`.
- **The target enforces a denylist**: no leading-underscore methods, no canonical ORM
  methods (use their dedicated tools — retrying `search` through here is blocked on
  purpose), and a hardcoded block on things like `browse` and `eval`.
- Use `introspect_list_methods` first when you are not sure the method exists on **that**
  environment.
- This is also how you reach the configuration wizard —
  `res.config.settings` + `execute` (`workflows/writing.md` §4) — and how you run a
  standard import (`workflows/imports.md`).

### The category it lands in

`orm_call_method` is `data_write`, whatever the method does. That has a practical
consequence worth knowing: **calling `button_immediate_uninstall` through here gets the
120 s data-write timeout**, not the 900 s module-admin one, because the category — not the
method — chooses the budget. For anything module-related use the `admin_*` tools
(`admin-tools.md`), which are `module_admin` and get the long budget.
