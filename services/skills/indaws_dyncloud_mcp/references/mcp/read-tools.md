# DynCloud MCP — read tools

Reading data and schema on the target Odoo. All of these are proxied, all require the five
location args, and all run as `target_user_login`. Shared conventions: `README.md`.

Category: `data_read` for the `orm_*` readers, `schema` for `introspect_*` and
`orm_fields_get`. On most stages both are open without a grant; `production` may still
gate them (`access-jit.md`).

---

## orm_search_read

**The default way to read.** One round-trip for filter + fields; prefer it over
`orm_search` followed by `orm_read` unless you already hold ids from an earlier call.

```json
{ "company_id": 3, "project_id": 900, "environment_id": 4021,
  "db": "odoo", "target_user_login": "admin",
  "model": "res.partner",
  "domain": [["is_company", "=", true], ["country_id.code", "=", "ES"]],
  "fields": ["name", "email", "city"],
  "limit": 20, "order": "name asc" }
```
```json
{ "ok": true,
  "data": { "ok": true, "result": { "records": [
      { "id": 15, "name": "Aurora Belmonte", "email": "hash-c4e29035",
        "city": "Bilbao", "_masked_fields": ["email"] } ] } },
  "elapsed_seconds": 0.06 }
```

Notes:

- Records live at **`data.result.records`**, each with `id` plus the fields you asked for.
- **`_masked_fields` names what the stage masked.** `hash-c4e29035` is not the stored
  value and not corruption — it is policy. If the real value is genuinely needed, that is
  the `unmask` JIT scope, with a reason (`access-jit.md`).
- `limit` defaults to 80, caps at 10000. **Ask for the fields you need**, never the whole
  record: every extra field crosses the proxy and lands in your context.
- A **large binary field is not returned inline** — you get a descriptor with a
  `download_url` instead (`file-tools.md`).

---

## orm_search

Ids only. Useful when you want a count-then-act pattern, or ids to feed `orm_write`.

```json
{ "model": "sale.order", "domain": [["state", "=", "draft"]], "limit": 100 }
```
```json
{ "ok": true, "data": { "ok": true, "result": { "ids": [12, 18, 34] } } }
```

---

## orm_search_count

How many match, without moving any data. **The right first call before a mass change**:
"this domain touches 4.812 records" is a sentence worth saying out loud before writing.

```json
{ "model": "product.template", "domain": [["sale_ok", "=", true]] }
```

---

## orm_read

Fields for ids you already have.

```json
{ "model": "res.partner", "ids": [15, 16], "fields": ["name", "email"] }
```

Same masking and same binary-descriptor behaviour as `orm_search_read`.

---

## orm_read_group

Server-side aggregation — count, sum, average grouped by a field. Far cheaper than reading
every record and adding them up yourself, and it keeps customer data out of your context.

```json
{ "model": "sale.order",
  "domain": [["state", "in", ["sale", "done"]]],
  "fields": ["amount_total:sum"],
  "groupby": ["partner_id"] }
```

**Aggregating over a masked field is refused**, deliberately: a sum or a count over
something the stage hid would leak the shape of the data the masking exists to protect.
If you hit that refusal, either aggregate on a non-masked field or request `unmask`.

---

## orm_fields_get

The model's field definitions as Odoo itself reports them: types, labels, relations,
required, readonly, selection values.

```json
{ "model": "res.partner", "fields": ["name", "email", "category_id"] }
```

Use it **before writing an unfamiliar model** — it is how you learn that a field is a
`many2one` (needs an id), a `selection` (needs one of its keys) or readonly (cannot be
written at all). Cheaper than a failed write on live data.

---

## orm_default_get

What Odoo would prefill on a new record. Worth calling before `orm_create` on a model with
non-obvious defaults (journals, warehouses, companies), so you create the record the way
the UI would rather than a technically valid but functionally wrong one.

```json
{ "model": "sale.order", "fields": ["company_id", "pricelist_id", "warehouse_id"] }
```

---

## orm_name_get

The display name of records — what the user actually sees in a many2one. Useful to confirm
you resolved the right record before writing its id somewhere.

```json
{ "model": "res.partner", "ids": [15, 16] }
```

---

## introspect_list_models / introspect_list_fields / introspect_list_methods

Discovery of the target's schema, when you do not yet know the model name, its fields, or
what you may call on it.

```json
{ "model": "sale.order" }        // list_fields / list_methods
```

These answer **what exists on that specific environment** — which is the point: a customer
with custom modules has models and fields that no generic Odoo reference will tell you
about. When the question is "does this environment have X", this is the tool; when it is
"what does standard Odoo 17 do with X", it is a documentation question, not this MCP.

`introspect_list_methods` matters before `orm_call_method`: it tells you what is callable
before you try, and the target enforces its own denylist regardless
(`write-tools.md` → `orm_call_method`).
