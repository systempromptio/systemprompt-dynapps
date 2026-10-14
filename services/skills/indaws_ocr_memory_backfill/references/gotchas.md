# Gotchas — ocr-memory-backfill

Los 20, enteros y con su evidencia. `SKILL.md` lleva el indice de una linea cada uno,
con la misma numeracion.


1. **Two `context_key` formats** (confirmed in the code): account and product-description use
   **`partner_id:<id>`**; taxes use **`partner:<id>`** (no `_id`). The skill already emits the
   right one per layer — do not unify them.
2. **`save_alias` defaults to `approved`, but it DOES take `state`** (verified in the Core:
   `ai_memory_alias.py`, the value lands in the `create` and is bumped on an existing row). Only
   `approved` resolves during extraction; `pending` waits in the review queue. So the solid
   evidence goes in `approved` and whatever is below the bar goes in `pending` — do not assume
   seeding is all-or-nothing.
3. **`name` is truncated to 128 chars** and the match is `=ilike` (case-insensitive). Very long
   descriptions are cut the same way when seeding and when looking up → consistent.
4. **An alias only matches if the AI extracts the SAME text.** VAT numbers and codes are stable
   (high hit rate); descriptions and trade names vary — hence the thresholds. It is not magic: it
   raises the initial accuracy, it does not guarantee 100%. Measured case: the alias `Vracht`
   did **not** match the real line `"Vracht week 33 — 4 ritten Eindhoven-Amsterdam"`. The match is
   on the **whole** text, never partial, so descriptions carrying a week, a batch or a month inside
   are not worth seeding.
5. **`account.tax.find_record` ignores `field_name`** (it matches by `name` on any field); that is
   why the tax layer seeds `field_name='name'` and the value is the tax label.
6. **Multi-company**: run once per company with its own `ODOO_COMPANY_ID`. `env.company` is set
   through the context; do not mix them.
7. **v19 / indaws_security transport**: XML-RPC is BLOCKED on Odoo ≥19 and on hostings with
   indaws_security. `_odoo.py` already supports **JSON-RPC** (`/jsonrpc`) as well as XML-RPC; with
   `ODOO_PROTOCOL=auto` (default) it detects and uses JSON-RPC if the endpoint answers like Odoo.
   Force it with `ODOO_PROTOCOL=jsonrpc|xmlrpc`.
8. **`display_type='product'` on v19** (critical): from Odoo 17 product lines carry
   `display_type='product'` (NOT `False`). Filtering by "truthy" skips ALL lines → 0 product/account
   aliases. Skip ONLY `line_section` / `line_note` (already fixed).
9. **Own / intercompany tax ids contaminate**: receipts with no tax id that were given the
   company's own end up making an own tax id "point at" some random supplier. `scan_and_plan` reads
   `res.company` and **excludes those tax ids** from the VAT layer automatically; the audit lists
   them so they can be fixed in the master.
10. **Source ≠ target (prod→staging)**: the `seed_plan.json` carries the `res_id`s of the Odoo that
    was scanned; they only match if the target is a CLONE of the same lineage. `apply_seed`
    validates by **clone signature** (supplier names), not by "equal ids": an OLD clone is valid,
    it merely has drift (new ids that are missing) → those are filtered, not fatal. Corporate moved
    from `indaws.es` to `portal.dynapps.es` (login `aylen.garces@dynapps.es`); our own stagings
    (indaws.cloud) may be old clones and may NOT carry the `dynapps_ai_*` stack deployed → the
    `ai-odoo-ocr` repo has to be deployed there first (git/deploy, not over RPC) and then installed
    over RPC. Login on pre-rename clones: `aylen@indaws.es` + the production API key.
11. **The jobrunner cron is born set to once a day** (`queue_job_cron_jobrunner`): useless. Bring it
    down to 5-10 minutes. It helps that `queue.job.create` calls `_ensure_cron_trigger()`, so with
    the cron properly configured the jobs start almost immediately through `ir.cron.trigger`,
    without waiting for the interval.
12. **`ai.extract.config` is born with no provider** → *"No AI provider set on the extraction
    config"* on the first extraction. Since each provider module creates its own `ai.provider`, you
    have to pick one and assign it on the config by hand.
