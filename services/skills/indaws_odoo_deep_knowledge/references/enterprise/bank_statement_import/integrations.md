# Bank Statement Import — Integrations

How the bank-statement-import family connects to the rest of Odoo. Focus on the reconciliation bridge, the bank rec widget, OCR, and localization formats.

---

## Bridges (oficiales)

### ↔ account_accountant (reconciliation engine + bank rec widget) — the core dependency

`account_bank_statement_import` **depends on `account_accountant`** (manifest). It does not implement reconciliation; it produces `account.bank.statement.line` records and hands them to the widget.

**Pipeline**:
```
journal._import_bank_statement(attachments)   [account_journal.py:33]
  └─ parse → create account.bank.statement(.line)
  └─ line_ids._cron_try_auto_reconcile_statement_lines(limit_time)   [:83]
       └─ (method defined in account_accountant) attempts rule-based auto-match
  └─ account.bank.statement.line._action_open_bank_reconciliation_widget(
         extra_domain=[('statement_id','in', statements.ids)],
         default_context={'search_default_not_matched': True, ...})   [:85-92]
```

- `_action_open_bank_reconciliation_widget` is **overridden** in this module only to inject the no-content help text ("Click New or upload a CAMT/CSV/OFX/QIF") listing installed formats (`account_bank_statement_import/models/account_bank_statement.py:21-28`, `AccountBankStatementLine._action_open_bank_reconciliation_widget`). The actual widget action is `account_accountant`'s.
- Immediate auto-reconcile after import means lines may already be matched when the widget opens; the default filter is `not_matched` so the user sees only the leftovers.
- `limit_time` derives from `tools.config['limit_time_real_cron']`, clamped to `0 < x < 180` else 180 (`:80-83`).

### ↔ bank reconciliation widget (frontend, account_accountant)

This module ships JS/XML assets that patch the widget's list/kanban/finish-buttons:
`account_bank_statement_import/static/src/bank_reconciliation/{list,kanban,finish_buttons}.{js,xml}`.
They surface the file-uploader inside the rec widget so users can import more files mid-reconciliation. The journal dashboard kanban also gets the `account_file_uploader` widget on bank journals (`account_bank_statement_import/views/account_bank_statement_import_view.xml:4-15`, `journal_dashboard_view_inherit`).

### ↔ account_bank_statement_extract (OCR / IAP)

Bridge that adds scan digitization. Architecture:
```
account.bank.statement  _inherit ['extract.mixin', 'account.bank.statement']   [extract/models/account_bank_statement.py:9]
journal._import_bank_statement overridden:                                       [extract/models/account_journal.py:32-55]
   PDF/JPEG/PNG-only batch → create statement(attachment) → _send_batch_for_digitization()
   else → super() (normal CAMT/CSV/OFX/QIF path)
OCR result → _fill_document_with_results: writes balances/date + creates line_ids   [:37-61]
          → triggers server action account_accountant.auto_reconcile_bank_statement_line._trigger()
```
- Depends `accountant`, `account_bank_statement_import`, `iap_extract`, `account_extract`.
- Endpoint: `https://extract.api.odoo.com/api/extract/bank_statement/1/...`, `OCR_VERSION=100` (`:4, 31-35`).
- Company toggle `extract_bank_statement_digitalization_mode` (`no_send`/`auto_send`, default auto_send) → settings UI `account_invoice_extract/views/res_config_settings_views.xml:11`, `account_ocr_settings` `. See BSIG18.
- `is_in_extractable_state = not line_ids` (`:11-13`) — once lines exist (parsed or OCR-filled) the statement is no longer re-sendable.

### ↔ l10n bank formats / localizations

- **CAMT.053** is the SEPA-recommended cash management format and the one localizations lean on for EU. The CAMT parser is namespace-version-agnostic (`namespace-uri(.)`), so camt.053.001.02/.04/.08 variants parse without a localization module.
- **FR** special cases baked into base: BNP (27↔11 char) and Crédit Lyonnais/LCL (27↔7) account-number substring matching in `_statement_import_check_bank_account` (`account_bank_statement_import/models/account_journal.py:157-168`, `AccountJournal._statement_import_check_bank_account`). CH non-unique account numbers handled by `split(" ")[0]`.
- **ES Norma 43**: NOT in EE. Provided by OCA `l10n-spain/l10n_es_account_statement_import_n43` which sits on the **OCA** `account_statement_import_file` stack (model `account.statement.import`, depends `chardet`), a parallel architecture to EE. Do not mix carelessly. See "Gotcha cross-module" and core.md.

---

## Diagrama

```
                    ┌─────────────────────────────────────────┐
   bank file        │  account.journal (extended per module)  │
  (CAMT/CSV/        │  create_document_from_attachment        │
   OFX/QIF) ───────▶│      └─ _import_bank_statement           │
                    │           └─ _parse_bank_statement_file  │◀── chain of responsibility
                    └───────────────┬─────────────────────────┘     (camt/ofx/qif _check_*)
   PDF/JPEG/PNG ─────┐              │ triplet(s)
                     ▼              ▼
         account_bank_statement_extract     _create_bank_statements
           (extract.mixin → IAP OCR)                │
                     │  bank_statement_lines        │ filter dups (unique_import_id),
                     ▼                               ▼ drop amount==0
              ┌──────────────────────────────────────────┐
              │ account.bank.statement / .line           │
              │  + unique_import_id (UNIQUE)              │
              └───────────────┬──────────────────────────┘
                              │ _cron_try_auto_reconcile_statement_lines
                              ▼
              ┌──────────────────────────────────────────┐
              │ account_accountant: bank rec widget       │
              │  auto_reconcile_bank_statement_line (srv)  │
              │  → account.move / reconciliation           │
              └──────────────────────────────────────────┘

  ES Norma 43 ✗ EE  ──▶  OCA l10n_es_account_statement_import_n43 (separate account.statement.import stack)
