---
name: dynapps-quotation-builder-value-based
description: >-
  Generates a value-based Odoo commercial proposal (closed price per app, no hours) as an interactive
  single price list in a web HTML + executive PDF under the Dynapps brand, from one shared extra_data.json,
  filling in the sale.order with the "Implantación v2" template. Multi-language via locale packs (language ⊕
  country composition: es/en/fr/nl + BE/CH layers; es = byte-parity reference, en = engine base); each
  operator uses their own Dynapps MCP connector. Modes: predemo (no prices) and live (interactive, real
  prices); both via generate_proposal.py. SOLE quote generator in the suite: quotation-builder (hours/Studio,
  template id=1) is DEPRECATED and only used to edit existing legacy quotes. A small client with 1-3 process
  lines inside the MVI conditions goes to quotation-builder-mvi instead. Triggers: create a quote,
  commercial proposal, Odoo implementation quote, per-app proposal, interactive price list, Dynapps
  proposal PDF, Odoo value-based quotation, multi-language proposal.
version: 7.3.0
model: opus
effort: high
---

# Quotation Builder — Value-Based

Odoo implementation quotes with a **closed price per application**: per-app cards, everything included, no hours. Core absorbed into the base price. The price is built **live** with the client on a **single interactive list**.

## Important (non-negotiable)

- **This skill is written in English; the OUTPUT language is decided by the COUNTRY, never by preference.** `resolve_locale` picks it (explicit `--locale`/`extra_data.locale` > Odoo DB > lead-country hint > default `en`) and the country layer composes on top (`nl-BE`, `fr-BE`, `de-CH`…), which is what also carries that country's fiscal and legal rules — see §Locale selection and `references/localization.md`. Resolve it ONCE and inject the same code everywhere. **The JSON/feature KEYS stay Spanish** whatever the locale, because the template JS branches on them (gotcha 15): never translate a key, a workbook sheet name, or an intake match keyword.
- **The client NEVER sees hours** — not in the cards, not in the PDF, not in the Excel `Propuesta` sheet; hours are internal (Timesheets). **Core is absorbed into the base**, never a line item.
- **The price IS shown to the client, live**, on the `live` **interactive single list** (checkable apps + support + custom dev + add-ons), TOTAL recalculated instantly; **one** "Implementation price" block, **NO "Recommended/Custom" concept** shown to the client. `predemo` shows **NO** prices, and it is what goes to the client **BEFORE the demo**, so they validate the scope; the price comes after the demo (Dyn Brain 5722).
- **General Configuration and Go-live support: always included, cannot be unchecked** (Config: €500 base minimum · support: dynamic 10% of the checked apps) · **±15% adjustment** free for the rep, **>15% requires management approval** · **⚙️** add-ons carry the Dynapps Core module, invisible to the client.
- **The productivity factor is a MANAGEMENT lever and it NEVER moves the price** — `rates.productivity_factor` (`Parameters`, 0.92 base) decides how much of the work product and AI absorb instead of consulting. Lowering it cuts the INTERNAL hours and raises the effective rate (`base ÷ factor`), so the gain is Dynapps **margin**: never a discount to the client, and never a shorter client-facing calendar (weeks come from UNFACTORED hours). The internal sheet states the target out loud — effective rate, budgeted hours, hours absorbed by product and AI, and **IA & Product Value = price × (1 − factor)**. Internal only.
- **Real content, not generic** — `app_overrides.features` and `dev_overrides` are MANDATORY and personalized; template text = rejected proposal.
- **The Analysis sells itself first** — the gateway panel lists its deliverables explicitly ("Al terminar el Análisis, te llevas:" blueprint, implementation plan, methodology, timeline, **locked budget** highlighted), next to the fixed price and "Se contrata primero, por separado · precio fijo", **above** the two rights (to stop, to the locked price). Web + PDF, every `live` deal; strings `deliv_*`/`gw_sep_chip` per pack.

