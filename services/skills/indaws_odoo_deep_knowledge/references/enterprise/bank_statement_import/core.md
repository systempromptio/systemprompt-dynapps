<!-- verified: v18.0 @ 2026-06 -->
# Bank Statement Import — Core (CAMT.053 / CSV / OFX / QIF + OCR)

Family of EE modules that turn bank-provided files into `account.bank.statement` + `account.bank.statement.line` records, then push the user into the bank reconciliation widget. All `OEEL-1`. `auto_install=True` on base/camt/csv/ofx — **NOT on QIF** (`account_bank_statement_import_qif/__manifest__.py` has no `auto_install` key, so QIF must be installed manually; the others pull in automatically once `account_accountant` is present).

| Module | Path | Adds |
|--------|------|------|
| `account_bank_statement_import` | `odoo_enterprise/18/account_bank_statement_import/` | Generic pipeline + chain-of-responsibility. Depends `account_accountant`, `base_import`. NO format on its own. |
| `account_bank_statement_import_camt` | `.../account_bank_statement_import_camt/` | CAMT.053 (SEPA XML). Depends import. |
| `account_bank_statement_import_csv` | `.../account_bank_statement_import_csv/` | CSV via `base_import` column mapping. Depends import + `base_import`. |
| `account_bank_statement_import_ofx` | `.../account_bank_statement_import_ofx/` | OFX (needs python `ofxparse`). |
| `account_bank_statement_import_qif` | `.../account_bank_statement_import_qif/` | QIF (Quicken). NOT auto_install — manual install required. Ships `views/account_journal_views.xml` for the per-journal QIF settings. |
| `account_bank_statement_extract` | `.../account_bank_statement_extract/` | OCR of PDF/JPEG/PNG scans via IAP. Depends `accountant`, import, `iap_extract`, `account_extract`. |

## Core models

| Model | Role |
|-------|------|
| `account.journal` (extended) | Holds ALL the import logic. No dedicated import wizard model — the journal IS the entry point. |
| `account.bank.statement.line` (extended) | Adds `unique_import_id` (Char, readonly, copy=False) with a UNIQUE sql constraint → dedup across imports. `account_bank_statement_import/models/account_bank_statement.py:15-19`, `class AccountBankStatementLine`. |
| `account.bank.statement` | Standard model (from `account_accountant`); statements are created from parsed vals. Extended by extract module with `extract.mixin`. |
| `account.setup.bank.manual.config` (extended) | `setup_wizards.py` — new bank journals default `bank_statements_source='file_import'` if any import format installed. |
| `base_import.import` (extended) | CSV only — reuses the generic Import wizard, not a custom one. |

There is NO `account.bank.statement.import` transient model. The legacy v13-era wizard was removed; entry is now the **file uploader widget** on the bank journal dashboard kanban (`account_file_uploader`, `account_bank_statement_import/views/account_bank_statement_import_view.xml:9-13`, `journal_dashboard_view_inherit`).

## Entry point: `create_document_from_attachment`

`account_bank_statement_import/models/account_journal.py:24-31`, `AccountJournal.create_document_from_attachment`. When a file is dropped on a bank/credit/cash journal, the generic `create_document_from_attachment` is overridden to call `journal._import_bank_statement(attachments)`. Other journal types fall through to `super()` (invoice digitization, etc.).

## Pipeline (`_import_bank_statement`, account_journal.py:33-102)

```
file(s) dropped on journal
  └─ for each attachment:
       1. _parse_bank_statement_file(attachment)        # format module parses, chain of responsibility
            → (currency_code, account_number, stmts_vals)
       2. _check_parsed_data(stmts_vals, account_number) # structural: ≥1 stmt, ≥1 transaction
       3. _find_additional_data(currency_code, acct_no)  # resolve res.currency + account.journal
       4. (journal must have default_account_id, else UserError)
       5. _complete_bank_statement_vals(...)             # partner/bank matching, unique_import_id namespacing
       6. _create_bank_statements(stmts_vals)            # filter dups, create statement + lines
       7. journal.sudo().bank_statements_source = 'file_import'
  └─ line_ids._cron_try_auto_reconcile_statement_lines(limit_time)   # immediate auto-reconcile attempt
  └─ open bank reconciliation widget (search_default_not_matched)
  └─ errors → RedirectWarning (partial) or UserError (total). Successful stmts committed before raising.
```