13. **The API key leaks in plain text into the chatter** when Gemini fails: `str(e)` of a
    `requests.HTTPError` includes the URL, and Gemini passes the credential in the query string
    (`...:generateContent?key=AIza...`). That text is persisted in `message_post` **and** in
    `ai.request.log.error_msg`, readable by anyone with access to the document. Fixed in the Core
    (`scrub_secrets`) on 16.0/17.0/18.0/19.0 — but check it is deployed before handing over an
    environment, and sanitise what is already written (`AIza`, `key=`, `api_key=`). OpenAI and
    Anthropic send the key in a header, so they never showed up.
14. **Spanish strings in the Core data**: `dynapps_ai_provider_gemini/data/ai_provider_models.xml`
    shipped `label`s in Spanish. Fixed on 17/18/19, but the file is `noupdate="0"`, so **any patch
    you apply over RPC to that data is reverted by the next module upgrade** — measured. The same
    applies to view `arch_db` patched by RPC.
15. **Pick a Pro-class model for the lines.** Measured on five real invoices:
    `gemini-flash-latest` returned **0 lines** on three of them and omitted `quantity` on the
    fourth (turning a €2,218.75 invoice into €61.25); `gemini-pro-latest` got all five right,
    discounts and PO matching included. Also set
    `ir.config_parameter dynapps_base_ai.temperature` to **0.0** — it ships at 0.2, i.e. random
    sampling on what is a transcription task.

16. **Not every scattered supplier is a data problem.** Before excluding one, look at WHAT
    discriminates the account. Measured case: a leasing company with 15 accounts had **one account
    per vehicle registration**, while the line description was always the literal `'Leasing'`. No
    key — product, description or supplier — can separate that: the discriminating fact lives in
    the contract, not in the document. That is accounting per asset, not something the memory can
    learn. Exclude it and say why; do not lower the thresholds to force it in.
17. **Regularisation accounts contaminate.** Year-end "pending invoices" accounts (`4009…` in
    l10n_es) appear sprinkled across good suppliers' history. They are not a classification: seed
    them and you teach the engine to book real invoices into the accrual account. Exclude them by
    code prefix.
18. **Measure which key discriminates BEFORE seeding, not after.** One `read_group` over the
    history answers it: for each candidate key (supplier / description / product / description +
    supplier), how many keys are ambiguous and what percentage of lines the dominant value would
    get right. Measured at a real customer: supplier alone 79.9%, description alone 91.0%,
    **description + supplier 94.2%** — which is exactly what the account mapping indexes. That
    number is the one to report to the customer, not how many entries the memory has.
19. **`ai.extract.config.memory_count` is NOT the memory counter.** It is `store=False`, has no
    `depends` and does not count the aliases: measured at **0 with 730 seeded**, and 21 against
    810 at another. Never use it to confirm the seeding worked — count over `ai.memory.alias`
    (that is what `verify.py` does).

20. **The history's line description is the WRONG key, and this is the skill's main trap.**
    Verified on real extractions (A2 `ai_raw_result` vs its own posted lines): the PDF said
    `CNL MODULAR 200 AISI316L L2530 H60-70 FONDO`, `ALAMBRE RECOCIDO NUMERO 8 (1,30)`,
    `AGUA NATURA 500 PET C/35`; Odoo had `Compra de otros aprovisionamientos` and
    `Gastos diversos`. **0 of 8 invoices matched.** Supplier names diverge too
    (`Nortcoat Galicia, S.L.U` vs `NORTCOAT GALICIA SL.`; `ALFREDO RODRIGUEZ CORRAL` vs the
    reversed `RODRIGUEZ CORRAL ALFREDO`). Only VAT survives. Note also that real document text
    often carries a period inside (`ROISAN JUNIO DIA - 25`,
    `PotenciaP1 15/05/2026-14/06/2026`), so it is single-use even when read from the PDF —
    filter it there too. Corollary for the report: a seeded row is not coverage. The only
    number worth giving a customer is how many lines resolve on documents they have not seen.

