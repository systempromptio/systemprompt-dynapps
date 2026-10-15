---
name: indaws-pdf-to-odoo
description: >
  Automate document intake from PDF to Odoo. Use this skill when the user needs
  to process incoming PDFs (invoices, credit notes, delivery notes, purchase
  orders, expense receipts) and create draft records in Odoo automatically.
  Activates for tasks involving: (1) processing scanned invoices or bills,
  (2) OCR extraction from PDF documents, (3) automatic Odoo record creation
  from documents, (4) setting up a PDF inbox/watcher for document processing,
  (5) batch processing multiple PDFs into Odoo. Uses GLM-OCR (via Ollama or
  Z.AI Cloud) for text extraction and odoo-pilot scripts for all Odoo API
  operations.
version: 1.1.1
model: sonnet
effort: low
allowed-tools:
  - Bash
  - Read
  - Write
---

# pdf-to-odoo

PDFs dropped into a watched folder are OCR-processed, classified, and routed to the correct Odoo
model **as draft records**. Draft is the whole safety model: everything downstream assumes a human
opens the record before it becomes an accounting fact.

## Quick start

```bash
pip install -r skills/pdf-to-odoo/requirements.txt
brew install poppler          # macOS · apt install poppler-utils on Debian/Ubuntu
ollama pull glm-ocr && ollama serve          # or set ZAI_API_KEY — see references/ocr-setup.md
cp skills/pdf-to-odoo/.env.example skills/pdf-to-odoo/.env    # then edit it
```

```bash
python skills/pdf-to-odoo/scripts/watcher.py --once        # process the inbox and exit
python skills/pdf-to-odoo/scripts/watcher.py               # daemon: watch for new PDFs
python skills/pdf-to-odoo/scripts/process_pdf.py file.pdf  # one document
```

Every setting, with the three that decide how much mess a bad OCR can make:
`references/configuration.md`.

## Workflow — what `process_pdf.py` does per file

| Step | Script | Output |
|---|---|---|
| 1 | `auth.sh` (odoo-pilot) | session against the target instance |
| 2 | `pdf_utils.py` | PDF pages → PIL images at 300 DPI |
| 3 | `ocr_extract.py` | structured JSON per page (GLM-OCR) |
| 4 | `pdf_utils.py` | boundary detection → split multi-document PDFs |
| 5 | `classify_document.py` | document type, or the configured fallback |
| 6 | `match_partner.py` | VAT → fuzzy name → auto-create |
| 7 | `create_record.py` | **draft** Odoo record |
| 8 | `attach_pdf.py` | the source PDF as `ir.attachment` |

Success → PDF moves to `archive/`. Failure → `errors/`, plus an email if `NOTIFY_EMAIL_TO` is set.

This pipeline is deterministic Python end to end — **no subagent, no model routing**. The skill
runs at `sonnet`/`low` because its job is to run scripts and read their output, not to reason about
the documents. If an OCR result needs judgement, that is a human's call, not an escalation to
`opus`.

## Supported document types

`account.move` (`in_invoice`, `in_refund`, `out_invoice`, `out_refund`), `stock.picking`,
`purchase.order`, `hr.expense`. Keywords per type, and the JSON payload each one builds, in
`references/document-types.md`.

## Scripts

| Script | Purpose |
|--------|---------|
| `config.py` | Load .env configuration with defaults |
| `watcher.py` | Folder watcher daemon (entry point) |
| `process_pdf.py` | Main pipeline orchestrator |
| `pdf_utils.py` | PDF → images, multi-doc split |
| `ocr_extract.py` | GLM-OCR extraction (Ollama / Z.AI) |
| `classify_document.py` | Document type classifier (3-tier) |
| `match_partner.py` | Partner matching: VAT → fuzzy → create |
| `create_record.py` | Build payload + create Odoo record |
| `attach_pdf.py` | Attach PDF to Odoo record |
| `notify.py` | Error email via Odoo `mail.mail` |

## GOTCHAS

1. **The VAT lookup uses `like`, and Odoo wraps `like` in `%…%`.** `scripts/match_partner.py:74`, `match_partner`, searches
   `["vat", "like", vat_no_prefix]`, and Odoo turns that into `LIKE '%value%'`
   (`odoo/models.py:3256`, `BaseModel._condition_to_sql`, in 18.0). A VAT that merely *contains* the digits matches, so tier 1 can
   return a different company and it looks like a confident hit, not a fuzzy one.
2. **Classification never fails — it falls back.** `classify(…)` returns
   `("account.move", default_type)` when nothing matched (`scripts/classify_document.py:120`, `classify`), and the
   shipped default is `in_invoice`. An unrecognised document does not land in `errors/`; it lands
   in accounting as a vendor bill.
3. **`PARTNER_AUTO_CREATE` is `true` out of the box.** Tier 3 creates a `res.partner` from OCR text
   (`scripts/match_partner.py:106-130`, `match_partner`). One misread name is one new partner, and the duplicate is only
   visible much later, from the accounting side.
4. **The fuzzy tier is a cutoff, not a confirmation.** `0.7` on `get_close_matches` means "closest
   above the line wins" — there is no second-best comparison and no ambiguity report.
5. **`notify.py` sends mail *from Odoo*** (`mail.mail`, created and sent). On an instance where
   outgoing mail is not deliberately configured, that is a real email to a real person. Leave
   `NOTIFY_EMAIL_TO` empty to disable it.
6. **`OCR_BACKEND=zai` ships the document image to a third party.** Fine if someone decided it;
   never a default you slide into because Ollama was not running.

## NEVER / ALWAYS

- **NEVER** post, validate or confirm anything this pipeline creates. It writes drafts; a person
  promotes them.
- **NEVER** run it against production the first time. Point it at a copy, process a real batch, and
  read what it created before trusting the classifier on live data.
- **NEVER** raise `PARTNER_MATCH_THRESHOLD` confidence by lowering the number — that is backwards:
  lower cutoff = more matches accepted = more wrong partners.
- **ALWAYS** check `errors/` and the log after a batch. A run that reports success only means no
  exception escaped, not that the records are right.
- **ALWAYS** verify partner and total on the created record before promoting a draft; OCR
  confidence is not record correctness.

## Delegates to

`common-skills:odoo-pilot` — every Odoo call goes through its scripts (`auth.sh`,
`search_records.sh`, `create_record.sh`); this skill writes no client of its own. The connection
contract, including which protocol per version, is `references/shared/odoo-connection.md`.

## Consumed by

**Nobody today.** It is an entry-point skill: a person points it at an inbox. Said plainly because
declaring consumers that do not cite it is the cheap way to look integrated.

## References

- `references/configuration.md` — every `.env` setting, and what the risky defaults do
- `references/document-types.md` — document type to Odoo model mapping with JSON payload examples
- `references/ocr-setup.md` — GLM-OCR installation guide for Ollama and Z.AI Cloud
- `references/partner-matching.md` — 3-tier partner matching algorithm details
- `references/tax-mapping.md` — Tax rate to Odoo tax mapping (Spain/IVA)