**Dates — legal constraint:**

- **Dates are indicative, never contractual** — the client-facing timeline is RELATIVE by design (phases in weeks, Gantt S1..Sn).
- Calendar milestone dates only exist when the deal opts in (`anchor_dates` / `deal.fecha_decision`) and **ALWAYS** render with the "orientativas, no contractuales" disclaimer (web + PDF).
- **Hard country gate**: locales composing a `_NO_CALENDAR_DATES_CC` country layer (**BE** — Belgian contracts forbid fixed milestone dates) **ABORT generation** if any calendar date would reach client output; **no layer can bypass it** (`_anchor_dates_gated()` → exit 3).

**Order and views** — triggers here, full render contracts in `references/workflow_detail.md` §Render contracts.

- **Client-first ordering (v5)**: opens with "How we understand your business" (**context + ambition ONLY**), not with Dynapps; problems appear **ONCE** (challenges, **max 4**); mutual commitment renders **AFTER** pricing. Order table: `references/html_structure.md`.
- **Process view**, auto-on at **≥6 scoped apps across ≥2 end-to-end processes**: the metro map **REPLACES** the mindmap at `#mapa`, **no prices on the map**, stations two-way synced with the checkboxes; small deals keep the flat view **untouched (byte-parity)**; override `extra_data.module_view = "process"|"flat"`.
- **Involvement per module in DAYS/week** on every app card; **% is internal** (20% = 1 day); always on in process view, flat view **only when explicit** (byte-parity). **Workbook app tabs grouped BY PROCESS** (v4.19+, per-app LEGACY fallback for older workbooks; deployed bases need a re-deploy).
- **Construction vertical** (`deal.vertical='inbuilt'`): narrative app card **without price** + the `#vert-block` Investment tile; closed price **WITHOUT hours** (no 10% support) but it **does** enter future migration. `references/inbuilt_app.md`.

## Locale selection

- **Locale packs** (`locales/<code>/`, 9 JSON files per language); the engine (`scripts/*.py`, `templates/*.html`, `shared/*`) is **shared and never forked** — a new language/country is a new pack, **never an engine edit** — and `locale_loader.py` is the single access point. Composition, BE/CH stub gate, KEYS-vs-titles: `references/localization.md`; governance: `locales/README.md`.
- **Workbook chrome (`sheet.*` + `params.*`) is ENGLISH in every locale**, carried by `en` alone and inherited; the sole exception is `sheet.formacion_line`, which renders a CLIENT-facing feature and stays per language (gotcha 22 — byte-parity does NOT cover that boundary; `tests/workbook_chrome_boundary_smoke.py` does).
- **`es` = byte-parity reference** · **`en` = engine base/default** · `fr`/`nl` = reference packs · **BE/CH compose but are stubs**: the loader **BLOCKS client-facing output** until `verify_from_odoo` + fiscal/legal sign-offs are in.
- **Connector-agnostic**: each operator uses **their own Dynapps MCP connector**; entity, currency, template id and product ids are **discovered at runtime** (`odoo_discovery.py`), never hardcoded; product ids **not variabilized**.
- **`Parameters` sheet = per-deal pricing AND dates**, overriding the pack for that deal only; `deal.fecha_decision` is the **date ANCHOR** and the calendar cascades from it. Priority `extra_data.anchor_dates` > sheet > nothing (**no sheet → no override**, byte-parity intact). The sheet's chrome is ENGLISH in every locale (`strings.params.*`, `en`-only, inherited); its column HEADERS are a parser contract — see `workflow_detail.md` §Parameters.
- **Resolution** (`resolve_locale`, highest first): explicit `--locale`/`extra_data.locale` > Odoo DB (`master`→`es`) > lead-country hint > default `en`; the code **is logged**. Resolve **ONCE**, inject the same `--locale` everywhere + into `extra_data.locale`. Fallback: UI strings → `en`; **pricing/fiscal/product ids → HARD ERROR**, never silently Spanish.

