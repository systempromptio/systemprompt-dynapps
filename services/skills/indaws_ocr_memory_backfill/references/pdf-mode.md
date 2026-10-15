# Mode B — a batch of PDFs (new implementation)

For customers with NO accounting history (a fresh implementation): the only material is the invoice
PDFs the customer provides. Here the **supplier** can be pre-trained reliably; the product layer
only when the customer's product master already exists (see below), and the account layer not at
all — it needs human judgement and is learned by validating in use.

## Why Claude and not Gemini or regex
- **A VAT regex over the PDF text is NOT reliable**: about a third of invoices are image scans with
  no text layer, and on those that do have text the regex often picks up the **recipient's** VAT
  (our own company), not the issuer's.
- **Claude's vision** reads both native PDFs and scans, and tells the issuer apart. It needs no
  Gemini provider configured and no API key, and it creates no drafts in production.

## Flow

### Step 1 — Prepare the batch
Unzip / gather the PDFs into a folder. Note that the file name often already carries the supplier,
which is useful as a fallback for the name match.

### Step 2 — Issuer extraction (Claude, with subagents)
Claude reads each PDF and returns the issuer. For large batches, split across subagents (say 10-15
PDFs each) running in parallel; each one returns a list of objects:
```json
[{"file": "<name>.pdf", "emitter_vat": "F20023065", "emitter_name": "ULMA C Y E S COOP"}]
```
Rules for the subagent:
- Extract the **issuer** (whoever ISSUED the invoice), NOT the recipient/customer.
- `emitter_vat`: the tax id exactly as it appears; empty if not visible.
- `emitter_name`: the issuer's trade name; if in doubt, use the file name.
- If the PDF does not allow identifying the issuer, leave both empty (it goes to `unmatched`).
Consolidate every output into a single `extracted.json`.

