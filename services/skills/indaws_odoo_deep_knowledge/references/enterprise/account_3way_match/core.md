<!-- verified: v18.0 @ 2026-06 -->
# account_3way_match — Vendor Bill Release-to-Pay (3-Way Match)

**EE only.** Module path: `odoo_enterprise/18/account_3way_match/`. Present in v19 with non-trivial diffs.
`license: 'OEEL-1'` (`account_3way_match/__manifest__.py:33`). Depends: `purchase` (`account_3way_match/__manifest__.py:28`). NO `auto_install`.

Gotchas → [[gotchas]]
v19 → [[v19_changes]]
Integrations → [[integrations]]

## Concept

Manufacturing/wholesale process: receive vendor bill BEFORE goods arrive. Don't pay until received quantities match billed quantities. The 3 "ways" matched:
1. **Purchase Order** quantity & price
2. **Vendor Bill** invoiced quantity & price
3. **Goods Receipt** received quantity

Bill flagged with `release_to_pay`:
- `yes` — match, pay it
- `no` — nothing received, don't pay
- `exception` — mismatch, requires review

## Models

| Model | Field | Definition | Purpose |
|-------|-------|------------|---------|
| `account.move` | `release_to_pay` (Selection) | `account_3way_match/models/account_invoice.py:22-31`, `class AccountMove`, `compute='_compute_release_to_pay'`, `store=True`, `copy=False` | Aggregate status from lines |
| `account.move` | `release_to_pay_manual` (Selection, string "Should Be Paid") | `account_3way_match/models/account_invoice.py:32-39`, `class AccountMove`, `compute='_compute_release_to_pay_manual'`, `store=True`, `readonly=False` | Editable manual value (NOT a plain field — it is computed-stored-editable) |
| `account.move` | `force_release_to_pay` (Boolean, "Force Status") | `account_3way_match/models/account_invoice.py:40-42`, `class AccountMove` | Use manual value instead of computed |
| `account.move.line` | `can_be_paid` (Selection, "Release to Pay") | `account_3way_match/models/account_invoice.py:171-176`, `class AccountMoveLine`, `compute='_can_be_paid'`, `store=True`, `copy=False` | Per-line release-to-pay status |
| `account.journal` | (method overrides only) | `account_journal_dashboard.py` | Dashboard queries filter by `release_to_pay` |

Status selection list `[('yes','Yes'),('no','No'),('exception','Exception')]` defined once at `account_3way_match/models/account_invoice.py:9`, `_release_to_pay_status_list` and reused by all three Selection fields.

## Auto Init

Custom `_auto_init` exists on BOTH models (not one):

- `account.move._auto_init` — `account_3way_match/models/account_invoice.py:14-20`, `AccountMove._auto_init`:
  ```python
  if not column_exists(self.env.cr, "account_move", "release_to_pay"):
      self.env.cr.execute("ALTER TABLE account_move ADD COLUMN release_to_pay VARCHAR DEFAULT 'exception'")
  ```
- `account.move.line._auto_init` — `account_3way_match/models/account_invoice.py:94-100`, `AccountMoveLine._auto_init`: same pattern for `account_move_line.can_be_paid VARCHAR DEFAULT 'exception'`.

Avoids heavy compute on install for huge tables. Default `'exception'` means: until next compute runs, all bills/lines are "exception" (safe, blocks payment).

## Per-Line Logic (`_can_be_paid`) — `account_3way_match/models/account_invoice.py:103-135`, `class AccountMoveLine`

`@api.depends('purchase_line_id.qty_received', 'purchase_line_id.qty_invoiced', 'purchase_line_id.product_qty', 'price_unit')` (`account_3way_match/models/account_invoice.py:103`, `class AccountMoveLine`).

For each invoice line linked to a `purchase.order.line`:

```python
precision = self.env['decimal.precision'].precision_get('Product Unit of Measure')  # line 112
po_line = invoice_line.purchase_line_id      # line 114
invoiced_qty = po_line.qty_invoiced          # line 116
received_qty = po_line.qty_received          # line 117
ordered_qty  = po_line.product_qty           # line 118

# Currency check FIRST (lines 121-127)
invoice_converted_price = invoice_currency._convert(
    invoice_line.price_unit, order_currency, invoice_line.company_id, fields.Date.today())
if order_currency.compare_amounts(po_line.price_unit, invoice_converted_price) != 0:
    invoice_line.can_be_paid = 'exception'   # Price drift between PO and bill
    continue

if po_line.product_id.purchase_method == 'purchase':   # 'on ordered' (line 129)
    invoice_line._can_be_paid_ordered_qty(...)
else:                                                  # 'on received' (line 131)
    invoice_line._can_be_paid_received_qty(...)
```

Lines without `purchase_line_id` → `can_be_paid = 'exception'` (`account_3way_match/models/account_invoice.py:134-135`, `AccountMoveLine._can_be_paid`, the `else` branch).

## On Ordered Qty (`_can_be_paid_ordered_qty`) — `account_3way_match/models/account_invoice.py:137-152`, `AccountMoveLine._can_be_paid_ordered_qty`

```python
if float_compare(invoiced_qty - self.quantity, ordered_qty, precision_digits=precision) >= 0:
    self.can_be_paid = 'no'          # line 147-148 — already over-billed
elif float_compare(invoiced_qty, ordered_qty, precision_digits=precision) <= 0:
    self.can_be_paid = 'yes'         # line 149-150 — fits within order
else:
    self.can_be_paid = 'exception'   # line 151-152 — partial overflow
```
`self.quantity` is the CURRENT line's qty (subtracted from total `invoiced_qty`).

