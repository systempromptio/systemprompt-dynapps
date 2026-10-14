<!-- verified: v18.0 @ 2026-06 (indaws src) -->
# Indaws Module Catalog — Odoo 18.0

154 Indaws modules (v18 tree) on top of standard Odoo CE + Enterprise. Source: `indawsodoo/18/`

> The **Module Map by Domain** below highlights the most distinctive modules per domain (not exhaustive). The **Full Module Index** at the end of this file lists all 154 modules with their manifest `summary` (verified against source).

> **How the suite patches the core** (auto_install cascade, create/write/compute overrides, triple-discount sale≠account asymmetry, section.line.mixin tree, ETL monkeypatching) → [[integrations]]. Deep architecture/migration/code patterns live in the **`indaws-core-developer`** skill (not duplicated here).

> **v19:** functionally near-identical to v18, but a technical migration is in progress (~72/159 migrated, +2 new modules `indaws_dataflow`/`indaws_hr_timesheet`, triple-discount redesign, `models.Constraint` adoption, `Domain` API). This catalog is the v18 canonical list — for v19 deltas, migration status and v19 gotchas see [[v19_changes]].

## Architecture: 5 Layers

```
Layer 1: indaws_base → root module, auto-installs all indaws_*
Layer 2: indaws_base_* (10 modules) → foundation per domain (account, sale, stock, purchase, mrp, pos, hr, crm, product, website)
Layer 3: indaws_{domain} → main extensions (indaws_sale, indaws_stock, indaws_account, etc.)
Layer 4: indaws_{domain}_{feature} → specific features (triple_discount, rappel, composition, etc.)
Layer 5: indaws_{domain}_{bridge} → cross-domain bridges (sale_stock_picking_invoice, purchase_stock_valued_picking, etc.)
```

All modules with `auto_install=True` install automatically when their dependencies are present.

## Module Map by Domain

### Stock (9 modules)

| Module | What it adds |
|--------|-------------|
| `indaws_stock_available_immediately` | `immediately_usable_qty` computed field — ATP (available-to-promise) logic |
| `indaws_stock_removal_priority` | `removal_priority` Integer on stock.location — custom removal when dates equal |
| `indaws_stock_valued_picking` | Valuation fields on pickings (subtotal, total, demand) + delivery slip report |
| `indaws_stock_picking_invoice` | Create invoices directly FROM pickings (not just from SO/PO) |
| `indaws_stock_delivery_portal` | Customer portal: delivery confirmation with signature + DNI + photo capture |
| `indaws_stock_portal_picking` | Customer portal: view/manage delivery orders |
| `indaws_delivery_state_tracking` | Carrier delivery states (pending→in_transit→delivered→failed) + tracking history |
| `indaws_stock_base`, `indaws_stock`, `indaws_stock_account` | Foundation layers |

### Sale (20+ modules)

| Module | What it adds |
|--------|-------------|
| `indaws_sale_approval` | Approval workflow: rules + responsible users + thresholds + cron |
| `indaws_sale_order_type` | Custom order types: sequences, journals, no_invoice flag, defaults |
| `indaws_sale_order_type_manual_delivery` | Manual delivery creation per order type |
| `indaws_sale_order_revision` | Track SO versions (revision_id, state per revision) |
| `indaws_sale_invoice_plan` | Planned invoicing wizard (independent of delivery) |
| `indaws_sale_product_composition` | Configurable product bundles with templates + options + portal |
| `indaws_sale_product_composition_print` | Print composition details |
| `indaws_sale_product_composition_timesheet` | Timesheet tracking for composition items |
| `indaws_sale_triple_discount` | 3-tier discount (discount1 * discount2 * discount3 → aggregated) |
| `indaws_sale_pricelist_global_rule` | Global pricelist rules (bypass standard logic) |
| `indaws_sale_restricted_qty` | Max qty/UoM restrictions per product/category/customer |
| `indaws_sale_stock_picking_invoice` | Invoice from picking (sale flow) |
| `indaws_sale_stock_valued_picking` | Valued picking in sale context |
| `indaws_sale_rappel` | Volume rebates for sales (see Rappel section) |
| `indaws_sale_margin` | Cost categories + templates + per-line cost tracking |
| `indaws_sale_purchase` | Bi-directional SO<->PO linking + wizard |
| `indaws_sale_purchase_requisition` | SO → purchase requisition bridge |
| `indaws_sale_mrp_bom` | MRP BOM integration with sales |
| `indaws_sale_document` | Document management on SO |
| `indaws_sale_detail_view`, `indaws_sale_line_print`, `indaws_sale_order_line_code`, `indaws_sale_order_line_edit` | UI enhancements |

