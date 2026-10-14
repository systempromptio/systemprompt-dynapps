---
name: indaws-quotation-builder
description: >
  Edits and maintains LEGACY hours-based Odoo implementation quotations (sale.order with Studio
  fields, template id=1) in the Indaws corporate Odoo: phases, concepts, apps, Indaws Core, pain
  points, experts, success cases, server, maintenance, migration. Triggers: "adjust the hours on
  phase 1 of PR3488", "change the server on this quotation", "add a Core module to quote PR####",
  "recalculate the maintenance on an old quote". Use ONLY on an existing legacy quotation — for any
  NEW proposal use sales-skills:quotation-builder-value-based, which supersedes this one. Requires
  common-skills:odoo-pilot.
version: 2.0.2
model: opus
effort: high
---

# Quotation Builder (LEGACY — hours-based)

## Critical

1. **DEPRECATED for new quotations.** `sales-skills:quotation-builder-value-based` is the sole
   generator for new commercial proposals (closed price per app, no hours). This skill exists to
   **edit existing legacy quotations** built on template id=1 with the hours/Studio model. If the
   user asks for a new proposal, hand off to `quotation-builder-value-based` and say why. Only
   proceed with a full new build here if the user explicitly insists on the legacy hours model.
2. **Output language follows the user.** This skill and its references are English; every artefact
   written to Odoo (`x_studio_resumen`, `x_studio_concepto`, pain points, `x_studio_toma_de_datos`)
   goes out in the **client's language** — that is what the client reads in the PDF. Default when
   unstated: Spanish, since the corporate Odoo and its QWeb template are Spanish. Never mix
   languages inside one quotation.
3. **Product searches ALWAYS carry `context: {"lang": "es_ES"}`** — this is a database-language
   requirement, not an output-language choice, and it is independent of rule 2. Without it, name
   mapping silently binds the wrong products (GOTCHA 1).
4. **Two estimation models exist in this skill and they disagree.** The governing model is
   `hour_estimation.md` (functional 12h/week ∥ development 25h/week). `timeline_rules.md`
   (40/50/60h/week pace profiles) and `classification_rules.md` (fixed analysis 25/40/60h) are the
   **older model**, kept because legacy quotations were built with them. When editing a legacy
   quotation, **stay in whichever model that quotation already uses** — do not mix the two. If it
   is unclear which one produced it, ask.
5. **Never write to Odoo without an approved plan** (Step 10) and, for anything touching hours, an
   approved calendar (Step 5).

## Workflow

Load `references/workflow_detail.md` for the full step-by-step. Summary:

See references/shared/odoo-connection.md for the connection pattern.

1. **Connection** — get credentials and authenticate via `common-skills:odoo-pilot` (`auth.sh`)
2. **Positioning** — create new or modify existing. Link the opportunity (`crm.lead`).
   **Creating new → first offer `quotation-builder-value-based`** (see `## Critical` 1)
3. **Gather information** — sources: attachments, URLs, the lead's data-collection field, chatter.
   Ask about: apps, hosting, developments, analysis, Rescue Pack (only if the client already runs
   Odoo)
4. **Apps and Core** — search `product.template` by category. Priority: Standard > Core > Studio >
   Development. Load `references/app_implementation_hours.md` and
   `references/shared/paquetes_indaws_core.md`
5. **Calendar** — two parallel tracks (functional 12h/week + development 25h/week). Load
   `references/hour_estimation.md`. **[STOP] — validate with the user before continuing**
6. **Phases and concepts** — Analysis + Implementation with the standard concepts. Load
   `references/concept_structure.md`. Order: phase.sequence > concept.sequence
7. **Migration** — EUR range in `x_studio_migration` (50h base, adjusted for complexity)
8. **Experts, cases, pain points** — max 5 deduplicated experts. `blog.post` blog_id=16. Pain
   points in an empathetic register
9. **Maintenance and fields** — tiered maintenance by project total (default; see
   `references/sale_order_fields.md` §10). Server, licence, `sale_order_template_id=1`,
   `x_studio_new=True`
10. **Present the complete plan** — **[STOP] — wait for approval** before writing to Odoo
11. **Write to Odoo** — order: sale.order > phases > concepts > apps > Core > pain points >
    experts > blog.post > sale.order.line > data collection. Load
    `references/sale_order_fields.md`

## Rules