## Chain of responsibility — `_parse_bank_statement_file`

`account_bank_statement_import/models/account_journal.py:104-136`, `AccountJournal._parse_bank_statement_file` (base). Each format module overrides it, sniffs the file, and either parses or returns `super()`. Order is incidental (all auto_install); each `_check_*` is a cheap discriminator:

- CAMT: `_check_camt` — `etree.parse` succeeds AND root tag contains `'camt.053'`. `account_bank_statement_import_camt/models/account_journal.py:25-32`, `AccountJournal._check_camt`.
- OFX: `_check_ofx` — raw starts with `b"OFXHEADER"` (v1) OR contains `b"<ofx>"` (v2). `account_bank_statement_import_ofx/account_journal.py:102-110`, `AccountJournal._check_ofx`.
- QIF: `_check_qif` — raw stripped starts with `b'!Type:'`. `account_bank_statement_import_qif/account_journal.py:47-48`, `AccountJournal._check_qif`.
- CSV: handled differently — NOT via `_parse_bank_statement_file`. CSV uses the `base_import` UI (column mapping), see below.

Base fallback raises a **RedirectWarning to Apps** ("Did you install the module to support this type of file?") — `account_bank_statement_import/models/account_journal.py:128-136`, `AccountJournal._parse_bank_statement_file`.

### Return contract (the triplet)

`(currency_code, account_number, stmts_vals)`:
- `currency_code`: ISO 4217 string or None.
- `account_number`: bank account the statement belongs to, or None.
- `stmts_vals`: list of dicts `{name, date, [balance_start], [balance_end_real], transactions:[...]}`.
  - Each transaction: `{name/payment_ref, date, amount, unique_import_id, [account_number], [note/narration], [partner_name], [ref], [sequence]}`.

The canonical field doc is in the base docstring `account_bank_statement_import/models/account_journal.py:104-127`, `AccountJournal._parse_bank_statement_file` and still says `'name'` per transaction. NOTE: the actual parsers emit **`payment_ref`** (not `name`) per line — verified in OFX (`_fill_transaction_vals_line_ofx`, `account_bank_statement_import_ofx/account_journal.py:115`, `AccountJournal._fill_transaction_vals_line_ofx`), QIF (`vals_line['payment_ref']`, `account_bank_statement_import_qif/models/account_journal.py:64,88,96-99`, `AccountJournal._parse_bank_statement_file`) and CAMT (`entry_vals['payment_ref']`, `account_bank_statement_import_camt/models/account_journal.py:85`, `AccountJournal._parse_bank_statement_file_camt`). `account.bank.statement.line` stores it in `payment_ref`. The docstring is stale; trust the parsers.

## Journal/currency resolution — `_find_additional_data`

`account_bank_statement_import/models/account_journal.py:170-215`, `AccountJournal._find_additional_data`. 
- Sanitizes account number via `res_bank.sanitize_account_number`.
- If `currency_code` matches company currency → set to False (no foreign currency).
- If journal has NO `bank_account_id` → `journal.set_bank_account(account_number)` (auto-creates it).
- If imported on a recordset with no journal → search by `bank_account_id.sanitized_acc_number` (exact, then `ilike` partial-IBAN fallback if exactly one match).
- If journal already has a bank account → `_statement_import_check_bank_account` must confirm it matches (with FR BNP/LCL substring special cases, `:157-168`).
- Currency mismatch journal↔statement → UserError.

## Partner / bank account matching — `_complete_bank_statement_vals`

