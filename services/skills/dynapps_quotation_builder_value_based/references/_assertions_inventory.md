# Assertion inventory — the measuring stick for the F3 SKILL.md reflow

**Purpose.** `SKILL.md` (149 lines / 34,962 bytes ≈ 8,740 tokens) is scheduled to be reflowed to
~10 KB in phase F3. The owner's condition is that the reflow be **provably lossless**. This file is
the oracle: every normative assertion currently carried by `SKILL.md` and `references/rules.md` is
listed once, with its location and a classification. **If an assertion is not in this table, F3 can
delete it silently.** Nothing here is new policy — this is a read-only extraction.

**Do not edit `SKILL.md` from this file.** The inventory records what *is*, including the stale and
contradictory bits (see §7); fixing them is a separate decision, not a reflow side effect.

## Classification legend

| Tag | Meaning |
|---|---|
| `CODE` | Already mechanically enforced. The enforcing script (and where findable, the function / literal message) is named. Safe to compress in prose: the gate survives in code. |
| `CODE-POSSIBLE` | Not enforced today, but a script could check it deterministically. Candidate for F4-style gates; until then it lives only in prose and MUST survive the reflow. |
| `PROSE-ONLY` | Requires human/model judgement (voice, relevance, sector fit, "does this read human"). Can never become a gate. Must survive the reflow as readable instruction. |
| `LEGAL` | Contractual or jurisdictional constraint. **Must survive verbatim** — never paraphrase, never compress. |
| `DUP` | Appears in both `SKILL.md` and `rules.md` (both locations given). F3 collapses the duplicate; the row records which side carries the unique detail. |

Counts: **248 assertions**, numbered 1-248 with no gaps (247-248 were added after the extraction and
sit next to their Gotcha 12 siblings). By strongest tag: **4 `LEGAL`**
(rows 46, 47, 48, 64), **122 `CODE`**, **41 `CODE-POSSIBLE`**, **81 `PROSE-ONLY`**. **64 rows are
`DUP`** (present in both `SKILL.md` and `rules.md`). Rows carrying two tags are counted once, by the
strongest: `LEGAL` > `CODE` > `CODE-POSSIBLE` > `PROSE-ONLY`.

---

## 1. Frontmatter / description (always loaded, even when the skill does not fire)

| # | Assertion | Location | Class |
|---|---|---|---|
| 1 | Produces a value-based Odoo proposal: closed price **per application**, no hours, as an interactive single price list (web HTML) + executive PDF, Dynapps brand | frontmatter `description` | PROSE-ONLY (routing) |
| 2 | Both deliverables come from **one shared `extra_data.json`** | frontmatter; §Important; Key rule 5 | CODE — `generate_proposal.py` single `--extra-data` input; `scripts/test_generate_proposal.py::TestRender` |
| 3 | Fills the `sale.order` with the **"Implantación v2"** template | frontmatter; Key rule 1 | CODE-POSSIBLE (`odoo_discovery.py` resolves by name; no gate asserts it was used) |
| 4 | Multi-language via locale packs; **language ⊕ country composition** (es/en/fr/nl + BE/CH layers) | frontmatter; §Locale selection | CODE — `locale_loader.load_locale`; `tests/composition_smoke.py` |
| 5 | `es` = byte-parity reference | frontmatter; §Locale selection; Gotcha 15 | CODE — `tests/golden_check.py` |
| 6 | `en` = engine base language | frontmatter; §Locale selection | CODE — `locale_loader` default; `scripts/validate_locale.py` |
| 7 | Each operator uses **their own** Dynapps MCP connector | frontmatter; §Locale selection | PROSE-ONLY (ops) |
| 8 | Two modes only: `predemo` (no prices) and `live` (interactive, real prices); both via `generate_proposal.py` | frontmatter; §Important; rules.md #26 · **DUP** | CODE — `TestModes` |
| 9 | **SOLE** quote generator in the suite; `quotation-builder` (hours/Studio, template id=1) is DEPRECATED and only edits legacy quotes | frontmatter | PROSE-ONLY (suite routing) |
| 10 | Frontmatter contract: `name` / `description` / `version: 5.0.0` / `model: opus` / `effort: high` | frontmatter | CODE-POSSIBLE (suite-level frontmatter lint) |
| 11 | Trigger phrase list (create a quote, commercial proposal, per-app proposal, …) | frontmatter | PROSE-ONLY (routing) |

## 2. `## Important (non-negotiable)`

