# Workflow — ocr-memory-backfill

Los cinco pasos con sus comandos, banderas y verificaciones. `SKILL.md` lleva el indice.


### Step 0 — Input [STOP]
Ask the user and export as environment variables (NEVER hardcode — see
`references/connection.md`):
1. **Connection**: `ODOO_URL`, `ODOO_DB`, `ODOO_USER`, `ODOO_KEY`, `ODOO_COMPANY_ID` (mandatory:
   the memory is per company). If they would rather not paste a password, have them generate an
   API key.
2. **Mode** (`--mode`, default `explore`): explore | report+seed | seed (see above).
3. **Scope** (DEFAULT = vendor bills **with an attachment, from the last year**): applies the
   attachment filter (`--no-attachment-filter` to include those without one) and a 12-month
   window (`--months N`, `--all-history` for everything, `--since YYYY-MM-DD` or `--limit N` for a
   different cut-off).
4. **Layers** (`--layers`, default `partner,product,account`; `tax` is opt-in). 5. **Thresholds**
   (`--min-support`, `--consistency`). 6. **Training** (`--train-min`, default 3).
7. **Report language** (`--lang es|en|fr`, default es) — for customers and countries in the group.

Defaults live in `config.json`; the CLI flags override them. (Customer STARTING with no history →
**PDFs** source, `references/pdf-mode.md`.)

### Step 1 — Verify stack [SONNET/inline]
```bash
python3 scripts/check_stack.py          # --json for machine-readable output
```
Exit code **0** = ready · **1** = blocking problems · **2** = warnings worth reporting.

Run it and read it before anything else. Seeding a memory into an environment whose OCR cannot
run is the most expensive failure of this skill: nothing errors, nothing happens, and the
customer is told they are trained. Every check comes from a real start-up where the seeding was
fine and the environment was not:

| Check | Why it is there |
|---|---|
| stack installed + `ai.memory.alias` + real company | without it there is nothing to seed into |
| a provider module, `ai.provider` **enabled** with a **key** | the config is born with `provider_id = False`; the first extraction dies |
| the config's **model** | flash-lite class returns 0 lines or drops `quantity` — measured; it wants a Pro-class model |
| mapping action + field mappings, the account one with `context_field` and `overwrite` | with no context the account aliases overwrite each other; with no `overwrite` the learned account never replaces the default |
| **IAP**: if `account_invoice_extract` is installed → the bridge must be too, and `no_send` per document type | otherwise two engines write the same fields and Odoo's OCR burns credits |
| **who runs the jobs**: a `queue_job` runner ≤10 min OR the Core fallback cron | without either, jobs sit in `pending` and the smoke test does not fail — nothing happens |
| `dynapps_base_ai.temperature` = 0.0 | it ships at 0.2: random sampling on a transcription task |
| health of the last `ai.request.log` entries, and API keys leaked into them | tells you whether this environment has ever extracted anything, and whether the credential is sitting in plain text |

### Step 2 — Audit + plan (dry-run) [SONNET]
```bash
python3 scripts/scan_and_plan.py <out_dir> --limit 300 --layers partner,product,account \
    --only-with-attachments          # optional: invoices with an attachment only

# Exclusions and confidence bar (see GOTCHAS 16-17):
python3 scripts/scan_and_plan.py <out_dir> --all-history \
    --exclude-partner-ids 1491 \    # supplier whose account depends on something not in the line
    --exclude-account-codes 4009 \  # regularisation account, not a real classification
    --pending-below 0.9              # below this consistency -> seeded as 'pending', not applied
```
`--keep-dated-names` disables the default filtering of single-use descriptions (those carrying a
date, a week or a batch). `skipped.csv` says WHY each row was dropped, and `summary.json` breaks
the plan down by `state` and the discards by reason.
Produces `seed_plan.json` + `preview.csv` + `skipped.csv` + `summary.json` + **`audit.md`** (the
audit of how they record + the proposal to change) + `quality.json`. **Show the user the
`audit.md` and the `summary.json`** plus a sample of `preview.csv`. **[STOP]** Confirm before
applying; ideally fix whatever the audit flags first (tax ids, duplicates). If a lot is discarded
for ambiguity, suggest lowering `--consistency` or reviewing the data.

### Step 3 — Apply [SONNET]
```bash
export OCR_BACKFILL_APPLY=1
# ODOO_* env pointing at the TARGET (which may DIFFER from the one scanned, e.g. staging)
python3 scripts/apply_seed.py <out_dir>            # or --limit 50 for a trial run first
```
The plan is portable: you can scan one Odoo (e.g. production) and seed another (e.g. staging) by
pointing the env at the target. Before writing, `apply_seed`:
1. **model preflight** — `ai.memory.alias` must exist (the OCR stack installed on the target).
2. **clone signature** — compares supplier names between source and target; if they do not match,
   the target is a different database (the `res_id`s would point at different records) → it aborts.
3. **drift filtering** — skips rows whose `res_id` no longer exists on the target (an older clone
   with records created later), so no dangling aliases are created (`--keep-dangling` to skip the
   filtering).

`save_alias` is idempotent (entries are born `state='approved'`); re-running does not duplicate.
Note: it is one RPC call per alias → thousands of aliases take minutes (run it in the background).

### Step 4 — Verify [SONNET]
```bash
python3 scripts/verify.py
```
Counts the approved aliases per model/context. Recommend a **smoke test**: re-extract an invoice
from a supplier already seen and check that it assigns supplier + account with no correction.

**If the smoke test fails, the real error is NOT where it looks like it is**:
`extract_error_message` is a compute of the IAP module that returns *"An error occurred"* whenever
`extract_status` is not in its map, and writing `False` to it does not clear it. The actual detail
(exception type, message and `file:line`) is in the **document's chatter** (`mail.message`) and,
if the request made it out, in `ai.request.log`. Order to diagnose: chatter → `queue.job.exc_info`
→ `ai.request.log`. And note: `queue.job` records **cannot be deleted** (no group has delete).

