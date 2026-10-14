---
name: indaws-lead-prospector
version: 1.0.2
effort: medium
model: sonnet
description: >-
  Finds, enriches, and qualifies leads matching the Indaws ICP (Spanish SMBs needing Odoo ERP)
  via Clay MCP tools, then creates them in Odoo CRM. Use when the user needs new leads, contact
  enrichment, prospect scoring, or a CRM import list.
  Triggers: "busca leads para X sector", "enriquece este contacto", "encuentra empresas que usen
  SAP en Valencia", "score these prospects", "lead list for CRM", "prospección outbound",
  "encuentra potenciales clientes", "qualify these companies", "lead research", "CRM import".
---

# Lead Prospector — Indaws

Generate, enrich, and qualify leads for Indaws using Clay MCP tools.

## ICP Quick Reference

| Dimension | Default criteria |
|-----------|-----------------|
| Company size | 10–250 employees |
| Revenue | 1M–50M EUR annual |
| Industries | Manufacturing, Distribution, Professional Services, Retail, F&B, Construction, Healthcare |
| Geography | Primary: Comunidad Valenciana. Secondary: All Spain. |
| Technology | No ERP, legacy ERP (SAP B1, Dynamics NAV, Sage, A3), or basic tools |
| Decision makers | CEO, Director General, CFO, COO, CTO, Director de IT, Director de RRHH |
| Disqualifiers | Already using Odoo, <5 employees, public sector |

Full ICP: `references/icp-definition.md`

---

## Workflow

1. **Parse user input for ICP filters** — Extract industry, location (default: Spain/Valencia), company size (default: 10–250), titles (default: C-level + Director roles), and any custom criteria. If a specific domain is given, switch to company-first flow (see `references/prospecting-pipeline.md`).
2. **Search contacts via Clay MCP** — Call `find-and-enrich-contacts-at-company` with `companyIdentifier`, `contactFilters` (job_title_keywords, locations, exclusions). Save the returned `taskId`.
3. **Enrich companies** — Call `add-company-data-points` with the same `taskId`. Priority data points: Tech Stack, Annual Revenue, Headcount Growth, Recent News, Open Jobs.
4. **Enrich contacts** — Call `add-contact-data-points` with the same `taskId`. Priority: Email, Summarize Work History, Find Thought Leadership.
5. **Poll for results** — Call `get-existing-search` every ~5 seconds, up to 5 polls. Check `state` field: `in-progress` → wait; `completed` → proceed.
6. **Apply Quality Gate scoring** — Score each lead per `references/quality-gate.md` (base 50, PASS ≥60, WARN ≥35, FAIL <35). Record score impacts.
7. **Present ranked lead table** — Output markdown table sorted by quality score descending. Include PASS/WARN/FAIL summary. See `references/output-formats.md` for exact format.
8. **Optional: import to Odoo CRM** — Confirm with user which leads to import. Delegate to odoo-pilot for `crm.lead` creation. Field mapping and process in `references/output-formats.md`.

---

## GOTCHAS

- **Clay tasks are async — must poll.** `find-and-enrich-contacts-at-company` returns a `taskId` immediately; data is NOT in the response. Always call `get-existing-search` and check `state: "completed"` before reading results.
- **Each enrichment step generates its own taskId.** Steps 2, 3, and 4 each return a different `taskId`. Track them separately — do not reuse the Step 2 `taskId` for company or contact enrichment calls that return new task IDs.
- **Spanish title variations are non-optional.** "CEO" and "Director General" are both valid for the same role. Always include the full alias list from `references/prospecting-pipeline.md` Step 4 when building `job_title_keywords`. Filtering on "CEO" alone will miss 60–70% of Spanish decision makers.
- **Email enrichment takes significantly longer than company data.** Tech Stack and Revenue usually complete in 1–2 polls; Email and Work History can take 4–5 polls. Do not abort early — wait the full 5 polls before reporting missing emails.
- **WARN leads (score 35–59) are worth pursuing.** They fail a non-critical check (e.g., revenue unverified, ERP unknown). Do not auto-discard WARN leads — surface them to the user with the specific reason for the lower score so they can decide.
- **Duplicate check before Odoo import is mandatory.** Search `crm.lead` by domain and `email_from` before creating. Clay may return the same contact across multiple runs.
- **`find-and-enrich-company` is a separate tool** from `find-and-enrich-contacts-at-company`. Use the former for company-first deep-dive (single domain research); use the latter for bulk contact search with filters.

---

## NEVER

- NEVER create Odoo leads without first confirming with the user which leads (PASS, WARN, or specific) to import.
- NEVER skip the duplicate check in `crm.lead` before creating new records.
- NEVER use `job_title_keywords: ["CEO"]` alone for Spanish prospects — always include full Spanish title aliases.
- NEVER report missing emails after only 1–2 polls; wait the full polling cycle (up to 5 polls, ~25 seconds).
- NEVER write ad-hoc Python/curl/xmlrpc code to connect to Odoo — always delegate to odoo-pilot.

## ALWAYS

- ALWAYS save the `taskId` from Step 2 before calling enrichment steps.
- ALWAYS apply Quality Gate scoring to every lead before presenting the table.
- ALWAYS include score breakdown (which checks passed/failed) in the lead output so the user understands the rating.
- ALWAYS present WARN leads separately from PASS, with the specific reason for the lower score.
- ALWAYS apply dynapps-brand guidelines to any outreach messaging generated from lead data.

---

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common-skills | Creating/updating `crm.lead` records in Odoo CRM |
| dynapps-brand | common-skills | Reviewing outreach message copy for brand compliance |

## Consumed by

| Skill | Suite | How it uses lead-prospector output |
|-------|-------|-----------------------------------|
| campaign-planning | ai-claude-marketing | Receives ranked lead lists as target audience input |
| performance-analytics | ai-claude-marketing | Tracks pipeline metrics from imported CRM leads |

---

## REFERENCES

| File | Purpose |
|------|---------|
| `references/icp-definition.md` | Full Ideal Customer Profile with scoring criteria |
| `references/clay-mcp-guide.md` | Clay MCP tool reference and authentication |
| `references/quality-gate.md` | Complete scoring system (checks, weights, thresholds) |
| `references/prospecting-pipeline.md` | Clay MCP step-by-step pipeline (Steps 1–8) and company-first flow |
| `references/output-formats.md` | Ranked lead table format, enriched profile format, Odoo CRM field mapping |
| `references/shared/odoo-connection.md` | Odoo connection setup for CRM import |