```

---

## Modelos (clave para extender)

| Model | Module | Notes |
|-------|--------|-------|
| `account.journal` | import (+ each format) | All logic; override `_parse_bank_statement_file` / `_get_bank_statements_available_import_formats` to add a format. |
| `account.bank.statement.line` | import | `unique_import_id` Char + UNIQUE constraint (v18 `_sql_constraints`; v19 `models.Constraint`). |
| `account.bank.statement` | extract | `extract.mixin` (v18) / `extract.mixin.with.words` (v19). |
| `res.company` | extract | `extract_bank_statement_digitalization_mode`. |
| `account.setup.bank.manual.config` | import | defaults `bank_statements_source='file_import'`. |
| `base_import.import` | csv | CSV column mapping (no dedicated model). |

---

## Config

1. **Install** a format module (CAMT/CSV/OFX/QIF) — all `auto_install=True`, so installing `account_accountant` + Enterprise pulls them in automatically. OFX additionally needs python `ofxparse` (BSIG13).
2. **Journal**: must have `default_account_id` set (BSIG15) and ideally `bank_account_id`/IBAN (mandatory for multi-IBAN CAMT, BSIG5). New bank journals default `bank_statements_source='file_import'`.
3. **QIF only**: per-journal `qif_date_format` (day_first default) + `qif_decimal_point` (BSIG12).
4. **OCR**: Settings → Accounting → `extract_bank_statement_digitalization_mode`. Default `auto_send` uploads scans to IAP (BSIG18); set `no_send` to disable. Requires IAP credits.
5. **Import**: drop the file on the bank journal dashboard card (`account_file_uploader`) — there is no menu (BSIG14).

---

## Gotchas cross-module (resumen, ver gotchas.md)

- **BSIG1/BSIG2** — `unique_import_id` is namespaced per journal+account; dedup is not global, and a fully-overlapping file is rejected wholesale.
- **BSIG5/BSIG6** — CAMT silently drops other-IBAN and other-currency statements.
- **BSIG10** — OFX ledger balance ≠ ending balance → rec end-balance mismatches.
- **BSIG18** — OCR auto-send is ON by default → scans leave the instance to Odoo IAP.
- **BSIG19** — auto-PDF on import in v18 only; removed v19.
- **BSIG21** — N43 unsupported on EE → needs OCA, which uses a DIFFERENT model (`account.statement.import`). Installing the OCA N43 stack alongside EE means two parallel import architectures touching `account.bank.statement(.line)`; verify no conflicting overrides and pick ONE entry path per journal.

---

## v19

API contract of `_parse_bank_statement_file` changed: now takes **raw bytes** (`raw_file`) and returns a **list of triplets** `[[currency, account, stmts_vals], ...]` to support multi-account files natively; the auto-PDF and post-import standalone auto-reconcile call were dropped (auto-reconcile is triggered inside statement creation via `auto_statement_processing` context). CSV adds a bank-rec-widget search filter for imported transactions. Extract switches to `extract.mixin.with.words`. Full detail in `v19_changes.md`.
