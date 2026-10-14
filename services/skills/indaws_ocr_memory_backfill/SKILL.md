---
name: indaws-ocr-memory-backfill
version: 1.8.0
effort: medium
model: sonnet
allowed-tools: [Read, Write, Bash, Glob, Grep, Agent, common-skills:odoo-pilot]
description: >-
  Prepares the start-up of the dynapps_ai_* OCR (Docky) at a customer: AUDITS how they record their information and issues a VERDICT ("does the way they work need to
  change before digitising?"), and SEEDS the alias memory (ai.memory.alias) so the engine recognises supplier, product and account from the very first document instead
  of starting cold. Multi-language report ES/EN/FR. Modes: explore (report only, read-only, default), report+seed, seed. Use when asked to "seed OCR memory", "backfill
  the memory", "preload / train the OCR", "information status report", "audit how they record for Docky" — or in Spanish, "sembrar memoria OCR", "backfill de memoria",
  "precargar/entrenar el OCR", "informe de estado de la información", "auditar el registro para Docky". Does NOT digitise documents into the production flow and does
  NOT install the module or create the extraction config.
---

# ocr-memory-backfill v1.8.0

Preloads the memory of the Dynapps OCR (`ai.memory.alias`) so it starts out recognising
suppliers — and, when there is history, products and accounts — from the first document instead
of cold.

## READ THIS FIRST — the memory keys on the SUPPLIER'S words, not the customer's

An alias only resolves if the AI extracts **the same text**, and the lookup is `=ilike` on the
**whole** string: the key has to be the text **printed on the document**. The history does not
hold that text — it holds what accounting typed. Measured at a customer (A2, Odoo 18, 2,129 vendor
bills): the PDF said `CNL MODULAR 200 AISI316L L2530 H60-70 FONDO`, the history had `Compra de
otros aprovisionamientos`. **0 of 8 real invoices matched**, and 735 aliases were seeded and
deleted again because counting them as "trained" was simply false. Supplier names fail the same
way (`Nortcoat Galicia, S.L.U` vs `NORTCOAT GALICIA SL.`); of the four layers only **VAT**
survives contact with a real document, and the engine resolves the supplier by VAT structurally
before it ever asks the memory. Full evidence: gotcha 20 in `references/gotchas.md`.

Consequences, enforced by the scripts and not left to good intentions: `--layers product,account`
from the history is **OFF by default** (`--trust-odoo-line-text` forces it, and only after
checking the recorded text IS the document text) · the only layer worth seeding from history alone
is **partner/VAT** · to seed account and product the key must come from the **document**, and
where the *destination* comes from is a separate question — **C1, with history**: extract 2-3
invoices per supplier and pair each extracted line against the line already posted on that invoice
(by amount and order); **C2, clean database**: the customer hands over PDFs before go-live, so the
destination is *proposed* against their master and seeded **always `pending`**
(`references/pdf-mode.md`) · report coverage as **the percentage of lines that resolve on real
documents**, never as rows in the memory.

## FIRST DECISION — does the customer have history? This changes the work radically

- **WITH history** (working in Odoo, invoices recorded and attached) → **mode A**. The first job
  is NOT to seed: **audit how they record** (dirty tax ids, duplicates, scattered accounts,
  product usage, references) and hand over a **proposal to change the way they work**
  (`audit.md`) — seeding on dirty data propagates the errors. Then seed supplier + product +
  account. Double value: starting trained **and** surfacing bad recording habits.
- **FROM SCRATCH** (new implementation, no history) → **mode B** (`references/pdf-mode.md`):
  nothing to audit, the supplier is extracted from the customer's PDFs with Claude's vision and
  only the supplier is seeded. With a batch of real invoices before go-live, go further with **mode
  C2** in the same file and seed account and product too, `pending` — a clean database is not a
  reason to arrive with an empty memory.

## Agent Routing

**Step 0 (Input)** runs in the main loop, because it is the interaction with the user.
**Steps 1-4** (verify stack — `check_stack.py` —, scan + plan, apply, verify) are **sonnet,
inline**. The History source runs inline throughout; the PDFs source uses subagents to read them
with vision (`references/pdf-mode.md`).

## ROLE and SOURCES
READ-ONLY except for `save_alias`: audits how they record, issues a verdict and — where
appropriate — seeds aliases. Never digitises into the production flow and never touches configs
(`ai.extract.config`, server actions, provider keys: different flow). Everything over RPC — the
transport contract is `references/shared/odoo-connection.md`, the local specifics
`references/connection.md`. Two sources: **History** (`scan_and_plan.py`) — invoices already
posted → report + supplier/product/account — and **PDFs** (`match_and_plan.py`) — no history, so
Claude reads each PDF, extracts the issuer, matches it and seeds ONLY the supplier; **mode C2** in
`references/pdf-mode.md` extends it to account and product, seeded `pending`.

## EXECUTION MODES (customer with history) — `--mode`
The question the skill answers: *"with this structure plus the memory, does the way they work need
to change?"* · **explore** (DEFAULT) — `scan_and_plan.py` only: **report + VERDICT** (`audit.md`), READ-ONLY,
seeds nothing; the starting point for a customer with history · **report+seed** — the report and
then, after review / [STOP], `apply_seed.py` · **seed** — skip the report and seed an existing
`seed_plan.json` (data already audited and cleaned).

The **verdict** comes from the severity of the findings (own tax ids, product↔account, partial
analytics, undefined wildcard, duplicates → "YES, fix first"; clean → "NO, go ahead"), and the
report follows the "information status report" template.

## THE SERVICE (how we deliver it)

**Not a stand-alone utility**: it is the *initial training* phase of the OCR start-up service —
what the quotation calls "Initial training: suppliers, accounts and formats".

| Phase | What we do | Deliverable | Who decides |
|---|---|---|---|
| 1 · Audit | `--mode explore` over the history (read-only) | `audit.md` with the **verdict**: does the way they work need to change before digitising? | We report |
| 2 · Clean-up | the customer fixes what the verdict flags (dirty tax ids, duplicates, scattered accounts) | a clean master | **The customer** |
| 3 · Seeding · 4 · Validation | `--mode seed` over the reviewed plan · smoke test with real invoices from different suppliers | `approved` memory loaded · % of lines resolved unaided | Us, after [STOP] · we measure, they validate |
| 5 · Handing over the queue | reviewing pending entries passes to whoever does the bookkeeping | a 5-min/week routine | **The customer** |

Two things to say up front, because they are what make start-ups fail: **phase 2 is theirs and it
decides the outcome** (seeding on dirty data propagates the errors; a thousand well-chosen aliases
pay off more than twenty thousand seeded any old how) and **if nobody reviews the pending queue
the memory stops growing** — learning is not automatic on free-text fields, somebody in accounting
has to approve, five minutes a week with an owner who has a name.

## WORKFLOW

**The five steps, with their commands, flags and checks: `references/workflow.md`.** Load it
before executing, not after. **Step 0 — Input [STOP]** · **Step 1 — Verify stack** · **Step 2 —
Audit + plan (dry-run)** · **Step 3 — Apply** · **Step 4 — Verify**. Steps 1-4 are sonnet inline.

## KEY RULES

**NEVER** write to anything other than `ai.memory.alias` through `save_alias` (`_odoo.py` blocks the rest even in apply mode) · apply without having shown `preview.csv` and got confirmation (Step 2 [STOP]) · seed the `account` layer without `context_key=partner_id:<id>` — the account depends on the supplier, the same "Freight" goes to a different account per supplier, and with no context the aliases overwrite each other · hardcode credentials, URLs or ids · use this skill to digitise PDFs or create the `ai.extract.config`: different flow.

**ALWAYS** set `ODOO_COMPANY_ID` (the memory and its unique index are per company) · start in dry-run and apply only after reviewing the plan · respect the unique key `(name, res_model, field_name, company_id, COALESCE(context_key,''))`, with `context_key=""` and not `False` for global entries · resolve conflicts by frequency (mode + consistency), discarding and logging the ambiguous ones · recommend a real smoke test after seeding.

## GOTCHAS

One line each; **all 20 in full, with their evidence, in `references/gotchas.md`** under the same numbering.

1. **Two `context_key` formats** · 2. **`save_alias` defaults to `approved`, but it DOES take `state`** · 3. **`name` is truncated to 128 chars**
4. **An alias only matches if the AI extracts the SAME text.** · 5. **`account.tax.find_record` ignores `field_name`** · 6. **Multi-company**
7. **v19 / indaws_security transport** · 8. **`display_type='product'` on v19** · 9. **Own / intercompany tax ids contaminate**
10. **Source ≠ target (prod→staging)** · 11. **The jobrunner cron is born set to once a day** · 12. **`ai.extract.config` is born with no provider**
13. **The API key leaks in plain text into the chatter** · 14. **Spanish strings in the Core data** · 15. **Pick a Pro-class model for the lines.**
16. **Not every scattered supplier is a data problem.** · 17. **Regularisation accounts contaminate.** · 18. **Measure which key discriminates BEFORE seeding, not after.**
19. **`ai.extract.config.memory_count` is NOT the memory counter.** · 20. **The history's line description is the WRONG key, and this is the skill's main trap.**

## Delegates to
- `common-skills:odoo-pilot` — the RPC transport layer, especially on Odoo ≥ 19 (JSON2).

## Consumed by
None so far. Candidate: an OCR start-up skill (`dynapps_ai_extract_account`) calling this one after installing the module and creating the config.

## REFERENCES

| File | What it carries |
|---|---|
| `references/workflow.md` | the five steps with their commands, flags and checks — **load it before executing** |
| `references/gotchas.md` | the 20 gotchas in full, with their evidence, same numbering as the index above |
| `references/what-to-seed.md` | the exact `(res_model, field_name, context_key)` tuples the mapping expects, where they come from in the history, and the two `context_key` formats |
| `references/methodology.md` | aggregation, thresholds, conflict resolution, idempotency, which layers and why, how to choose the scope |
| `references/pdf-mode.md` | mode B and C2: extracting the supplier with Claude's vision (subagents per batch), matching against the master, seeding. Flow and gotchas |
| `references/localization.md` | **Docky per country** — language, VAT, taxes / equivalence surcharge / reverse charge, chart of accounts, analytics, wildcard, provider/key, memory, the `noupdate` gotcha. The Dynapps group is multi-country |
| `references/connection.md` · `references/shared/odoo-connection.md` · `references/CHANGELOG.md` | environment variables, API key, apply mode, protocol per version · the group-wide transport contract · version history, which used to live here costing 48 lines that are not instruction |
| `config.json` | tunable defaults: invoice domain, scope, layers, thresholds, the two `context_key` formats. **Credentials never here** — they go through the environment |

**Related, not delegated:** `developer-skills:odoo-importation` / `indaws-data-import` load data,
a different job (nothing is imported here, an alias memory is seeded) · the `dynapps_ai_*` stack
(repo `ai-odoo-ocr`) is the OCR being preloaded, and this skill is the recommended step before
putting `dynapps_ai_extract_account` into production.
