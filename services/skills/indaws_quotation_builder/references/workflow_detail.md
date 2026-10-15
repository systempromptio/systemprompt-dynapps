# Workflow Detail — Quotation Builder (LEGACY)

Detailed steps, question blocks, calculations and critical rules for quotation generation.

> **Language.** This reference is English. Everything written into Odoo goes out in the client's
> language (default Spanish). The `lang: es_ES` product-search context is a separate,
> non-negotiable database requirement. See SKILL.md `## Critical`.
>
> **Deprecation.** For a NEW proposal, hand off to `sales-skills:quotation-builder-value-based`.
> This flow is for editing existing legacy hours-based quotations.

---

## User Question Flow

### BLOCK 1: Connection

Ask for the credentials to reach the corporate Odoo:

| Item | Variable | Default | Required |
|------|----------|---------|----------|
| URL | `ODOO_URL` | `https://www.indaws.es` | No (confirm) |
| Database | `ODOO_DB` | `master` | No (confirm) |
| Version | — | `19.0` (JSON2) | No (confirm) |
| User | `ODOO_USER` | — | **Yes** |
| API Key | `ODOO_KEY` | — | **Yes** |

Authenticate via `common-skills:odoo-pilot`:

```bash
export ODOO_URL="..." ODOO_DB="..." ODOO_USER="..." ODOO_KEY="..."
eval $(auth.sh)
```

### BLOCK 2: Positioning

1. **Create or update?** — a new quotation or a modification of an existing one.
2. **If creating new** → offer `quotation-builder-value-based` first and say why (SKILL.md
   `## Critical` 1). Only continue here if the user explicitly wants the legacy hours model.
3. **If updating** → ask for the quotation number/ID (e.g. PR3488). Read the whole quotation, and
   identify which estimation model it was built on (SKILL.md `## Critical` 4).
4. **Opportunity** (new only) → name or ID of the `crm.lead`. On duplicates, show a list with name
   + ID to choose from. The `sale.order` links through `opportunity_id`.

### BLOCK 3: Project (new quotations only)

1. **Information sources** — ask where the project information should come from:
   - Attachments
   - Websites / URLs
   - The opportunity's data-collection field (`crm.lead.xx_data_collection`, HTML)
   - The opportunity description (`crm.lead.description`)
   - The opportunity's full chatter (`mail.message`)
   - Several sources combined

2. **General information** — what the user supplies: client type, expectations, price range,
   phases, hours per phase, pain points. If they do not supply it, look for it in the sources.

3. **Does the client already run Odoo?** — Yes/No. Determines whether the Rescue Pack applies.

4. **Rescue Pack** — only offer it if the client **already runs Odoo**.

   > **BLOCKING prerequisite:** it requires access to **the source code** (via git, zip, shared
   > folder or any other route) **AND to the database** (dump/backup or direct access). Without
   > **both** it cannot be delivered — the Rescue Pack *is* the code audit (technical debt,
   > security, performance, conventions) plus the X-ray of the real database. Confirm
   > unambiguously that the client will provide both **before** quoting it.

5. **Analysis** — three options: no analysis, online analysis, on-site analysis.

6. **On-site days** — only if on-site analysis. Ask how many days.

7. **Odoo applications** — consult `app_implementation_hours.md`. Show a numbered list for quick
   selection. Complement it with whatever the information sources yielded.

8. **Hosting** — options: Indaws Cloud (default), Odoo Online, Odoo.sh, On-Premise. Confirm.

9. **Developments** — ask whether custom developments are needed. Also derive them from the
   information sources.

### BLOCK 4: Modification (existing quotations only)

- **What should change?** — suggest phrasings such as:
  - "Adjust the hours on phase 1"
  - "Raise the total to €50,000"
  - "Add an analysis phase between phases 2 and 3"
  - "Change the server to Odoo.sh"
  - "Add the min/max Core module"

---

## Automatic Calculations

### Analysis Hours (governing model)

**Online:**
```
(n_apps × 1.5h × 2) + (1h × n_developments) + 5h_preparation + 4h_presentation
```

**On-site:**
```
(n_days × 8h × 2) + (1h × n_developments) + 5h_preparation + 4h_presentation
```

> The **legacy model** instead uses flat analysis hours by classification (Simple 25h / Medium 40h
> / Complex 60h — see `classification_rules.md`). Use one model per quotation, never both.

### Implementation Hour Estimation (two tracks)