### Purchase (8 modules)

| Module | What it adds |
|--------|-------------|
| `indaws_purchase_triple_discount` | 3-tier discount for purchase (mirrors sale) |
| `indaws_purchase_supplier` | Supplier evaluation: types, concepts, scoring, points |
| `indaws_purchase_account` | Purchase advance payments + deposits (down payments for PO) |
| `indaws_purchase_stock_picking_invoice` | Invoice from picking (purchase flow) |
| `indaws_purchase_stock_valued_picking` | Valued picking for purchase receipts |
| `indaws_purchase_rappel` | Volume rebates from vendors (see Rappel section) |
| `indaws_purchase_sale_margin_project` | Cross SO→PO margin tracking per project |
| `indaws_purchase_stock` | Lot/move line extensions for purchase receipts |

### POS (10 modules)

| Module | What it adds |
|--------|-------------|
| `indaws_pos` | Customer discount, force pricelist, optional products, print averages |
| `indaws_pos_analytic` | Analytic distribution config for POS |
| `indaws_pos_cashkeeper` | **CashKeeper terminal** — socket communication + middleware, proactive monitoring |
| `indaws_pos_location` | **Multi-location stock visibility** — show stock across locations, location-based picking |
| `indaws_pos_lot_auto_selection` | **Auto lot selection** — FIFO/LIFO based on removal strategy |
| `indaws_pos_loyalty` | Loyalty with usage date ranges |
| `indaws_pos_paytef` | PayTEF payment terminal |
| `indaws_pos_receipt` | Receipt capture as image + PDF + WhatsApp send |
| `indaws_l10n_es_pos` | Spanish simplified invoice (ticket) |
| `indaws_base_pos` | Foundation |

### Account (10 modules)

| Module | What it adds |
|--------|-------------|
| `indaws_account` | Invoice sections/notes, payment term context, custom reports |
| `indaws_account_triple_discount` | **Triple Discount Mixin** — 3 cascading discount fields on invoice lines |
| `indaws_account_invoice_exchange` | Invoice exchange/refund wizard + POS integration |
| `indaws_account_move_merge` | **Merge multiple account moves** into one |
| `indaws_account_payment_paytef` | PayTEF payment integration |
| `indaws_account_edi` | EDI document enhancements |
| `indaws_l10n_es_edi_sii` | **SII** improvements (Spanish tax reporting) |
| `indaws_l10n_es_edi_verifactu` | **Veri*Factu** (Spanish real-time invoice validation) |
| `indaws_l10n_es_account_ipc` | **IPC** (Consumer Price Index) configuration for invoicing |
| `indaws_account_retention` | Retention/withholding on invoices (delegated by inBuilt certifications) |
| `indaws_mail_account` | Send invoices to CC/BCC partners |
| `indaws_base_account` | Foundation |

## Unique Indaws Features (Not in Standard Odoo)

### Rappel System (Volume Rebates)

3 modules: `indaws_rappel` (core) + `indaws_sale_rappel` + `indaws_purchase_rappel`

```
rappel → rappel_section (tiers) → rappel_settlement → credit note
  ├── Calculation: fixed / variable (tiered) / formula
  ├── Basis: amount or quantity
  ├── Periods: monthly, quarterly, yearly
  ├── Settlements: auto-computed → generates credit notes
  └── Evolution tracking over time
```

**Models:** `rappel`, `rappel.settlement`, `rappel.settlement.line`, `rappel.section`, `rappel.evolution`, `rappel.partner`

Standard Odoo has loyalty (points) but not periodic commercial rebates based on cumulative purchases/sales.

### Triple Discount

4 modules across sale/purchase/account:

```
line.discount1 = 10%
line.discount2 = 5%
line.discount3 = 2%
→ aggregated_discount = 1 - (1-0.10)(1-0.05)(1-0.02) = 16.29%
```

Standard Odoo has a single `discount` field (0-100%). Triple discount is common in B2B industrial/wholesale.

### Product Composition

4 modules: `indaws_sale_product_composition` + print + timesheet + (portal)

