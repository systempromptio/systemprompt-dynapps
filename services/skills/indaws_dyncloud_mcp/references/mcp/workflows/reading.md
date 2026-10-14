# Workflows — reading data and configuration

Reading is cheap, reversible and the right way to start almost everything. The only real
cost is context: every field you ask for crosses the proxy and lands in the conversation.

---

## 1. Answer a question about the customer's data

```text
orm_search_read {
  ...five args...,
  model: "res.partner",
  domain: [["customer_rank", ">", 0], ["country_id.code", "=", "ES"]],
  fields: ["name", "city", "email"],
  limit: 20, order: "name asc"
}
```

**Ask for the fields you need and nothing else.** `fields: []` returns everything the model
has — dozens of columns per record, most of them noise.

If the answer is a number rather than a list, do not read the records to count them:

```text
orm_search_count { model: "res.partner", domain: [...] }   → 4812
```

Records arrive at `data.result.records`.

---

## 2. Read a record and what hangs off it

Odoo relations come back as ids (or `[id, "display name"]`). Two options:

**Dotted paths in the domain** to filter by a related field — one call:
```text
domain: [["partner_id.country_id.code", "=", "ES"]]
```

**A second read** when you need the related record's own fields:
```text
orm_search_read { model: "sale.order", domain: [["state","=","sale"]], fields: ["name","partner_id"] }
orm_search_read { model: "res.partner", domain: [["id","in",[15,16,22]]], fields: ["name","vat"] }
```

Two calls with an `in` domain beat N calls one per id, and beat asking for every field on
the first call hoping the data is in there.

---

## 3. Review a company's configuration

The functional audit consultants ask for. All of it is data, so it is ordinary reads:

```text
system_list_modules { ...five args..., state: "installed" }        # what functionality exists
orm_search_read { model: "res.company",  fields: ["name","currency_id","country_id"] }
orm_search_read { model: "account.journal", fields: ["name","type","code"] }
orm_search_read { model: "account.tax",     domain: [["active","=",true]], fields: ["name","amount","type_tax_use"] }
orm_search_read { model: "stock.warehouse", fields: ["name","code"] }
```

For the settings themselves see `writing.md` §4 — they live on `res.config.settings`, which
is a transient wizard, not a table you can read historically. To see the **current** value
of a setting, read the model it writes to (often `ir.config_parameter`, or a field on
`res.company`), or create a `res.config.settings` record with `orm_default_get` and read
what Odoo prefills.

---

## 4. "The client says X does not appear"

The most common consulting question, and the one where the MCP shines — because you can
look **as that user**.

```text
list_target_users { environment_id: ..., db: ... }        # find their login
orm_search_read { ...five args..., target_user_login: "<their login>",
                  model: "sale.order", domain: [["name","=","S00123"]], fields: ["name","state"] }
```

If it comes back empty for them and non-empty for `admin`, the record exists and **their
permissions or record rules hide it** — an access question, not a missing record. That is
a real answer, obtained in two calls, and it is far better than "no lo veo yo tampoco".

Other explanations worth ruling out in the same pass:

- The record is **archived** (`active: false`): add `["active","in",[true,false]]` to the
  domain and see whether it appears.
- It is in a **different company** than the one they are looking at.
- The **module that provides the menu** is not installed (`system_list_modules`).

---

## 5. Aggregate without pulling the data

For totals and counts, let the server do it:

```text
orm_read_group {
  ...five args...,
  model: "sale.order",
  domain: [["state","in",["sale","done"]], ["date_order",">=","2026-01-01"]],
  fields: ["amount_total:sum"],
  groupby: ["partner_id"]
}
```

Cheaper, faster, and it keeps thousands of customer records out of your context.

**Aggregating over a masked field is refused on purpose.** A sum or a count over data the
stage hid would leak the shape of what the masking protects. Either group by something else
or request the `unmask` scope with a real reason (`access.md` §4).