## Workflow

- **Fast start:** sheet already filled and the rep just wants the HTML/PDF (URL/ID) → **skip STEP 1-3**, go to STEP 4.
- **Odoo connection (STEP 1, 3):** **odoo-pilot** (`auth.sh` + `search_records.sh`; **required** to create the spreadsheet and pull binaries/photos) or the **Odoo MCP** (`orm_search_read`/`orm_write`, superuser, Cowork without `auth.sh`). Proxy gotchas (no binaries, big outputs to file, spreadsheet keyed by `order_id`): `workflow_detail.md` §1.1.bis.
- **One local chain, not eight turns:** `python3 scripts/qb_run.py prepare|generate|all …` — `prepare` = `order_totals` → `match_cases` → `fetch_case_images` → `scaffold_extra_data` → `brand_lint`(json); `generate` = `pregate_extra_data`(pre-render) → `generate_proposal` + the **two post-render gates concurrently**. Never touches Odoo; **never crosses a human STOP** (`all` refuses `--only pdf`).
- **Commands, gates and traps per step live in `references/workflow_detail.md`** — §1 CRM, §3 spreadsheet, §4.0 STEP 4 sub-steps, §STEP 5. Load it at the start of execution and follow it, don't improvise.

| Step | Runs as | Core contract |
|---|---|---|
| **1 — CRM extraction** | `Agent(model: sonnet)` + the STEP 1 prompt (`references/subagent_prompts.md`), `lead_input` injected | returns `parsed` + `raw` + `existing_orders`; `{"status":"missing_toma"}` → run `sales-skills:lead-data-collection`, retry. **The orchestrator NEVER reads `xx_data_collection` or raw attachments.** Resolve existing quotes ("new v2 vs. convert") **HERE**, before pricing |
| **2 — Apps + pricing** | opus, with the rep | detection (`references/pricing_apps.md`) → iterate apps / add-ons / adjustments → closed prices, analysis, custom dev (`analysis_pricing.md`), duration (`timeline_rules.md`). Complexity triggers (multi-company, >5 apps, >100 users, regulated sector, >2 devs) → size support/add-ons up. **Present the summary and wait for confirmation before writing anything** |
| **3 — Fill the spreadsheet** | `Agent(model: sonnet)` + the STEP 3 prompt, `sale_order_id` + `budget_json` + pricing injected | returns `spreadsheet_id`, `impl_total_final`, **`completeness`** (B44 / B51-B52 / Gantt / `check`); fail or missing → **relaunch STEP 3**. **Only write path = `fill_spreadsheet.py`** (direct cell edits FORBIDDEN); creating the sheet and pulling binaries/photos goes through **odoo-pilot**; size the write with `scripts/spreadsheet_payload.py measure|verify` |
| **4 — Draft + web HTML** | opus | `extra_data.json` → `generate_proposal.py --only web`; gates below |
| **5 — Publish, then attach the PDF** | opus | 5.1 publish + note · 5.2 PDF from the copied selection; idempotent (an existing version is UPDATED, never duplicated) |

**🛑 MANDATORY STOP after STEP 3 — wait for the sales rep's review.** The rep MUST open the spreadsheet in Odoo (URL from the pack's `instance.json` `sales_url_pattern`, e.g. `{odoo_url}/odoo/sales/<sale_order_id>`; `odoo_url` from runtime discovery — **never hardcode**) and review apps/add-ons/devs/prices. **Do NOT move to STEP 4 until the rep explicitly replies "ok, go ahead" or equivalent.** Rep edits in Odoo are picked up: STEP 4 re-reads the sheet.

**STEP 4 — always ask first:** (1) **mode**, `predemo` (no prices; the default while the demo has not happened yet) or `live` (interactive, source of the PDF); (2) **team** names (for `fetch_team.py`). **ALL corrections go into `extra.json`** and are re-injected before regenerating — the generator renders `extra_data` only, never auxiliary files.