**Architecture:** Operates at SO line level via `composition_uuid`/`composition_parent_uuid` parent-child linking. Propagates to invoice lines with namespaced UUIDs. Independent of BOM/MRP.

```
product.template
  ├── composition_product_ids → product.composition (mixin-based)
  │     Fields: product_id, quantity, uom_id, discount, price_type (pricelist|fixed)
  │     Flags: link_qty, link_delivered, mandatory, by_default, repeated
  └── composition_configure_new → enables adding new compositions from SO

SO line with has_comp=True → _compute_comp_lines() creates child SOLs
  ├── link_qty=True → child qty = composition.qty × parent qty (auto-sync)
  ├── link_delivered=True → child qty_delivered_method='parent_composition'
  ├── mandatory=True → child cannot be deleted
  ├── Recursive: children with compositions nest sub-children
  └── Configurator wizard for optional/new compositions

Works on ANY product type (storable, service, consumable) — _compute_has_comp checks composition_ids, NOT service_tracking.
```

**Key pattern — Mixed VAT with single delivery item:**
Storable product (0€, manufacturing BOM) + service compositions per tax rate (carrying actual prices, `link_delivered=True`). Picking shows only the storable; invoice shows tax-split lines.

### Picking-to-Invoice

3 modules: `indaws_stock_picking_invoice` + `indaws_sale_stock_picking_invoice` + `indaws_purchase_stock_picking_invoice`

```
Stock Picking → "Create Invoice" button → generates account.move
  ├── Links: picking.invoice_ids (M2M)
  ├── Supports: partial invoicing from picking
  └── Works for: sales, purchases, and standalone pickings
```

Standard invoicing goes SO→Invoice or PO→Invoice. Picking-to-invoice skips the order.

### Data Import/Export ETL

2 modules: `indaws_data_import` + `indaws_data_export`

```
Import (god class: indaws.import, ~1600 lines):
  Template → Parser (CSV/Excel/JSON/txt) → Variables (field mapping)
  → Scheduler (backing ir.cron) → queue.job (OCA, async) → state machine driven by job 'done'

Export:
  Template → Variables → Parameters → Registry (audit) → Log
  ⚠️ _register_hook MONKEYPATCHES create/write/unlink on arbitrary target models
```

Standard Odoo has basic CSV wizard. Indaws provides enterprise-grade ETL with scheduling, async (OCA `queue_job`), error handling, and retry logic. **Friction:** the export engine patches ORM methods on any target model process-wide — see [[gotchas]] IG31/IG32 and [[integrations]] §8.

### REST API

`indaws_api`

```
api.endpoint → api.access.token (JWT)
  ├── Configurable endpoints
  ├── Endpoint inheritance
  └── Token-based authentication
```

v18 has JSON-RPC only. v19 introduces JSON2. Indaws provides a REST API layer with JWT.

### Delivery Portal

`indaws_stock_delivery_portal` + `indaws_stock_portal_picking`

```
Customer portal → Delivery confirmation
  ├── Digital signature capture
  ├── DNI (ID number) capture
  ├── Photo capture
  ├── Barcode scanning
  └── Carrier state tracking integration
```

Standard portal shows delivery status. Indaws adds interactive confirmation with legal compliance (signature + ID).

## Key Differentiators vs Standard Odoo

| Feature | Standard Odoo | Indaws Adds |
|---------|---------------|-------------|
| Discounts | Single % field | Triple cascading discount |
| Volume rebates | None (loyalty only) | Full rappel engine with settlements |
| Product configurator | Studio/custom | Product composition templates |
| Invoice from picking | Not possible | Direct picking→invoice |
| POS lot selection | Manual | Auto FIFO/LIFO selection |
| POS multi-location | Single warehouse | Stock visibility across locations |
| Data import | CSV wizard | ETL with scheduling + async |
| REST API | JSON-RPC only | JWT REST API endpoints |
| Delivery portal | Read-only status | Signature + photo + DNI capture |
| Supplier evaluation | None | Scoring + evaluation framework |
| Sale approval | None native (CE) | Rules + thresholds + cron |
| Order types | None | Custom types with behaviors |
| Order revisions | None | Version tracking |
| Spanish compliance | SII base | Enhanced SII + Verifactu + IPC |
| Payment terminals | Stripe/Adyen | CashKeeper + PayTEF |
| DOCX reports | PDF only | Word document generation |

## Full Module Index (154 modules, verified against v18 source)

