# Domain Playbooks

Curated exposures per business domain. These are **starting points**, not dogma:
trim fields the client doesn't need, and never add a write/method that the agreed
capabilities don't justify. Field lists assume Odoo 17–19; verify names against
the live model with `describe_model` / the GOD server's `inspect_model` (field
names and method signatures drift across versions — verify against
`common-skills:odoo-version-api-reference` (per-version API + diffs) and the
indaws-core-developer odoo18/19 fixes).

## Universal rule

Whatever the domain, expose the **entities the agent must turn names into ids**,
or `resolve` is useless:

- `res.partner` — read: `name`, `display_name`, `email`, `phone`, `vat`,
  `country_id`, `customer_rank`, `supplier_rank`, `company_id`. Almost every
  domain needs it.
- `product.product` (or `product.template`) — read: `name`, `display_name`,
  `default_code`, `list_price`, `standard_price`, `uom_id`, `categ_id`,
  `barcode`. Needed by sales, purchase, inventory.

And always keep `core_tools` enabled so `whoami` / `describe_model` / `resolve` /
`aggregate` / `list_models` / `get_selection` are available — that is what makes
the server self-sufficient without per-client configuration.

## Files & binaries (any domain that touches documents/images)

Binary and image fields (`datas`, `image_1920`, report PDFs, attachments) **never
come back as bytes** from `search_read`/`describe` — the read returns only a
*value-ref placeholder*, never base64. Retrieval is a separate, deliberate path:

- **Download**: take the record id and call `attachment_search` (find the file)
  → `attachment_link` (get a short-TTL pre-signed URL). The raw bytes never pass
  through the ORM read.
- **Upload**: `request_upload` returns a pre-signed target; the client PUTs the
  file there, then you link it. Never try to push base64 through `create`/`write`.

Implication for the plan: if the capability involves documents/images, you must
**enable the attachment tools** (part of `core_tools` on this server family) and
expose `ir.attachment` narrowly (`name`, `res_model`, `res_id`, `mimetype`,
`type`) — *not* `datas`. State in `instructions` that files travel via the
pre-signed URL, not inline, so the agent doesn't ask for base64.

---

## Table of contents