| # | Assertion | Location | Class |
|---|---|---|---|
| 12 | The client **NEVER** sees hours — not in the cards, not in the PDF, not in the Excel `Propuesta` sheet; hours live in Timesheets (internal) | §Important; rules.md MP1, #1 · **DUP** | CODE-POSSIBLE (no gate greps rendered HTML for hour tokens) |
| 13 | **Core absorbed** into the base price; never shown as a line item | §Important; rules.md MP2, #2 · **DUP** | CODE-POSSIBLE |
| 14 | The price **IS** shown to the client, **live**, on the interactive single list (checkable apps + support + custom dev + add-ons) with the TOTAL recalculated instantly — **supersedes the old "Golden Rule 3"** | §Important; Key rule 3; rules.md MP6, #1, #26 · **DUP** | CODE — `TestModes::test_live_lista_unica_y_gateway`, `TestContext::test_pricing_totals` |
| 15 | `predemo` shows **NO** prices | §Important; rules.md #26 · **DUP** | CODE — `TestModes::test_predemo_has_no_prices` |
| 16 | **Single list, not two cards**; there is **NO** "Recommended/Custom" concept shown to the client | §Important; rules.md MP6, #26 · **DUP** | CODE — `TestNoRecomendadoCustom::test_sin_recomendado_custom` |
| 17 | Client-first ordering (v5): the document opens with "How we understand your business" — **context + ambition ONLY** | §Important; rules.md #18 · **DUP** | CODE (presence) — `TestModes::test_understanding_opener_present_all_modes`; ordering itself CODE-POSSIBLE |
| 18 | Problems appear **ONCE**, in challenges, **max 4** | §Important; DoD; rules.md #18, #32 · **DUP** | CODE — `generate_proposal.py` ~L865 `WARNING: %d pains recibidos …` + hard truncation `pains[:4]` |
| 19 | Mutual commitment ("The calendar, together") renders **AFTER** pricing, never before | §Important; rules.md #34 · **DUP** | CODE-POSSIBLE (`TestNewMejoras::test_anchor_note_and_juntos_lane` checks presence, not position) |
| 20 | Full section-order table lives in `references/html_structure.md` | §Important; rules.md #18 | PROSE-ONLY (pointer) |
| 21 | **Process view auto-switch**: ≥6 scoped apps across ≥2 end-to-end processes | §Important | CODE — `tests/process_view_smoke.py::TestProcessView` |
| 22 | The Odoo **metro map REPLACES** the mindmap/solution-map section (same `#mapa` anchor, before the module breakdown); the mindmap survives only in flat view | §Important | CODE — `process_view_smoke.py` |
| 23 | Hovering **any** station — lit or unlit — lights up its icon and its name | §Important | CODE-POSSIBLE |
| 24 | The **FULL** Odoo Enterprise network is always drawn: 6 lines, no milestones/terminus | §Important | CODE-POSSIBLE (data in `shared/app_registry.json` `flows`) |
| 25 | Line identities are fixed: Market-to-Lead carmine feeding into CRM ◎; Lead-to-Cash gradient; Procure-to-Pay violet; Record-to-Report midnight; Hire-to-Retire orange with the whole employee journey; Project-to-Result tangerine born **VERTICALLY** at Ventas ◎ ("de una venta nace un proyecto") and tunnelling under P2P/H2R | §Important | CODE-POSSIBLE (`app_registry.json`) |
| 26 | **ALL** money lines (L2C via Facturación, P2P direct, H2R via Gastos) converge into the **Contabilidad final hub** — the visual proof of Odoo's integration | §Important | CODE-POSSIBLE |
| 27 | Station states: lit = in scope & selected; dashed = in scope not preselected ("Fase 2"); greyed-out ghost = Odoo has it, this proposal doesn't (curated `flows.ghosts`, official icons, upsell without money) | §Important | CODE — `process_view_smoke.py` |
| 28 | **NO prices on the map** — it sells processes; money lives in the single list | §Important | CODE-POSSIBLE |
| 29 | Lit stations are **two-way synced** with the single-list checkboxes (station tap = checkbox toggle) | §Important | CODE-POSSIBLE |
| 30 | The L1 line **renames itself** by where the lit flow starts ("Del clic al cobro" → "Del lead al cobro" → "Del presupuesto al cobro"/Quote-to-Cash) | §Important | CODE-POSSIBLE |
| 31 | PDF gets the same map as a **static SVG**; `predemo` gets a non-interactive map | §Important | CODE-POSSIBLE |
| 32 | Small deals keep the flat view **untouched** (byte-parity) | §Important | CODE — `tests/golden_check.py` |
| 33 | Per-deal override: `extra_data.module_view` = `"process"` or `"flat"` | §Important | CODE — `process_view_smoke.py` |
| 34 | Network + app→process mapping live in `shared/app_registry.json` (`apps[*].process`, `processes`, `flows`); rationale in `references/process_taxonomy.md` | §Important | CODE (data location) |
| 35 | The co-signed workbook groups app tabs **BY PROCESS** (v4.19+): Order-to-Cash / Procure-to-Pay / Record-to-Report / Personas y soporte; Config General and the vertical keep their own tabs | §Important | CODE — `fill_spreadsheet.py` |
| 36 | `fill_spreadsheet` keeps a **per-app LEGACY fallback** for pre-v4.19 workbooks (old category tabs) | §Important | CODE — `fill_spreadsheet.py` |
| 37 | Deployed base templates need a **re-deploy** to gain the process tabs | §Important | PROSE-ONLY (ops) |
| 38 | Customer involvement per module is expressed in **DAYS/week** ("≈ 1 día a la semana"); % is internal, 20% = 1 day | §Important | CODE — `generate_proposal.py` involvement block (~L780-830) |
| 39 | Price-weighted involvement **roll-up chip** per process header | §Important | CODE — `generate_proposal.py` `_g["involvement_pct"]` weighted mean |
| 40 | Involvement renders as a **5-dot half-day scale (½–2½)** | §Important | CODE-POSSIBLE |
| 41 | Involvement source priority: `app_overrides.<APP>.involvement_pct` > the app block's per-app sheet row 'Dedicación cliente (días/sem)' > `involvement_default_pct` > `deal.involvement_dias` > 1 day | §Important | CODE — `_sheet_app_involvement()` + priority chain |
| 42 | Involvement is clamped to **20–50%**, with a stderr `WARNING: involvement_pct … fuera del rango 20–50; se recorta.` | (implicit in §Important's %-is-internal claim) | CODE — `_inv_clamp()` |
| 43 | Non-numeric `deal.involvement_dias` is ignored with a stderr warning | (not in SKILL.md) | CODE — `generate_proposal.py` ~L809 |
| 44 | Single-list checkboxes take the **metro-line colour** of their process in process view | §Important | CODE-POSSIBLE |
| 45 | Involvement is **always on** in process view; in flat view **only when set explicitly** (byte-parity) | §Important | CODE — `involvement_active = _inv_explicit or module_view == "process"` |
| 46 | **Dates are indicative, never contractual**; the client-facing timeline is RELATIVE by design (phases in weeks, Gantt S1..Sn) | §Important | **LEGAL** |
| 47 | Calendar milestone dates exist **only** when the deal opts in (`anchor_dates` / `deal.fecha_decision`) and **ALWAYS** render with the "orientativas, no contractuales" disclaimer (web + PDF) | §Important | **LEGAL** + CODE-POSSIBLE (disclaimer presence) |
| 48 | **Hard country gate**: a locale composing a `_NO_CALENDAR_DATES_CC` country layer (**BE** — Belgian contracts forbid fixed milestone dates) **ABORTS generation** if any calendar date would reach client output; **no layer can bypass it** | §Important | **LEGAL** + CODE — `generate_proposal.py:482` `_NO_CALENDAR_DATES_CC = ("BE",)`, `:497` `_anchor_dates_gated()` → stderr `ERROR: el locale … PROHÍBEN fechas de hito fijas …` + `raise SystemExit(3)` |
| 49 | `app_overrides.features` and `dev_overrides` are **MANDATORY** and personalized; template text = rejected proposal | §Important; Gotcha 6; rules.md #11, #11.b, #15 · **DUP** | PROSE-ONLY (quality); presence CODE-POSSIBLE |
| 50 | The Analysis gateway panel lists the deliverables explicitly ("Al terminar el Análisis, te llevas:" — blueprint, implementation plan, methodology, timeline, **locked budget** highlighted) | §Important | CODE — `TestModes::test_live_lista_unica_y_gateway`; strings validated by `validate_locale.py` |
| 51 | The gateway shows the fixed price + the claim "Se contrata primero, por separado · precio fijo", **above** the two rights (right to stop, right to the locked price); web + PDF, **all live deals** | §Important | CODE (presence) / CODE-POSSIBLE (position) |
| 52 | Gateway strings are pack-owned: `deliv_*` / `gw_sep_chip` | §Important | CODE — `validate_locale.py` key coverage |
| 53 | **General Configuration** is always included and **cannot be unchecked**; €500 base minimum | §Important; Key rule 2; rules.md MP3, #4 · **DUP** | CODE — `TestConfigAppNoDuplicada`, `TestItemsIncluidosNoClickables::test_incluidos_sin_checkbox` |
| 54 | **Go-live support** is always included and cannot be unchecked; dynamic **10%** of the checked apps | §Important; Key rule 2; rules.md #4, #13 · **DUP** | CODE — `TestItemsIncluidosNoClickables::test_salida_produccion_dinamica`, `TestContext::test_soporte_dinamico_10pct_horas` |
| 55 | **±15% adjustment** is free for the sales rep; **>15% requires management approval** | §Important; Key rule 3; rules.md MP4, #3 · **DUP** | CODE-POSSIBLE (>15% highlighted per rules.md #3; no gate) |
| 56 | Add-ons marked **⚙️** include the Dynapps Core module, invisible to the client | §Important; rules.md MP5, #5 · **DUP** | PROSE-ONLY |

## 3. `## Locale selection`

| # | Assertion | Location | Class |
|---|---|---|---|
| 57 | The engine (`scripts/*.py`, `templates/*.html`, `shared/*`) is shared and **never forked**: adding a language/country = adding a pack, **never editing code** | §Locale selection | PROSE-ONLY + CODE-POSSIBLE (`tests/add_country_smoke.py` covers the add path) |
| 58 | `locale_loader.py` is the **single access point** to a pack | §Locale selection | CODE-POSSIBLE (convention; not gated) |
| 59 | A language pack is **9 JSON files** under `locales/<code>/` | §Locale selection | CODE — `scripts/validate_locale.py` |
| 60 | Pack architecture + governance live in `locales/README.md` | §Locale selection; §References | PROSE-ONLY (pointer) |
| 61 | `xx-YY` composes a **language layer** (`locales/<lang>/`: strings/prose/lint/matching) with a **country layer** (`locales/_countries/<CC>/`: instance/pricing/fiscal/currency) | §Locale selection | CODE — `tests/composition_smoke.py` |
| 62 | Plain `es`/`en`/`nl`/`fr` resolve **exactly as before** (ES parity intact) | §Locale selection | CODE — `golden_check.py` + `composition_smoke.py` |
| 63 | BE/CH already compose (`nl-BE`/`fr-BE`; `fr-CH`/`en-CH`) — **CH = FR+EN**, dynapps.ch has no DE/IT site | §Locale selection | CODE — `composition_smoke.py` |
| 64 | BE/CH country layers are **stubs**: the loader **BLOCKS client-facing output** until `verify_from_odoo` + **fiscal/legal sign-offs** are filled in | §Locale selection | **LEGAL** + CODE — `load_locale(allow_stub=False)` M5 gate; `TestM5PlaceholderGate::test_load_locale_rechaza_stub` |
| 65 | Plan, intake checklists and per-locale case URLs live in `references/add_be_ch_plan.md` | §Locale selection; §References | PROSE-ONLY (pointer) |
| 66 | `es` byte-parity means: with `--locale es`, **every artifact is byte-identical** to the Spain-only version | §Locale selection | CODE — `golden_check.py`, `golden_check_helpers.py`, `golden_check_brand_lint.py` |
| 67 | `fr`/`nl` are **reference packs** | §Locale selection | CODE-POSSIBLE |
| 68 | **Connector-agnostic**: entity, currency, "Implantación v2" template id and product ids are **discovered at runtime** (`scripts/odoo_discovery.py`) — NOT hardcoded | §Locale selection; Key rule 1 | PROSE-ONLY + CODE-POSSIBLE |
| 69 | Pricing **and dates** per deal/country live in the `Parámetros` sheet (v2 "settings page" layout, editable yellow `Valor` column) and **override the pack per deal** via `proposal_data.sheet_overrides` → `Locale.with_price_overrides` | §Locale selection | CODE — `TestSheetPricingOverrides::test_sheet_overrides_parse`, `::test_overrides_flow_through_context` |
| 70 | Overridable keys: support %, migration, POC, on-site work, MCP, Field Service, AI prices, per-user license, maintenance plan, adjustment band | §Locale selection | CODE — `TestPricingDesdeCtx` (mcp/fs/migración truqueados fluyen a web y PDF) |
| 71 | Backward-compatible with the **legacy `clave` / `valor` / `etiqueta` (pipe-separated)** layout | §Locale selection | CODE — `proposal_data.sheet_overrides` |
| 72 | `deal.fecha_decision` (dd/mm/yyyy) is the **ANCHOR**; the engine derives team assignment (**+3 business days**), kick-off (**1st Monday of the following month**), analysis/rollout (weeks from 'Interna Comercial') and support **in cascade** | §Locale selection | CODE — `_derive_anchor_dates()` |
| 73 | `deal.anchor_<milestone>` pins a milestone; `deal.fecha_propuesta` sets the document date | §Locale selection | CODE — `_derive_anchor_dates()` |
| 74 | Anchor priority: `extra_data.anchor_dates` > sheet > nothing (**no sheet → no override**, ES byte-parity intact) | §Locale selection | CODE — `TestSheetPricingOverrides::test_no_sheet_no_override_byte_parity` |
| 75 | Per-country templates pre-fill `Parámetros` (`generate_spreadsheet_template.py --locale <code>`); retrofit existing workbooks with `scripts/insert_params_sheet.py` | §Locale selection | PROSE-ONLY (ops) |
| 76 | **Product ids are NOT variabilized** | §Locale selection | PROSE-ONLY |
| 77 | `resolve_locale` priority (highest first): explicit `--locale` / `extra_data.locale` > connected Odoo DB (`master`→`es`) > lead-country hint > default `en` | §Locale selection | CODE — `locale_loader.resolve_locale`; `tests/locale_smoke.py` |
| 78 | The resolved locale code **is logged** | §Locale selection | CODE-POSSIBLE |
| 79 | The orchestrator resolves the code **ONCE** and injects the **same** `--locale` into every script and into `extra_data.locale` (spreadsheet writer and HTML readers share one locale, no drift) | §Locale selection | PROSE-ONLY (orchestration discipline) |
| 80 | Fallback policy: UI strings → `en`; **pricing/fiscal/product ids → HARD ERROR**, never a silent fallback to Spanish | §Locale selection | CODE — `locale_loader` |

## 4. `## Workflow` — connection + STEP 1..5

| # | Assertion | Location | Class |
|---|---|---|---|
| 81 | **Fast start**: if the rep already has the spreadsheet filled and just asks for the HTML/PDF (URL/ID), **skip STEP 1-3** and go straight to STEP 4 | §Workflow | PROSE-ONLY (routing) |
| 82 | Two Odoo paths: **odoo-pilot** (`auth.sh` + `search_records.sh`, RPC with API key — **required** to create the spreadsheet and pull binaries/photos) or **Odoo MCP** (`orm_search_read`/`orm_write`, superuser, Cowork without `auth.sh`) | §Workflow; §Delegates | PROSE-ONLY |
| 83 | MCP proxy gotchas: doesn't serve binaries, large outputs go to file, spreadsheet keyed by `order_id` | §Workflow; Gotcha 9 · **DUP** | PROSE-ONLY |
| 84 | STEP 1 runs as `Agent(model: sonnet)` with the STEP 1 prompt from `references/subagent_prompts.md`, injecting `lead_input` (ID or name) | §STEP 1; §Agent Routing · **DUP** | PROSE-ONLY |
| 85 | STEP 1 returns JSON: `lead_id`, `partner`, `parsed` (apps_detectadas, complementos_sugeridos, metadata, resumen), `raw` (chatter, attachments), `existing_orders` | §STEP 1 | CODE-POSSIBLE (schema check) |
| 86 | `{"status": "missing_toma"}` → run `sales-skills:lead-data-collection` first, then retry | §STEP 1; Gotcha 1; §Delegates · **DUP** | PROSE-ONLY |
| 87 | The orchestrator **NEVER** reads `xx_data_collection` or raw attachments (tens of thousands of tokens); the subagent returns only `parsed` + a short summary | §STEP 1 | PROSE-ONLY (token discipline) |
| 88 | Existing quotes on the opportunity must be resolved ("new v2 order vs. convert") **HERE**, before pricing, not midway | §STEP 1 | PROSE-ONLY |
| 89 | STEP 2.1: present the detection to the rep (read `references/pricing_apps.md`): detected apps + suggested add-ons + totals | §STEP 2 | PROSE-ONLY |
| 90 | STEP 2.1: then **iterate** — add/remove apps and add-ons, quantity on `por_unidad` add-ons (`"Name:N"`), % adjustment per app or global, internal notes | §STEP 2; Key rule 2; rules.md #12 · **DUP** | CODE (`"Name:N"`) — `fill_spreadsheet.parse_extra_entry()` |
| 91 | Price per app = `(base + Σ add-ons × quantity) × (1 + adjustment%)` | §STEP 2 | CODE — `proposal_data.calc_app_price_split()`; `TestContext::test_app_price_split` |
| 92 | The adjustment **%** applies over `base + add-ons`, **not just the base** | §STEP 2; Gotcha 1 · **DUP** | CODE |
| 93 | Functional analysis: internal online/on-site calculation → closed price; the client does NOT see hours | §STEP 2; `references/analysis_pricing.md` | PROSE-ONLY |
| 94 | Custom development: estimated hours **× €75/h** internally → closed price | §STEP 2 | CODE-POSSIBLE (rate = sheet B5, default 75.0 — `soporte_rates()`) |
| 95 | Write a **specific description** for each dev item (never the generic fallback) and **link it** to an app or mark it general | §STEP 2; Gotcha 6; rules.md #15 · **DUP** | PROSE-ONLY |
| 96 | Duration: derive `duracion.semanas_*` and `gantt.semanas_por_app` with the pace profiles in `references/timeline_rules.md`; internal hours never shown | §STEP 2 | CODE (weeks computation) — `fill_spreadsheet.py` `SESSIONS_PER_WEEK = 3`, `Análisis = ceil(total_sesiones / 3)` |
| 97 | Training sessions are included in the go-live support fee (dynamic 10%, "support, migrations and training"); **always state 2h per session** | §STEP 2; Key rule 2; rules.md #14, #30 · **DUP** | CODE — `TestPdfLiveModel::test_pdf_formacion_2h` |
| 98 | Complexity triggers (multi-company, >5 apps, >100 users, regulated sector, >2 devs) → tell the rep to size support/add-ons accordingly | §STEP 2 | PROSE-ONLY |
| 99 | STEP 2.5: present the definitive summary and **wait for confirmation before writing anything** | §STEP 2; rules.md #6 · **DUP** | PROSE-ONLY |
| 100 | STEP 3 runs as `Agent(model: sonnet)` with the STEP 3 prompt, injecting `sale_order_id`, `budget_json`, `ana_price`, `apps_total`, `extras_total`, `devs_total` | §STEP 3; §Agent Routing · **DUP** | PROSE-ONLY |
| 101 | STEP 3 returns `spreadsheet_id`, `impl_total_final` (apps + 10% support + devs, already rounded) **and** the `completeness` object (B44 sessions / B51-B52 weeks / Gantt bars / `check`) | §STEP 3 | CODE-POSSIBLE (schema) |
| 102 | If `completeness.check` is `fail` or missing → **relaunch STEP 3**, don't present to the rep; **matching totals do NOT prove the sheet is complete** | §STEP 3; §STEP 4.1 | CODE (detection) — `generate_proposal.py` ~L627 `WARNING: semanas de análisis/implantación = …`, ~L636 `WARNING: la hoja 'Gantt' no tiene celdas pintadas (0 barras)`; `fill_spreadsheet.py` ~L780-792 completeness |
| 103 | The **ONLY** write path is `fill_spreadsheet.py` → upload the complete JSON; **direct cell edits are FORBIDDEN** | §STEP 3 | PROSE-ONLY |
| 104 | Creating the spreadsheet and pulling binaries/photos **MUST** go through **odoo-pilot**; reading/writing fields is also valid via the Odoo MCP | §STEP 3; Gotcha 9; §Delegates · **DUP** | PROSE-ONLY |
| 105 | An order created via ORM without lines/spreadsheet → the subagent creates them per `workflow_detail.md` §3.0 | §STEP 3; Gotcha 3 · **DUP** | PROSE-ONLY |
| 106 | **MANDATORY STOP after STEP 3**: the rep must open the spreadsheet in Odoo and review apps/add-ons/devs/prices | §STEP 3; Key rule 1; rules.md #19 · **DUP** | PROSE-ONLY |
| 107 | The spreadsheet URL comes from the pack's `instance.json` `sales_url_pattern` (e.g. `{odoo_url}/odoo/sales/<sale_order_id>`); `odoo_url` comes from runtime discovery — **never hardcode** | §STEP 3 | CODE-POSSIBLE |
| 108 | Do **NOT** move to STEP 4 until the rep explicitly replies "ok, go ahead" or equivalent | §STEP 3; rules.md #19 · **DUP** | PROSE-ONLY |
| 109 | If the rep adjusts the spreadsheet in Odoo (quantities, prices, % adjustments), STEP 4 **re-reads** the updated state | §STEP 3 | PROSE-ONLY |
| 110 | STEP 4 **ALWAYS asks at the start**: (1) mode — `predemo` or `live`; (2) team names (for `fetch_team.py`) | §STEP 4 | PROSE-ONLY |
| 111 | **ALL corrections go into `extra.json`** and are re-injected before regenerating — the generator only renders `extra_data`, never auxiliary files | §STEP 4 | PROSE-ONLY |
| 112 | **Two-phase flow**: STEP 4 generates **ONLY the web HTML** (`--only web`); the PDF waits for the client's copied selection (STEP 5.2, `--only pdf`, body = ONLY what was checked) | §STEP 4; §STEP 5 | CODE — `--only` flag; `TestCopySelection`, `TestPdfLiveModel` |
| 113 | web = v5 client-first order; PDF = static executive **A4 mirror** rendered by **headless Chrome** | §STEP 4; rules.md #21, #22 · **DUP** | CODE — `find_chrome()` (`generate_proposal.py:4848`) |
| 114 | Mode matrix + section structure + `extra_data` spec live in `references/html_structure.md` → "STEP 4 — Execution detail" | §STEP 4; §References | PROSE-ONLY (pointer) |
| 115 | The full STEP 4 sub-step spec (commands, gates, traps) lives in `references/workflow_detail.md` §4.0; SKILL.md carries only the index | §STEP 4 | PROSE-ONLY (pointer) |
| 116 | STEP 4.1: re-download the spreadsheet **+ line re-sync guard** (`order_totals.py` → rewrite `sale.order.line.price_unit` on drift, §3.4) | §STEP 4 | CODE — `scripts/order_totals.py` |
| 117 | STEP 4.1: **completeness gate ABORT → STEP 3** if B44/B51/B52 = 0 or the Gantt has no painted cells | §STEP 4 | CODE (warns; the abort is the orchestrator's) — see #102 |
| 118 | STEP 4.2: `extra_data.json` with MANDATORY personalized content — `app_overrides.<APP>.features` with **EXACT spreadsheet keys**, `dev_overrides`, `understanding` | §STEP 4; Gotcha 6 · **DUP** | PROSE-ONLY (content); exact keys CODE-POSSIBLE |
| 119 | Non-ES: prose in the **target language** + `extra_data.locale` | §STEP 4; Gotcha 15 · **DUP** | CODE — `brand_lint.py` IDIOMA (stopword profile + pack `leak_denylist`) |
| 120 | STEP 4.3: glossary + team (`fetch_team.py`) + cases (`match_cases.py --locale <code>`) | §STEP 4 | PROSE-ONLY (sequence) |
| 121 | ⚠️ `--locale` **MUST** match the toma's language or every case scores `sector=0` (false `weak_match`) | §STEP 4 | CODE (detection) — `generate_proposal.py` `WARNING: caso de éxito '%s' llega con 'weak_match': true …` |
| 122 | References pipeline order: `<CC>.json` freshness → `match_cases.py` → `fetch_case_images.py` | §STEP 4 | CODE — cache >90 days warns on stderr (`workflow_detail.md` §4.0 step 0) |
| 123 | `metric` **and** `image` are MANDATORY per case; `match_cases.py` output is used **AS-IS** | §STEP 4 | CODE — `generate_proposal.py` warnings: `caso de éxito '%s' sin 'image'`, `… con 'image' http(s) sin embeber`, `… sin 'metric' (metric-first)` |
| 124 | Quotes are **never invented** | §STEP 4; DoD; rules.md #33 · **DUP** | CODE (provenance via `fetch_references.py`) + PROSE-ONLY (judgement) |
| 125 | Team photos via MCP: `references/mcp_photo_access.md` | §STEP 4; Gotcha 11 · **DUP** | PROSE-ONLY (pointer) |
| 126 | STEP 4.4: **brand lint on the JSON** (`brand_lint.py [--locale]`) → fix findings → **rep sign-off on `extra_data.json` before generating** | §STEP 4; rules.md #27 · **DUP** | CODE (lint) — `brand_lint.lint()`; sign-off PROSE-ONLY |
| 127 | STEP 4.5: recurring costs from sale.order values or `RECURRENT_DEFAULTS`; **future migration = 15% of checked apps+devs / 24, `ceil_to_10`** | §STEP 4; Key rule 5; rules.md #16, #26 · **DUP** | CODE — `TestRecurrentesMigracion::test_migracion_futura_15pct_24` |
| 128 | STEP 4.6: generate **ONLY the web HTML** (`--only web`); M5 gates: a stub pack needs `--allow-stub`, a **placeholder leak → exit 3, nothing written** | §STEP 4 | CODE — `check_no_rendered_placeholders()` (`generate_proposal.py:4928`), stderr `… placeholders … sin rellenar; NO se ha escrito ningún fichero.`; `TestM5PlaceholderGate` |
| 129 | **Post-generation gates are MANDATORY before delivering ANY rendered HTML** (STEP 4 and STEP 5.2): `brand_lint.py --html` **then** `verify_totals.py --html … --extra-data …`; exit 1 → fix and regenerate, **never deliver a failing file** | §STEP 4; §STEP 5; rules.md #35 · **DUP** | CODE — both scripts; `verify_totals.py` exit 1 = FAIL, `test_verify_totals.py` |
| 130 | The two gates run **in that order** (brand_lint before verify_totals) | §STEP 4 | CODE-POSSIBLE |
| 131 | The placeholder guard applies **always**, with or without `--allow-stub` | (code comment `generate_proposal.py:4984`) | CODE — `TestM5PlaceholderGate::test_guard_caza_placeholders_escapados` |
| 132 | STEP 5 is **idempotent**: an existing version on the sale.order is UPDATED, never duplicated | §STEP 5 | PROSE-ONLY |
| 133 | Full command-level STEP 5 detail is in `references/workflow_detail.md` §STEP 5 — **follow it, don't improvise** | §STEP 5 | PROSE-ONLY (pointer) |
| 134 | 5.1 delivers the HTML and **does NOT touch the PDF** | §STEP 5 | PROSE-ONLY |
| 135 | 5.1: publish via `utilities:publish` or the hoster; re-publish via `upload_id` → public URL | §STEP 5; §Delegates · **DUP** | PROSE-ONLY |
| 136 | 5.1: post/update a **rich-HTML internal note on the `sale.order`** (the order's id, **NOT** the crm.lead) via `orm_post_message` (preserves HTML — verified 2026-07) | §STEP 5 | PROSE-ONLY |
| 137 | 5.1: link the text **"HTML proposal"**, never the raw URL | §STEP 5 | PROSE-ONLY |
| 138 | 5.1: delete any previous "Proposal PR<NUM>" note **first** (`orm_unlink`) | §STEP 5 | PROSE-ONLY |
| 139 | 5.2: inject the "Copy selection" JSON into `extra.json` (mapping in §STEP 5) → `--only pdf` → **re-run both gates on the `-pdf.html`** (FAIL blocks the upload) | §STEP 5 | CODE (gates) + PROSE-ONLY (sequence) |
| 140 | 5.2: upload out-of-band — `odoo_request_upload` **WITHOUT pre-targeting `sale.order`** + `curl` → `attachment_id` | §STEP 5 | PROSE-ONLY (verified end-to-end, ANRO PR4021) |
| 141 | 5.2: re-post the note with the link + `attachment_ids=[<attachment_id>]` | §STEP 5 | PROSE-ONLY |
| 142 | Return contract: 5.1 → public URL + note; 5.2 → attached PDF + updated note | §STEP 5 | PROSE-ONLY |

## 5. `## Key rules` (1-5) — the SKILL.md digest of rules.md

| # | Assertion | Location | Class |
|---|---|---|---|
| 143 | The non-negotiables are in `## Important`; the full list (35) is in `references/rules.md` | §Key rules | PROSE-ONLY (pointer) — **count is inconsistent with §References, see §7** |
| 144 | "Implantación v2" template id is **discovered by name at runtime** via `odoo_discovery.py`, not hardcoded; on the `master`/`es` DB it is 15 | Key rule 1; rules.md #9 · **DUP + CONTRADICTION** (rules.md #9 states `id=15` flatly) | CODE-POSSIBLE |
| 145 | Single list composition: Config General (included, non-uncheckable) + checkable apps (pre-checked via `apps_recomendadas`) + Go-live support (included, non-uncheckable) + checkable custom dev items | Key rule 2; rules.md #4, #5 · **DUP** | CODE — `TestModes::test_live_lista_unica_y_gateway`, `TestEdgeCases::test_soporte_incluido_no_desmarcable` |
| 146 | Go-live support = **10% of the implementation HOURS** (config + checked apps + checked custom dev) **plus analysis**, mirroring the sheet's **F36/E36** cells; **recalculates live** | Key rule 2; rules.md #13 · **DUP** | CODE — `proposal_data.calc_soporte_salida()`, `TestContext::test_soporte_dinamico_10pct_horas`, `TestFormulaRobustness::test_soporte_formula_cubre_todas_las_horas` |
| 147 | `line_hours = ROUND(€ × factor / rate, 1)`; factor/rates read from sheet **B4/B5/B6** (defaults 0.92 · 75 €/h implementation · 80 €/h analysis); `€_support = ROUND(ROUND(Σ hours × 0.10, 1) × implementation_rate, 0)` | rules.md #13 **only** | CODE — `proposal_data.soporte_rates()` (`proposal_data.py:884`) |
| 148 | The rep does **NOT** duplicate the support line manually | rules.md #13 **only** | PROSE-ONLY |
| 149 | Custom developments are **selectable**; their HOURS **do** count toward the 10% support figure (as in sheet row F36) even though their € is not part of the "apps base" | rules.md #5 **only** | CODE — see #146 |
| 150 | Add-on matching is by **substring** | Key rule 2; rules.md #10 · **DUP** | CODE — `fill_spreadsheet.match_extra()`: equality, both-way containment, **plus an undocumented 20-char-prefix heuristic when `len(cn) > 10`** |
| 151 | Verify add-on names if an extra doesn't activate | rules.md #10 | PROSE-ONLY |
| 152 | **Add-ons / AI block**: everything **unchecked by default**; conditional items hide when their app is unchecked | Key rule 3; rules.md #14 · **DUP** | CODE — `TestValorAnadido::test_addons_condicionales`, `::test_catalogo_exclusivo` |
| 153 | AI catalog composition: MCP/Starter **mutually exclusive**, Payroll/Delivery-Note OCR, conditional CRM/Field Service (ids **305, 288, 212** hidden if their app isn't selected), POC, on-site presence | rules.md #14 **only** | CODE — `TestValorAnadido::test_plataforma_ia_checkbox_excluyente_opcional`, `::test_305_288_removidos_del_catalogo`, `::test_ocr_facturas_328` |
| 154 | POC = **20% of the analysis** | rules.md #14 (implicit) | CODE — `TestValorAnadido::test_poc_es_20pct_analisis` |
| 155 | Zero brochure adjectives · sentence-case capitalization · sector-specific language (glossary); validate with `brand_lint.py` | Key rule 4; rules.md #27 · **DUP** | CODE — `brand_lint` FOLLETO / CAPS / JERGA |
| 156 | Metric-first cases with a link **to the Dynapps case-studies site** (never the Odoo backend) | Key rule 4; rules.md #28 · **DUP** | CODE-POSSIBLE; `TestCasesMetricFirst::test_metric_rendered`, `::test_links_present` |
| 157 | The case URL is composed **per locale** by `match_cases.py` as `instance.cases_base_url` + the case's `slug`; the shared `_base_url` in `cases_library.json` (`https://www.dynapps.es/en/cases`, `_base_url_TEMP: false`) is only the fallback when the pack declares none | rules.md #28 **only** | CODE — `tests/references_smoke.py::TestMatchCasesV2` |
| 158 | Team sourced from `hr.employee`; **never invent an email** | Key rule 4; rules.md #29 · **DUP** | PROSE-ONLY |
| 159 | Do **NOT** rewrite the employee email domain; if it must be `@dynapps.es`, the employee changes it in Odoo | rules.md #29 **only** | PROSE-ONLY |
| 160 | Team/cases are **optional**: hidden if there's no data; never invent client photos, names or results | Key rule 4; rules.md #25 · **DUP** | CODE — `TestTeamCasos::test_absent_by_default`, `::test_only_team_no_casos` |
| 161 | Self-contained output: Readex Pro + logos as **base64 data URIs**, no external resources | Key rule 5; rules.md #21 · **DUP** | CODE — `TestPdfAssets::test_no_empty_img_src` |
| 162 | Official **Odoo icons** per app, no emoji, no generic icons | Key rule 5; rules.md #24 · **DUP** | CODE-POSSIBLE |
| 163 | Name→file mapping lives in `APP_ICON_FILES` (`generate_proposal.py:98`); unmapped apps fall back automatically (initial/bar) | rules.md #24 **only** | CODE |
| 164 | Dynapps brand + hosting called **"Dyn Cloud"** | Key rule 5; rules.md #22, #31 · **DUP** | CODE — `brand_lint` MARCA; `RECURRENT_DEFAULTS` |
| 165 | Brand palette detail: Midnight Denim `#050543`, Unicorn Violet, Ferrarri Carmine, Sunset Tangerine; 45° gradient violet → carmine at **52%** → orange `#FF7759`; central mindmap node = Odoo purple `#714B67`; web adds the dark glass top nav + dark footer with gradient line | rules.md #22 **only** | CODE-POSSIBLE (frozen by `golden_check.py` for `es`) |
| 166 | Output file naming: lowercase **`propuesta-pr<NUM>.html`** | Key rule 5; rules.md #20 · **DUP** | CODE-POSSIBLE (not enforced in `generate_proposal.py`) |
| 167 | The file stays local; publishing is decided **outside** the skill | rules.md #20 **only** · **CONTRADICTION** with §STEP 5 (which publishes) | PROSE-ONLY |
| 168 | PDF body = **ONLY what was checked**, including selected AI items; **no showcase appendix** | Key rule 5; rules.md #22 · **DUP** | CODE — `TestPdfLiveModel::test_pdf_ia_no_anexo_solo_marcado_en_cuerpo`, `::test_presencialidad_no_en_cuerpo_pdf` |
| 169 | Recurring costs: from the spreadsheet or `RECURRENT_DEFAULTS`; override with `extra_data.costes_recurrentes` | Key rule 5; rules.md #16 · **DUP** | CODE — `proposal_data.extract_costes_recurrentes()`, `TestContext::test_costes_recurrentes` |
| 170 | Recurring source cells: `Interna Comercial` rows **60, 61, 62** (col A title, col B formatted price); unevaluated `=ODOO.LIST(...)` formulas are **discarded** in favour of `RECURRENT_DEFAULTS` (Dyn Cloud 1,080€/yr · Maintenance 360€/month · Odoo License 334.80€/user·yr) | rules.md #16 **only** | CODE — `extract_costes_recurrentes()` |
| 171 | Future migration, **MCP (€50/month)** and **Field Service (+€2/user/month)** enter recurring costs **if checked** | Key rule 5 | CODE — `TestValorAnadido::test_no_doble_conteo_recurrente_mcp_fs`, `TestRecurrentesMigracion::test_mcp_field_service_visibles_marcados` |
| 172 | Partner manager **Micaela Gil Rother (migi@odoo.com)** | Key rule 5 | CODE — `TestEdgeCases::test_cierre_contacts_partner_manager_and_odoo_color`; omitted whole if the pack value is a placeholder (`TestM5PlaceholderGate::test_pm_note_omitida_si_placeholder`) |
| 173 | Odoo context `lang: es_ES` whenever products are searched | Key rule 5; rules.md #7 · **DUP** | PROSE-ONLY |
| 174 | **Never invent data**; if the intake data is thin, ask the rep — don't assume add-ons without clear signals | Key rule 5; rules.md #8 · **DUP** | PROSE-ONLY |
| 175 | Recurring block is split into **annual / monthly** sub-blocks, allocated per period | rules.md #16 (implicit) | CODE — `TestRecurrentesAnualMensual::test_sub_bloques_anual_mensual`, `::test_reparto_por_periodo` |
| 176 | Template subtotals live in rows **39-42** (`SUM(...)` tolerating empty cells); internal names, not client-facing; the skill does **NOT** rebuild these formulas | rules.md #17 **only** | CODE — `TestFormulaRobustness` |
| 177 | Deleting rows must **NEVER shift/renumber** rows: `Interna Comercial` formulas use FIXED refs (`=ROUND((F10+F15+…+F34)*pct,1)`, `E38`/`E40`, `='Operaciones'!D3`); an empty cell evaluates to **0, not `#REF!`** | `fill_spreadsheet.py:146` docstring (**not in SKILL.md**) | CODE — `TestFormulaRobustness::test_delete_rows_no_shift` |
| 178 | Per-app extras block totals by **RANGE** (`D20: =SUM(D16:D18)`) with per-row `Dx: =Ax*Cx`; `D22: =D3+D20` = base + extras | `fill_spreadsheet.py` (**not in SKILL.md**) | CODE |
| 179 | Sticky side navigation on the web: scroll-spy + progress bar, **only present sections**, hidden below **1280px**; web only, no PDF; requires no rep action | rules.md #30 **only** | CODE-POSSIBLE |
| 180 | `extra_data.contacto` defaults to Dynapps (info@dynapps.es · www.dynapps.es · 960 202 942) | rules.md #31 **only** | CODE (pack `instance.contact`; placeholder segments omitted — `TestM5PlaceholderGate::test_contacto_placeholder_omitido`) |
| 181 | Legacy modes `proposal` (ranges) and `close` are **removed** and collapse to `live`; the Recommended/Custom concept is removed | rules.md #26 **only** | CODE — `TestModes::test_legacy_modes_collapse_to_live` |
| 182 | Future-migration cycle = renewal **every 2 years** (hence /24) | rules.md #26 **only** | CODE |
| 183 | Register per pack: **es keeps professional "tú" untouched**, nl uses je/jij, fr uses vous | rules.md #27 **only** | CODE — `brand_lint` REGISTRO (pack `register` list) |
| 184 | **No em-dashes (—)**; the en-dash (–) is valid **only** inside numeric/date ranges ("S12–S14", "jul–ago") | DoD; Gotcha 14; rules.md #27 · **DUP** | CODE — `brand_lint` DASH + `_endash_findings()` |
| 185 | **One language per document**, no leaks (per-pack denylist) | DoD; rules.md #27 · **DUP** | CODE — `brand_lint` IDIOMA |
| 186 | Talk about the client **2:1**; "if any competitor could sign the sentence, delete it" | rules.md #27 **only** | PROSE-ONLY |
| 187 | Warn-only lint checks exist and are not gates: **EXCL** (exclamations), **PASIVA** (passive voice heuristic; `es` defines no patterns by product decision) | rules.md #27 | CODE (warn-only) — `brand_lint` checks 12 & 13 |
| 188 | `verify_totals.py` accepts `--ss/--toma` for the **exact-match** model | rules.md #35 **only** | CODE — `verify_totals.verify()` |
| 189 | `verify_totals` also enforces **sanity bounds**: plausible per-user licence rate (Custom 334.80 reference / Standard 178.80 unless explicitly overridden), **hard FAIL over €500/user with >1 user**, and a monthly total within [0.5x, 2x] of expectation | `verify_totals.py` docstring (**not in SKILL.md**) | CODE — `STANDARD_LICENCE_RATES`, `LICENCE_RATE_HARD_CAP = 500.0` |
| 190 | `verify_totals` asserts each summary equals the sum of its **own visible detail rows** (catches hand-edited `costes_recurrentes`) | `verify_totals.py` docstring | CODE |

## 6. `## Definition of done`, `## Gotchas`, `## Agent Routing`, `## Delegates to`

### 6.1 Definition of done (Ine checklist — 10 boxes)

| # | Assertion | Location | Class |
|---|---|---|---|
| 191 | ☐ One language, zero leaks | DoD (= #185) | CODE — `brand_lint` IDIOMA |
| 192 | ☐ Zero em-dashes | DoD (= #184) | CODE — `brand_lint` DASH |
| 193 | ☐ **No all-caps anywhere, CSS included** | DoD; Gotcha 7; rules.md #27 · **DUP** | CODE for visible text (`brand_lint` CAPS) / **CODE-POSSIBLE for the CSS half** — `brand_lint.strip_html_visible()` deliberately strips `<style>`, so `text-transform: uppercase` is **not** detected today |
| 194 | ☐ v5 section order (problems once; commitment after pricing) | DoD (= #17, #19) | CODE-POSSIBLE |
| 195 | ☐ Max 4 challenges | DoD (= #18) | CODE |
| 196 | ☐ Real client logos when available — never a text name if the logo exists, **never a fabricated logo** | DoD; rules.md #33 · **DUP** | PROSE-ONLY |
| 197 | ☐ Quotes never invented (genuine site quote or factual headline) | DoD (= #124) | PROSE-ONLY + CODE (provenance) |
| 198 | ☐ Rep photo + next-step line on closing, **no phone number** | DoD; rules.md #31 · **DUP** | CODE-POSSIBLE |
| 199 | ☐ Recurring totals verified (`verify_totals.py` PASS) | DoD (= #129) | CODE |
| 200 | ☐ File complete: opens fully, no truncated sections, **no placeholders** | DoD | CODE — `TestRender::test_no_leftover_placeholders` + the M5 render-time gate (#128) |

### 6.2 Gotchas 1-16

| # | Assertion | Location | Class |
|---|---|---|---|
| 201 | **G1** Empty intake data → do NOT parse; ask the rep to run `lead-data-collection` first | Gotcha 1 (= #86) | PROSE-ONLY |
| 202 | **G1b** The adjustment % applies over `base + add-ons` | Gotcha 1 (= #92) | CODE |
| 203 | **G2** Per-unit add-ons (multi-warehouse, multi-company, extra pages) are price × quantity — **always ask for the quantity** | Gotcha 2; rules.md #12 · **DUP** | CODE (`"Name:N"` parsing) / PROSE-ONLY (asking) |
| 204 | **G3** Empty `sale.order.spreadsheet` = wrong template or an ORM-created order (the UI's onchange doesn't apply a template on `create`); create lines + spreadsheet with the §3.0 recipe; **don't hand-craft `spreadsheet_data`** | Gotcha 3 (= #105) | PROSE-ONLY |
| 205 | **G4** Invoicing and Accounting are **separate apps**; a client can have Invoicing without Accounting (sole traders) — don't assume they go together | Gotcha 4 | PROSE-ONLY (domain knowledge) |
| 206 | **G5** Max **10** analysis sessions / **5** custom dev items — template slot limit; extend the template if more are needed | Gotcha 5 | CODE — `fill_spreadsheet.py:418` `sesiones[:10]`, blanking loop `range(len(sesiones), 10)` |
| 207 | **G6** Templated `extra_data.json` = rejected proposal; extract the exact keys with the one-liner in `references/html_structure.md` | Gotcha 6 (= #49, #118) | PROSE-ONLY |
| 208 | **G7** Sentence case everywhere; proper nouns and acronyms (CIF/IVA/CRM) excepted; **never** caps for emphasis; **no `text-transform: uppercase` in the CSS** (the "eyebrow caps via CSS" doctrine is retired); the brand is always **"Dynapps"** (capital D) in prose | Gotcha 7; DoD; rules.md #27 · **DUP** | CODE (CAPS/MARCA on text) / CODE-POSSIBLE (CSS) — see #193 |
| 209 | **G8** Typography and breathing room: don't enlarge fonts (hero ≤ `clamp(40px,6vw,72px)`, titles ≤ `clamp(23px,3vw,34px)`) or padding (~64px); tighten before stretching | Gotcha 8; `html_structure.md` #14 · **DUP** | CODE-POSSIBLE |
| 210 | **G9** Creating/writing the spreadsheet → **odoo-pilot**, not the generic MCP (no `describe_model`, no onchange, no binaries, no spreadsheet creation; it CAN read/write fields) | Gotcha 9 (= #104) | PROSE-ONLY |
| 211 | **G9b** Two `spreadsheet_data` formats: `fill_spreadsheet.py` needs **dict `{content, style}`** cells; old plain-`str` ones fail → copy a recent "Hoja de implantación base" | Gotcha 9 | CODE-POSSIBLE |
| 212 | **G10** Chrome in the Linux sandbox (Cowork): no Chrome, no root; `find_chrome()` already checks `QB_CHROME_BIN` and `PLAYWRIGHT_BROWSERS_PATH`; no-root recipe (playwright to `~/pw`, `.so` via `apt-get download` + `dpkg-deb -x`, wrapper with `LD_LIBRARY_PATH` + `--no-sandbox`) in `workflow_detail.md` §4.bis | Gotcha 10; rules.md #22 (Chrome required) · **DUP** | CODE — `find_chrome()` (`generate_proposal.py:4848`), stderr `⚠️ Chrome no encontrado: PDF no generado.` |
| 213 | **G11** A base64 photo with a bytes repr (`b'iVBOR...'`) renders broken; it must be clean ASCII. `fetch_team.py` and the renderer already sanitize. Source of truth = `extra_data.equipo` (re-inject after fixing) | Gotcha 11 | CODE — `TestNewMejoras::test_photo_clean_strips_bytes_wrapper` |
| 214 | **G11b** Photo via MCP (no API key): `orm_search_read('ir.attachment', res_model/res_id/res_field='image_512')` → `attachment_id` → `odoo_attachment_link(attachment_id)` → `curl`, then `scripts/embed_team_photos.py`. The `odoo_attachment_link(model,res_id,field)` path **IS blocked** for `hr.employee`/`res.partner`; by `attachment_id` it is NOT. Alternative: `fetch_team.py` (odoo-pilot RPC + API key) | Gotcha 11 | PROSE-ONLY (ops recipe) |
| 215 | **G12** Odoo license is a **PER-USER rate**; the engine multiplies it by `num_usuarios` (live selector on web, scaled in PDF) | Gotcha 12 | CODE |
| 216 | **G12b** **NEVER** inject the already-multiplied total into `price` (60 × 334.80 = 20,088 → re-multiplied = 1,205,280, the PR4021 bug class). Correct: set `extra_data.num_usuarios` **and** a per-user `price`, or the explicit `amount_per_user` | Gotcha 12 | CODE — `generate_proposal.py` ~L886 `WARNING: Licencia Odoo tarifa/usuario=%s € × %d usuarios …`; **hard FAIL** in `verify_totals.py` (`LICENCE_RATE_HARD_CAP = 500.0`) |
| 217 | **G12c** Reference rate **Custom 334.80** €/user·year (27.90/month, yearly, list price, never the first-year discount) · **Standard 178.80** sanctioned | Gotcha 12 | CODE — `verify_totals.STANDARD_LICENCE_RATES`, pack fallback `recurrent_lines[odoo_license]`, `TestLicenciaReferencia` |
| 218 | **G12d** If the number of users is `[ASK]`, **ask before generating** | Gotcha 12 | PROSE-ONLY |
| 219 | **G12e** `generate_proposal.py` warns on stderr when the resolved per-user rate exceeds **€500** (symptom of an injected total) | Gotcha 12 | CODE — as #216 (guard fires only when `users > 1`) |
| 247 | **G12f** **Light User** = its own annual line from **`num_usuarios_light`** (pack `recurring_monthly.odoo_light_user`, 4.90 €/user/month in es), never scaled by the users selector, never hand-written in `costes_recurrentes`; `-1` pack rate → omitted + warning | Gotcha 12 | CODE — `generate_proposal._resolve_light_user`, `_is_licencia_odoo_item`, `verify_totals._check_light`, `TestLightUser` (both test files) |
| 248 | **G12g** odoo.com prices by **geo-IP**: from Spain every locale URL returns the ES pricelist — only ES is verified, never "verify" another market from here | Gotcha 12 | PROSE-ONLY |
| 220 | **G13** `hr.employee` `image_512`/`image_256` comes portrait (e.g. 341×512); with `object-fit:cover` the face gets cropped. **ALWAYS** normalize to a centred square with padding (neutral `#EEEEF2`, 512×512) before injecting into `extra_data.equipo`. Ideally `fetch_team.py` should do this by default | Gotcha 13 | CODE-POSSIBLE (**known gap**: not implemented in `fetch_team.py`) |
| 221 | **G14** Text that sounds like AI = rejected proposal — `understanding`, `executive_summary_html`, `cost_of_delay`, `app_overrides`, `dev_overrides` must read human | Gotcha 14; `html_structure.md` #15 · **DUP** | PROSE-ONLY (judgement) |
| 222 | **G14b** No filler words ("empower", "leverage", "revolutionize", "a wide range of"…), no filler intros/outros ("in an ever-changing world…", "in conclusion") | Gotcha 14; rules.md #27 · **DUP** | CODE — `brand_lint` SLOP / FLUFF (pack lexicons) |
| 223 | **G14c** No 4+ consecutive sentences of the same length (the robotic drone effect); active voice, varied sentence length, the "café test" | Gotcha 14 | CODE (partial) — `brand_lint` RITMO via `_monotony_run()` / `_MONO_RUN` |
| 224 | **G14d** Fix SLOP / FLUFF / RITMO warnings **before generating**; the literal tags are what `brand_lint.py` emits | Gotcha 14 | CODE (tags) + PROSE-ONLY (act on them) |
| 225 | **G15-1** `--locale` only localizes the static chrome + apps' `display_name`; **the prose in `extra_data` is PER LANGUAGE** (one `extra_<lang>.json` per language, each with a top-level `extra_data.locale` so it renders without a flag) | Gotcha 15; `localization.md` · **DUP** | CODE-POSSIBLE |
| 226 | **G15-2** The **KEYS** of `app_overrides.features` / `dev_overrides` and the `data-app`/checkbox values **stay in SPANISH** (the template JS branches on them); only visible titles get localized (`display_name`, `dev_overrides[k].title`) | Gotcha 15; §Important (implicit) | CODE-POSSIBLE (**i18n trap; highest-risk assertion to lose**) |
| 227 | **G15-3** `app_overrides.features` values accept `{"title": …, "expl": …}` besides the legacy plain string; for **non-es always use the object form** (key stays the Spanish spreadsheet name) | Gotcha 15 | CODE — `TestRender::test_app_override_features_render` |
| 228 | **G15-4** License = per user (`num_usuarios` + per-user override in `costes_recurrentes`); the recurring block **doesn't render `desc`** | Gotcha 15 | CODE — `TestQuirksCongeladosEs::test_desc_operador_de_licencia_descartada` |
| 229 | **G15-5** ES byte-parity gate = `python3 tests/golden_check.py` **without** a flag | Gotcha 15 | CODE |
| 230 | **G16** `spreadsheet_data` (and any big **Text** field, ~55-75 KB) is `ttype='text'`, **not** Binary/Image; the MCP `{'__ref__':'attachment','id':N}` value-ref is for Binary/Image **ONLY** — writing it to a Text field stores the literal dict-repr and **silently corrupts** the record (fails later client-side with `OwlError` / `SyntaxError: Expected property name or '}' in JSON`) | Gotcha 16 | PROSE-ONLY (ops hazard) |
| 231 | **G16b** Correct big-Text recipe, 7 steps: (1) `odoo_request_upload` with **no pre-target**; (2) `curl` the RAW text; (3) a **temporary `ir.cron`** (confirm `model_id`) doing `attachment.raw.decode('utf-8')` → `write`; (4) trigger via an `ir.cron.trigger` record (**`method_direct_trigger` and `ir.actions.server.run()` are both blocked** by this proxy); (5) poll until `lastcall` is populated; (6) **mandatory read-back verify**; (7) clean up the temp cron + attachments | Gotcha 16 | PROSE-ONLY (ops recipe) |
| 232 | **G16c** **ALWAYS** do the read-back + `json.loads()` verify after **ANY** write to `spreadsheet_data`, plain `orm_write` included | Gotcha 16 | PROSE-ONLY |

### 6.3 Agent Routing, Delegates to, Consumed by

| # | Assertion | Location | Class |
|---|---|---|---|
| 233 | STEP 1 (CRM extraction) and STEP 3 (fill the spreadsheet) run as **sonnet subagents** (Pattern A) — mechanical work | §Agent Routing | PROSE-ONLY |
| 234 | STEP 2 runs as **opus, Main** — business decisions + interaction with the rep | §Agent Routing | PROSE-ONLY |
| 235 | STEP 4 runs as **opus, Main** — personalized copy | §Agent Routing | PROSE-ONLY |
| 236 | STEP 5 runs as **opus, Main**; 5.1 publishes HTML + note, 5.2 generates and attaches the PDF; idempotent | §Agent Routing | PROSE-ONLY |
| 237 | **Subagent contract**: self-contained prompts injecting `lead_input` (STEP 1) or `budget_json + sale_order_id + pricing` (STEP 3); they return **structured JSON, not prose**; they do **not** decide pricing and do **not** write PDF copy | §Agent Routing | PROSE-ONLY |
| 238 | `common-skills:odoo-pilot` is **REQUIRED** to create/write `sale.order.spreadsheet`, upload HTML/PDF as `ir.attachment`, and pull photos/titles from `hr.employee`; prerequisite = **per-operator Odoo API key** (`auth.sh`). Field read/write is also valid (OPTIONAL) via the operator's MCP | §Delegates | PROSE-ONLY |
| 239 | `common-skills:dynapps-brand` = **VISUAL identity only** (colors, Readex Pro, logos, gradient) | §Delegates | PROSE-ONLY |
| 240 | `dynapps-voice` (group skill, draft) = **VERBAL identity**; the `brand_lint.py` v2 lexicons (DASH/MARCA/REGISTRO/PRESION…) implement its rules — **defer voice questions there, not to dynapps-brand** | §Delegates | PROSE-ONLY (routing) |
| 241 | `common-skills:content-collection` supplies real success cases for `casos_exito` by sector | §Delegates | PROSE-ONLY |
| 242 | `utilities:publish` publishes the HTML in STEP 5 and re-publishes to the same destination if one existed | §Delegates | PROSE-ONLY |
| 243 | `sales-skills:lead-data-collection` (optional) is the prerequisite when `xx_data_collection` is missing | §Delegates | PROSE-ONLY |
| 244 | **No other skill consumes this one** — it is an endpoint of the commercial workflow | §Consumed by | PROSE-ONLY |
| 245 | Urgency is framed as **planning transparency** (how team assignment works), never as sales pressure | rules.md #34 **only** | CODE — `brand_lint` PRESION (pack `pressure` list) |
| 246 | `references/localization.md:74`: the Python `eur()` and the template's JS rounding **are frozen — do not unify them** | `references/localization.md` (**not in SKILL.md**) | CODE — `TestPricingDesdeCtx::test_js_defaults_es_frozen`, `TestParseAmountFloatIdentity` |

### 6.4 `## References` table — the **Content** column

⚠️ **Why this subsection exists.** §8.1 records the §References table as a *load map* (file +
when-to-load + token cost) and in doing so dropped the table's **Content** column, which carries
normative statements that appear **nowhere else in SKILL.md**. The most serious casualty was the
Inbuilt render contract (row 247): SKILL.md:143 is its only mention in the whole file, yet it
describes a live engine contract. Without these rows, F3 would have been licensed to delete them
silently. Every row below is `KEEP` — the reflow may relocate them, never drop them.

| # | Assertion | Location | Class |
|---|---|---|---|
| 247 | **Inbuilt / construction vertical render contract**: scoped via `deal.vertical='inbuilt'` (Parámetros sheet) or `extra_data.vertical`; the vertical is a **closed price WITHOUT hours** so it does **not** add to the 10% go-live support, but it **does** enter future migration; the pack **includes** the App and the App can be sold standalone; post-demo selection via `extra_data.vertical_sel = {pack, app}`. It renders as the **`#vert-block`** price tile on the web and as the PDF's `vert_block` inside the same value-added group | SKILL.md:143 (§References, Content col) — **only mention in SKILL.md** | CODE — `generate_proposal.py:922-944` (`vertical` dict, `L.price("vertical.inbuilt.pack_oneshot"/"app_oneshot")`), web anchor `:3488`, PDF `:4468-4487`; pack key `pricing.vertical.inbuilt` |
| 248 | **Double-charge suppression**: if the vertical already enters as a **first-class app** (sheet tab with base cell `Dynapps Construcción — Base` → present in `active`/`c['apps']`), the one-shot Parámetros path is **switched off** (`vertical["active"] = False`); the one-shot route survives ONLY for older bases without that tab | SKILL.md:143 implies it; the rule itself is **code-only** | CODE — `generate_proposal.py:932-937`; `_VERT_APP_NAME = "Dynapps Construcción"` (registry app id `inbuilt`) |
| 249 | The vertical's app card renders **narrative WITHOUT price** (the price lives in the Inversión block), with the same tier cards as any other app, and only when the deal has the vertical scoped and `app_overrides` carries personalized content (keys `Dynapps Construcción` or the legacy `Inbuilt (construcción)`) | code-only; SKILL.md:143 contradicts it (see S9) | CODE — `generate_proposal.py:2541-2548` |
| 250 | Emitting the vertical only when it is active is what keeps the **ES reference render byte-identical** — the vertical must never leak into a non-vertical deal's output | code comment, not in SKILL.md | CODE — `generate_proposal.py:2545`; guarded by `tests/golden_check.py` |
| 251 | `references/pricing_apps.md` is the pricing table of **13 apps × base + priced add-ons** — the canonical app count for STEP 2.1 detection | SKILL.md:137 (§References, Content col) | CODE-POSSIBLE — cross-checkable against `locales/*/pricing.json` `app_base_prices` (14 keys incl. `config_general`) |
| 252 | Connection and batch patterns are **not** documented in this skill: they live in the cross-repo pointer **`references/shared/odoo-connection.md`** | SKILL.md:136 (§References, Content col) | PROSE-ONLY (cross-repo pointer — breaks silently if dropped) |
| 253 | `references/timeline_rules.md` holds the internal timeline algorithm: **3 pace profiles (40/50/60 h/week)** → `duracion.semanas_*` and `gantt.semanas_por_app`; the hours are **never visible to the client** | SKILL.md:140 (§References, Content col) — complements #96 | CODE (weeks) + PROSE-ONLY (profile choice) |
| 254 | `references/spreadsheet_filling.md` defines the full `budget_json` structure in which **`duracion` and `formacion` are MANDATORY and > 0**, plus the formula cleanup/rebuild rules | SKILL.md:141 (§References, Content col) | CODE-POSSIBLE — the B44/B51/B52 completeness gate (#see STEP 4.1) catches the symptom, not the schema |
| 255 | `references/sector_glossary.md` is an **internal asset** — a map of Odoo terms → the client's sector vocabulary — and it **governs the writing** (it is not reference trivia) | SKILL.md:144 (§References, Content col) | PROSE-ONLY |
| 256 | The E2E process view rests on documented **literature analysis (APQC PCF, SAP Signavio, big-4)** plus explicit boundary decisions and the app→process mapping; `references/process_taxonomy.md` is where the mapping is edited, in `app_registry.processes` | SKILL.md:145 (§References, Content col) | PROSE-ONLY (provenance) + CODE (`shared/app_registry.json`) |
| 257 | `references/mcp_photo_access.md` carries the **attachment-id recipe** for pulling employee photos via MCP, **why the obvious paths fail**, and a standing feature request to the MCP team — the "why it fails" is what stops the next operator retrying the blocked path | SKILL.md:147 (§References, Content col) — complements Gotcha 11 | PROSE-ONLY |
| 258 | `locales/README.md` carries pack **governance**: per-file owners, sign-offs and the fallback policy. Governance is what makes "adding a country = adding a pack" safe | SKILL.md:148 (§References, Content col) | PROSE-ONLY (governance) |
| 259 | Onboarding a country goes **stub → live** via `scripts/add_country.py` + `scripts/odoo_discovery.py`; the approved BE/CH two-layer plan, intake checklists and per-locale case URLs live in `references/add_be_ch_plan.md` | SKILL.md:149 (§References, Content col) | CODE (`add_country.py`, `tests/add_country_smoke.py`) + PROSE-ONLY (checklists) |
| 260 | `references/rules.md` carries **model principles** alongside the numbered rules — the principles are not duplicated in SKILL.md and are the part most at risk when the duplicate collapses | SKILL.md:134 (§References, Content col) | PROSE-ONLY |

**Revised counts:** **260 assertions**, numbered 1-260 with no gaps. §6.4 adds 14 rows
(247-260): 6 `CODE`, 3 `CODE-POSSIBLE`, 7 `PROSE-ONLY` by strongest tag (rows 253/256/259 carry
two tags and are counted once). `LEGAL` remains **4** (rows 46, 47, 48, 64).

---

## 7. Stale text and contradictions F3 must resolve deliberately (not silently)

| Item | Detail |
|---|---|
| S1 | `SKILL.md` §Key rules says "Full list (**35**) in `references/rules.md`"; the §References table says "Full rule list (**31**)". `rules.md` actually carries **6 model principles + 36 numbered rules** (1-35 plus an extra `11.b`). All three counts disagree. F3 should stop quoting a count. |
| S2 | `rules.md` line 3 points the reader to "`SKILL.md` (`## Rules clave`)" — that heading no longer exists; it is now `## Key rules`. |
| S3 | `rules.md` #9 states the template is "id=15" flatly, while `SKILL.md` Key rule 1 and §Locale selection say the id is **discovered at runtime and never hardcoded** (15 is merely the value on the `master`/`es` DB). The SKILL.md form is the current doctrine. |
| S4 | `rules.md` #20 says "the file stays local; publishing is decided outside the skill" — contradicted by §STEP 5, which owns publishing (`utilities:publish` + the sale.order note). |
| S5 | `rules.md` #16 describes recurring costs as read from `Interna Comercial` rows 60/61/62, while `SKILL.md` Key rule 5 says "sale.order values or `RECURRENT_DEFAULTS`". Both are true of the code path; the rules.md wording carries the unique cell references (#170) and must not be lost when the duplicate collapses. |
| S6 | The all-caps rule claims CSS coverage ("CSS included", "no `text-transform: uppercase`") but `brand_lint.strip_html_visible()` strips `<style>` before linting, so the CSS half is **unenforced**. Either keep it as prose or make it a gate — do not let the reflow imply it is checked. |
| S7 | Gotcha 13 says "ideally `fetch_team.py` should do this by default" — the square-normalization is a known unimplemented gap, not a description of current behaviour. |
| S8 | `match_extra()` has a third matching heuristic (20-char prefix when the candidate is longer than 10 chars) that neither `SKILL.md` nor `rules.md` documents; "matching by substring" is an incomplete description of the code (relevant to the "new names must not collide" constraint). |
| S9 | **SKILL.md:143 is stale about the Inbuilt vertical.** It states "current price-tile behavior (`deal.vertical=inbuilt` → `#vert-block`, **NO app card yet**) and the approved first-class-app migration spec" — but the migration is **already partly implemented**: `generate_proposal.py:2541-2548` renders the vertical "como app de primera clase" with narrative tier cards, and `:932-937` carries the double-charge suppression that only makes sense once the first-class path exists. F3 must reconcile this deliberately (assertions #247-#249), not copy the stale sentence forward. Do NOT resolve it by deleting the row: the render contract is documented nowhere else in SKILL.md. |
| S10 | `references/localization.md:73-74` is now stale about the golden gate: it says "`es` is hardcoded; passing `--locale es` errors, it treats argv as fixture names". After the F0 parameterization, `es` is a `DEFAULT_LOCALE` and argv is parsed for locales as well as fixtures. The operative instruction (run it **bare**) and assertion #229 stay correct — only the parenthetical rationale is wrong. `references/workflow_detail.md:531` describes the same gate as "es byte-parity" only, which is now narrower than reality (48 artifacts across es/en/fr/nl). Whoever owns those files in F3 must fix both. |

---

## 8. Measured facts F3 needs — reference load map and token cost

### 8.1 Which references each STEP loads

| Phase | References loaded | Tokens pulled in (≈) |
|---|---|---|
| Always (skill fires) | `SKILL.md` | **8,740** |
| Start of execution | `workflow_detail.md` | 10,376 |
| STEP 1 | `subagent_prompts.md` (STEP 1 prompt), `workflow_detail.md` §1 | 2,255 (+ shared) |
| STEP 2.1 | `pricing_apps.md`, `addon_details.md` | 2,969 + 2,773 |
| STEP 2.2-2.4 | `analysis_pricing.md` | 949 |
| STEP 2.5 | `timeline_rules.md` | 1,302 |
| STEP 3 | `subagent_prompts.md` (STEP 3 prompt), `spreadsheet_filling.md`, `workflow_detail.md` §3 | 958 (+ shared) |
| STEP 4 (all of it) | `html_structure.md`, `workflow_detail.md` §4.0 | **12,907** + shared |
| STEP 4.3.b | `sector_glossary.md` | 809 |
| STEP 4.3.c | `mcp_photo_access.md` | 951 |
| STEP 4, non-ES | `localization.md` | 1,984 |
| STEP 4, process-view deals | `process_taxonomy.md` | 2,273 |
| STEP 4, construction deals | `inbuilt_app.md` | 2,694 |
| STEP 5 | `workflow_detail.md` §STEP 5 | (shared) |
| On demand ("a rule's detail") | `rules.md` | 3,233 |
| Language/pack work | `locales/README.md`, `add_country.md`, `add_be_ch_plan.md` | 2,945 + 1,016 + 3,113 |

### 8.2 File sizes (bytes → ≈ tokens at 4 bytes/token)

| File | Bytes | ≈ Tokens |
|---|---|---|
| `SKILL.md` | 34,962 | **8,740** |
| `references/html_structure.md` | 51,628 | 12,907 |
| `references/workflow_detail.md` | 41,506 | 10,376 |
| `references/rules.md` | 12,935 | 3,233 |
| `references/add_be_ch_plan.md` | 12,455 | 3,113 |
| `references/pricing_apps.md` | 11,879 | 2,969 |
| `references/addon_details.md` | 11,094 | 2,773 |
| `references/inbuilt_app.md` | 10,776 | 2,694 |
| `references/subagent_prompts.md` | 9,022 | 2,255 |
| `references/process_taxonomy.md` | 9,092 | 2,273 |
| `references/localization.md` | 7,939 | 1,984 |
| `references/timeline_rules.md` | 5,208 | 1,302 |
| `references/add_country.md` | 4,064 | 1,016 |
| `references/spreadsheet_filling.md` | 3,834 | 958 |
| `references/mcp_photo_access.md` | 3,806 | 951 |
| `references/analysis_pricing.md` | 3,798 | 949 |
| `references/sector_glossary.md` | 3,236 | 809 |
| `locales/README.md` | 11,782 | 2,945 |
| **Total prose corpus** | **248,016** | **≈ 62,000** |

### 8.3 The cost centre

`SKILL.md` is the **only file paid on every invocation** (≈8,740 tokens). Reflowing 35 KB → 10 KB
saves **≈6,200 tokens per run** and touches nothing else. The two big references
(`html_structure.md` 12.9k, `workflow_detail.md` 10.4k) are the next targets but are pulled
conditionally, so they are a smaller expected-value win per invocation.

Note for F3: 64 of the 246 assertions above are duplicated between `SKILL.md` and `rules.md`
(marked **DUP**). Collapsing those is the single largest lossless reduction available — but rows
#147-#149, #153-#154, #157, #159, #163, #165, #167, #170, #176, #179-#183, #186 and #245 carry
detail that exists **only** in `rules.md`, and rows #12-#19, #48, #64, #128-#131 carry detail that
exists **only** in `SKILL.md`. Neither side is a superset of the other.