`account_bank_statement_import/models/account_journal.py:217-247`, `AccountJournal._find_additional_data`. Per transaction:
- `unique_import_id` is **namespaced**: `{sanitized_acct}-{journal_id}-{raw_unique_id}` (`:226`). This makes the same bank ID unique per journal/account.
- If line has `account_number` (counterparty IBAN) and no `partner_bank_id`: search `res.partner.bank` by `acc_number` (+ partner_id if known, else company scope).
- Only assigns partner/bank if EXACTLY ONE match — never guesses on ambiguity (`:244`).

## Dedup + creation — `_create_bank_statements`

`account_bank_statement_import/models/account_journal.py:249-299`, `AccountJournal._complete_bank_statement_vals`.
- Skips lines with `amount == 0` OR whose `unique_import_id` already exists in DB (sudo search). Counts ignored.
- On a skipped line, if `balance_start` known, it is **adjusted** by adding back the line amount (`:268-269`) so opening balance stays coherent.
- Builds statement with `line_ids = [[0,False,line]...]`, `with_context(default_journal_id=...)`.
- If 0 lines imported across the file → UserError "You already have imported that file." (configurable via `raise_no_imported_file`).
- If statement `is_complete` → `action_generate_attachment()` generates a PDF report (skippable via `skip_pdf_attachment_generation` context). REMOVED in v19.

## Format specifics

### CAMT.053 (SEPA — the format relevant for ES)
- Parser lib: `account_bank_statement_import_camt/lib/camt.py` — pure-python `CAMT` class, namespace-agnostic xpath getters. Namespace auto-detected: `ns = {'ns': root.xpath('namespace-uri(.)')}` (`account_bank_statement_import_camt/account_journal.py:41`, `AccountJournal._parse_bank_statement_file_camt`).
- One CAMT file can hold **multiple `Stmt` for multiple IBANs**. `_parse_bank_statement_file_camt` groups `statements_per_iban`; if >1 IBAN it keeps only those matching the current journal's IBAN, warns about the rest, and errors if the journal has no IBAN set (`:160-174`).
- Multi-currency: per-entry `InstdAmt`/`TxAmt`/`CntrValAmt` with `CcyXchg` rate handling (`_get_signed_amount`, `:630-717`); foreign amounts set `amount_currency` + `foreign_currency_id` (`_set_amount_in_currency`). Swiss CHF /100 rule for SEK/DKK/YEN/NOK (`:649`).
- Balances: start = OPBD/PRCD/OPAV/ITBD; end = CLBD/CLAV/ITBD (first match wins) — `:546-569`. Sign from `CdtDbtInd` (DBIT→negative).
- `unique_import_id` from `AcctSvcrRef` (+ `NtryRef`), falling back to `{name}-{date}-{sequence}` when missing/all-zeros/`NOTPROVIDED` (`_get_unique_import_id`, `:759-771`).
- Rich `narration` built from Entry Info, Counter Party, Address, Transaction/Instruction/EndToEnd/Mandate IDs, Check Number (`:110-137`). Transaction type label from BkTxCd domain/family/subfamily codes via the big `codes` dict (`:774-784`).
- Multi-entry-detail rounding diff is dumped onto the largest line (`:146-150`).

### CSV — reuses base_import (no custom parser)
- **CSV does NOT go through `_parse_bank_statement_file`.** `account_bank_statement_import_csv/models/account_journal.py` overrides `_import_bank_statement`: if the (single) file matches `.csv/.xls/.xlsx` (`_check_file_format`, `:16-17`) it short-circuits to the interactive `base_import` client action (`tag: 'import_bank_stmt'`, `:40-48`) instead of the parse pipeline. Only ONE CSV per upload: mixing CSV with other types → UserError "Mixing CSV files with other file types is not allowed"; >1 CSV → "Only one CSV file can be selected" (`:19-27`). XLS/XLSX are accepted too (not just `.csv`).
- `account_bank_statement_import_csv/wizard/account_bank_statement_import_csv.py` extends `base_import.import`.
- Adds 3 virtual columns to map: `balance` (cumulative), `debit`, `credit` (`get_fields_tree`, `:16-46`).
- Mapping rule (`_parse_import_data`, `:51-125`): you must provide EITHER `amount` XOR (`debit` AND `credit`). `amount = abs(credit) - abs(debit)`.
- **Rows must be sorted by date** (asc or desc); unsorted → UserError. Descending is auto-reversed (`:74-83`).
- `balance` column → derives `balance_start` (first balance minus first amount) and `balance_end_real` (last balance) (`:94-102`).
- `execute_import` (`:132-149`) creates the statement after rows imported (in a savepoint), `line_ids = Command.set(res ids)`.
- CSV does **not** dedup (no reliable unique id) and does **not** handle multicurrency — manifest warns to prefer OFX.

