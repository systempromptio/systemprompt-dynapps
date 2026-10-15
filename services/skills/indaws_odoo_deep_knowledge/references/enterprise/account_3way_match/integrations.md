<!-- verified: v18.0 @ 2026-06 -->
# account_3way_match Integrations

EE module. All paths under `odoo_enterprise/18/account_3way_match/` unless noted. Cross-module hooks verified against v18 source.

## (a) `account.move.line` ↔ `purchase.order.line` (the core join)

`_can_be_paid` reads the link `invoice_line.purchase_line_id` (`account_3way_match/models/account_invoice.py:114`, `AccountMoveLine._can_be_paid`) — the M2o on `account.move.line` provided by the `purchase` dependency (`account_3way_match/__manifest__.py:28`). From the PO line it pulls three quantities:
- `po_line.qty_invoiced` (`:116`)
- `po_line.qty_received` (`:117`)
- `po_line.product_qty` (ordered) (`:118`)

The depends chain `'purchase_line_id.qty_received', 'purchase_line_id.qty_invoiced', 'purchase_line_id.product_qty'` (`:103`) means: any receipt (`qty_received` changes via stock pickings → `purchase.order.line`), any other bill on the same PO line (`qty_invoiced`), or a PO quantity edit recomputes `can_be_paid` on EVERY invoice line sharing that PO line — then bubbles up to `release_to_pay` via `invoice_line_ids.can_be_paid` (`:44`).

## (b) Dependency on `product.purchase_method`

Branch selector at `account_3way_match/models/account_invoice.py:129`, `AccountMoveLine._can_be_paid`:
- `purchase_method == 'purchase'` → `_can_be_paid_ordered_qty` ("on ordered quantities") (`:130`)
- else (`'receive'`) → `_can_be_paid_received_qty` ("on received quantities") (`:131-132`)

`purchase_method` is the field added by `purchase`/`purchase_stock` on `product.template`. Consequence: a product invoiced "on ordered" never blocks on receipts — only on `ordered_qty` (`:147,149`); a product invoiced "on received" blocks until `qty_received` catches up (`:164`). Changing a product's `purchase_method` mid-flow flips which formula a future recompute uses. Tests exercise both via `invoicing_policy='purchase'`/`'receive'` (`account_3way_match/tests/test_release_to_pay_invoice.py:28-31`, `TestReleaseToPayInvoice.check_release_to_pay_scenario` y `account_3way_match/tests/test_release_to_pay_invoice.py:76-90`, `TestReleaseToPayInvoice.test_3_way_match`).

## (c) `payment_state` → recompute

`release_to_pay` depends on `payment_state` (`account_3way_match/models/account_invoice.py:44`, `class AccountMove`) and short-circuits to `'no'` once `payment_state == 'paid'` (`:52-54`). `payment_state` is the `account` field driven by reconciliation (`account.payment` / `account.move` matching). Consequence: every reconciliation/payment registration that moves a vendor bill's `payment_state` recomputes its stored `release_to_pay`, which then changes what the dashboard tiles count. `release_to_pay_manual` is also re-mirrored unless forced (`:79-83`).

## (d) Dashboard `account.journal` queries

`account.journal.open_action`, `_get_open_sale_purchase_query`, `_get_draft_sales_purchases_query` are OVERRIDES of `account`'s journal-dashboard machinery (`account_3way_match/models/account_journal_dashboard.py:11`, `AccountJournal.open_action` y `account_3way_match/models/account_journal_dashboard.py:20`, `AccountJournal._get_open_sale_purchase_query` y `account_3way_match/models/account_journal_dashboard.py:44`, `AccountJournal._get_draft_sales_purchases_query`).
- The "to pay" tile select `release_to_pay IN ('yes','exception')` (`:39`) is injected into the base `account` dashboard query — so the purchase journal kanban gains a payable count the sale journal lacks.
- Draft purchases surface when `invoice_date_due < today` OR `release_to_pay = 'yes'` (`:54-57`).
- View wiring: `views/account_journal_dashboard_view.xml` inherits `account.account_journal_dashboard_kanban_view` and binds tile clicks to `search_default_bills_to_validate` / `search_default_bills_to_pay` / `search_default_late`, which resolve against the search filters defined in `account_3way_match/views/account_invoice_view.xml:28-30`, `account_invoice_filter_inherit_account_3way_match`.

## (e) Multi-currency `_convert` interaction

`account_3way_match/models/account_invoice.py:121-125`, `AccountMoveLine._can_be_paid`:
```python
invoice_currency = invoice_line.currency_id
order_currency   = po_line.currency_id
invoice_converted_price = invoice_currency._convert(
    invoice_line.price_unit, order_currency, invoice_line.company_id, fields.Date.today())
if order_currency.compare_amounts(po_line.price_unit, invoice_converted_price) != 0:
    invoice_line.can_be_paid = 'exception'
```
`_convert` and `compare_amounts` come from `res.currency` (base). Integration consequence: the price match is done in the PO's currency, using `res.currency.rate` valid TODAY (not the PO/bill date). A bill in a different currency, or the same currency revalued between PO and check time, can fall into `exception` purely from FX. The company used for the rate lookup is `invoice_line.company_id`. (See gotcha A3M3 in [[gotchas]].)

## (f) Module-presence coupling (install context)

`_compute_release_to_pay` inspects `self.env.context.get('module') == 'account_3way_match'` (`account_3way_match/models/account_invoice.py:47`, `AccountMove._compute_release_to_pay`) — the install context set by `ir.module.module` during module loading — to bulk-set `'no'` on irrelevant moves cheaply (`:49-50`). Any external code that calls the compute with that context key spoofed would trigger the bulk branch.