**🛑 Two-phase flow: STEP 4 generates ONLY the web HTML** (`--only web`); the PDF waits for the client's copied selection (STEP 5.2, `--only pdf`, body = ONLY what was checked). Same `extra_data.json` for both: web = v5 client-first order, PDF = static executive A4 mirror via headless Chrome. Mode matrix, sections and the `extra_data` spec: `references/html_structure.md` → "STEP 4 — Execution detail".

STEP 4/5 gates — **index only; the full spec, with commands, is `workflow_detail.md` §4.0.** Run in this order, every generation:

1. **Line re-sync guard** (`order_totals.py`) + **completeness gate → ABORT back to STEP 3** if B44/B51/B52 = 0 or the Gantt has no painted cells. Matching totals never prove completeness (ANRO PR4035).
2. **`scaffold_extra_data.py`** derives the derivable half; its `[[WRITE: … — verify]]` markers **abort generation** while prose is missing. Non-ES: prose in the target language + `extra_data.locale`.
3. **`match_cases.py --locale <code>`** — ⚠️ `--locale` MUST match the intake's language or every case scores `sector=0`. `metric` **and** `image` mandatory, output used AS-IS, **quotes never invented**.
4. **`brand_lint.py` on the JSON** → fix → **rep sign-off on `extra_data.json` before generating**.
5. **`pregate_extra_data.py` (PRE-render)** — FAIL (exit 1, nothing rendered) only for a wrong figure, an illegal one, a prose marker or list-shaped `features`; everything else WARNs. Never `--strict`.
6. **M5 render gates** — a stub pack needs `--allow-stub`; a **placeholder leak → exit 3, nothing written**, always, `--allow-stub` included.
7. **Post-render gates, MANDATORY before delivering ANY rendered HTML (STEP 4 and 5.2):** `brand_lint.py --html` **then** `verify_totals.py --html … --extra-data …`; exit 1 → fix and regenerate, **never deliver a failing file**. They stay post-render (PR4021 is a render-time bug class).
8. **STEP 5.1** publishes and posts/updates a rich-HTML internal note on the **`sale.order`** (not the crm.lead), linking the text **"HTML proposal"**, deleting the previous "Proposal PR<NUM>" note. **STEP 5.2** re-runs both gates on the `-pdf.html`, uploads out-of-band and re-posts with `attachment_ids` (ANRO PR4021).

## Key rules

Non-negotiables are above; the **complete list (model principles + numbered rules) is `references/rules.md`** — single source of truth, load it for a rule's detail. Operationally critical:

1. "Implantación v2" template, id **discovered by name at runtime** (`odoo_discovery.py`), never hardcoded (15 on `master`/`es` only) · context `lang: es_ES` when searching products · **never invent data** (thin intake → ask the rep).
2. Support = **10% of the implementation PRICE** (config + checked apps + checked custom dev) **plus analysis**, mirroring the sheet's **E36 -> F36** (the € leads, the hour is derived), recalculated live. Price base, not hours, so the **productivity factor never moves the price** (gotcha 19). Apps pre-checked via `apps_recomendadas`; **add-on matching is by substring** — verify names when an extra doesn't activate. **Add-ons / AI unchecked by default**, conditional items hidden when their app is unchecked; future migration selectable (15%/24).
3. Zero brochure adjectives · sentence case · sector language (glossary), validated with `brand_lint.py` · **metric-first cases linking to the Dynapps case-studies site**, never the Odoo backend · team from `hr.employee`, **never invent an email**, never rewrite the domain · team/cases optional, hidden without data, never invented.
4. One `extra_data.json` → web + PDF · self-contained (Readex Pro + logo data URIs) · official Odoo icons, no emoji · **"Dyn Cloud"** hosting · lowercase `propuesta-pr<NUM>.html` · PDF body = **ONLY what was checked**, no showcase appendix.
5. Recurring costs from the sheet or `RECURRENT_DEFAULTS`, override via `extra_data.costes_recurrentes`; future migration, **MCP (€50/month)** and **Field Service (+€2/user/month)** join them if checked · partner manager **Micaela Gil Rother (migi@odoo.com)**.