### Step 3 — Match + plan (READ-ONLY)
```bash
python3 scripts/match_and_plan.py <out> --extracted extracted.json --company-id <cid>
```
Matches each issuer against `res.partner`: by normalised VAT (excluding the VAT numbers of the
group's own companies) and, failing that, by name with `=ilike`. Produces `seed_plan.json` +
`preview.csv` + `unmatched.csv` + `summary.json`. **Review before applying** — especially
`unmatched.csv` (suppliers missing from the master: decide whether to create them separately).

### Step 4 — Apply (per company)
```bash
export ODOO_COMPANY_ID=<cid> OCR_BACKFILL_APPLY=1
python3 scripts/apply_seed.py <out>
```
Seeds `res.partner/name` and `res.partner/vat` (both global) → the supplier is recognised from the
first document. The memory is per company: repeat for each target `company_id` (the same plan
works; only `ODOO_COMPANY_ID` changes).

## Can products be seeded too?
Yes, **on one condition**: the product has to exist in the master already, because seeding an alias
needs its `res_id`. In a fresh implementation that is often the case — the customer has no invoices
yet but has imported their catalogue and their suppliers.

What is worth extracting from the PDF lines, given that the match is on the whole text and never
partial:

| From the PDF line | Worth seeding? | Why |
|---|---|---|
| Supplier reference / article code → `product.product` / `default_code` | **Yes, the best payoff** | codes are identical invoice after invoice |
| Barcode → `product.product` / `barcode` | Yes | just as stable |
| Free-text description | **No** | it carries the week, the batch or the month inside and never matches again |
| General account | **No** | it depends on an accounting policy nobody has decided yet |

Seed the article reference **with a supplier scope** (`context_key=partner_id:<id>`): two suppliers
use the same code for different things.

## Mode B gotchas
- **Suppliers by default.** Product only under the condition above; account never (nobody has
  decided it yet) — it is learned when the first invoices are validated (`save_correction`).
- **The supplier must exist in the master** in order to seed (we need its `res_id`). Whatever comes
  out in `unmatched.csv` has to be created first (or you accept that the OCR creates them on the
  fly in use).
- **Discard own VAT numbers**: the recipient is one of the group's companies; `match_and_plan`
  excludes them by reading `res.company`.
- **Idempotent**: `apply_seed` uses `save_alias`; re-running is safe.
- **It creates no account.move**: this mode does NOT push the invoices into the production flow; it
  only learns the supplier. Uploading the invoices for real is the normal OCR flow, separately.

---

# Mode C — DOCUMENT text as the key (the right way to seed account/product)

**Status: designed, not implemented.** Mode A (history) cannot seed the account or product layer
correctly, for the reason at the top of SKILL.md: it keys on the description accounting typed,
while the alias needs the description the supplier printed. This mode is what replaces it.

## The idea

**The key always comes from the document. Where the destination comes from depends on whether
there is accounting yet** — and those are two independent choices, so do not couple them:

| | key (the alias text) | destination (account / product) | seeded as |
|---|---|---|---|
| **C1 · customer with history** | the invoice's own attachment, extracted | the line already posted on that same invoice | `approved` / `pending` by consistency |
| **C2 · clean database** | PDFs the customer sends (email, a folder, a shared drive) | nobody has booked anything yet → proposed against their master | **always `pending`** |

C2 is the normal case in a new implementation, and it is worth doing: the customer hands over a
batch of real invoices before go-live, and the memory starts populated with pending rows they
approve in one pass — so the very first invoice through the engine is already recognised, instead
of the memory starting empty and learning one correction at a time.

Never seed C2 as `approved`. There is no evidence behind those rows: a proposal is a question,
and the answer belongs to whoever does the bookkeeping.

## C1 — with history

1. Pick the suppliers worth training — those with enough history to be predictable (`>= 3`
   invoices) — and cap the run (e.g. 100 suppliers x 3 invoices).
2. For each chosen invoice, extract its attachment and keep the per-line
   `product_description` the AI returns. That string is the alias key.
3. Pair each extracted line with the line already **posted** on that same invoice — by amount
   first, then order, then quantity — and read its `account_id` and `product_id`. That is the
   destination.
4. Aggregate per `(key, supplier)` exactly as mode A does: mode, support, consistency. Seed the
   confident ones `approved`, the rest `pending`.
5. Drop keys that carry a date, a week or a batch (`ROISAN JUNIO DIA - 25`,
   `PotenciaP1 15/05/2026-14/06/2026`) — single-use even when read from the PDF.

## What it settles that mode A cannot

- Whether a "scattered" supplier is really unlearnable. A leasing company with one account per
  vehicle looks hopeless in the history (the description is always the literal `Leasing`) but is
  trivial if its PDF prints the registration on the line. Only the document can answer that.
- Which texts are genuinely single-use, measured on the document rather than guessed.
- The real coverage number, because step 3 gives you a labelled set: extract, resolve against the
  seeded memory, count.

## C2 — clean database, PDFs handed over

1. **Get the PDFs in**: a local folder is the simplest contract (`--pdf-dir`); a mailbox works
   too, but do not build mail plumbing into the skill — have somebody drop the attachments in a
   folder. One invoice per file, any supplier mix, no naming convention needed.
2. **Read each one**. Two options, and the choice matters:
   - **Claude's vision** (subagents, as mode B already does): no dependency on the customer's
     provider or API key, and it works before the OCR stack is even configured. Right for a
     pre-go-live batch.
   - **The customer's own `ai.extract.config`**: slower and costs their API budget, but the text
     is exactly what their engine will return in production — which is the whole point of keying
     on document text. Prefer this when the stack is already installed.
3. **Match the supplier** against `res.partner` by VAT first, then name — same as mode B. A PDF
   whose issuer is not in the master is reported, not invented (see the autocreate parameter).
4. **Propose the destination** for each extracted line: search the customer's master for a
   product (`default_code`, `barcode`, name) and, failing that, leave the account to be chosen.
   Where there is genuinely nothing to match, list the line in the report instead of guessing —
   an alias pointing at a wrong account is worse than no alias.
5. **Seed everything `pending`**, scoped `partner_id:<id>` for the account layer exactly as C1.
   Hand over one screen (`Settings → AI → AI Memory`, filter `pending`) and let them approve,
   correct or delete in one sitting. That review IS the training, and it is the moment to agree
   who owns the queue from then on.

## Costs, to state up front (C1 and C2 alike)

One AI call per invoice: 100 suppliers x 3 invoices = **300 extractions**, real API spend and
real time (2-8 s each on a Pro-class model) when going through the customer's provider. Claude's
vision in C2 avoids their budget but still costs wall-clock. Never digitise into their flow —
extract a copy of the attachment, exactly as `smoke_test.py` already does.

## Reuse

`smoke_test.py` already uploads an attachment, extracts it and reads the result back;
`match_and_plan.py` already reads PDFs with vision and matches the issuer, which is most of C2
step 2-3. What is new: the per-invoice line pairing of C1 step 3, the destination proposal of C2
step 4, and the aggregation — which is the same code mode A uses, fed a different key.
