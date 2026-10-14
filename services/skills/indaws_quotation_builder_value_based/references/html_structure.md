# HTML Proposal — Structure, content, and writing rules

## Output modes (`extra_data.mode`)

The **same `extra_data.json`** feeds **two** modes. Defaults to `live`.

| Mode | Purpose | Prices | Price block | Anchors | Devs |
|------|----------|---------|------------------|--------|------|
| `predemo` | Send **before/around the demo** | **None** | hidden | hidden | hidden |
| `live` | Build the price **live with the client** (source of the PDF) | **Real** | **single interactive list** with live TOTAL | yes (if data exists) | checkable, with price |

> **The price IS shown to the client in `live`.** It's built LIVE with the client on top of
> the single interactive list (checkable apps + dynamic 10% support + developments + added value),
> with the TOTAL recalculated instantly. **This SUPERSEDES the old "Golden Rule 3"** (which
> prohibited showing the price in writing). `predemo` still has no prices. The `proposal` (ranges)
> and `close` modes are removed: a single mode variable with two values, `predemo` and `live`.

## Document structure — client-first order (v5)

Same order in web and PDF; the mode controls which sections appear. **Mutual commitment always
comes AFTER pricing** — the calendar is only credible once the client has seen the number.

| # | Section (anchor) | Content |
|---|---------|-----------|
| 1 | Cover / Hero | Logo, client, sector, date, number, sales rep. Optional `extra_data.hero_outcome` → outcome-led H1 with the label "Propuesta Dynapps para {cliente}" above it |
| 2 | **Understanding your business** (`#entendemos`) | "What you told us": **context + ambition ONLY** — who they are, what they want to achieve, how they'll measure success. **The problem list does NOT live here** (it lives ONCE, in the challenges section). Opens the document, before talking about Dynapps |
| 3 | **Your challenges** (`#retos`) | challenge → solution, **max 4 pains** (data-level cap; the engine warns on stderr → MERGE related pains, don't drop context) + "the cost of staying the same" strip (qualitative, no €, no FOMO) |
| 4 | **Comparable cases / proof** (`#casos`) | Logo wall + **quote-first cards** + solid industry pills. Quote only if it genuinely exists on the market site; otherwise a factual headline. See the proof data contract below |
| 5 | Solution map (`#mapa`) | Radial mindmap (flat view) or Odoo metro map (process view) |
| 6 | Timeline (`#cronograma`) | Phase flow + Gantt (PDF: merged with methodology into one "How we deliver it" section) |
| 7 | Applications (`#apps`) | 1 card/app: **✓ Included** vs **+ Tailored to your case** bullets. **Web only, accordion (progressive disclosure):** each card is a native `<details class="app-card app-acc">` collapsed by default — the `<summary>` shows icon, name, outcome, a compact count chip (`7 ✓ · 2 +`) and a violet chevron; intro/fit/tiers live intact in `.app-acc-body`. One "Expand all / Collapse all" button (`.acc-toggle-all`, labels from `sections.apps_expand_all` / `apps_collapse_all`). JS opens all on `beforeprint` and restores on `afterprint` (closed `<details>` don't print). The PDF keeps every card fully expanded, unchanged |
| 8 | **Why Dynapps** (`#dynapps`) | 3 pillars (escala / partner único / predecible) + figures band (900+ implementations "#1 worldwide" · 280+ · 52 · security · QA). Security figure = ISO 27001 ONLY where `credibility.iso27001` is true (BE market); elsewhere the generic high-security-standards claim (`fig_security_*`) — never claim ISO outside Belgium. + philosophy |
| 9 | Your team (`#equipo`) | Team cards from `hr.employee` |
| 10 | **Investment** (`#inversion`) | **Single interactive list** with live TOTAL (apps + support + devs) + **Added value / AI** block. Hidden in `predemo` |
| 11 | **The calendar, together** (`#calendario`) | Mutual client/Dynapps anchors (`live` only). **Always AFTER pricing** |
| 12 | Closing (`#cierre`) | Recurring + CTA. Sales rep photo when available, next-step line derived from `anchor_dates`, **NO phone number** |

Web extras: dark glass top nav + dark footer with the gradient line; scroll-reveal animations
with a **1.5s safety timeout** (content never stays hidden if JS stalls) and
`prefers-reduced-motion` respected.

**Web only, reading mode (executive vs full):** a compact two-button switch (`.mode-switch`,
labels `sections.mode_exec` / `mode_full`, group aria label `mode_aria`) on the right of the top
nav toggles `.exec-mode` on `<html>`. Executive mode hides via pure CSS: `#mapa`, `#cronograma`,
`#equipo` (plus their topnav/docnav links — no dead anchors), the apps accordion **bodies** (the
collapsed summaries stay as an index; the toolbar and chevrons hide and summaries become inert),
and the added-value block's **unchecked** rows / all-unchecked groups (`:has()`). It keeps hero,
understanding, challenges, proof, apps summaries, why Dynapps, investment, calendar and closing.
Default is always the **full document**; the mode is NOT persisted. `beforeprint` forces full mode
and `afterprint` restores it, so print never misses content. Without JS the switch is hidden
(under `.js-anim`) and the document is always full.

---

## Pricing model: single interactive list

From the client's perspective there is a single **"Implementation price"** block with a **live TOTAL**
that the sales rep builds by checking/unchecking items with the client. **There is no Recommended/Custom concept.**

### Block order

1. **Applications · "check the ones you need"**
   - **General Configuration**: INCLUDED, **cannot be unchecked**.
   - **Apps**: each one checkable, with its **closed price** (= base + spreadsheet extras, same as today).
     - **Pre-checked**: based on `extra_data.apps_recomendadas` (list of app names) or the per-app flag if
       provided. If there's no flag, **all** scoped apps are checked.
   - **Go-live**: INCLUDED, **cannot be unchecked**. Amount is **DYNAMIC = 10% of the sum of CHECKED apps**
     (apps only, **never** developments), rounded to €10, **recalculates live** as apps are toggled.
     Includes "support, migrations, and training".
2. **Custom developments · "check the ones you want"**: checkable, with their price (the devs from the spreadsheet).
   **Web only, collapsible rows:** each dev row is `<li class="li-row li-dev-acc">` with the checkbox
   `<label>` as a SIBLING of a native `<details class="dev-acc">` (summary = title + price + chevron,
   body = the full technical description from `dev_overrides` / the dev catalog). Because the checkbox
   lives outside the `<details>`, toggling the selection never expands/collapses, and the live recalc
   reads `checked` state regardless of collapse. JS opens dev accordions too on `beforeprint`
   (restored on `afterprint`). The PDF keeps its own devs page, unchanged.
3. (**added-value** products go in their own block, see below).

### Live TOTAL

```
TOTAL implementation = Σ checked apps
                     + support (10% of implementation HOURS: config + checked apps + checked devs + analysis — sheet F36/E36)
                     + Σ checked developments
                     + Σ checked added value (one-shot amounts)
```

**Recurring** amounts (MCP €50/month, Field Service €2/user/month, future migration) **do NOT** enter
the implementation TOTAL — they go to the separate recurring block.

The JS amount format matches `eur()` (thousands with ".", no decimals). In `live`, the TOTAL and the
subtotals (support, added value) **recalculate instantly** as items are checked/unchecked. **In live
mode, app cards and map nodes are NOT hidden when unchecked** — the fine-grained trimming happens in
the final PDF.

Removed from the client-facing output: the "2-level" accumulation chart, the Recommended/Custom
breakdown, and the old devs bento (all reworked into the single list).

---

## Added value / AI block

Its own block, everything **unchecked by default**. FIXED catalog with prices already resolved
from Odoo (**no need to check Odoo**, use these amounts). The sales copy for 343/344/345 is
provided by the Copy phase; for the rest, a brief, accurate text.

**EXCLUSIVE group (radio, pick 1; both unchecked):**
- **MCP Odoo** (product 345): **€2,500 one-shot + €50/month recurring**.
- **AI Starter Pack — Training and Infrastructure** (product 339): **€7,500 one-shot**.

**Add-ons (multi-select, unchecked):**
- **Payroll OCR** (343): €1,000.
- **Delivery Note OCR** (344): €1,000.
- **Monitor CRM opportunities with AI** (305): €850 — **ONLY if the CRM app is checked**.
- **CRM Prospecting** (288): €250 — **ONLY if CRM is checked**.
- **Field Service / Mobile Apps** (212): €2,500 one-shot **+ €2/user/month** license — **ONLY if
  the Projects app is checked**.

**POC: Essential Flow** (347): amount = **20% of the Functional Analysis price** (the product's
€1,000 is only indicative, **do NOT use it**). It's tied to the Analysis (in the Analysis section), selectable.

**On-site presence**: **€900/day × number of days** (quantity selector).

**Conditional items (305, 288, 212) are HIDDEN live** if their app stops being checked. The one-shot
amounts of checked added value **add to the implementation TOTAL**; recurring ones (MCP €50/month,
Field Service €2/user/month) go to recurring.

---

## Recurring + future migration

Existing recurring costs block (Dyn Cloud / Maintenance / Odoo License). Added:

- **Future migration** — **SELECTABLE** monthly product. Amount = **15% of (checked apps +
  checked developments) / 24 = X €/month**, recalculates live. Note to client: *"renewal of the
  migration cycle every 2 years"*.
- **Recurring MCP** (€50/month) and **Field Service** (+€2/user/month) go here if checked.
- Keep the **partner manager** Micaela Gil Rother · migi@odoo.com (already present in recurring).

> Document the percentage (15%) and the divisor (24) as configurable, in case the model evolves.

---

## extra_data.json (CRITICAL)

The skill MUST write personalized content into this JSON. **Do not let the script auto-generate
text** — templated copy comes out generic and unprofessional.

### Structure

```json
{
  "fecha_completa": "April 10, 2026",
  "num_presupuesto": "PR3671",
  "comercial": "Iñaki Núñez",
  "executive_summary_html": "<p><strong>COMPANY SL</strong> is...</p><p>...</p>",
  "app_overrides": {
    "CRM": {
      "intro": "<p>Personalized text about how CRM helps the client</p><div class=\"highlight-box\"><p>Key benefit</p></div>",
      "features": {
        "Pipeline comercial (etapas, probabilidades, ≤3 comerciales)": "Client-specific explanation",
        "...": "..."
      }
    }
  }
}
```

### Getting exact feature names

The `features` keys must match the spreadsheet names EXACTLY. To extract them:

```bash
python3 -c "
import json, sys
sys.path.insert(0, 'scripts')
from proposal_data import extract_app_details
with open('/tmp/spreadsheet.json') as f:
    ss = json.load(f)
details = extract_app_details(ss, 'CRM')
print('--- Base (de serie) ---')
for cat, items in details['base_features'].items():
    for it in items: print(f'  {cat}: {it}')
print('--- Avanzadas (extras) ---')
for cat, items in details['extras'].items():
    for it in items: print(f'  EXTRA {cat}: {it}')
"
```

The features in `details["base_features"]` render under the **"Included"** tier (bullet marker
**✓**); those in `details["extras"]` under **"Tailored to your case"** (bullet marker **+**) —
the internal `base_features`/`extras` KEYS don't change; only the visible labels/markers change
so it doesn't look basic/advanced. The override by name works in both blocks.

**Localized feature titles (non-ES proposals).** A `features` entry accepts, besides the legacy
plain string, an **object form**: `"<spanish-key>": {"title": "<localized bullet name>",
"expl": "<personalized explanation>"}`. The key stays the exact Spanish spreadsheet name (machine
contract); `title` is what the client sees as the bullet name. **For any non-es proposal the
object form is mandatory** — it closes the old gap where bullets rendered the Spanish key.

---

## Executive summary / "What you told us" — How to write it

The opener is **the selling moment**. It proves we listened: **context + ambition ONLY**.
Recommended structure (3-4 paragraphs):

1. **Who the client is** (sector, business model, size, markets) — 1 sentence
2. **Their ambition, in their own words**: what they want to achieve and why now
3. **How they'll measure success**: the concrete metrics/outcomes they named
4. **Timeline and phases**: analysis + implementation, without going into operational detail

**Rules:**
- **The problem list does NOT go here.** Problems live ONCE, in the challenges section
  (`#retos`, max 4) — repeating them in the opener makes the document read as pain-hammering
  instead of understanding. *(This replaces the old v2 instruction to list pain points here.)*
- Do NOT copy verbatim from the intake data. Rewrite it in professional prose.
- Do NOT use flat lists of data points (+30 years, X clients, Y countries).
- DO use natural sentences that explain the context.
- Focus on **the client and their business**, NOT on the Odoo apps (that goes in each card).
- The philosophical callout "closed value, no meters running" does NOT go here — it's in "Why Dynapps".

---

## Recurring costs (optional but recommended)

`extra_data.costes_recurrentes` is a list of 3 cards that appears on the Investment page. It
overrides what the script reads from the spreadsheet (the cells usually have `=ODOO.LIST(...)`
formulas that don't get evaluated when exporting the JSON).

```json
"costes_recurrentes": [
  {"title": "Dyn Cloud", "price": "1.080,00 € / año",
   "desc": "High-performance hosting, daily backups, and SSL security."},
  {"title": "Mantenimiento", "price": "480,00 € / mes",
   "desc": "Extended support for the group (60 users, 6 companies)."},
  {"title": "Licencia Odoo", "id": "odoo_license", "amount_per_user": 334.80,
   "price": "334,80 € / año (27,90 € / mes)",
   "desc": "Direct contract with Odoo S.A. — Custom plan, per user."}
]
```

⚠️ **The Odoo License uses the PER-USER RATE, not the total.** The engine multiplies it by
`extra_data.num_usuarios` (live selector on the web + scaled in the PDF), so if you inject the
already-multiplied total it gets **duplicated** (20,088 × 60 = 1,205,280). Set `price` per user (or
the explicit `amount_per_user` field) and fix `num_usuarios`; the engine calculates the TOTAL.
`generate_proposal.py` warns via stderr if the rate/user exceeds €500.

If `costes_recurrentes` is omitted, the script uses the `RECURRENT_DEFAULTS` (Dyn Cloud
€1,080/year, Maintenance €360/month, Odoo License €334.80/user/year) and scales the license by
`num_usuarios`. For negotiated terms, override it here — keep in mind that **all 3 cards get
replaced as a block** (if you touch one, include all three).

**Light User is NOT a `costes_recurrentes` card.** Set `extra_data.num_usuarios_light` (or
`deal.num_usuarios_light` in `Parámetros`): the engine adds its own annual card "Licencia Odoo ·
Light User" = pack rate (4.90 €/user/month in `es`) × 12 × n, a plain fixed card that the
`#recur-users` selector never rescales. Absent at 0; omitted with a stderr warning when the pack
rate is `-1` (unverified). Gotcha 12b.

---

## Per-app customizations

- **intro:** HTML with `<p>` explaining what the app does FOR THIS CLIENT. Focused on real pain
  points. Mention concrete data (number of users, warehouses, revenue).
- **fit (hyper-personalization, recommended):** a short sentence rendered as a highlighted callout
  **"For &lt;Client&gt;: …"** right under the intro. Connects the module to the SPECIFIC need of
  the deal (their companies, their current system, their sector, their timeline). A single line,
  no filler — the client should see themselves reflected. E.g.:
  `"fit": "Replaces A3/A3Bank: closing and consolidation of the 6 group companies with automatic intercompany eliminations."`
- **features:** 🛑 **MANDATORY — not optional.** Each spreadsheet feature explained in 1-2
  sentences personalized to the client. The key must match the spreadsheet name EXACTLY (valid
  for both base and advanced features).

### Why `features` is mandatory

If only `intro` is set and `features` is omitted, each app card comes out like this (unacceptable
in a professional proposal):

```
❌ BAD — plain spreadsheet text, not personalized:
   • Configuración del sistema (empresa, logo, moneda, idioma)
   • Multi-compañía
   • Trazabilidad por lotes
```

With `features` personalized to each client:

```
✓ GOOD — each feature contextualized to the real client:
   • Configuración del sistema → "A single Odoo instance with the identity
     of the group's 6 companies (IMA, Anrobox, Anrotech, Ogar, EDC, Vipcasa)
     and operating currencies EUR and USD"
   • Multi-compañía → "The group's 6 companies with per-company access
     rules and intercompany operations enabled"
   • Trazabilidad por lotes → "Traceability of steel, profiles, and
     consumable materials by lot, with full history from receipt through
     use in production"
```

### Validation with the sales rep BEFORE generating HTML

Once `extra_data.json` is written, **show the sales rep a summary** (full executive summary +
2-3 feature examples per app) and **ask for explicit confirmation** before running
`generate_proposal.py`. Content quality gets validated in the JSON, not by reading the full
HTML/PDF afterward.

---

## Generation: web HTML + executive PDF (Dynapps brand)

`scripts/generate_proposal.py` builds a single `context` from the spreadsheet + intake data
(reuses the extraction from `proposal_data.py`) and renders it into TWO Dynapps templates:

- `templates/propuesta_web.html` — **rich web page, high-contrast light background** (light
  variant of dynapps-brand; dark glass top nav + dark footer with the gradient line;
  scroll-reveal with a 1.5s safety timeout + `prefers-reduced-motion` respected). **v5 order**
  (see the section-order table above): hero (optional outcome-led H1 via `hero_outcome`) →
  understanding → challenges (max 4) → **cases** (logo wall + quote-first cards + solid industry
  pills) → **radial mindmap / metro map** (SVG, light theme; central node in Odoo purple
  #714B67) → timeline (flow + bar gantt) → apps with tiers **✓ "Included" vs + "Tailored to your
  case"** → **Why Dynapps** (3 pillars + figures band: *900+ implementations, #1 worldwide* ·
  280+ · 52 · security(ISO only if credibility.iso27001) · QA + philosophy) → **Your team** → **"Implementation price" — single
  interactive list** (non-uncheckable General Config + checkable pre-checked apps +
  non-uncheckable dynamic 10% support + checkable developments) with **live TOTAL** + **added
  value / AI block** (MCP/Starter radio + conditional add-ons + POC + on-site presence) → **the
  calendar, together** (after pricing) → closing (recurring + future migration + rep photo +
  next-step line, no phone). In `predemo` there are no prices or price/added-value block.
- `templates/propuesta_pdf.html` — **executive PDF, NOT interactive** (no checkboxes or JS):
  sober light layout, A4 with `@page` margins **17mm 15mm** and ~22px card padding, clean page
  breaks (`page-break-inside: avoid`). **Mirrors the web's v5 order** (challenges before cases;
  calendar/anchors after the investment): full-page cover → understanding → challenges →
  **proof page** (embedded client logos + industry pills + quote-or-headline cards) →
  **"How we deliver it"** (methodology + timeline MERGED into one section) → apps (✓/+ bullets)
  → Why Dynapps (3 pillars + figures band) + Your team → **Investment that REPLICATES the web**:
  Analysis panel (gradient, large price, entitlements, POC+total), light "Closed price" card,
  **card-type blocks with ONLY what's checked** — Applications (Config "Included" + apps +
  Go-live "Included"), Custom developments, and **AI (chosen platform + checked add-ons)** —
  each with its subtotal, **implementation TOTAL in a dark panel**, and recurring costs in two
  columns (annual | monthly). AI is just another selection: what's NOT checked doesn't appear
  (no showcase appendix). → calendar, together → closing strip (rep photo + next-step line, no
  phone). **Colour discipline:** non-content blocks are de-coloured — colour only on the cover,
  Why Dynapps, closing, price/value/total panels, industry pills and the timeline. Rendered with
  Chrome headless.

```bash
# 1. Save the sale.order spreadsheet
search_records.sh sale.order.spreadsheet '[["order_id","=",<ORDER_ID>]]' '["spreadsheet_data"]' \
  | python3 -c "import sys,json; d=json.load(sys.stdin); open('/tmp/ss.json','w').write(d[0]['spreadsheet_data'])"

# 2. Save the intake data
search_records.sh crm.lead '[["id","=",<LEAD_ID>]]' '["xx_data_collection"]' \
  | python3 -c "import sys,json; d=json.load(sys.stdin); open('/tmp/toma.html','w').write(d[0]['xx_data_collection'])"

# 3. Write extra_data.json (the SKILL writes the personalized content — feeds BOTH formats)
cat > /tmp/extra.json << 'EOF'
{...}
EOF

# 4. STEP 4 — generate ONLY the web HTML (NOT the PDF, until the demo selection is available)
python3 scripts/generate_proposal.py --only web /tmp/ss.json /tmp/toma.html /tmp/propuesta-pr<NUM> /tmp/extra.json
#   -> /tmp/propuesta-pr<NUM>.html      (interactive web)

# 5. MANDATORY post-generation gates (both paths, before ANY delivery):
python3 scripts/brand_lint.py [--locale <code>] --html /tmp/propuesta-pr<NUM>.html
python3 scripts/verify_totals.py --html /tmp/propuesta-pr<NUM>.html --extra-data /tmp/extra.json \
        [--locale <code>] --ss /tmp/ss.json --toma /tmp/toma.html
#   verify_totals recomputes the recurring totals with the same engine and hard-FAILS on
#   summary-vs-detail drift (the PR4021 bug class). Exit 0 = deliverable; exit 1 = fix first.

# 4.bis STEP 5.2 — after injecting the selection copied from the demo into extra.json, generate ONLY the PDF
python3 scripts/generate_proposal.py --only pdf /tmp/ss.json /tmp/toma.html /tmp/propuesta-pr<NUM> /tmp/extra.json
#   -> /tmp/propuesta-pr<NUM>-pdf.html  (PDF source)
#   -> /tmp/propuesta-pr<NUM>.pdf       (Chrome headless)
# ... then re-run BOTH gates on the PDF source (-pdf.html) before attaching the PDF.
```

Notes:
- Requires **Google Chrome / Chromium** for the PDF. If it's not available, the script warns and
  leaves the `-pdf.html` (it can be printed to PDF from the browser).
- Brand asset override via `DYNAPPS_BRAND_ASSETS` (points to `dynapps-brand/assets`).
- `extra_data.pain_points` (list of strings) overrides the pains from the intake. Each pain is
  automatically matched against the Odoo capability that solves it (`solution_for_pain`); for
  full control, write the `executive_summary_html`.

### Official Odoo icons per app

Each active app is rendered with its **official Odoo icon** (multicolor SVG on a transparent
background, embedded as a data URI → self-contained deliverables). They appear in **three places**:

1. **App cards (web)** — rounded tile tinted with the app's accent color (`app-ico`).
2. **App cards (executive PDF)** — small icon next to the title (`p-app-ico`, replaces the old
   color bar).
3. **Radial solution mindmap** — each app node shows its icon inside an accent ring.

The SVGs live in `assets/odoo-icons/` and the app-name → file mapping is in `APP_ICON_FILES`
(`scripts/generate_proposal.py`). If an app has no mapping/file, there's an **automatic
fallback** (initial on a color in web, color bar in PDF) — rendering never fails. To add a new
app: copy its official SVG to `assets/odoo-icons/` and add the entry to the mapping. Do NOT
replace icons with emojis or generic icons.

### Optional hyper-personalization sections

Both render **only if there's data** (if missing, nothing appears — the engine is robust):

```json
"equipo": [
  {"name": "Iñaki Núñez Iturain", "role": "Your sales rep", "photo": "data:image/png;base64,...",
   "area": "Sales", "email": "inaki@dynapps.es", "focus": "Your point of contact throughout the project.",
   "skills": ["Odoo Functional Certified", "English"]},
  {"name": "Jesús Lloret Quero", "role": "Project manager", "photo": "data:...", "area": "Consulting", "email": "jesus@dynapps.es"}
],
"casos_exito": [
  {"cliente": "TM Technics", "sector": "Construcción",
   "industry_label": "Construcción",
   "metric": "Obra, producción y costes por proyecto en 1 plataforma",
   "resultado": "Tras un primer ERP fallido, unificó obra, producción y costes por proyecto en una sola plataforma Odoo.",
   "quote": "…only if it genuinely exists on the market site (fetch_references.py provenance)…",
   "url": "https://www.dynapps.es/es/cases/tmtechnics",
   "image": "data:image/jpeg;base64,...",
   "logo": "data:image/png;base64,...",
   "local": true}
]
```

⚠️ `metric` and `image` are **NOT optional**: paste the output of `match_cases.py` AS-IS (it
already carries both) and run `fetch_case_images.py` to embed the hero image (see 4.3.d). A
hand-written case without `image` renders a bare card **silently** (the engine only warns on
stderr).

- **equipo** → "Your team / People, not tickets" section (web) and "Your project team" block on
  the "Why Dynapps" page (PDF). Fields per member: `name`, `role`, `photo`, `area`
  (department), `email` (**company email only — no phone, it's usually personal**), `focus`
  (focus line; if missing it's derived from role/area), and `skills` (list of
  competencies/certifications → chips). If a member has no `photo`, an avatar with initials over
  a gradient is drawn. Generate with `scripts/fetch_team.py "Name:Role" ...` (authenticated Odoo
  environment): it pulls the photo (`image_256`, ~100 KB), `department_id` → area, `work_email`
  from `hr.employee`, and the skills from `hr.employee.skill` **filtering out hobbies/sports**
  (they only show up when employees fill in skills/certifications on their Odoo profile). The
  sales rep provides the names. Matching is by `ilike`; if the full name fails, it retries by
  last name (e.g. "Antonio Hermosilla" might be in Odoo as "Toni Hermosilla").
  🛑 **The source of truth for rendering is `extra_data.equipo`** (the generator does NOT read a
  separate `team.json`): if you fix team photos/data, **re-inject them into `extra.json`** before
  regenerating, or the issue persists. The `photo` must be a **data URI with clean ASCII base64**
  (`data:image/png;base64,iVBOR…`) — **no** `b'...'` wrapper, no quotes, no line breaks;
  `fetch_team.py` and the renderer already sanitize it, and if the photo comes back empty/`false`
  it falls back to the initials avatar. **Cowork gotcha:** the MCP proxy does NOT serve binaries
  (`image_256` comes back `false`) → get the photo with `fetch_team.py` (RPC), not via MCP.
- **casos_exito** → proof section (`#casos`): **logo wall + quote-first cards + solid industry
  pills**. Picks the most relevant by **country-local origin, industry and pain** (match_cases v2
  scoring, see the engine section below). **Proof data contract per case:** `metric` (headline
  figure, **mandatory**), `cliente`, `sector`, `industry_label` (exact site-taxonomy label →
  solid pill, colour from `shared/industries.json`), `resultado` (1 sentence), `url` (link to the
  case), `image` (hero, **mandatory** — embed with `fetch_case_images.py`), `logo` (client logo
  data URI — `fetch_case_images.py` embeds it too; real logo when available, the client name as
  text otherwise, **never a fabricated logo**), `quote` (client quote — **ONLY if it genuinely
  exists on the market site**, scraped with provenance by `fetch_references.py`; quote absent →
  the card leads with the factual headline. **NEVER invent a quote**) and `local` (country-local
  case flag). See "Comparable cases engine" below.

---

## Value-based fields in `extra_data.json`

```json
"mode": "predemo | live",
"hero_outcome": "Una sola plataforma para obra, producción y costes",
"apps_recomendadas": ["CRM", "Ventas", "Inventario"],
"forma_pago": "SEPA B2B",
"contacto": {"email": "info@dynapps.es", "web": "www.dynapps.es", "tel": "960202942"},
"understanding": {
  "sector_intro": "<p>Who the client is and what we've understood about them…</p>",
  "objetivos": ["What they want to achieve, in their own words", "…"],
  "success_measures": ["How they'll measure success (metric)", "…"]
},
"cost_of_delay": ["Qualitative sentence about the cost of staying the same (NO €)", "…"],
"cliente": "Davis Fabrics Spain SLU",
"anchor_note": "We plan teams during the last week of each month. Your decision date defines your start month; one extra week deciding can mean one extra month before starting.",
"anchor_dates": [
  {"owner": "Tú",      "date": "Jun 30", "milestone": "Decision date"},
  {"owner": "dynapps", "date": "Jul 1",  "milestone": "Team assignment"},
  {"owner": "Juntos",  "date": "Jul 7",  "milestone": "Kick Off"},
  {"owner": "dynapps", "date": "Jul–Aug","milestone": "Analysis"},
  {"owner": "Juntos",  "date": "Sep",    "milestone": "Implementation"},
  {"owner": "dynapps", "date": "after go-live", "milestone": "Support"}
],
"sector_glossary": { "...": "see references/sector_glossary.md (internal, not rendered)" }
```

- **cliente** → **ALWAYS wins** over the name extracted from the intake. The intake parser can
  confuse the **partner signature** in the header ("Odoo Gold Partner", "Dynapps", "Dynapps")
  with the client's name (`extract_client_info` already filters out those signatures, but the
  reliable name is `res.partner.name` from the sale order's `partner_id` — inject it here).
- **understanding** → "Understanding your business" opener. If missing, falls back to
  `executive_summary_html` or the default summary. `objetivos`/`success_measures` only render
  if present.
- **cost_of_delay** → "The cost of staying the same" strip (qualitative). If omitted, it's
  derived from the pains. **Never a number in €** (the quantified opportunity cost is stated
  live). Tone: **factual and consequence-based, not FOMO** — describe what staying the same
  costs in their operation (rework, blind spots, manual hours), never pressure phrasing
  ("don't miss out", "now or never"). `brand_lint.py` PRESION flags violations.
- **apps_recomendadas** → list of app names **pre-checked** when the single list loads (or the
  per-app flag if the spreadsheet provides it). If missing, **all** scoped apps get checked.
  General Config and Go-live are always checked and are **NOT** uncheckable.
- **Single interactive list** → in `live` (only the `.html`, not the PDF), apps, developments,
  and added-value products are **selectable** (checkbox/radio). A self-contained JS recalculates
  **live** as items are checked/unchecked: **go-live support** (10% of implementation hours:
  config + checked apps + checked devs + analysis, sheet F36/E36), the added-value subtotals,
  **future migration** (15% of checked apps+devs / 24 = €/month), and the **implementation
  TOTAL** (Σ apps + support + Σ developments + Σ one-shot added value). Conditional added-value
  items (305, 288 if CRM; 212 if Projects) **hide** when their app is unchecked. **App cards and
  map nodes are NOT hidden when unchecked.** The **PDF is static** (body = ONLY what's checked,
  including the selected AI; no appendix). The JS amount format matches `eur()` (thousands with
  ".", no decimals).
- **anchor_dates** → "A mutual commitment, with dates" section (`live` only). These are the
  **sales-cycle** milestones (Decision date → Team assignment → Kick Off → Analysis →
  Implementation → Support), each with an **owner** (`Tú`/client = orange · `dynapps` = violet ·
  `Juntos` = carmine) and a date. It's a mutual commitment that drives the close, not data
  logistics. Dates can be **real** (anchored to the end-of-month planning window, calculated
  from TODAY) or **relative** (Month 0 / Week 1). ⚠️ If you use real dates, **recalculate them
  based on the generation date** — don't leave fixed dates from a previous proposal.
  **Editable from the spreadsheet:** if `extra_data` does NOT provide `anchor_dates`, the engine
  **derives them from the `Parámetros` sheet** of the spreadsheet: `deal.fecha_decision`
  (dd/mm/yyyy) is the anchor and the cascade uses the Interna Comercial weeks (assignment +3
  business days · kick-off 1st Monday of the following month · analysis + semanas_analisis ·
  implementation + semanas_impl · support "after go-live"); `deal.anchor_<milestone>` overrides
  specific milestones and `deal.fecha_propuesta` fixes the document date. Priority:
  `extra_data.anchor_dates` > sheet > `[]` (section hidden). Details in
  `references/workflow_detail.md` §Parámetros.
- **anchor_note** → highlighted callout (banner with the brand's 45° gradient) **above** the
  timeline, framed as **planning transparency, never as sales pressure**: it factually explains
  how our planning works ("teams are assigned during the last week of each month, so your
  decision date defines your start month") and its concrete consequence. No FOMO, no countdown
  language, no "last chance" phrasing — `brand_lint.py` flags pressure wording (PRESION). Only
  renders if present (web and PDF). *(Reframed in v5: this used to be documented as an "urgency
  callout / closing lever".)*
- **hero_outcome** → optional one-line outcome. When present, the hero renders an
  **outcome-led H1** (the client's desired outcome as the headline) with the label
  "Propuesta Dynapps para {cliente}" above it. Write it as the client's result, not our service
  ("Una sola plataforma para obra, producción y costes", not "Implantación de Odoo"). If
  omitted, the classic hero (client + proposal title) renders unchanged.
- **sector_glossary** → internal; governs the vocabulary used when writing (not rendered).
  `sector_glossary.permitir` (list) silences false positives from `brand_lint.py`.
- **forma_pago** → text in the investment fine print ("Payment method: …"). Defaults to
  "bank transfer".
- **contacto** → `{email, web, tel}` for the CTA/contact strip. Defaults to Dynapps
  (info@dynapps.es · www.dynapps.es · 960 202 942).

## Comparable cases engine

0. **References cache (per-country market-site scrape).** `scripts/fetch_references.py
   --country <CC>` scrapes the country market site into `shared/references/<CC>.json`
   (ES/BE/CH cached), each entry with **provenance + `scraped_at`**. Quotes are captured only
   when they genuinely exist on the site. `shared/industries.json` holds the canonical industry
   ids + pill colour; each country layer's `instance.json` carries an `industries` key with the
   exact site-taxonomy labels (note: **BE has NO construction industry**). A cache older than
   **90 days** makes downstream scripts warn on stderr → refresh with `fetch_references.py`.
1. `python3 scripts/match_cases.py --locale <code> "<sector>" "<pains/toma>" [N]` — **v2
   scoring**: country-local case first (**+8**) > exact industry (**+6**) > adjacent industry
   (**+3**) (+ pain-archetype points), and it emits `industry_label`, `logo`, `quote` and
   `local` per case alongside `metric`, `url` and `image`. `--locale` = the language the
   sector/pains text is written in (scoring runs against the pack's localized sector tokens —
   see §4.3.d step 1).
2. **Relevance gate:** if the best case has `sector = 0`, the script prints a stderr WARNING and
   marks the affected cases with `"weak_match": true` — the library doesn't cover the sector and
   the ranking is archetype-only. Do NOT use weak matches silently: first re-check the `--locale`
   matches the text's language, then find a local case (content-collection) or get the sales
   rep's explicit sign-off on relevance.
3. Use the output **AS-IS** (don't retype cases by hand — that's how `image`/`metric` get lost).
   The sales rep **confirms/changes** and pairs it with **1 local case** (content-collection).
4. `python3 scripts/fetch_case_images.py <extra.json>` — **always, before generating** — embeds
   the hero images **and the client logos** as data URIs so the HTML/PDF stays self-contained.
5. Quote-first render (factual headline when no genuine quote exists) with a link, industry pill
   and logo, in `predemo` and `live`.

---

## HTML writing rules

1. **Correct accents and ñ are mandatory** — everything in proper Spanish
2. **Professional "tú" (informal you) tone** — never "usted" (formal you)
3. **Vocabulary of value** — "investment" not "expense", "profitable" not "cheap"
4. **No prices on app cards** — only in the Investment section (pricing cards + breakdown)
5. **No "included" in headers** — "Features" not "Included features"
6. **Real personalization** — each app must mention concrete client data (number of users,
   warehouses, revenue, current system, pain points)
7. **Outcome-oriented executive summary** — pain points + how Odoo solves them, NOT a scope
   description
8. **Mandatory disclaimer** in the executive summary: the document is a pre-analysis estimate
9. **"Included" vs "Tailored to your case" tiers visually separated** in each app card —
   "Included" always shown (bullets marked **✓**), "Tailored to your case" only if there are
   extras (bullets marked **+**). Do NOT use basic/advanced-style labels
10. **Generic payment terms:** "The analysis is invoiced at the start. Implementation is
    invoiced monthly throughout the process." No month-by-month detail (the client chooses the
    package).
11. **Zero brochure adjectives** (v2 Golden Rule 10) — short sentences, concrete numbers.
    Banned: "innovative", "leading", "cutting-edge", "end-to-end", "360º solution". **Test: if
    any competitor could sign the sentence, delete it.** Talk about the client 2:1 (their
    business, not ours).
12. **Sector language** — use `sector_glossary` (see `references/sector_glossary.md`): the
    client's sector vocabulary, not Odoo jargon.
13. **Sentence case — all-caps is forbidden, with NO exceptions (v5)** — all written text
    (titles, intros, features, `understanding`, `cost_of_delay`, case results, eyebrows,
    buttons) uses only the first letter capitalized; respect proper nouns and acronyms (CIF,
    IVA, CRM, MRP, EPI). Never use caps for emphasis — and **no `text-transform: uppercase`
    anywhere in the CSS either**: the old "eyebrow micro-labels in CSS caps" doctrine is
    retired. The brand is always **"Dynapps"** (capital D) in prose — the all-lowercase form
    lives only inside the logo artwork. `brand_lint.py` (CAPS + MARCA) validates both, on the
    JSON and with `--html` on the rendered output.
14. **Typography and breathing room** — don't enlarge fonts or padding when touching templates.
    Hero ≤ `clamp(40px,6vw,72px)`, `sec-title` ≤ `clamp(23px,3vw,34px)`, sections ~64px vertical
    padding. Goal: clean and orderly, not a giant template full of gaps.
15. **Sounds human, not AI (anti-slop rule)** — the text has to read like a consultant
    explaining it to the client over coffee, not like a self-generated brochure. Hard rules:
    - **No AI crutch words:** banned: "potenciar", "impulsar", "maximizar", "revolucionar", "un
      abanico de", "una amplia gama de", "llevar al siguiente nivel" (and the English tells:
      delve, leverage, elevate, seamless, unlock, empower…). Say it with the plain word instead.
    - **Get to the point, no filler:** no filler intros ("En un mundo cada vez más…", "Hoy en
      día…", "Es importante destacar que…") and no summary closers ("En conclusión…", "En
      definitiva…", "En resumen…"). Start with what matters; stop when you're done.
    - **Vary sentence length:** mix short, punchy sentences with the occasional longer one.
      Avoid 4+ sentences in a row of the same length (the robotic "drone" that gives AI away). A
      three-word sentence is fine.
    - **Active, direct voice:** "Odoo reconciles your stock", not "stock is reconciled by the
      system". No hedging ("generally", "arguably"). Commit: take a stance.
    - **Simple reading level + natural contractions** (in Spanish: del, al) and the informal
      "tú". Clear isn't dumb: it's direct.
    - **No em-dashes (—), one language, right register (v5):** the em-dash is banned in all
      client-facing text (an AI tell and off-voice); the en-dash (–) is only valid inside
      numeric/date ranges ("S12–S14", "jul–ago") — otherwise use a comma, colon or period.
      The whole document is written in ONE language (no leaks; per-pack denylist), in the
      pack's register: **es keeps the professional "tú" untouched**, nl uses je/jij, fr uses
      vous. Urgency is framed as planning transparency, never pressure.
    - **Bar test:** if you wouldn't say it this way to a client over coffee, rewrite it.
      Validate it with `brand_lint.py` (SLOP / FLUFF / RITMO, plus the v5 mechanical checks
      DASH / MARCA / IDIOMA / REGISTRO / PRESION and warn-only EXCL / PASIVA) — and again with
      `brand_lint.py --html <rendered.html>` after generating.

---

## STEP 4 — Execution detail (moved from SKILL.md)

Summary of STEP 4 in SKILL.md: re-download spreadsheet → write `extra_data.json` →
lint+validation → generate both formats. Here's the granular detail for each sub-step.

### 4.2 — MANDATORY fields of `extra_data.json`

- `mode` — `predemo` | `live` (see the modes table above). Defaults to `live`.
- `apps_recomendadas` — list of apps pre-checked in the single list (if missing, all scoped apps).
- `understanding` — client-first opener: `sector_intro` (who they are and what we've understood
  about them), `objetivos` (in their own words), `success_measures` (how they'll measure
  success). If missing, falls back to `executive_summary_html`.
- `executive_summary_html` — oriented to **pain points + outcomes**, not scope. Link each pain
  point to the Odoo capability that solves it.
- `cost_of_delay` — 2-4 sentences on the **cost of staying the same** (qualitative, **no €**;
  the number is stated live). If omitted, it's derived from the pains.
- `anchor_dates` (`live` only) — mutual milestones `{owner: cliente|dynapps, date, milestone}`.
  The client's dates carry the same weight as ours.
- `app_overrides.<APP>.intro` and `app_overrides.<APP>.fit` — "For <Client>: …" callout, 1 line
  tying the module to the deal's specific need (recommended for EVERY app so the client sees
  themselves reflected, without bloating).
- `app_overrides.<APP>.involvement_pct` — the CLIENT team's expected involvement for that
  module, integer 20–50 (20% = 1 day a week; out-of-range values are clamped with a warning).
  **The visible unit is DAYS/week everywhere** (managers don't think in %): a 5-dot
  working-week scale (whole/half dots) on the app card (web + PDF). When the sheet's Gantt
  paints the app, the value shows amount AND timing: "2,5 días · S12–S14" (weeks from the
  app's Gantt row); without a painted row it falls back to "≈ N días a la semana", plus a price-weighted roll-up chip
  on each process header ("Tu equipo: ≈ 1 día/sem"). Priority: extra_data override > the app block's per-app row 'Dedicación cliente
  (días/sem)' in the co-signed workbook (below each TOTAL; template emits it, legacy
  workbooks simply lack it) > `extra_data.involvement_default_pct` > `deal.involvement_dias`
  ('Parámetros', days/week, days × 20 = %) > 20. Emission rule: process view → always shown
  (base case 1 day/week); flat view → only if the deal sets a value explicitly (extra_data or
  sheet; small-deal byte-parity intact). Effort ≠ money: it also renders in `predemo`.
- `app_overrides.<APP>.features` for EVERY active app. **`features` is NOT optional**: for each
  exact feature name from the spreadsheet, write 1-2 sentences explaining that feature **in the
  client's specific context** (this client's multi-company setup, the warehouses they have, the
  sector they operate in, etc.). Without this, the cards end up with generic plain text like
  "Configuración del sistema (empresa, logo, moneda, idioma)" — unacceptable in a professional
  proposal. Values accept a plain string (es) or the object form
  `{"title": "<localized bullet name>", "expl": "<explanation>"}` — **mandatory for non-es
  proposals** (the key stays the exact Spanish spreadsheet name; `title` is what renders).
- `dev_overrides.<DEV_NAME>` — specific description per development based on the intake/RFP.

The KEYS of `app_overrides.<APP>.features` are the exact feature names from the spreadsheet
(extraction one-liner further down in this doc).

### 4.3.b — Sector glossary (client's language)

Build `extra_data.sector_glossary` (a map of Odoo term → lead's sector vocabulary) and **write
all the content using it**. It's internal, not rendered. See `references/sector_glossary.md`.

### 4.3.c — Project team (hyper-personalization)

Ask the sales rep **which people** make up the team (sales rep + project manager + consultant).
With those names, run `scripts/fetch_team.py "Name:Role" ...` (authenticated Odoo environment)
→ it pulls the photo (`image_256`), title, and **email** (`work_email`) from `hr.employee`
as-is and returns the JSON for `extra_data.equipo`. **Do NOT rewrite the email domain**: if it
should be `@dynapps.es`, the employee updates it in the Odoo backend. The "Your team / People,
not tickets" section only renders if `equipo` is present.

### 4.3.d — Comparable cases (metric-first, with a link and a photo)

Numbered procedure (each step is mandatory):

1. Run `python3 scripts/match_cases.py --locale <code> "<sector>" "<pains/toma>"` and use its
   JSON output **AS-IS** — it carries `metric`, `url` AND `image` per case. Do NOT hand-write
   `casos_exito` from memory or from doc examples: that is how cases end up without a photo
   (real ANRO PR4035 failure — the card rendered bare, silently).
   ⚠️ **`--locale` must match the language the sector/pains text is written in** (the deal's
   resolved locale). Without the flag the default resolves to `en` and a Spanish sector like
   "Construcción industrializada" scores `sector=0` against every case (scoring compares the
   pack's LOCALIZED sector tokens; "construcción" never matches the English token
   "construction") → the whole output comes back `weak_match` and you would wrongly discard
   perfectly valid cases.
2. **Check relevance:** if the script printed the `weak_match` WARNING (no case matches the
   sector), replace the weak cases with a local case from `common-skills:content-collection` or
   get the sales rep's **explicit sign-off** on their relevance. Acceptance criterion: ≥ 1 case
   with a matching sector, or explicit sales-rep validation. Once resolved, **delete the
   `weak_match` key** from every entry you keep — it is an internal signal, not part of the
   `casos_exito` schema, and `generate_proposal.py` warns if it reaches `extra_data`.
3. The sales rep confirms the final selection (and pairs with 1 local case if valuable).
4. Run `python3 scripts/fetch_case_images.py <extra.json>` — **ALWAYS, before generating** — to
   embed the hero images (download + downscale + data URI; on failure it falls back to the URL).
   `generate_proposal.py` warns on stderr if a case is missing `image`/`metric` or the image is
   a non-embedded http(s) URL: fix those warnings, don't ship them.

Each case carries `metric` (headline figure) and `url` (the international/local `scope` is
**no longer rendered** — cases go by relevance, not origin). Case links **ALWAYS point to the
Dynapps case-studies website** (composed per locale from `instance.cases_base_url` + slug;
fallback `cases_library.json._base_url`) — never Odoo backend URLs in the client-facing
document. The section only renders if `casos_exito` is present.

### 4.5 — Recurring costs + future migration

If the `sale.order` has lines with `recurring_invoice=True`, their real prices are used (via
`extra_data.costes_recurrentes`). If not, the script uses `RECURRENT_DEFAULTS` (Dyn Cloud /
Maintenance / Odoo License at standard prices). If the sales rep wants negotiated prices
specific to this client (e.g. 60+ users → a higher license tier), inject them via
`extra_data.json`. See rule 16.

**The Odoo License is defined by RATE PER USER + `num_usuarios`; the engine calculates the
total.** Do NOT inject the already-multiplied total in `price` (the engine multiplies it again
by `num_usuarios` → an absurd figure; that was the PR4021 bug class: 20.088 × 60 = 1.205.280).
Per-user rates (odoo.com list price, yearly billing, Spain, verified 2026-09-28):
- **Custom (Personalizado) — the reference**: €334.80/user/year (€27.90/user/month).
- **Standard**: €178.80/user/year (€14.90/user/month), sanctioned alternative.
- Never the first-year discount (Custom €22.90 · Standard €11.90/month) nor monthly billing.
- **Light User**: €4.90/user/month (€58.80/year), employees without core apps (Sales, Purchases,
  Projects, Accounting), contracted via an Odoo Advisor. Own line, own count
  (`num_usuarios_light`), see above. Only ES is verified: odoo.com prices by geo-IP (Gotcha 12c).

Set `extra_data.num_usuarios` and the license card with `price` **per user** (or the
`amount_per_user` field to make it unambiguous). E.g.: `amount_per_user: 334.80` +
`num_usuarios: 28` → the engine displays **€9,374.40/year (€781.20/month)**.
`generate_proposal.py` warns via stderr if the resolved rate/user exceeds €500 (a symptom of
having injected the total). If the number of users is `[PREGUNTAR]`, ask the sales rep before
generating.

Additionally, in `live` the recurring block includes: **selectable future migration** (15% of
checked apps+devs / 24 = €/month, recalculates live; note "renewal of the migration cycle every
2 years"), **recurring MCP** (€50/month) and **Field Service** (+€2/user/month) if checked, and
the **partner manager** Micaela Gil Rother (migi@odoo.com). See "Recurring + future migration"
above.

### 4.5.b — Added value / AI, training, and on-site presence

- **Added value / AI block** (fixed catalog, see dedicated section): everything unchecked by
  default; conditional items (305, 288, 212) hidden if their app isn't checked. One-shot amounts
  add to the TOTAL; recurring ones go to recurring.
- **Training**: state **"2h per session"** wherever training sessions are mentioned (protects
  the team). Go-live support includes "support, migrations, and training".
- **On-site presence**: product €900/day × number of days (selector in the added-value block).
  The on-site presence surcharge is **only mentioned in Terms and Conditions**, NEVER in the
  body of the quote.

### 4.6 — Generating both formats

Generate with `scripts/generate_proposal.py [--only web|pdf|both] <ss.json> <toma.html>
<out_basename> <extra.json>`. The script builds a single `context` from the same `extra.json`
and renders it into two Dynapps templates (`templates/propuesta_web.html` rich +
`templates/propuesta_pdf.html` executive), embeds Readex Pro and the Dynapps logos as data
URIs, and (for the PDF) calls Chrome headless. **Two-stage flow:** in **STEP 4** only the HTML
is generated (`--only web` → `<base>.html`); the **PDF is NOT generated until STEP 5.2**, after
injecting into `extra.json` the selection the sales rep copied from the demo (`--only pdf` →
`<base>-pdf.html` + `<base>.pdf`, body with only what's checked). Without `--only`, both are
generated (legacy/test use). If it emits a WARNING on stderr, review it before delivering.

**Post-generation gates (MANDATORY on every rendered HTML — web in STEP 4 AND the PDF source in
STEP 5.2):** (1) `python3 scripts/brand_lint.py [--locale <code>] --html <rendered.html>` — the
rendered output must be clean, not just the JSON; (2) `python3 scripts/verify_totals.py --html
<rendered.html> --extra-data <extra.json> [--locale <code>] [--ss <ss.json> --toma <toma.html>]
[--strict]` — recomputes recurring totals from the engine, asserts rendered summary == its own
detail rows and sane per-user licence rates (the PR4021 bug class); exit 1 = do NOT deliver.
Pass `--ss/--toma` whenever available (exact-match model).
