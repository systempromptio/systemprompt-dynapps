# Changelog — ocr-memory-backfill

Historia de versiones. Vivia en `SKILL.md`, donde ocupaba 48 lineas que no son
instruccion: nadie necesita la 1.4.0 para ejecutar la skill hoy.

- **1.8.0** — La `SKILL.md` baja de 436 a 150 lineas: este CHANGELOG, los GOTCHAS y el WORKFLOW
  se mudan a `references/` (nada se pierde, comprobado por tokens), y la skill cita
  `references/shared/odoo-connection.md` en vez de reexplicar la conexion.
- **1.7.1** — **Estudio 360 OCR Deploy Fixes (v18.0.37+)**. Multi-tenant `fiscal_position_id`
  validation + re-extraer idempotent cleanup. Ver PRs indawsodoo/ai-odoo-ocr#160-162
  (v17.0, 18.0, 19.0).
- **1.7.0** — **The account/product layers no longer seed from the history's line text.** Mode C
  (designed) splits into **C1** (history: destination from the posted line) and **C2** (clean
  database: PDFs handed over before go-live, destination proposed and seeded `pending`) — the key
  always comes from the document, and where the destination comes from is a separate question. Measured
  at a customer: the recorded description is the customer's internal vocabulary, the alias needs
  the supplier's, and `=ilike` on the whole string makes a near-miss a miss — 0 of 8 real invoices
  matched, 735 aliases seeded and deleted. Those layers are off unless `--trust-odoo-line-text` is
  passed, the limitation is the first thing the skill says, and gotcha 20 carries the evidence.
- **1.6.0** — **`scripts/check_stack.py`**: Step 1 stops being a checklist in prose and becomes a
  script with an exit code (0 ready / 1 blocking / 2 warnings). It verifies the model in use
  (rejects flash-lite class), the provider + its key, the IAP bridge and `no_send` per document
  type, who runs the jobs, `temperature`, and the health of the last extractions. Seeding
  **`pending`** is now possible (`--pending-below`, `save_alias` does take `state`): what falls
  below the confidence bar goes into accounting's review queue instead of being inherited blind.
  `scan_and_plan` gains `--exclude-partner-ids` / `--exclude-account-codes` /
  `--keep-dated-names`, and `skipped.csv` now says WHY each row was dropped. `verify.py` counts
  both states. Gotchas 16-19.
- **1.5.0** — Skill translated to English (the marketplace is mixed, but Dynapps is a multi-country
  group and this skill already produces reports in ES/EN/FR). Activation triggers kept bilingual.
- **1.4.0** — Step 1 now checks **the queue runner and the provider configuration** (the two
  failures that leave the seeding useless without warning), Step 4 explains **where the real error
  is** (the chatter, not `extract_error_message`), warning about the **API key leak** in the
  chatter with Gemini, and a new **THE SERVICE** section (what we deliver and in which phase).
  Gotchas 11-14.
- **1.3.2** — `scripts/smoke_test.py`: end-to-end validation (uploads N real invoices from source
  to target, extracts and compares supplier / base / VAT / total invoice by invoice → JSONL).
  Source via `SRC_ODOO_*`.
- **1.3.1** — The `description` and Step 0 reflect the multi-language support (`--lang`); correct
  routing.
- **1.3.0** — **Multi-language report ES/EN/FR** (`--lang`, default es) for the whole Dynapps
  group: title, verdict, actions, sections and tables translated (`TR` dict).
- **1.2.2** — `references/localization.md`: per-country tuning guide (the Dynapps group is
  multi-country: ES/BE/FR…) — language, country-agnostic VAT, taxes / equivalence surcharge /
  reverse charge, chart of accounts, provider/key, memory per company, `noupdate` gotcha.
- **1.2.1** — Default scope = vendor bills **with an attachment, from the last year**
  (`only_with_attachments=true` + a 12-month window; `--months` / `--all-history` /
  `--no-attachment-filter` to adjust). Fixes "None" labels on products (uses name /
  `default_code` / id).
- **1.2.0** — Execution modes `--mode` (explore / report+seed / seed; default explore). The report
  now ends in a **VERDICT** ("change the way they work? yes/no + actions"). Report template
  aligned with the "information status report" (products↔accounts, suppliers for training,
  rounding, analytics, wildcard 'sundry creditors' + supplier hygiene).
- **1.1.0** — JSON-RPC transport (Odoo ≥19 / indaws_security block XML-RPC). Fixes
  `display_type='product'` (v19 was seeding 0 on product/account). `--only-with-attachments`
  filter. Automatic denylist of own / intercompany tax ids. `audit.md` as a first-class
  deliverable (previously just counts). Seeding into a target different from the one scanned
  (prod→staging) with an aligned-ids check + model preflight.