## On Received Qty (`_can_be_paid_received_qty`) — `account_3way_match/models/account_invoice.py:154-169`, `AccountMoveLine._can_be_paid_received_qty`

```python
if float_compare(invoiced_qty, received_qty, precision_digits=precision) <= 0:
    self.can_be_paid = 'yes'         # line 164-165 — bill <= what arrived
elif received_qty == 0 and float_compare(invoiced_qty, ordered_qty, precision_digits=precision) <= 0:
    self.can_be_paid = 'no'          # line 166-167 — nothing received yet, within order
else:
    self.can_be_paid = 'exception'   # line 168-169 — billed > received, or over ordered
```

## Bill-Level Aggregation (`_compute_release_to_pay`) — `account_3way_match/models/account_invoice.py:44-77`, `class AccountMove`

`@api.depends('invoice_line_ids.can_be_paid', 'force_release_to_pay', 'payment_state')` (`account_3way_match/models/account_invoice.py:44`, `class AccountMove`).

```python
for invoice in records:
    if invoice.payment_state == 'paid' or not invoice.is_invoice(include_receipts=True):
        invoice.release_to_pay = 'no'                       # line 52-54
    elif invoice.force_release_to_pay:
        invoice.release_to_pay = invoice.release_to_pay_manual  # line 55-57
    else:
        result = None                                       # line 60
        for line in invoice.invoice_line_ids.filtered(
                lambda l: l.display_type not in ('line_section', 'line_note')):  # line 61
            line_status = line.can_be_paid
            if line_status == 'exception':
                result = 'exception'; break                 # line 63-66
            elif not result: result = line_status           # line 67-68
            elif line_status != result:
                result = 'exception'; break                 # line 69-71
        invoice.release_to_pay = result or 'no'             # line 77
```

Mixed `yes`+`no` lines → `exception`. Empty/section-only bill → `result=None` → `'no'`.

## Manual Override

`release_to_pay_manual` is itself computed by `_compute_release_to_pay_manual` (`account_3way_match/models/account_invoice.py:79-83`, `class AccountMove`): when the bill is NOT paid, IS an invoice, and is NOT forced, the manual value mirrors `release_to_pay`. Once `force_release_to_pay=True` the compute stops touching it, so the user-entered value sticks.

`@api.onchange('release_to_pay_manual')` (`account_3way_match/models/account_invoice.py:85-88`, `class AccountMove`): if user changes the manual value AND it differs from the computed `release_to_pay`, `force_release_to_pay` is set to `True` automatically.

`force_release_to_pay=True` + `release_to_pay_manual='yes'` forces a bill payable regardless of computed status (`_compute_release_to_pay` line 55-57 reads the manual value).

## Module Install Behaviour

`_compute_release_to_pay` checks `context.get('module') == 'account_3way_match'` (`account_3way_match/models/account_invoice.py:47-50`, `AccountMove._compute_release_to_pay`):
- During install, all paid bills + non-invoice moves get `'no'` in one bulk write (`(self - records).release_to_pay = 'no'`, line 50).
- Only records with `payment_state != 'paid' and move_type in ('in_invoice', 'in_refund')` are computed properly (line 49).

## Views — `views/account_invoice_view.xml`

- Form inherit `account_invoice_form_inherit` (`:3`) injects after `fiscal_position_id` in `other_tab`; only visible for `move_type in ('in_invoice','in_refund')` (`:9-10`). `release_to_pay` is `invisible` + `force_save="1"` (`:11`); `force_release_to_pay` only visible when truthy (`:13-14`).
- Search-view inherit `account_invoice_filter_inherit_account_3way_match` (`:20-34`, `mode="primary"`, inherits `account.view_account_bill_filter`) adds three filters: `bills_to_validate` (`:28`), `bills_to_pay` (`:29`), `exception` (`:30`).
- Rebinds `account.action_move_in_invoice_type` and `account.action_move_in_refund_type` search views to the inherited filter (`:40-46`).

## Dashboard — `models/account_journal_dashboard.py`

- `open_action` override (`:11-18`) swaps the vendor-bill search view to `account_invoice_filter_inherit_account_3way_match` and zeroes `search_default_in_invoice`.
- `_get_open_sale_purchase_query` override (`:20-42`) adds tile select `SQL("release_to_pay IN ('yes', 'exception') AS to_pay")` (`:39`).
- `_get_draft_sales_purchases_query` override (`:44-65`): purchase journals show a draft bill if `invoice_date_due < today` OR `release_to_pay = 'yes'` (`:54-57`).
- Dashboard kanban inherit `views/account_journal_dashboard_view.xml` wires the tiles' click contexts to `search_default_bills_to_validate`, `search_default_bills_to_pay`, `search_default_late`.

## Tests

- `tests/test_release_to_pay_invoice.py` — `test_3_way_match` (`:75-90`) drives both invoicing policies through receive/invoice scenarios and asserts `release_to_pay`. Confirms: price change between order and invoice always → `exception` (`:88-90`); section/note lines added to the bill do NOT affect status (`:60-63`).
- `tests/test_account_journal_dashboard.py` — dashboard tile counts.