- [Sales](#sales)
- [Purchases](#purchases)
- [Accounting](#accounting)
- [Inventory](#inventory-secondary)
- [CRM](#crm-secondary)

---

## Sales

"An MCP for sales" almost always means: answer questions about quotations/orders
and (sometimes) take or confirm orders.

| Model | Read-only profile | Transactional adds |
|-------|-------------------|--------------------|
| `sale.order` | `name`, `partner_id`, `date_order`, `state`, `amount_untaxed`, `amount_tax`, `amount_total`, `currency_id`, `user_id`, `team_id`, `order_line`, `validity_date`, `client_order_ref` | write: `partner_id`, `order_line`, `validity_date`, `client_order_ref`, `pricelist_id`; create: same |
| `sale.order.line` | `order_id`, `product_id`, `name`, `product_uom_qty`, `price_unit`, `discount`, `tax_id`, `price_subtotal`, `price_total` | write/create: `product_id`, `product_uom_qty`, `price_unit`, `discount` |
| `res.partner` | universal (above) | — |
| `product.product` | universal (above) + `taxes_id` | — |
| `product.pricelist` | read: `name`, `currency_id` | — |
| `crm.team` *(optional)* | read: `name`, `member_ids` | — |

- **Methods** (transactional only, gate carefully): `action_confirm`
  (quotation → sales order), `action_quotation_send`. Avoid `action_cancel`
  unless the use case is explicitly "let the agent cancel".
- **Relations to resolve**: `partner_id` → `res.partner`, `product_id` →
  `product.product`, `pricelist_id` → `product.pricelist`.
- **Aggregate wins**: "how many quotations by state", "total sold by salesperson"
  → `aggregate` on `sale.order` grouped by `state`/`user_id` — never page rows.
- **Never**: `unlink` on orders/lines.

**Instructions snippet** (drop into `instructions`):
> You manage the Sales area of this Odoo. Quotations and orders live in
> `sale.order` with their lines in `sale.order.line`. Resolve customers
> (`res.partner`) and products (`product.product`) by name before creating or
> filtering. Use `aggregate` for counts/totals by state, salesperson or period.
> Amounts are in the order's `currency_id`.

---

## Purchases

Mirror of sales on the buy side: track POs, sometimes raise/confirm them.

| Model | Read-only profile | Transactional adds |
|-------|-------------------|--------------------|
| `purchase.order` | `name`, `partner_id`, `date_order`, `state`, `amount_untaxed`, `amount_tax`, `amount_total`, `currency_id`, `user_id`, `order_line`, `date_planned` | write/create: `partner_id`, `order_line`, `date_planned` |
| `purchase.order.line` | `order_id`, `product_id`, `name`, `product_qty`, `price_unit`, `taxes_id`, `price_subtotal`, `date_planned` | write/create: `product_id`, `product_qty`, `price_unit` |
| `res.partner` | universal — here partners are **vendors** (`supplier_rank > 0`) | — |
| `product.product` | universal + `seller_ids` (read) | — |

- **Methods**: `button_confirm` (RFQ → PO). Avoid `button_cancel` unless required.
- **Relations to resolve**: `partner_id` (vendor), `product_id`.
- **Aggregate wins**: spend by vendor (`group_by partner_id`, `amount_total:sum`),
  POs by state.
- **Never**: `unlink`.

**Instructions snippet**:
> You manage Purchasing. RFQs and purchase orders are `purchase.order` with lines
> in `purchase.order.line`. Vendors are `res.partner` records with
> `supplier_rank > 0`; resolve them and products by name first. Use `aggregate`
> for spend-by-vendor or counts by state.

---

## Accounting

The most sensitive domain — legal/audit weight. **Default hard to read-only**
(reporting, balance/AR-AP questions, invoice lookups). Treat any write as an
exception requiring explicit client sign-off, and never expose reconciliation
internals or journal-entry posting unless that is the literal requirement.

| Model | Read-only profile | Notes |
|-------|-------------------|-------|
| `account.move` | `name`, `move_type`, `state`, `partner_id`, `invoice_date`, `invoice_date_due`, `amount_untaxed`, `amount_tax`, `amount_total`, `amount_residual`, `payment_state`, `currency_id`, `journal_id`, `ref` | invoices/bills/entries; filter `move_type` |
| `account.move.line` | `move_id`, `account_id`, `partner_id`, `name`, `debit`, `credit`, `balance`, `date`, `tax_ids` | read-only; heavy — prefer `aggregate` |
| `account.payment` | `name`, `partner_id`, `amount`, `payment_type`, `state`, `date`, `journal_id` | read |
| `res.partner` | universal + `credit`, `debit` (read) | AR/AP balances |
| `account.account` | read: `code`, `name`, `account_type` | chart lookups |
| `account.journal` | read: `name`, `type`, `code` | — |

- **Transactional (only on explicit demand)**: create a **draft** `account.move`
  (`move_type=out_invoice`) with lines. `action_post` is high-stakes — expose only
  with written approval and a restricted user/group; most "AI for accounting"
  asks are reporting and need none of this.
- **Aggregate wins**: AR by partner (`group_by partner_id`,
  `amount_residual:sum`, domain `move_type=out_invoice`, `payment_state!=paid`),
  revenue by month/journal. This is where `aggregate` shines — `account.move.line`
  has millions of rows; never dump them.
- **Never**: `unlink`; exposing internal reconciliation models; broad
  `account.move.line` writes.

**Instructions snippet**:
> You answer Accounting questions. Invoices, bills and journal entries are
> `account.move` (filter by `move_type`); lines are `account.move.line`.
> Customer/vendor balances are on `res.partner` (`credit`/`debit`) and on
> `account.move.amount_residual`. Always use `aggregate` for totals and ageing —
> never page individual move lines. You have read access only unless told
> otherwise; amounts are in each document's `currency_id`.

---

## Inventory *(secondary)*

| Model | Read-only profile |
|-------|-------------------|
| `stock.picking` | `name`, `partner_id`, `state`, `scheduled_date`, `picking_type_id`, `origin`, `move_ids_without_package` |
| `stock.move` | `product_id`, `product_uom_qty`, `quantity`, `state`, `location_id`, `location_dest_id` |
| `stock.quant` | `product_id`, `location_id`, `quantity`, `reserved_quantity` |
| `product.product` | universal + `qty_available`, `virtual_available` |

- **Methods**: `button_validate` only if the agent must confirm transfers.
- **Aggregate wins**: on-hand by location/product.

## CRM *(secondary)*

| Model | Read-only profile | Transactional adds |
|-------|-------------------|--------------------|
| `crm.lead` | `name`, `partner_id`, `email_from`, `phone`, `stage_id`, `expected_revenue`, `probability`, `user_id`, `team_id` | write/create: `name`, `partner_id`, `email_from`, `stage_id`, `expected_revenue` |
| `res.partner` | universal | — |

- **Methods**: `action_set_won_rainbowman` / stage moves if the agent qualifies leads.
- **Aggregate wins**: pipeline by stage (`expected_revenue:sum`).

---

## Combining domains

Clients rarely want one domain in isolation. "Sales" + "see what's been invoiced"
= sales playbook + a read-only slice of accounting (`account.move` filtered to
`out_invoice`). Compose playbooks, but keep the union minimal and re-run the
least-privilege check on the combined plan.
