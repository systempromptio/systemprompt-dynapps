# OCA Accounting Modules — Odoo 18.0

What OCA adds over standard Odoo accounting. 9 repos, ~120 modules.
Source: `OCA/v18/` — excludes l10n-spain (separate analysis).

> **Gotchas** (48 source-verified, file:line) → [[gotchas]]. **How OCA patches/conflicts with the core + Indaws** → [[integrations]]. **l10n-spain** (AEAT/SII/Verifactu/Facturae) → [[l10n_spain]]. **v18→v19 survival** → [[v19_changes]].


## 1. account-financial-tools — Assets, Chart Update, Cash/Check Management

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_asset_management` | Full asset lifecycle: profiles, groups, depreciation lines, batch compute, revaluation, transfer (AUC→Fixed). Replaces standard `account_asset`. | **High** |
| `account_chart_update` | Detect + apply chart of accounts template changes without losing data | Medium |
| `account_fiscal_year` | Explicit fiscal year model (not just calendar year). Overlap validation. | Medium |
| `account_cash_deposit` | Physical cash → bank deposit workflow with cash unit definitions | **High** |
| `account_check_deposit` | Check → bank deposit tracking. In-hand check account. | **High** |
| `account_asset_compute_batch` | Cron-based batch depreciation computation | Medium |
| `account_asset_transfer` | Transfer assets between categories (AUC → Fixed Asset) | Medium |
| `account_fiscal_position_vat_check` | VAT validation when applying fiscal position | Low |

**Path:** `OCA/v18/account-financial-tools/`


## 2. account-invoicing — Invoice Enhancements

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_invoice_refund_link` | Visual link between invoice ↔ credit note | Low |
| `account_invoice_section_sale_order` | Auto-sections by SO on multi-SO invoices | Medium |
| `account_global_discount` | Company-wide discount rules with per-tax-rate splitting | Medium |
| `account_invoice_pricelist` | Track which pricelist was used on invoice | Low |
| `account_invoice_fixed_discount` | Fixed € discount (not just %) on invoice lines | **Medium** |
| `account_invoice_date_due` | Override due date independently of payment terms | Medium |
| `account_invoice_check_total` | Manual total verification before posting | Low |
| `account_invoice_crm_tag` | CRM tags on invoices for cross-functional reporting | Low |
| `account_billing` | Group invoices by billing cycle (monthly/project) | **Medium** |
| `account_invoice_fiscal_position_update` | Change fiscal position on posted invoice + recalculate | Medium |

**Path:** `OCA/v18/account-invoicing/`


## 3. account-payment — Due List, Returns, Extended Terms

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_due_list` | Dedicated AR/AP aging list by due date | Medium |
| `account_payment_return` | Payment return/bounce management (reversal, tracking, reasons) | **High** |
| `account_payment_return_import` | Import bank bounce files (extensible per format) | Medium |
| `account_payment_term_extension` | Weeks/months delay, rounding, multiple payment days, untaxed basis | **Medium** |
| `account_payment_promissory_note` | Promissory note as payment instrument | Low |
| `account_payment_notification` | SMS/email payment reminders | Low |
| `account_cash_invoice` | Pay invoice directly from bank statement line | Low |

**Path:** `OCA/v18/account-payment/`


## 4. bank-payment — SEPA, Payment Orders, Mandates

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_payment_order` | **Payment order workflow**: draft→open→generated→uploaded. Batch payments to bank. | **Critical** |
| `account_payment_mode` | Payment modes (bank transfer, check, draft) linked to journals | **High** |
| `account_banking_mandate` | SEPA DD mandates: reference, expiration, partner bank validation | **High** |
| `account_banking_pain_base` | PAIN XML generation base (ISO 20022) | **High** |
| `account_banking_sepa_credit_transfer` | SEPA CT file export (PAIN 001.002.03) | **Critical** |
| `account_banking_sepa_direct_debit` | SEPA DD file export (PAIN 008.003.02) + mandate cron | **Critical** |
| `account_payment_partner` | Payment mode on partner/invoice, extends report | Medium |
| `account_invoice_select_for_payment` | Batch-select invoices for payment order | Low |

**Path:** `OCA/v18/bank-payment/`

**Note:** Indaws already uses `account_banking_sepa_credit_transfer` + `account_banking_sepa_direct_debit`.