See **`hour_estimation.md`** for the full criteria. Algorithm in brief:

1. **Assess each app** against the complexity criteria → assign weeks per app
2. **Functional track**: sum the weeks (sequential) × 12h/week
3. **Development track**: total development hours / 25h/week
4. **Project duration** = max(functional weeks, development weeks)
5. **Project management** = 1h/week × project duration
6. **Functional split**: 70% implementation / 30% meetings and training
7. **Present the calendar to the user** with both tracks in parallel

**MANDATORY VALIDATION**: the user must approve the calendar and the hours before the quotation
continues.

### Future Migration

Calculate a EUR range considering:

- Base: 50h (standard Indaws Core migration → a horizontal development, it does not accrue hours
  per module)
- Adjust for project complexity: bigger projects mean more hours
- Typical range: €2,500–5,000 for medium projects, €5,000–10,000 for large ones
- Write it as text in `x_studio_migration` (e.g. "3750-5000")

### Concepts per Phase

See `concept_structure.md` for the standard structure. Implementation concepts:

| # | Concept | Hours | Calculation |
|---|---------|-------|-------------|
| 1 | Technical configuration | 10h | Fixed (1 week) |
| 2 | Internal project management | Variable | 1h/week × project duration |
| 3 | Data migration | Variable | Included in the 70% implementation share |
| 4 | Module implementation | Variable | 70% of the distributable functional hours |
| 5 | Meetings and training | Variable | 30% of the distributable functional hours |
| 6 | Go-live support | 24–48h | 2–4 weeks depending on complexity |
| 7+ | Development: [name] | Own hours | One concept per development, type "Desarrollo" |

**Note**: "Print formats" is NOT a separate concept. Standard formats sit inside implementation;
custom formats are developments.

`concept_price = hours × phase_hourly_rate` (Analysis: €80/h, Implementation: €75/h)

### Maintenance

- **Default — tiered by project total**: <€12,000 → €250/month; €12,000–30,000 → €350/month;
  >€30,000 → €460/month
- **Override — scalable**, only if the consultant asks for it: base €320/month with 5h included,
  €64/h beyond that (`320 + extra_hours × 64`)
- Covers: functional questions on the implemented apps, bug fixing, development warranty
- Does not cover: training, new developments, Indaws Core products, Studio changes, apps out of
  scope

### Sale Order Lines

Create `sale.order.line` only in these cases:

- **With Rescue Pack**: a line with product ID 330, quantity 1, price from the product record
  (look it up with `search_records.sh`)
- **Without Rescue Pack but with analysis**: a line with the analysis product, quantity = analysis
  phase hours, price = the analysis phase hourly rate (€80/h)

---

## Execution Process

> **RULE**: before writing anything to Odoo, propose a summarised, ordered plan to the user.

### Step 1 — Gather and Understand

- Review all documentation and everything the user supplied
- Ask the questions that are genuinely needed, and only those
- Consult the information sources named
- Build the project summary for `x_studio_resumen`: HTML with `<p>` tags, an executive summary of
  four paragraphs, in the client's language:
  1. **Client context**: business, sector, number of users, current system, company structure
  2. **Apps and modules**: the Odoo applications selected and the Indaws Core modules to be
     installed, each with a brief note on what it does
  3. **Project phases**: describe the phases (Analysis and Implementation) and what happens in
     each. **NEVER mention hours, prices, weekly velocities (12h/week) or numeric totals.**
  4. **Custom developments**: list the custom developments with a brief description of each
     (integration with X, control of Y, system for Z…). No hours, no prices.

  Professional, client-facing tone, without excessive technical jargon.

### Step 2 — Apps and Indaws Core Products

- Determine the project's Odoo apps
- **Find the app products in Odoo**: `product.template` with
  `[["categ_id.name","ilike","Plantilla Presupuestos"]]` and **`context: {"lang": "es_ES"}`**. Map
  each selected app to the right `product.template` by its Spanish name.
- Consult `references/shared/paquetes_indaws_core.md`. Look for Core modules that fit the
  needs.
- **Find the Core products in Odoo**: `product.template` with `[["categ_id.name","ilike","CORE"]]`
  and **`context: {"lang": "es_ES"}`**. Take the real catalogue IDs and prices.
- **Priority**: Standard > Indaws Core > Odoo Studio > custom development.

### Step 3 — Implementation Calendar