### OFX
- Uses python `ofxparse`; if absent, only a warning at load + UserError on import.
- `PatchedOfxParser` monkey-patches ofxparse to treat `,` as decimal separator (European banks) and to parse `YYYY-MM-DD` dates (`account_bank_statement_import_ofx/account_journal.py:24-90`, `PatchedOfxParser`).
- ofxparse gives no account number string → partner matched by `transaction.payee == res.partner.name` lookup (`:144-156`).
- WARNING in code: ledger balance is not guaranteed to be the ending balance (`:170-171`); `balance_start = balance - sum(amounts)`.

### QIF
- Hand-rolled line parser (`D`/`T`/`N`/`P`/`M`/`^` markers). `account_bank_statement_import_qif/account_journal.py:50-113`, `AccountJournal._parse_bank_statement_file`.
- Per-journal config fields: `qif_decimal_point` (default `.`) and `qif_date_format` (`day_first` default, vs `month_first`) — QIF has no canonical date/decimal format (`:26-40`). Exposed in `qif/views/account_journal_views.xml`.
- Returns `currency=None, account=None` → relies entirely on the journal it was dropped on; partner matched by payee name. No dedup, no multicurrency.

### OCR (account_bank_statement_extract)
- `account.bank.statement` gains `extract.mixin` (`account_bank_statement_import/models/account_bank_statement.py:13`, `AccountBankStatementLine`). Sends PDF/JPEG/PNG scans to `https://extract.api.odoo.com/api/extract/bank_statement/1/...` via IAP (`_contact_iap_extract`, `:31-35`).
- `_get_bank_statements_available_import_formats` extended with `PDF, JPEG, PNG` (`account_bank_statement_import/models/account_journal.py:27-30`, `AccountJournal.create_document_from_attachment`) so OCR-able files reach the OCR path; mixing OCR + non-OCR files in one upload → UserError (`:35-36`).
- `_fill_document_with_results` writes `balance_start`, `balance_end`, `date`, and creates `line_ids` from `bank_statement_lines` (amount/date/description) then triggers auto-reconcile server action `account_accountant.auto_reconcile_bank_statement_line` (`:37-61`).
- Company setting `extract_bank_statement_digitalization_mode` (`no_send` / `auto_send`, default `auto_send`) on `res.company` (`res_company.py`). Auto-send fires on main attachment set (`_message_set_main_attachment_id` → `_autosend_for_digitization`).

## After import
`_import_bank_statement` calls `line_ids._cron_try_auto_reconcile_statement_lines(limit_time=…)` immediately (limit derived from `limit_time_real_cron`, capped 0<x<180 else 180), then opens the bank reconciliation widget filtered to `not_matched`. The actual reconciliation engine lives in `account_accountant` (bank rec widget) — see integrations.md.

## Spanish context — Norma 43 (N43 / AEB43 / Cuaderno 43)
**NOT supported by Odoo Enterprise.** The EE family ships CAMT/CSV/OFX/QIF only. Spanish banks predominantly export N43. Coverage requires **OCA**: `l10n-spain/l10n_es_account_statement_import_n43` (`OCA/v18/l10n-spain/l10n_es_account_statement_import_n43`), which depends on the OCA stack `account_statement_import_file` (NOT the EE modules — different architecture, `account.statement.import` wizard). For an Indaws client on EE wanting N43 you either: (a) install the OCA N43 module (mind the EE/OCA architecture clash on `account.bank.statement.line`), or (b) ask the bank for a CAMT.053 export and use native EE. Many ES banks support CAMT.053 on request.
