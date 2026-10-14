# What to seed, and where it comes from

The invoice mapping in `dynapps_ai_extract_account` resolves each field by querying
`ai.memory.alias` (only rows with `state='approved'`, per company). An alias is
`name (the text the AI emits) → (res_model, field_name) = res_id`, with an optional `context_key`.
Seeding those aliases from the history = starting out trained.

## Tuples the mapping queries (confirmed in the code)

| Layer | res_model | field_name | context_key | Where the mapping looks it up |
|---|---|---|---|---|
| Supplier by trade name | `res.partner` | `name` | `""` (global) | server action + declarative mapping |
| Supplier by tax id | `res.partner` | `vat` | `""` (global) | server action + declarative mapping |
| Product by code | `product.product` | `default_code` | `""` | server action (direct search) |
| Product by barcode | `product.product` | `barcode` | `""` | server action |
| Product by description | `product.product` | `name` | `""` (or `partner_id:<id>`) | server action + description mapping |
| **General account** | `account.account` | `code` | **`partner_id:<id>`** | declarative mapping `invoice_line_ids.account_id.code` |
| Tax | `account.tax` | `name` | **`partner:<id>`** + global | server action (`find_record`) |

## Where each value comes from in the history (posted `account.move`, in_invoice/in_refund)

- **Supplier**: for each invoice, `partner_id`. Alias `name=partner.name → partner.id` (field
  `name`) and `name=partner.vat → partner.id` (field `vat`). Both global.
- **Product** (per line with a `product_id`): `name=product.default_code → product.id` (field
  `default_code`), `name=product.barcode → product.id` (field `barcode`), and `name=line.name
  (description) → product.id` (field `name`).
- **Account** (per line): `name=line.name (description) → account.id` (field `code`), with
  `context_key=partner_id:<id of the invoice's supplier>`. This is the key layer: the right account
  depends on the supplier.
- **Tax** (from the line's `tax_ids`): `name=tax.name → tax.id` (field `name`), with
  `context_key=partner:<id>`. Opt-in (generic names, lower value; the engine already falls back by
  type/rate).

## The TWO context_key formats (do not unify them)
- Account and product-description / PO seeding: **`partner_id:<id>`**
  (`extract_mixin._ai_mapping_context_key`).
- Taxes in the server action: **`partner:<id>`** (no `_id`).

Use `context_key=""` (an empty string, not `False`/`None`) for global entries: the unique index
uses `COALESCE(context_key,'')`, so empty and NULL collide.

## Seeding method
`ai.memory.alias.save_alias(name, res_model, res_id, field_name, company=None, state="approved",
context_key="")`. Idempotent: if the same tuple already exists pointing at the same `res_id` it
does nothing; if it points elsewhere, it updates. Entries are born `approved` (they resolve during
extraction). `company` is taken from `env.company` (set through the `company_id` context in
`_odoo.py`).

## What is worth seeding, and what is not
The match is on the **whole** text (`=ilike`), never partial. That is why stable values pay off and
variable ones do not:

- **Worth it**: tax ids, product codes, supplier references, barcodes — identical on every document.
- **Not worth it**: line descriptions that carry a week, a batch or a month inside. Measured case:
  the alias `Vracht` did **not** match the real line
  `"Vracht week 33 — 4 ritten Eindhoven-Amsterdam"`.
