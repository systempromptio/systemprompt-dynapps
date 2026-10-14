# OCA v18→v19 — Massive Module Reduction

OCA repos lost 80%+ of modules in v19. Most are NOT YET PORTED, not intentionally removed. Check OCA GitHub for porting status.

## Module Survival Summary

| Repo | v18 | v19 | % Lost |
|------|-----|-----|--------|
| account-financial-tools | 37 | 6 | 84% |
| account-invoicing | 47 | 8 | 83% |
| account-payment | 20 | 3 | 85% |
| bank-payment | 20 | 9 | 55% |
| bank-statement-import | 15 | 4 | 73% |
| account-reconcile | 11 | 1 | 91% |
| account-financial-reporting | 7 | 3 | 57% |
| account-analytic | 23 | 3 | 87% |
| account-closing | 7 | 1 | 86% |
| **l10n-spain** | **49** | **9** | **82%** |
| sale-workflow | 130+ | 27 | 79% |
| purchase-workflow | 70+ | 13 | 81% |

## What SURVIVED in v19

### account-financial-tools (6/37)
`account_journal_restrict_mode`, `account_move_name_sequence`, `account_move_post_date_user`, `account_move_print`, `account_usability`, `account_account_tag_code` (NEW)
**LOST critical:** `account_asset_management`, `account_chart_update`, `account_fiscal_year`, `account_cash_deposit`, `account_check_deposit`

### account-invoicing (8/47)
`account_global_discount`, `account_invoice_fixed_discount`, `account_invoice_pricelist`, `account_invoice_pricelist_sale`, `account_invoice_refund_link`, `account_invoice_triple_discount`, `portal_account_personal_data_only`, `sale_order_invoicing_qty_percentage`
**LOST critical:** `account_billing`, `account_invoice_section_sale_order`, `account_invoice_date_due`, `account_invoice_fiscal_position_update`

### account-payment (3/20)
`account_check_printing_report_base`, `account_due_list`, `account_payment_method_base`
**LOST critical:** `account_payment_return`, `account_payment_term_extension`, `account_payment_promissory_note`

### bank-payment (9/20)
`account_banking_mandate`, `account_banking_mandate_sale`, `account_banking_pain_base`, `account_banking_sepa_direct_debit`, `account_payment_mode`, `account_payment_order`, `account_payment_order_grouped_output`, `account_payment_purchase`, `account_payment_sale`
**LOST:** `account_banking_sepa_credit_transfer`, `account_payment_partner`, `account_invoice_select_for_payment`

### bank-statement-import (4/15)
`account_statement_import_base`, `account_statement_import_camt`, `account_statement_import_file`, `account_statement_import_online`
**LOST:** ALL specific online providers (GoCardless, Ponto, Stripe, Wise, PayPal), OFX, sheet import

### account-reconcile (1/11)
`account_statement_base` only
**LOST critical:** `account_reconcile_oca` (CE reconciliation widget), `account_reconcile_model_oca`, `account_partner_reconcile`

### account-financial-reporting (3/7)
`account_financial_report`, `account_tax_balance`, `partner_statement`
**LOST:** `mis_template_financial_report`, `mis_builder_cash_flow`

### account-analytic (3/23)
`account_analytic_tag`, `account_move_update_analytic`, `purchase_analytic`
**LOST critical:** `account_analytic_required`, `account_analytic_parent`, `account_analytic_sequence`

### account-closing (1/7)
`account_invoice_start_end_dates` only
**LOST critical:** `account_cutoff_base`, `account_cutoff_picking`, `account_fiscal_year_closing`

## l10n-spain (9/49) — Spanish Localization

### Survived
`l10n_es_aeat` (base), `l10n_es_aeat_mod111`, `l10n_es_aeat_mod115`, `l10n_es_aeat_mod123`, `l10n_es_aeat_mod347`, `l10n_es_aeat_partner_check`, `l10n_es_partner`, `l10n_es_toponyms`, `l10n_es_account_statement_import_n43`

### LOST (Critical for Spain)
- **`l10n_es_aeat_mod303`** — IVA trimestral (OBLIGATORIO)
- **`l10n_es_aeat_mod390`** — resumen anual IVA
- **`l10n_es_aeat_mod349`** — intracomunitarias
- **`l10n_es_aeat_sii_oca`** — SII (OBLIGATORIO grandes empresas)
- **`l10n_es_verifactu_oca`** — Verifactu (NUEVA OBLIGACION)
- **`l10n_es_facturae` + `_face`** — Facturae/FACe (obligatorio B2G)
- **`l10n_es_vat_book`** — libro de IVA
- **`l10n_es_vat_prorate`** — prorrata IVA
- **`l10n_es_aeat_mod190`** — resumen retenciones anual
- **`payment_redsys`** — pasarela pago
- **All delivery carriers** — SEUR, GLS, DHL

**IMPACTO:** Empresa espanola en v19 con OCA pierde IVA trimestral, SII, Verifactu, Facturae, libro IVA. **BLOQUEANTE** hasta que se porten.

## sale-workflow (27/130+)
**Key survivors:** `sale_automatic_workflow`, `sale_blanket_order`, `sale_by_packaging`, `sale_commercial_partner`, `sale_delivery_date`, `sale_discount_display_amount`, `sale_exception`, `sale_force_invoiced`, `sale_last_price_info`, `sale_order_general_discount`, `sale_order_type`, `sale_procurement_group_by_line`, `sale_product_set`, `sale_quotation_number`, `sale_stock_picking_blocking`, `sale_stock_reference_by_line` (NEW), `sale_cancel_restrict` (NEW), `sale_fully_invoiced` (NEW)
**LOST critical:** `sale_invoice_plan`, `sale_order_revision`, `sale_mrp_bom`, `sale_pricelist_*`, `sale_order_product_recommendation`, `sale_sourced_by_line`

## purchase-workflow (13/70+)
**Key survivors:** `purchase_advance_payment`, `purchase_blanket_order`, `purchase_deposit`, `purchase_exception`, `purchase_force_invoiced`, `purchase_last_price_info`, `purchase_order_approved` (NEW), `purchase_order_type`, `purchase_order_weight_volume` (NEW), `purchase_partner_incoterm`, `purchase_stock_price_unit_sync`
**LOST critical:** `purchase_request_*` (all 4), `purchase_requisition_*`, `purchase_delivery_split_date`, `purchase_invoice_plan`

## Key Takeaway

v19 OCA is in EARLY PORTING phase. For migration planning:
1. Check each module's GitHub issue/PR for porting status
2. Budget for manual porting of critical unported modules
3. l10n-spain is most impacted — mod303, SII, Verifactu are ESSENTIAL and missing
4. Some functionality may now be in Odoo core v19
