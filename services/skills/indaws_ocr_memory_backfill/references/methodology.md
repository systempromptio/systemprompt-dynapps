# Backfill methodology

## The idea
The OCR starts out "cold": the memory is empty, so automatic supplier assignment sits around 0-2%
on the first invoice and only climbs after 2-3 corrections per supplier. If the customer already
has a history of posted invoices, that history IS human-labelled knowledge: which supplier, which
product and which account correspond to each piece of text. The backfill turns it into `approved`
aliases so the system gets it right from the first document.

## Flow
1. **Scan** (`scan_and_plan.py`, read-only): reads the last N invoices (or from a given date),
   their lines, products, accounts and taxes, and derives alias candidates.
2. **Aggregate and resolve conflicts**: for each `(res_model, field_name, context_key, name)`
   tuple, count which `res_id` it pointed at across the history. A text may have gone to several
   destinations (noise, one-off mistakes). Take the **mode** (most frequent destination) and
   compute the **consistency** = mode / total.
   - It is **seeded** if `support ≥ min-support` and `consistency ≥ consistency`.
   - It is **discarded** (to `skipped.csv`) if support is low or it is ambiguous (several
     destinations split evenly). Better not to seed than to seed an alias that misleads.
3. **Preview [STOP]**: `preview.csv` + `summary.json`. Review before writing.
4. **Apply** (`apply_seed.py`, requires `OCR_BACKFILL_APPLY=1`): idempotent `save_alias`.
5. **Verify** (`verify.py`) + a real smoke test.

## Thresholds — how to choose them
- `--min-support 1` (default): seeds even if a supplier has a single invoice. A recurring supplier
  will have high support; an occasional one, 1. Raise it to 2 if you only want the recurring ones.
- `--consistency 0.6` (default): if the majority destination is not reached at least 60% of the
  time, it is treated as ambiguous and not seeded. Lowering it tolerates more noise; raising it is
  more conservative.
- For VAT numbers and codes (stable) there is almost no ambiguity. Ambiguity shows up mainly in
  description → account (the same label used for different accounts on different invoices).

## Which layers, and why
- **partner (name + vat)** — ON. Maximum value, minimum risk. The VAT is stable → high hit rate.
- **product (default_code + barcode)** — ON. Precise codes.
- **product (description → product)** — ON with a threshold. Covers invoices with no code.
- **account (description → account, per supplier)** — ON with a threshold and a context. The Health
  case: the same concept goes to different accounts depending on the supplier →
  `context_key=partner_id:<id>`.
- **tax** — OFF by default (opt-in). Tax names are generic, of little value, and the engine already
  falls back by type/rate.

## Idempotency and safety
- `save_alias` is idempotent: re-running the backfill is safe (it does not duplicate; it updates if
  the destination changed). You can widen the scope (more invoices) and run it again.
- The only thing ever written is `ai.memory.alias` through `save_alias`. `_odoo.py` blocks any other
  write even in apply mode → it is impossible to touch invoices, products or any other customer
  data.
- Recommended gradual test: `apply_seed.py --limit 50` first, verify, then the whole set.

## An honest limitation
An alias only matches if, when a new invoice is digitised, the AI extracts the **same text** that
was seeded. VAT numbers and codes → very reliable. Trade names and descriptions → they vary; hence
the thresholds, and hence the backfill **raises** the initial accuracy without taking it to 100%.
The rest is filled in by the normal learning in day-to-day use.

Note that the match is on the whole text, never partial: a description carrying a week or a month
inside (`"Vracht week 33 — 4 ritten Eindhoven-Amsterdam"`) will never match the seeded `Vracht`.