1. **Never hardcode credentials** — always ask the user
2. **Template always id 1** — `sale_order_template_id = 1`
3. **`x_studio_new` always True**
4. **Prefer standard** — Standard > Core > Studio > Development
5. **Calendar before quotation** — NEVER generate concepts without a user-validated calendar
6. **Plan before execution** — present a summarised plan and wait for approval before writing
7. **Summary in HTML** — `x_studio_resumen`: four `<p>` paragraphs (context, apps/Core, phases,
   developments). No hours, prices or velocities
8. **Concepts in HTML with bullets** — `x_studio_concepto`:
   `<p><strong>Title</strong></p><ul><li>…</li></ul>`. 4–8 project-specific bullets
9. **One concept per development** (type `Desarrollo`). A single line for all Studio changes
10. **No "print formats" concept** — standard formats live inside implementation; custom formats
    are a development concept. This applies to the presentation template too
11. **Concept order** — by phase.sequence, then concept.sequence
12. **Maintenance: tiered is the default** (250 / 350 / 460 €/month by project total). The
    scalable 320 €/month + 64 €/h formula is an **override**, only when the consultant asks for it

## Gotchas

1. **`lang: es_ES` context is mandatory on `product.template`** — without `{"lang": "es_ES"}` the
   base (English) names do not match the UI. Real case: ID 119 base = "Compras (copia)" but in
   Spanish = "Inventario". Fails silently, binding the wrong product.
2. **`product.template` vs `product.product`** — `x_studio_product_id` (apps/Core) takes
   `product.template`. Only `sale.order.line.product_id` takes `product.product`.
3. **Rescue Pack only if the client already runs Odoo** — never offer it to a new client. It also
   has a **blocking prerequisite**: access to both the source code AND the database. Confirm both
   before quoting it.
4. **Duplicate experts** — one employee can hold several records (one per skill). Deduplicate by
   `x_hr_employee_id` and pick the area most relevant to the project. Max 5.
5. **Pain points: register** — never frame problems as failures or accusations. Use "challenges"
   or "areas for improvement".
6. **The data-collection field excludes internal information** — no indicative budget, no
   consultant notes, no unresolved `[ASK]` fields, no contact details, no source references.
7. **Analysis hours: two different formulas.** Under the governing model:
   online `(apps×1.5h×2) + (devs×1h) + 9h`; on-site `(days×8h×2) + (devs×1h) + 9h`. The legacy
   model instead uses flat 25/40/60h by classification. Pick one per quotation (`## Critical` 4).
8. **Concepts need BOTH `x_name` and `x_studio_concepto`** — leaving the HTML empty renders a
   blank concept in the Odoo UI, and leaving `x_name` empty on Core lines hits a NOT NULL error.
9. **Editing a legacy quotation with existing order lines** — delete them first with
   `[2, <id>, 0]` or ask the user; appending silently double-counts the project.

## Agent Routing

**opus** — complex multi-phase workflow: user interview, estimation arithmetic, calendar
validation, and Odoo API writes across 10+ models.

## Delegates to

- **sales-skills:quotation-builder-value-based** — ALL new commercial proposals (`## Critical` 1)
- **common-skills:odoo-pilot** — all Odoo CRUD (auth, search, create, update)
- **common-skills:odoo-effort-estimator** — centralised velocity/hours/complexity reference
  (consistent with `hour_estimation.md`)

## Consumed by

None currently — this skill is the legacy endpoint. New proposals flow through
`quotation-builder-value-based`.

## References

| File | Content | When to load |
|------|---------|--------------|
| `references/workflow_detail.md` | Full question blocks (1–4), calculations, execution steps (1–10.5), product search rules | Start of workflow execution |
| `references/sale_order_fields.md` | Models, fields, types and relations of sale.order + the Studio models, write sequence, standard prices | Step 11 (write to Odoo) |
| `references/app_implementation_hours.md` | Odoo apps catalogue with implementation hours per complexity level | Step 4 (apps selection) |
| `references/shared/paquetes_indaws_core.md` | Indaws Core module catalogue with prices | Step 4 (Core selection) |
| `references/concept_structure.md` | Standard concept structure per phase with real examples | Step 6 (phases/concepts) |
| `references/hour_estimation.md` | **Governing** estimation model: weekly velocities per app | Step 5 (calendar) |
| `references/classification_rules.md` | **Legacy model** — classify a project Simple/Medium/Complex (sets flat analysis hours) | Only when editing a quotation built on the legacy model |
| `references/meeting_migration_rules.md` | Formulas for meetings, per-app training and migration hours | Step 5 / Step 7 (migration) |
| `references/timeline_rules.md` | **Legacy model** — pace profiles used by the meeting/migration formulas | Only with the legacy model |
| `templates/quotation_presentation.md` | Presentation format shown to the user before writing to Odoo | Step 10 |