One line per module = manifest `summary` (empty summary → short note from category/depends). Verified 2026-06 against `indawsodoo/18/*/__manifest__.py`.

### Base / Foundation (auto_install roots)

| Module | Summary / role |
|--------|----------------|
| `indaws_base` | Root module (Hidden) — auto-installs the suite, depends `base_setup` |
| `indaws_base_account` | Foundation: account |
| `indaws_base_crm` | Foundation: crm |
| `indaws_base_delivery` | Common delivery fields and wizards for all carrier modules |
| `indaws_base_fleet` | Foundation: fleet |
| `indaws_base_hr` | Foundation: hr |
| `indaws_base_hr_attendance` | Foundation: hr_attendance |
| `indaws_base_mass_mailing` | Foundation: mass_mailing |
| `indaws_base_mrp` | Foundation: mrp |
| `indaws_base_payment` | Foundation: payment |
| `indaws_base_pos` | Foundation: point_of_sale |
| `indaws_base_product` | Foundation: product + stock |
| `indaws_base_project` | Foundation: project |
| `indaws_base_purchase` | Foundation: purchase |
| `indaws_base_sale` | Foundational Sale module: shared menu, config settings hook, base extensions |
| `indaws_base_stock` | Foundation: stock (via indaws_base_product) |
| `indaws_base_survey` | Foundation: survey |
| `indaws_base_website` | Foundation: website |
| `indaws_base_website_sale` | Foundation: website_sale |
| `indaws_base_website_sale_stock` | Foundation: website_sale_stock |
| `indaws_security` | Suite security groups/rules (Hidden) |
| `indaws_storage` | Storage backend abstraction (Hidden) |
| `indaws_storage_s3` | S3 storage backend (Hidden) |

### Product

| Module | Summary / role |
|--------|----------------|
| `indaws_product` | Add brand management on products and variants |
| `indaws_product_data_feed` | Product data feed (website) |
| `indaws_product_data_feed_gmc` | Google Merchant Center product feed |
| `indaws_product_description_replace` | Show only the product description on document lines (base) |
| `indaws_product_description_replace_account` | …on invoice lines |
| `indaws_product_description_replace_purchase` | …on purchase order lines |
| `indaws_product_description_replace_sale` | …on sale order lines |
| `indaws_product_lot_sequence` | Lot/serial sequence on products |
| `indaws_product_supplierinfo_for_customer` | Define prices for customers in the products |
| `indaws_product_supplierinfo_for_customer_partner` | Per-partner customer pricing |
| `indaws_product_supplierinfo_for_customer_sale` | Loads customer code defined in product into each sale line |

### Sale (additional, beyond domain map)

| Module | Summary / role |
|--------|----------------|
| `indaws_sale` | Main sale extension (auto) |
| `indaws_sale_base` | Sale base layer (Hidden) |
| `indaws_sale_amazon_mrp` | Amazon integration for phantom BoM (kit) stock sync |
| `indaws_sale_amazon_mrp_available_immediately` | Amazon phantom BoM with immediately_usable_qty support |
| `indaws_sale_loyalty` | Loyalty integration for sales |
| `indaws_sale_margin_purchase` | Margin tracking with purchase data |
| `indaws_sale_order_type_analytic` | Analytic config per order type |
| `indaws_sale_order_type_sale_stock` | Order type + sale_stock bridge |
| `indaws_sale_product_composition_stock_picking` | Include composition children when invoicing from picking |
| `indaws_sale_purchase_stock` | SO↔PO with stock bridge |
| `indaws_sale_recurring_order` | Lightweight recurring orders without subscriptions |
| `indaws_sale_stock_picking_invoice_mrp` | Kit (phantom BoM) support for picking invoicing |

### Purchase (additional)

| Module | Summary / role |
|--------|----------------|
| `indaws_purchase` | Main purchase extension (auto) |

### Stock (additional)

| Module | Summary / role |
|--------|----------------|
| `indaws_stock_mrp` | Manage reordering rules for BoM components centrally |

### Website / eCommerce / Portal