## 5. bank-statement-import — File + Online Bank Sync

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_statement_import_file` | Base wizard for file upload/parse | **High** |
| `account_statement_import_camt` | CAMT 053/054 XML import (ISO 20022 — European standard) | **Critical** |
| `account_statement_import_ofx` | OFX format import (US/legacy) | Medium |
| `account_statement_import_online` | Online bank sync framework (async providers) | **High** |
| `account_statement_import_online_gocardless` | GoCardless Open Banking connector | **High** |
| `account_statement_import_online_ponto` | Ponto/Isabel Open Banking connector (European banks) | **High** |
| `account_statement_import_sheet_file` | Excel/CSV import with configurable column mapping | Medium |
| `account_statement_import_move_line` | Convert existing journal items to statement lines | Low |

**Path:** `OCA/v18/bank-statement-import/`


## 6. account-reconcile — Reconciliation (CE Alternative to Enterprise)

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_reconcile_oca` | **Full reconciliation widget** — CE alternative to Enterprise. JS/SCSS frontend. Auto-rules. | **Critical** for CE |
| `account_reconcile_model_oca` | Reconciliation rule engine (match by amount, partner, tags) | **High** |
| `account_partner_reconcile` | Partner-level AR/AP reconciliation view | Medium |
| `account_in_payment` | "In Payment" intermediate status for items being paid | Low |
| `account_move_base_import` | GL entry import from XLSX/XLS | Medium |
| `account_reconcile_oca_queue` | Async reconciliation via queue_job (large datasets) | Medium |

**Path:** `OCA/v18/account-reconcile/`


## 7. account-financial-reporting — Reports (CE Must-Have)

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_financial_report` | **Trial Balance, General Ledger, Journal Ledger, Aged Partner Balance, Open Items, VAT Report** — all with XLSX export | **Critical** for CE |
| `account_tax_balance` | Tax detail report by period (for VAT declarations) | **High** |
| `partner_statement` | Customer/vendor statements (activity, outstanding, aging buckets) — PDF + XLSX | **High** |
| `mis_template_financial_report` | Pre-built P&L + Balance Sheet for MIS Builder | Medium |

**Path:** `OCA/v18/account-financial-reporting/`


## 8. account-analytic — Analytic Enhancements

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_analytic_required` | Force analytic distribution on specified GL accounts | **Medium** |
| `account_analytic_parent` | Restore parent-child hierarchy (v17+ regression) | Medium |
| `account_analytic_sequence` | Auto-numbering for analytic accounts | Low |
| `account_analytic_tag` | Analytic tags (removed from standard in v17+) | Medium (legacy) |
| `account_move_analytic_link` | Smart button: journal entry → analytic lines | Low |

**Path:** `OCA/v18/account-analytic/`


## 9. account-closing — Period End Accruals

| Module | What it adds | Value |
|--------|-------------|-------|
| `account_cutoff_base` | Accrual/prepaid cutoff framework with tax support | **High** |
| `account_cutoff_picking` | Auto-create cutoffs from unshipped POs / uninvoiced SOs | **High** |
| `account_invoice_start_end_dates` | Start/end date on invoice lines (for cutoff calculation) | Medium |
| `account_fiscal_year_closing` | FY closing wizard (closing moves, archival, balance prep) | Medium |

**Path:** `OCA/v18/account-closing/`


## What Standard Odoo CE Lacks (Filled by OCA)

| Gap in CE | OCA Solution |
|-----------|-------------|
| No reconciliation UI | `account_reconcile_oca` |
| No financial reports (Trial Balance, GL, Aged) | `account_financial_report` |
| No payment order workflow | `account_payment_order` + SEPA suite |
| No bank statement import (CAMT/OFX) | `account_statement_import_*` |
| No online bank sync | `account_statement_import_online_*` |
| No asset management (CE) | `account_asset_management` |
| No payment return tracking | `account_payment_return` |
| No period-end cutoffs | `account_cutoff_*` |
| No customer/vendor statements | `partner_statement` |
| No fiscal year definition | `account_fiscal_year` |
| Single % discount only | `account_invoice_fixed_discount` |
| No billing cycles | `account_billing` |


## Indaws Overlap

| OCA Module | Indaws Equivalent | Priority |
|-----------|-------------------|----------|
| `account_banking_sepa_credit_transfer` | Used directly from OCA | OCA ✅ |
| `account_banking_sepa_direct_debit` | Used directly from OCA | OCA ✅ |
| `account_reconcile_oca` | Not installed (uses Enterprise?) | OCA if CE |
| `account_invoice_fixed_discount` | `indaws_account_triple_discount` (more features) | Indaws |
| `account_global_discount` | `indaws_sale_pricelist_global_rule` (different scope) | Both |
| `account_invoice_section_sale_order` | `indaws_invoice_section` (more flexible) | Indaws |