See `hour_estimation.md` for the per-app duration criteria.

1. **List the apps with their complexity**: for each app, weigh the complexity signals and assign
   weeks
2. **Functional track**: sum the sequential weeks × 12h/week
3. **Development track**: total development hours / 25h/week
4. **Determine duration**: max(functional, development)
5. **Project management**: 1h/week × total duration
6. **Go-live support**: 2–4 weeks depending on complexity
7. **Build the calendar**: a week-by-week table with both tracks in parallel

### Step 3.5 — Calendar VALIDATION (MANDATORY) [STOP]

**STOP** and present to the user (in their language):

```
PROPOSED IMPLEMENTATION CALENDAR

FUNCTIONAL TRACK (12h/week):
| Week   | Activity                 | Hours |
|--------|--------------------------|-------|
| W1     | Initial configuration    | 10h   |
| W2-W3  | [App 1]                  | 24h   |
| ...    | ...                      | ...   |
| WN     | Go-live support          | 12h   |

DEVELOPMENT TRACK (25h/week):
| Week   | Activity                 | Hours |
|--------|--------------------------|-------|
| W1-WX  | [Development 1, 2…]      | XXXh  |

SUMMARY:
- Total duration: X weeks
- Functional hours: XXXh (70% impl. + 30% meetings)
- Development hours: XXXh
- Project management: Xh
- TOTAL: XXXh

Validate, adjust, or parallelise apps?
```

Wait for the user:

- **Validate** → continue to Step 4
- **Adjust** → change weeks/apps and present again
- **Parallelise** → shorten by overlapping apps and recalculate

### Step 4 — Phases and Concepts

With the calendar approved:

- Rescue Pack present → Phase 0 (1 week)
- Always at least one independent analysis phase
- Create the implementation phase(s) with the standard concepts
- On large projects, consider intermediate analyses between implementation phases

See `concept_structure.md`:

- Analysis: one main concept (its description names the apps analysed)
- Implementation: the standard concepts (configuration, project management, migration,
  implementation, meetings, support)
- Developments: type = `Desarrollo`. One concept per development.
- Studio changes: a single line grouping all of them

**MANDATORY concept ORDER**: concepts sort first by phase (`x_studio_fase_id`) following the
phases' `x_studio_sequence`, then within each phase by the concept's `x_studio_sequence`.

### Step 5 — Migration

Calculate the future-migration range (field `x_studio_migration`).

### Step 6 — Experts, Success Cases and Pain Points

- **Experts**: find records in `x_hr_employee_line_1e36b` and link them to the quotation via M2M.
  - The same employee (`x_hr_employee_id`) may hold several records (one per skill/area).
    **Deduplicate by `x_hr_employee_id`**: keep one record per employee, choosing the one whose
    `x_studio_product_id` (area/role) is most relevant to the project.
  - **Maximum 5 employees** on the expert team.
- **Success cases**: find `blog.post` records with `blog_id = 16` by sector or similarity.
- **Pain points**: identify the client's pain points and propose solutions (standard/Core).
  - **Empathetic, professional register**: describe problems as challenges or areas for
    improvement, never as failures, deficiencies, or accusations against the client. Avoid
    alarmist or dismissive wording.
  - Bad: "Your stock management is chaotic and constantly loses money"
  - Good: "Current stock management relies on manual steps that make traceability difficult"

### Step 7 — Maintenance and Other Fields

- Calculate maintenance (tiered by default; scalable formula only on request)
- Server: per the hosting chosen. Indaws Cloud = €1,080/year
- Licence: €27.90/month (reference, Custom yearly, list price, Spain)
- `sale_order_template_id`: **always ID 1**
- `x_studio_new`: **always True**

### Step 8 — Sale Order Lines

- Rescue Pack → `sale.order.line` with product ID 330
- Analysis without Rescue Pack → `sale.order.line` with the analysis hours and price

### Step 9 — Present the Complete Plan [STOP]

**BEFORE writing to Odoo**, present a summary — use `templates/quotation_presentation.md`:

- The approved implementation calendar (Step 3.5)
- Phases and concepts with hours and prices
- Selected apps
- Proposed Indaws Core modules
- Identified pain points
- Quotation total
- Maintenance, server, licence, migration

Wait for the user's approval or adjustments.

### Step 10 — Write to Odoo

With the plan approved, execute in this order:

1. Create/update `sale.order` with the scalar fields
2. Create phases (`x_sale_order_line_864da`) ordered by `x_studio_sequence`
3. Create concepts (`x_sale_order_line_36b23`) linked to phases — iterating phases by their
   `x_studio_sequence` and, within each phase, concepts by theirs
4. Create apps/services (`x_sale_order_services`)
5. Create Indaws Core lines (`x_sale_order_line_04801`)
6. Create pain points (`x_sale_order_line_3c439`)
7. Create/link experts (`x_hr_employee_line_1e36b`)
8. Link success cases (`blog.post` M2M)
9. Create `sale.order.line` where applicable — **delete pre-existing lines first** with
   `[2, <id>, 0]`, or ask the user; appending double-counts the project
10. Write the data collection into `x_studio_toma_de_datos` (Step 10.5)

### Step 10.5 — Data Collection on the Sale Order

Write the data collection as structured HTML into the `sale.order` field
`x_studio_toma_de_datos`. This is NOT a dump of the internal information but a **rewritten,
structured report** holding what is relevant to the client, in the client's language.

**Content** (rewritten from the project sources):

1. **General information**: company data, sector, users, locations
2. **Project objectives**: motivation, pain points, phased approach
3. **Functional scope**: detail per module/app with key decisions and requirements
4. **Customisations and developments**: integrations, developments identified with an hour estimate
5. **Data migration**: data to migrate, format, volume, source system

**EXCLUDE** (internal information):

- Commercial information (indicative budget, decision process, evaluation of other ERPs)
- The consultant's internal notes
- Unresolved `[ASK]` fields
- References to sources (chatter, attachments, web search)
- Detailed contact data (phone, email)

**Format**: clean HTML with no inline styles (Odoo's QWeb template renders the PDF). Use semantic
tags: `<h2>`, `<h3>`, `<table>`, `<ul>/<li>`, `<p>`, `<strong>`.

```bash
update_record.sh sale.order '[<sale_order_id>]' '{"x_studio_toma_de_datos": "<html_content>"}'
```

See `sale_order_fields.md` for the technical model and field names.

---

## Product Search (CRITICAL)

**ALWAYS** pass `context: {"lang": "es_ES"}` on every `product.template` search. The base
(internal English) names do NOT match the Spanish names shown in the UI. Without the language
context, name mapping fails silently and binds the wrong products.

**Real example of the failure**:

- `product.template` ID 119: base name = "Compras (copia)" → in Spanish = "Inventario"
- `product.template` ID 120: base name = "Inventario (copia)" → in Spanish = "Gestión de proyectos"

Searching without the Spanish context, "Inventario" matches ID 120, which the UI shows as
"Gestión de proyectos".

### Searches needed

| What | Domain | For |
|------|--------|-----|
| Apps | `[["categ_id.name","ilike","Plantilla Presupuestos"]]` | `x_sale_order_services.x_studio_product_id` |
| Core | `[["categ_id.name","ilike","CORE"]]` | `x_sale_order_line_04801.x_studio_product_id` |
| Analysis/Implementation | `[["name","ilike","Análisis de proyecto"]]` | `sale.order.line.product_id` (needs `product.product`) |
| Rescue Pack | `[["id","=",330]]` on `product.product` | `sale.order.line.product_id` |

### Search rules

1. **Always `product.template`** for `x_studio_product_id` fields (apps and Core). NEVER
   `product.product` — the IDs do not line up.
2. **Always `product.product`** for `sale.order.line.product_id`. To get it from a template:
   `[["product_tmpl_id","=",<template_id>]]`.
3. **Always with `context: {"lang": "es_ES"}`** on EVERY product search.

### A correct search

```bash
# With odoo-pilot's search_records.sh (context as the 4th JSON argument):
search_records.sh product.template \
  '[["categ_id.name","ilike","Plantilla Presupuestos"]]' \
  '["id","name","list_price"]' 100 \
  '{"context": {"lang": "es_ES"}}'

# With curl directly (JSON2 API):
curl -s -X POST "${ODOO_URL}/json/2/product.template/search_read" \
  -H "Content-Type: application/json" \
  -H "Authorization: bearer ${ODOO_KEY}" \
  -H "X-Odoo-Database: ${ODOO_DB}" \
  -d '{
    "domain": [["categ_id.name","ilike","Plantilla Presupuestos"]],
    "fields": ["id","name","list_price"],
    "limit": 100,
    "context": {"lang": "es_ES"}
  }'
```
