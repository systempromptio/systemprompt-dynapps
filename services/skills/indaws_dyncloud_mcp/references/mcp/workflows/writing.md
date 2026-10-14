# Workflows — changing data

Every write here lands on a live customer. The habits below are what separate a change you
can explain from one you have to undo.

---

## 1. Create a record safely

```text
orm_fields_get  { ...five args..., model: "res.partner",
                  fields: ["name","email","country_id","category_id"] }   # types, required, readonly
orm_default_get { ...five args..., model: "res.partner",
                  fields: ["company_id","lang","property_account_position_id"] }
orm_create      { ...five args..., model: "res.partner",
                  vals: { name: "Aurora Belmonte", email: "aurora@example.com",
                          country_id: 68 } }
  → data.result.ids: [9]
```

`orm_fields_get` before an unfamiliar model saves the classic failures: writing a string
into a `many2one`, a free value into a `selection`, or a `readonly` computed field.
`orm_default_get` is what makes the record look like one created in the UI — with the
company, the language and the accounting position Odoo would have set.

---

## 2. Change many records at once

**Count first, and say the number out loud.**

```text
orm_search_count { ...five args..., model: "res.partner",
                   domain: [["city","=","Bilbo"]] }        → 412
```

> "Son 412 contactos los que tienen la ciudad como *Bilbo*. ¿Los cambio todos a *Bilbao*?"

Then, and only then:

```text
orm_search { ...same domain..., limit: 500 }               → ids
orm_write  { ...five args..., model: "res.partner", ids: [...], vals: { city: "Bilbao" } }
```

Two things worth doing on anything large: **run it on one id first** and read the result
back, and **keep the id list** in your answer so the change is reversible by whoever comes
next. A write has no dry-run — the count and the single-record test are the substitute.

---

## 3. Archive instead of deleting

"Quitar", "borrar" and "eliminar" almost never mean `orm_unlink` in Odoo.

```text
orm_write { model: "product.template", ids: [...], vals: { active: false } }
```

Archiving hides the record from every view while preserving history — invoices still refer
to their product, orders to their customer. `orm_unlink` destroys the row and Odoo will
usually refuse when anything references it; **that refusal is protection, not an
obstacle**. Delete only when the user explicitly asked for a permanent delete and the
record has no history worth keeping.

---

## 4. Turn an Odoo option on or off

Settings are **not** rows you edit. `res.config.settings` is a transient wizard: you create
one with the values you want and call `execute` on it, exactly as pressing *Save* in
Settings does. Writing the underlying parameter directly skips the `set_values` logic that
creates records, activates groups and sets defaults — which is how a setting ends up
"enabled" with nothing behind it.

```text
# 1. Find the technical name of the option
orm_fields_get { ...five args..., model: "res.config.settings" }
#    → e.g. group_discount_per_so_line, group_uom, group_stock_multi_locations…

# 2. See the current state (the wizard prefills from the live configuration)
orm_default_get { ...five args..., model: "res.config.settings",
                  fields: ["group_discount_per_so_line"] }

# 3. Create the wizard with the change
orm_create { ...five args..., model: "res.config.settings",
             vals: { group_discount_per_so_line: true } }
  → ids: [812]

# 4. Apply it — this is the step that does the work
orm_call_method { ...five args..., model: "res.config.settings",
                  method_name: "execute", args: [[812]] }
```

Notes:

- Field names are the wizard's, not the model's: `group_*` for feature groups,
  `module_*` to install a module as part of saving the setting.
- **A `module_*` field installs a module**, with everything that implies — it is a module
  operation wearing a checkbox. Treat it with the care of `modules.md`, and expect it to
  take minutes.
- Settings are per company in multi-company databases: create the wizard with the right
  `company_id` in context, and verify afterwards on the company that matters.
- Verify by re-reading with `orm_default_get`, not by trusting the `execute` answer.

> Not exercised end to end in the session that produced this skill — the mechanism is
> Odoo's standard one and the calls are ordinary ORM, but treat the first run on a customer
> as a first run: do it on a development environment if you have one.

---

## 5. Run a business action

Confirming an order, validating a picking, calling a customer's own method:

```text
introspect_list_methods { ...five args..., model: "sale.order" }     # does it exist here?
orm_call_method { ...five args..., model: "sale.order",
                  method_name: "action_confirm", args: [[42]] }
```

- `args: [[ids]]` is the usual shape for a method that acts on records.
- The target blocks private methods (leading `_`), the canonical ORM methods (use their own
  tools) and a hardcoded list. If you get `E_METHOD_FORBIDDEN`, the tool you want is a
  different one, not a workaround.
- **A business action is a cascade.** Confirming a sale order creates deliveries, moves
  stock and may generate invoices. Say what will happen before calling it, and read the
  record back afterwards to report what actually did.