| Module | Summary / role |
|--------|----------------|
| `indaws_website_sale` | Main website_sale extension |
| `indaws_website_sale_collect` | Click & collect integration |
| `indaws_website_sale_hide_price_partner` | Hide product prices per-partner (cart/checkout still work) |
| `indaws_website_sale_loyalty` | Loyalty on website sale |
| `indaws_website_sale_partner_categ_filter` | Restrict shop products by contact categories/tags (parent inheritance) |
| `indaws_website_sale_product_brand` | Brand filter on shop |
| `indaws_website_sale_restricted_qty` | Max qty restrictions on website |
| `indaws_website_sale_stock` | website_sale_stock extension |
| `indaws_b2b_portal_sale` | B2B portal core: per-contact permissions, profile templates, approval flow, checkout filtering |
| `indaws_portal_sale_edit_lines` | Portal users edit SO lines (qty, delete) per contact permission |
| `indaws_portal_sale_search` | Advanced search/filters/sort on portal orders + "Created By" column |

### Project

| Module | Summary / role |
|--------|----------------|
| `indaws_project` | Indaws Project extension |
| `indaws_project_stock_material` | Project stock material base |
| `indaws_project_mrp_stock_material` | Project material + MRP |
| `indaws_project_purchase_stock_material` | Project material + purchase |
| `indaws_project_sale_stock_material` | Project material + sale |
| `indaws_project_sale_margin_stock_material` | Project material + sale margin |

### CRM

| Module | Summary / role |
|--------|----------------|
| `indaws_crm_project` | CRM ↔ project link |
| `indaws_crm_prospection` | CRM prospection (Hidden) |

### HR

| Module | Summary / role |
|--------|----------------|
| `indaws_hr` | Indaws HR |
| `indaws_hr_timesheet_attendance` | HR timesheet/attendance base |
| `indaws_hr_attendance_publish` | Publish hr.attendance records to portal users by project |

### Reporting (DOCX)

| Module | Summary / role |
|--------|----------------|
| `indaws_report_docx` | Word (DOCX) report engine (Hidden) |
| `indaws_report_docx_account` | DOCX reports for account |
| `indaws_report_docx_iot` | DOCX reports + IoT |
| `indaws_report_docx_portal` | DOCX reports on portal |
| `indaws_report_docx_product` | DOCX reports for product |
| `indaws_report_docx_sale` | DOCX reports for sale |
| `indaws_report_docx_stock` | DOCX reports for stock |

### Mail / Marketing

| Module | Summary / role |
|--------|----------------|
| `indaws_mail` | inDAWS Mail |
| `indaws_mail_account` | Send invoices to CC/BCC partners |
| `indaws_mass_mailing` | Mass mailing + themes extensions |

### Payment

| Module | Summary / role |
|--------|----------------|
| `indaws_payment_paytef` | Add PayTEF payment method to website |
| `indaws_payment_order_modification` | Modification on payment orders |

### Data Import/Export (additional)

| Module | Summary / role |
|--------|----------------|
| `indaws_data_import_templates_product` | Predefined product import templates |

### Other / Cross-domain

| Module | Summary / role |
|--------|----------------|
| `indaws_loyalty` | Loyalty program usage-dates management |
| `indaws_survey` | Survey usage across other models with dynamic linking + response tracking |
| `indaws_invoice_section` | Adds sections from ResConfig settings grouped by picking transfer and sale order |
| `indaws_mobile` | Mobile API controller for Flutter applications |
| `indaws_helpdesk_rma` | Portal product returns (RMA) via helpdesk |
| `indaws_fleet_analytic` | Fleet ↔ analytic integration |
| `indaws_treasury_forecast` | Cash flow forecast: projected bank balance, receivables, payables, manual entries |

## Vertical: inBuilt (Construction Industry)

16 modules on top of indaws_core. Source: `vertical-odoo-inbuilt/`
Full documentation: `references/inbuilt/` (core, gotchas, integrations, config_guide)

**Key features:**
- Progressive certification billing (origin/current modes)
- Measurement-based pricing (L x A x H x Uds)
- FIEBDC3/BC3 budget import (Spanish construction standard)
- Project profitability analysis (30+ computed margin fields)
- Retention/withholding management on certifications
- Hierarchical task creation from SO (chapter/part/subpart)

**Critical indaws_core dependencies:**
- `indaws_sale_order_type` — adds `create_project` and `measurements` fields
- `indaws_sale_margin` — extends margin engine with indirect costs, reads `cost_summary_ids`
- `indaws_sale_order_line_code` — uses `parent_id`/`line_child_ids` for certification hierarchy
- `indaws_account_retention` — delegates retention invoice creation
- `indaws_sale_order_line_edit` — measurement UX depends on inline editing

See `references/inbuilt/integrations.md` section 7 for field-level dependency map.
