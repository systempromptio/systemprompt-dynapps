# Bank Statement Import — v18 → v19 changes

Source: diff of `odoo_enterprise/18/account_bank_statement_import*` vs `odoo_enterprise/19/...`. All modules gained `'author': 'Odoo S.A.'` in their manifest (cosmetic).

---

## 1. `_parse_bank_statement_file` signature + return contract CHANGED (breaking)

The single most important migration point.

| | v18 | v19 |
|--|-----|-----|
| Param | `_parse_bank_statement_file(self, attachment)` (ir.attachment record) | `_parse_bank_statement_file(self, raw_file)` (raw bytes) |
| Return | ONE triplet `(currency, account, stmts_vals)` | LIST of triplets `[[currency, account, stmts_vals], ...]` |

- Base: `account_bank_statement_import/models/account_journal.py:110-116`, `AccountJournal._parse_bank_statement_file`, v19 (v19 docstring "rtype: list of triplets").
- Format modules now sniff `raw_file` bytes directly instead of `attachment.raw`:
  - CAMT `_check_camt(self, raw_file)` / returns `[[currency, account_no, statement_list]]` (v19 `:175`).
  - OFX `_check_ofx(self, raw_file)` / returns `[[currency_lst, account_lst, vals_bank_statement]]` (v19 `:110`).
  - QIF `_check_qif(self, raw_file)` / returns `[[None, None, [vals_bank_statement]]]` (v19 `:113`).
- **Migration impact**: any custom format module (e.g. an Indaws N43 or proprietary bridge) overriding `_parse_bank_statement_file` MUST update both the parameter (read bytes, not `attachment.raw`) and wrap its triplet in a list. Failing to wrap → the loop iterates over the triplet's elements as if each were a triplet → crash.

## 2. `_import_bank_statement` loop restructured for multi-account files

v18 looped over attachments, one triplet each. v19 loops over the **list** returned per attachment and over each `(currency, account, stmts_vals)` inside, with per-statement try/except so one bad account in a multi-account file doesn't abort the rest (`account_bank_statement_import/models/account_journal.py:40-86`, `AccountJournal._import_bank_statement`).
- `errors` is now a `defaultdict(list)` — multiple errors per file are collected and joined.
- When a single triplet is returned, the journal `self` is used; when multiple, an empty recordset forces `_find_additional_data` to resolve the journal from each statement's account (v19 `:45-48`).
- New journal-not-found message includes the account number when known (`:217-221`).

## 3. Auto-reconcile moved INTO statement creation (post-import call removed)

- v18: after import, `_import_bank_statement` explicitly called `line_to_reconcile._cron_try_auto_reconcile_statement_lines(limit_time=…)` (`:77-83`). The `tools.config['limit_time_real_cron']` clamp logic lived here.
- v19: that block is GONE. Instead `_create_bank_statements` creates the statement `with_context(auto_statement_processing=True)` (`:283-287`), and reconciliation is triggered downstream by `account_accountant` reacting to that context. Custom code that depended on the explicit post-import call no longer fires it.

## 4. Auto-PDF generation on import REMOVED

- v18 `_create_bank_statements` ended each created statement with `if statement.is_complete: statement.action_generate_attachment()` plus a `skip_pdf_attachment_generation` context escape (`:282-285`).
- v19: removed entirely. No PDF is generated at import time. The attachment now linked to the statement is the **source file** itself: `st_vals['attachment_ids'] = [Command.set(attachment.ids)]` (`:228`). (BSIG19.)

## 5. `_create_bank_statements` return value simplified

- v18 returned `(statement_ids, statement_line_ids, notifications)` and built a notifications list internally.
- v19 returns `(statement_ids, ignored_qty)` — an int count; the "N transactions already imported" notification text is now assembled by the caller (`_import_bank_statement`, v19 `:71-79`). Custom callers unpacking 3 values break.
- `line_ids` built with `Command.create(line)` instead of `[0, False, line]`.

## 6. `unique_import_id` constraint migrated to `models.Constraint`

`account_bank_statement.py`:
- v18: `_sql_constraints = [('unique_import_id', 'unique (unique_import_id)', '…')]`.
- v19: `_unique_import_id = models.Constraint('unique (unique_import_id)', "…")` — the v19 declarative constraint API.
- v19 also ADDS `get_import_templates()` returning an XLSX template `/account_bank_statement_import/static/xls/bank_statement_line_import_template.xlsx` (new built-in import template for statement lines).

## 7. CSV module gains a bank-rec-widget filter + class rename

- New file `account_bank_statement_import_csv/views/bank_rec_widget_views.xml` (added to manifest `data`): inherits `account_accountant.view_bank_statement_line_search_bank_rec_widget` and adds an invisible filter `imported_statement_lines` on `context.get('imported_transaction_ids', [])` — lets the rec widget scope to just-imported transactions.
- Wizard class renamed `AccountBankStmtImportCSV` → `Base_ImportImport` (v19 naming convention, class name = model name camelized).
- `get_fields_tree` is now guarded by `model == 'account.bank.statement.line'` and only adds the `debit`/`credit` virtual columns if the model doesn't already define them (`fields_get` check) — v19 may have native debit/credit on the line.

## 8. OFX parser refactor: ofxparse moved out, lazy import

- v18 monkey-patched ofxparse inline via `PatchedOfxParser` in the model file, importing `ofxparse` at module load inside a try/except.
- v19: the patched parser lives in a dedicated `account_bank_statement_import_ofx/ofx_parser.py`, imported lazily inside `_parse_bank_statement_file` (`from odoo.addons.account_bank_statement_import_ofx.ofx_parser import OfxParser`); presence checked at load with `importlib.util.find_spec('ofxparse')` (warning only). Same UserError if missing. Regex uses raw bytes literal `rb'encoding="(.*?)"'`.

## 9. Extract (OCR) module changes

- Mixin changed: `extract.mixin` → **`extract.mixin.with.words`** (`account_bank_statement_import/models/account_bank_statement.py:9`) — richer OCR word-box support.
- `_get_iap_account().account_token` → `.sudo().account_token` (`:33`).
- OCR-filled lines created `with_context(auto_statement_processing=True)` (aligns with change #3); the explicit `auto_reconcile_bank_statement_line._trigger()` call was REMOVED (now implicit via context).
- Manifest adds `web.assets_backend` JS (`static/src/js/*.js`).

## 10. Norma 43 / ES — unchanged

Still NOT supported in v19 EE (no N43 format module added). OCA `l10n_es_account_statement_import_n43` remains the path; verify its v19 port exists before promising it. CAMT.053 export from the bank remains the native EE alternative.

---

## Migration checklist (custom format/bridge modules)

1. Update `_parse_bank_statement_file(self, raw_file)` — read bytes, not `attachment.raw`.
2. Wrap the return in a list: `return [[currency, account, stmts_vals]]`.
3. If you call `_create_bank_statements`, unpack 2 values not 3.
4. Drop any reliance on auto-PDF at import (BSIG19) and on the explicit post-import `_cron_try_auto_reconcile_statement_lines` call.
5. Replace `_sql_constraints` overrides on the line with `models.Constraint` if you extended them.
6. OFX customizations: import the parser from `ofx_parser.py`, not the model.
