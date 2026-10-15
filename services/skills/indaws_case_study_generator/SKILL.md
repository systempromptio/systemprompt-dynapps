---
name: indaws-case-study-generator
version: 1.0.2
effort: high
model: sonnet
description: >-
  Generates a Hero-tier Indaws customer case study as a .docx with Word comments distinguishing
  Odoo-filled data from interview gaps. Use whenever the user asks for a case study, caso de éxito,
  customer story, or client reference document.
  Triggers: "crea el caso de éxito de X", "genera el caso de cliente", "customer case study",
  "historia de cliente", "documento de referencia", "prepara el caso", "case study para X",
  "documenta la implementación", "client reference", "caso de X".
---

# Case Study Generator

Generates a structured Hero-tier case study DOCX for an Indaws customer. Pulls all available data from Odoo (partner, sales, projects, tasks, tickets, invoices, installed modules) and produces a Word document with annotated Word comments:
- **FILLED from Odoo:** data confirmed directly from records (with source reference)
- **GAP:** content requiring a customer interview (with a specific suggested question)

## Prerequisites

Odoo credentials configured via `onboarding` skill or environment variables. See `references/shared/odoo-connection.md`.

Odoo corporativo: `https://portal.dynapps.es` (v19 Enterprise, JSON2, Bearer). No uses `www.indaws.es` para la API: redirige y la autenticación falla sin avisar.

## Workflow

1. **Resolve the customer** — Accept name or partner ID. Search by name if needed; confirm if multiple matches. See `references/odoo-queries.md` for the resolution query. Always anchor on `commercial_partner_id`.
2. **Fetch all Odoo data** — Run queries for partner+contacts, sale orders+lines, projects+tasks, posted invoices, helpdesk tickets, and Indaws Core modules. Full query set: `references/odoo-queries.md`.
3. **Analyse the data** — Extract relationship timeline, modules implemented, custom developments, scale figures, and team. Full patterns: `references/data-analysis.md`.
4. **Build the DOCX** — Run `scripts/build_docx.py` or build inline with python-docx. Comment injection pattern: `references/docx_pattern.md`. Every placeholder gets one Word comment: `FILLED from Odoo: <source>` or `GAP — <what>: "<concrete question>"`.
5. **Present the result** — Save to `~/Desktop/case-<customer-slug>-hero-<YYYY-MM-DD>.docx`. Report: N FILLED / M GAP counts, top 3 priority gaps, whether database.env had module data, suggested interview questions in priority order.

## Document sections

| # | Section | Key content | Main source |
|---|---------|-------------|-------------|
| 0 | Verification banner | Customer, tier, data sources, counts | Auto-generated |
| 1 | Identifier band | Sector, footprint, scale, partner | Odoo |
| 2 | How it started | Pre-Odoo situation, company history | Interview |
| 3 | The challenge | Pain points, specific vignettes | Interview |
| 4 | The decision | Why Odoo, why Indaws, alternatives considered | Interview |
| 5 | What we built | Modules, custom dev, integrations, AI | Odoo tasks/tickets |
| 6 | The rollout | Timeline, team, friction moments | Odoo + interview |
| 7 | The outcome | Metrics, before/after numbers | Interview + Odoo scale |
| 8 | In their words | Direct quotes from customer | Interview |
| 9 | What comes next | Future plans | Interview |
| 10 | Related cases | 3 slug links | Marketing team |

## GOTCHAS

- **odoo.com SaaS customers** will not appear in `database.env` — normal. Derive modules from tasks/tickets.
- **Always use commercial_partner_id** as filter, not partner_id — otherwise you miss most records for multi-contact companies.
- **Cancelled sale orders** still reveal what was proposed — include in timeline with `(cancelled)` note.
- **First helpdesk ticket ≈ go-live** — better proxy than first invoice (maintenance subscriptions often predate rollout).
- **Migration task descriptions** contain data comparison tables with record counts — parse them for scale figures.
- **Word comments need XML injection** — use `references/docx_pattern.md`, not python-docx's nonexistent comment API.
- **GAP comments must end with a concrete question in quotes** — never a vague instruction.

## NEVER

- Never filter by `partner_id` alone — always `commercial_partner_id` as the anchor
- Never emit a GAP comment without a specific interview question in quotes
- Never skip Step 3 analysis before building the DOCX — comment quality depends on it
- Never publish or send the DOCX — save to Desktop only, user reviews first

## ALWAYS

- Always confirm customer identity before fetching data (multiple matches = ask user)
- Always cite the source in every FILLED comment (model + record ID)
- Always report gap count and top 3 priority gaps at the end
- Always use `references/shared/odoo-connection.md` for auth pattern

## Delegates to

| Skill | When |
|-------|------|
| `common-skills:onboarding` | If Odoo credentials are not configured |
| `common-skills:odoo-pilot` | For auth pattern and script reference |

## Consumed by

| Skill | Why |
|-------|-----|
| `campaign-planning` (ai-claude-marketing) | Pulls finished case studies as proof materials for campaign briefs |
| `content-creation` (ai-claude-marketing) | Uses case study format and section templates for long-form content |

## Agent Routing

- Single customer, name or ID given → run full workflow (Steps 1-5)
- Multiple customer names listed → run one at a time, confirm each before fetching data
- User only wants the interview question list → run Steps 1-3 only, skip DOCX build
- User asks to update an existing case study → re-run Steps 2-3, rebuild DOCX with same output path

## REFERENCES

| File | Contents |
|------|----------|
| `references/odoo-queries.md` | All Step 2 queries + customer resolution query |
| `references/data-analysis.md` | Step 3 analysis patterns: timeline, modules, custom dev, scale, team |
| `references/docx_pattern.md` | Python code for Word comment injection via zip/XML |
| `references/template_sections.md` | Full prose templates for all 10 sections with placeholder text |
| `references/module_inference.md` | Signal→module mapping table |
| `scripts/build_docx.py` | Standalone script: reads collected data, builds annotated DOCX |
| `references/shared/odoo-connection.md` | Odoo auth and connection setup |
