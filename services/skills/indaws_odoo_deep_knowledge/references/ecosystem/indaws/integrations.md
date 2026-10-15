<!-- verified: v18.0 @ 2026-06 (indaws src) -->
# Indaws Suite — How It Patches/Extends the Core

Where the Indaws core suite hooks into standard Odoo, the create/write/compute overrides that silently mutate fields a caller didn't touch, and the friction when migrating or coding on top. Source: `indawsodoo/18/` (modules `indaws_*`).

> This file is the **integration/friction map**. For module catalog see [[catalog]]; for gotchas [[gotchas]]; for v19 deltas [[v19_changes]]. For deep architecture (5 layers, dependency tree, migration scripts, bridge patterns, code patterns) the canonical source is the **`indaws-core-developer`** skill (`~/.claude/skills/indaws-core-developer/`) — NOT duplicated here.

---

## 1. The auto_install cascade — you cannot install vanilla Odoo

- `indaws_base/__manifest__.py:25-26` — depends only on `base_setup`, `auto_install: True` → installs the moment any DB exists.
- Every layer module is `auto_install: True` with NO explicit auto-install list → Odoo's default rule applies: auto-install when ALL `depends` are already present.
- Consequence: installing standard `account` pulls `indaws_base_account` (depends `["account","indaws_base"]`); installing `sale` pulls `indaws_base_sale` → `indaws_base_account` → `indaws_account`/`indaws_sale`. The whole layer materializes silently.
- Two-tier split: `indaws_base_account` (thin foundation) vs `indaws_account` (heavy; depends `indaws_product` + `indaws_base_account`). So `indaws_account` only auto-installs once the product chain is present. **A field you expect "always on account.move" may live in `indaws_account` and only exist if `product` is installed too.**
- To run vanilla Odoo for comparison/debugging: remove/disable the indaws addons path; you cannot selectively install standard `sale` while the indaws addons are visible.

## 2. Create/write/compute overrides that mutate fields you didn't touch

These are the highest-surprise patches when debugging "why did this field change":

| Core model | Override | Hidden mutation | Source |
|-----------|----------|-----------------|--------|
| `sale.order` | `create()` | If `name=="New"` + order type has `sequence_id`, replaces SO reference from the TYPE's sequence (not `sale.order` seq) | `indaws_sale_order_type/models/sale_order.py:60-69`, `SaleOrder.create` |
| `sale.order` | `write()` | Changing `order_type_id` on a draft/sent order **renumbers** the SO reference from the new type's sequence | `indaws_sale_order_type/models/sale_order.py:71-100`, `SaleOrder.write` |
| `sale.order` | `_compute_journal_id` | `order_type_id.journal_id` overrides the standard invoice journal; `_prepare_invoice` copies it to the invoice | `indaws_sale_order_type/models/sale_order.py:102-108`, `SaleOrder._prepare_invoice` y `indaws_sale_order_type/models/sale_order.py:109-114`, `SaleOrder._compute_journal_id` |
| `account.move.line` | `create`/`write` | Auto-recomputes `parent_id` from `section_parent_uuid` unless `parent_id` is explicit or `no_compute_line_parent_id` context is set | `indaws_account/models/section_line_mixin.py:201-218`, `SectionLineMixin.create` |
| `account.move.line` | `discount` field | Redefined as a STORED COMPUTED aggregate of `discount1/2/3`; writing `discount` is intercepted → moved into `discount1`, 2/3 zeroed | `indaws_account_triple_discount/models/triple_discount_mixin.py:18-84`, `TripleDiscountMixin.discount` |

## 3. Triple discount — sale vs account are DIFFERENT mechanisms (migration trap)

There are **two divergent implementations**, and `sale.order.line.discount` ≠ `account.move.line.discount` semantically:

- **Account side** (`triple.discount.mixin`, `indaws_account_triple_discount/models/triple_discount_mixin.py:18-68`, `TripleDiscountMixin.discount`): `discount` IS the aggregate. Stored computed from `discount1/2/3` via `_get_aggregated_multiple_discounts` (multiplicative). Writing `discount` directly is redirected to `discount1`. 3 SQL CHECK constraints `discount1/2/3 <= 100` (lines 29-45) — **legacy import data with discount >100 hard-fails at the DB**.
- **Sale side** (`indaws_sale_triple_discount/models/sale_order_line.py:65-118`, `SaleOrderLine._compute_amount`): SOL does NOT inherit the mixin. Instead `_compute_amount` wraps super in `_aggregated_discount()` — a transient context-manager that temporarily overwrites `discount` with the final discount and restores it. So **the stored `discount` on a saved SO line is the FIRST discount, not the aggregate**; the aggregate exists only during amount compute. A `discounting_type` (additive/multiplicative) extends the sale side; purchase/account are multiplicative-only.
- **Trap:** integrations reading `sale.order.line.discount` as "the effective discount" are wrong (it's tier 1). Reading `account.move.line.discount` gives the aggregate. `_convert_to_tax_base_line_dict` (`:120-136`) only returns the aggregate when `discount_is_aggregated` context is set. (v19 redesign aligns sale to account — see [[v19_changes]] §5.)

## 4. Invoice section hierarchy — `section.line.mixin` makes lines a stored tree

- `indaws_account/models/section_line_mixin.py:8-43`, `SectionLineMixin` — el modelo `section.line.mixin` adds `parent_id`/`parent_path` (`_parent_store=True`), `line_child_ids`, `section_uuid`/`section_parent_uuid`, recursive `section_price_subtotal/total`. Inherited by `account.move.line` (and `sale.order.line` in siblings). **Invoice lines now form a stored materialized-path tree — raw SQL / bulk writes that ignore `parent_id`/`parent_path` corrupt the hierarchy.**
- `rebuild_hierarchy` (`:283-347`) copies the SO-line tree onto invoice lines by index alignment (`mail_notrack`, `tracking_disable`, `no_compute_line_parent_id`, `prefetch_fields=False`). If `len(source)!=len(target)` it silently returns without rebuilding.
- `indaws_account/models/account_move.py:9-13`, `AccountMove._compute_needed_terms` — `_compute_needed_terms` injects `payment_term_move` context; bridge code reading payment terms must not strip context.
- ⚠️ `indaws_invoice_section/models/res_company.py:6-24`, `DEFAULT_PYTHON_CODE` — `res.company.dynamic_invoice_section` stores **free-form Python** (`DEFAULT_PYTHON_CODE`, exposes `env/self/sale/invoice/picking`, `result=`), eval'd during invoicing. This module is `auto_install: False`, depends only on `sale` — the odd one out. Security review item: arbitrary server-side code on the company record.

## 5. Margin engine — `purchase_price_total` & `cost_summary_ids`

- `indaws_sale_margin/models/sale_order.py:9-24`, `SaleOrder.purchase_price_total` — `purchase_price_total = amount_untaxed - margin` (stored). It INHERITS all of Odoo's `sale_margin` quirks; it is NOT a sum of the new cost lines.
- `indaws_sale_margin/models/sale_order.py:12-43`, `SaleOrder.cost_summary_ids` — `cost_summary_ids` is a STORED One2many fully rebuilt on compute (`Command.clear()` + recreate, `compute_sudo=True`). Expensive on edits, and child record IDs are NOT stable across recomputes (don't store references to them).
- `indaws_sale_margin/security/res_groups.xml:2-5`, `group_sale_advanced_costs` — `group_sale_advanced_costs` gates the entire cost-detail UI. Users without it never see the purchase-price breakdown. (inBuilt depends on this group for its margin-project views — see [[../inbuilt/integrations]] §7.2.)

## 6. Picking-to-invoice — the m2m is COMPUTED, no manual join logic

- `indaws_stock_picking_invoice/models/stock_picking.py:11-40`, `StockPicking.invoice_ids` — `invoice_ids` IS a `fields.Many2many("account.move")` but `compute="_compute_invoice_ids"` + `store=True` (depends `move_ids.invoice_lines`), mapping `move_ids.invoice_lines.move_id`. The reciprocal `account.move.picking_ids` (`indaws_stock_picking_invoice/models/account_move.py:9-26`, `AccountMove.picking_ids`) maps `invoice_line_ids.invoice_stock_move_id.picking_id`. **No hand-maintained m2m relation** — both directions derive entirely from the custom `account.move.line.invoice_stock_move_id` (inverse declared on `stock.move`, `indaws_stock_picking_invoice/models/stock_move.py:23`, `StockMove.invoice_lines`). Break that field and the picking↔invoice link severs silently.
- `indaws_stock_picking_invoice/models/stock_picking.py:64-82`, `StockPicking._get_record_invoice` — `_get_record_invoice` direction-disambiguates: a picking with both `sale_id` and `purchase_id` (MTO/dropship) → incoming = vendor bill, outgoing = customer invoice. Uses `hasattr` to avoid hard sale/purchase deps.
- Actual move creation is delegated: the sale bridge (`indaws_sale_stock_picking_invoice/models/stock_picking.py:14-38`, `StockPicking.action_invoice_create`) launches the standard `sale.action_view_sale_advance_payment_inv` wizard with `default_invoice_from='picking'` (also catches sales whose only pending work is a RETURN picking → returns become linked credit notes). **To extend, hook the advance-payment wizard, not a custom create.**

## 7. Rappel — settlements are CREDIT NOTES, volume is invoice-based

- `indaws_rappel/models/rappel.py:30-39`, `Rappel.rappel_type` — `rappel_type` has 7 values: `fixed`, `variable`, `formula`, `single_target`, `global_scale`, `marginal_tiers`, `per_order` (NOT just fixed/variable). `formula` evals stored Python; tiered types require `section_ids`.
- `indaws_rappel/models/rappel.py:430-470`, `Rappel._get_period_volume` — `_get_period_volume` reads POSTED `account.move.line` (`display_type='product'`, scoped move types), subtracting refund lines (sign -1). Rappel volume is **invoice-based, not order-based**, date-filtered on `invoice_date`; draft invoices don't count, credit notes reduce the base.
- `indaws_rappel/models/rappel_settlement.py:249-326`, `RappelSettlement._create_invoice` — `_create_invoice` builds an `out_refund`/`in_refund` (credit note), sets `invoice.rappel_settlement_id`, auto-searches journal by type+company (a missing journal silently `limit=1`s or fails). The only link is `account.move.rappel_settlement_id` (`indaws_stock_picking_invoice/models/account_move.py:10-16`, `AccountMove.picking_ids`, `copy=False`) — **duplicating a rappel credit note drops the link**.

## 8. Data ETL — the real friction is process-wide monkeypatching

- `indaws_data_import/models/indaws_import.py` — the "god class" is the `indaws.import` model (~1600 lines), NOT a file named `import_record.py` (that file does not exist). Extend `indaws.import`.
- `indaws_data_import/models/indaws_import.py:682-706`, `IndawsImport.action_import_file` — dispatch is blocking OR `queue_job.with_delay(identity_key=...)`. Hard dependency on OCA `queue_job` (imports `RetryableJobError`). Without a running job runner, async imports never execute.
- `indaws_data_import/models/queue_job.py:14-31`, `QueueJob.create` — `queue.job` `create`/`write` are PATCHED to stamp `job_import_id` and auto-advance the import state machine when a job hits `done`. The import state machine is driven by job side-effects.
- `indaws_data_import/models/indaws_import_scheduler.py:144-259`, `IndawsImportScheduler.create` — scheduler `create`/`write`/`unlink` manage a backing `ir.cron`; deleting a scheduler deletes its cron.
- ⚠️ `indaws_data_export/models/export.py:487-595`, `IndawsExport._register_hook` — **monkeypatches `create`/`write`/`unlink`/`_compute_field_value` on ARBITRARY target models** (adapted from `base_automation`, via `_patch_method`). Any model chosen as an export target has its core ORM methods wrapped at registry load → global, cross-cutting, interacts with every other override on that model. **"Why does saving res.partner trigger X?" leads here.**
- ⚠️ `indaws_data_export/models/export_registry.py:14-21`, `convert_to_cache` — module-level reassignment `Serialized.convert_to_cache = convert_to_cache` patches the OCA `base_sparse_field` Serialized field type **process-wide** at import time. A real migration/debugging surprise.

## 9. Upgrade risk zones (suite-wide)

- Triple discount: v19 redesign moves `discount`→`discount1` semantics on the SALE side. Integrations reading `sale.order.line.discount` change meaning. See [[v19_changes]] §5.
- `section.line.mixin` materialized-path tree: any migration touching invoice/SO lines with raw SQL must preserve `parent_path`.
- Data export monkeypatching: on a v19 migration, the `_patch_method` / `Serialized.convert_to_cache` patches must be re-validated against the new core ORM internals.
- SQL CHECK constraints on discount fields → in v19 must become `models.Constraint` or they're silently lost (IGV19-1).

---

## Cross-references
- Catalog → [[catalog]] · Gotchas → [[gotchas]] · v19 → [[v19_changes]]
- Deep architecture / migration / code patterns → **`indaws-core-developer`** skill (do not duplicate here)
- inBuilt builds on this suite → [[../inbuilt/integrations]]
- OCA overlap → [[../oca-accounting/integrations]] §7
- Triple discount math → functional/account; analytic indaws → cross-module/analytic/indaws