## Definition of done (Ine checklist — verify on EVERY deliverable before handing it over)

☐ One language, zero leaks · ☐ zero em-dashes · ☐ no all-caps anywhere (CSS included) · ☐ v5 section order (problems once, in challenges; commitment AFTER pricing) · ☐ max 4 challenges · ☐ real client logos when available (never a text name if the logo exists; never a fabricated logo) · ☐ quotes never invented (genuine site quote or factual headline) · ☐ rep photo + next-step line on closing, no phone · ☐ recurring totals verified (`verify_totals.py` PASS) · ☐ file complete (opens fully, no truncated sections, no placeholders).

## Gotchas

One line each; **`references/gotchas.md`** carries the incident, the recipe, the enforcing script and the known gaps under the same numbering.

1. **Empty intake data** — don't parse, run `lead-data-collection` first. The adjustment **%** applies over `base + add-ons`, never the base alone.
2. **Per-unit add-ons** (multi-warehouse, multi-company, extra pages) are price × quantity — **always ask for the quantity** (`"Name:N"`).
3. **Spreadsheet not found** = wrong template or an ORM-created order → §3.0 recipe; **never hand-craft `spreadsheet_data`**.
4. **Invoicing and Accounting are separate apps** — a client can have Invoicing without Accounting.
5. **Max 10 analysis sessions / 5 custom dev items** — template slot limit (`fill_spreadsheet.py`).
6. **Templated `extra_data.json` = rejected proposal** — `features` (EXACT spreadsheet keys) + `dev_overrides` in real client context. `dev_overrides` reads **`description`**, not `desc`: with `desc` the title renders and the body silently falls back to the generic catalogue line.
7. **No all-caps ANYWHERE (v5)**, not even `text-transform: uppercase`; brand always **"Dynapps"**. `brand_lint.py` CAPS/MARCA gates the text — **the CSS half is NOT gated**, it is on you.
8. **Typography** — don't enlarge fonts (hero ≤ `clamp(40px,6vw,72px)`, titles ≤ `clamp(23px,3vw,34px)`) or padding (~64px).
9. **Spreadsheet writes → odoo-pilot, NOT the generic MCP** (no `describe_model`/onchange/binaries/sheet creation; fields yes). `fill_spreadsheet.py` needs **dict `{content, style}`** cells.
10. **No Chrome and no root in the Cowork sandbox** — `find_chrome()` reads `QB_CHROME_BIN` / `PLAYWRIGHT_BROWSERS_PATH`; no-root recipe in `workflow_detail.md` §4.bis.
11. **Team photo must be clean ASCII base64** (a `b'iVBOR...'` repr renders broken); source of truth = `extra_data.equipo`.
12. **Odoo license is a PER-USER rate and the engine multiplies it — NEVER inject the total** (60 × 334.80 = 20,088 re-multiplied = **1,205,280**, the PR4021 class). Set `num_usuarios` **+** a per-user `price`/`amount_per_user`: reference **Custom 334.80 €/user·year (27.90/month, yearly, list price, no first-year discount)** · Standard 178.80; `verify_totals.py` hard-FAILS above €500/user. **Every proposal splits staff into full users and Light Users** (Dyn Brain 5723): ask BOTH counts, `[ASK]` either → ask first. **Light User** (4.90 €/user/month = 58.80/year, same on annual or monthly billing, no discount; HR-record-only or employee features only) = its own annual line from **`num_usuarios_light`** (`Parameters` `deal.num_usuarios_light`), never hand-written; deployed bases get the rows with `scripts/patch_light_user.py`. odoo.com prices by **geo-IP**: from Spain every URL returns the ES pricelist, so only ES is verified.
13. **Normalize team photos to a centred 512×512 square** before injecting (portrait + `object-fit:cover` crops the face). **Known gap:** `fetch_team.py` doesn't.
14. **Text that sounds like AI = rejected proposal** — no filler words or intros/outros, no 4+ same-length sentences in a row, active voice, the "café test". `brand_lint.py` SLOP / FLUFF / RITMO: fix **before** generating.
15. **7 multi-language traps** — `--locale` moves only the chrome; **prose is per language**; the **KEYS stay in SPANISH** (the template JS branches on them); non-es `features` use the `{"title","expl"}` object form; license is per user; non-es recurring cards need a numeric **`amount`** (the `en` pack's own `€1,080.00 / year` parses as **1.08**); the **pain→answer matcher keys on Spanish substrings**; and the hero's **sector + companies count come only from the toma**. **ES byte-parity gate: `python3 tests/golden_check.py` bare, no flag.**
16. **Big Text-field writes — NEVER the attachment value-ref.** `{'__ref__':'attachment','id':N}` in `spreadsheet_data` writes fine and **silently corrupts** the record (later `OwlError` / `SyntaxError`). Measure the path (`spreadsheet_payload.py measure`), then **always read back + `json.loads()` verify**, plain `orm_write` included.
17. **A redeployed canonical base silently wipes the deal's sheet** — new `sale.order.spreadsheet`, no revisions, no snapshot, nothing to recover. The tell: `apps_total` = the sum of **every** base price (16,250 € for the 13 apps) with `devs_total: 0` and the completeness gate firing. Check the ACTIVE apps are the deal's, not all of them; rebuild from the last delivered HTML and require the delivered totals to reproduce exactly. **Archive `extra_data.json` next to every delivered file** (STEP 5) so this never costs a session again.
18. **Sheet references in formulas are always quoted** — use `sheet_ref()`; an unquoted hyphenated tab (`Order-to-Cash`) parses as subtraction and breaks the workbook (`#BAD_EXPR`) while the HTML still looks correct. Gate: `tests/sheet_ref_quoting_smoke.py`. Workbooks from before the 2026-08-05 base repair still carry it: repair in place (`ir.cron` recipe), never redeploy the base (gotcha 17).
19. **The productivity factor never touches the price** — `rates.productivity_factor` (`Parameters`, 0.92 base) is a MANAGEMENT lever: lowering it cuts the INTERNAL hours and raises the effective rate (`base ÷ factor`), and the gain is **margin**. That is why go-live support is 10% of the **price**, not of the hours: derived from factored hours, lowering the factor discounted the CLIENT (203 € from 0.92 to 0.70 on PR4086, ~9 € per hundredth). It does not shorten the client-facing calendar either. Rate priority: **a LITERAL in B4/B5/B6 (a deliberate local act) > `Parameters` > pack defaults**; new workbooks emit B4/B5/B6 as formulas pointing at `Parameters`, so management governs unless someone types a number over it. No silent override in either direction — the `Parameters` value used to be read by `sheet_overrides()` and then ignored by `soporte_rates()`. Gate: `tests/productivity_factor_smoke.py`.

## Agent Routing

| Phase | Model | Runs as | Why |
|---|---|---|---|
| STEP 1 CRM extraction · STEP 3 fill the spreadsheet | sonnet | Subagent (Pattern A) | Mechanical: auth + search + parsing / JSON + script + update |
| STEP 2 apps + pricing | **opus** | Main | Business decisions, interaction with the rep |
| STEP 4 draft + generate HTML | **opus** | Main | Personalized copy |
| STEP 5 publish (5.1), attach the PDF (5.2) | **opus** | Main | Publish + note; PDF `--only pdf` from the copied selection. Idempotent |

**Subagent contract:** self-contained prompts injecting `lead_input` (STEP 1) or `budget_json + sale_order_id + pricing` (STEP 3); they return **structured JSON, not prose**, don't decide pricing, don't write PDF copy. Fan-out may be disabled — then run the step inline and say so.

## Delegates to

- **common-skills:odoo-pilot** — **REQUIRED** to create/write `sale.order.spreadsheet`, upload HTML/PDF as `ir.attachment` and pull photos/titles from `hr.employee`; needs a **per-operator Odoo API key** (`auth.sh`). Field read/write also works (OPTIONAL) via the operator's Dynapps MCP connector
- **common-skills:dynapps-brand** — **VISUAL identity only** (colors, Readex Pro, logos, gradient). **VERBAL identity** is **dynapps-voice** (group skill, draft), implemented by the `brand_lint.py` v2 lexicons (DASH/MARCA/REGISTRO/PRESION…): defer voice questions there, not to dynapps-brand
- **common-skills:content-collection** — real success cases for `casos_exito` by sector
- **utilities:publish** — STEP 5: publishes the HTML → public URL (same destination on re-publish). **sales-skills:lead-data-collection** (optional) — prerequisite when `xx_data_collection` is missing

## Consumed by

No other skill consumes this one. It's an endpoint of the commercial workflow.

## References

| File | Content | Load when |
|---|---|---|
| `references/rules.md` | model principles + the complete numbered rules (single source of truth) | a rule's detail |
| `references/gotchas.md` | gotchas in full: incidents (PR4021, ANRO PR4035), recipes, enforcing scripts, known gaps | a gotcha bites |
| `references/subagent_prompts.md` | self-contained STEP 1 / STEP 3 sonnet prompts | STEP 1, STEP 3 |
| `references/workflow_detail.md` | commands for STEP 1/3/4/5 (§4.0 + gates + `qb_run.py`), §Parameters, §Render contracts, §Test suite index. Connection/batch patterns are NOT in this skill: `references/shared/odoo-connection.md` | start of execution, STEP 4/5 |
| `references/pricing_apps.md` | pricing table: **13 apps** × base + priced add-ons (canonical count) | STEP 2.1 |
| `references/addon_details.md` | add-on detection signals from the intake data | STEP 2.1 |
| `references/analysis_pricing.md` | analysis hours (online/on-site), hours by dev type (€75/h), adjustment rules | STEP 2.2-2.4 |
| `references/timeline_rules.md` | timeline algorithm, **3 pace profiles (40/50/60 h/week)**; hours never shown | STEP 2.5, STEP 3 |
| `references/spreadsheet_filling.md` | `budget_json` structure (`duracion`/`formacion` MANDATORY > 0), formula rules, §Write path | STEP 3 |
| `references/html_structure.md` | modes, client-first order, web+PDF sections, brand, `extra_data.json` + involvement spec, case matching, `generate_proposal.py`, writing rules | STEP 4 (all of it) |
| `references/sector_glossary.md` | Odoo terms → the client's sector vocabulary. **Governs the writing** | STEP 4.3.b |
| `references/localization.md` | packs and composition, language layers, KEYS-vs-titles, per-language content, per-locale money, golden gate | non-ES, pack edits |
| `references/_assertions_inventory.md` | every SKILL.md assertion mapped to the code or prose that enforces it — check it before changing a rule | editing a rule or a gate |
| `references/inbuilt_app.md` | Dynapps Construcción: render contract, detection signals, migration state | construction deals |
| `references/process_taxonomy.md` | process view: APQC PCF / Signavio analysis, boundaries, app→process mapping | process-view deals |
| `references/mcp_photo_access.md` | employee photos via MCP, and **why the obvious paths fail** | STEP 4.3.c |
| `locales/README.md` | pack architecture + governance (per-file owners, sign-offs, fallback) | pack work |
| `references/add_country.md` | country **stub → live** (`add_country.py` + `odoo_discovery.py`); BE/CH checklists in `references/add_be_ch_plan.md` | new country, BE/CH |
