---
name: indaws-crm-report
description: >
  Generate the weekly INDAWS CRM report from Odoo data. Fetches opportunities,
  calculates 21+ KPIs, produces a brand-compliant HTML report, publishes to the
  Odoo website, and delivers personalized emails to each salesperson. Use when
  the user asks to generate the CRM report, check pipeline health, or review
  weekly commercial metrics.
version: 1.1.1
model: opus
effort: high
---

# INDAWS CRM Weekly Report

Weekly commercial control report: XML-RPC to Odoo, 21+ KPIs, brand-compliant HTML, Odoo website publishing, personalized emails per salesperson.

## Workflow

> Setup, env vars, config and architecture: `references/setup_guide.md`

See references/shared/odoo-connection.md for connection pattern.

1. **Test connection**: `python scripts/main.py --test`
2. **Generate local**: `python scripts/main.py --output report.html`
3. **Publish to Odoo**: `python scripts/main.py --odoo` (website + activities)
4. **Send team email**: `python scripts/main.py --email`
5. **Send personalized**: `python scripts/main.py --email-salespeople`
6. **Full pipeline**: `python scripts/main.py --odoo --email --email-salespeople`

## Rules

- **ALWAYS** test connection (`--test`) before generating
- **NEVER** skip brand compliance — Montserrat font, zero emojis, Indaws colors from brand-tokens.json
- **ALWAYS** generate locally first before publishing to Odoo or sending emails
- KPI calculations are in data_processor.py — NEVER calculate manually

## Gotchas

1. XML-RPC uses `ODOO_USERNAME`/`ODOO_PASSWORD`, not JSON-RPC `ODOO_USER`/`ODOO_KEY`
2. **Demos are `mail.activity` records, not fields.** Reading them REQUIRES the
   `active_test: False` context — a done demo is archived, so without it the query returns
   almost nothing and demo KPIs silently collapse to zero. The "Demo" activity type is
   resolved BY NAME (38 in production, but it differs per instance — never hardcode it).
   The real date is `date_done`, not `date_deadline`
3. **KPI 12 counts demos, not leads**, and attributes each to the activity's `user_id`
   (whoever ran it).
4. Stage classification is keyword-based and must match in **both** en_US and es_ES — the
   names come back in the connected user's language. There is no "lost" stage: a lost lead
   is marked on the lead itself. Analysis is an ADVANCED stage, never an early one
5. `SALESPEOPLE_OBJECTIVES` must be updated monthly or forecasts will be wrong
6. Email HTML must be inline-styled — Gmail strips `<style>` blocks
7. Odoo website publishing creates activities for all team members — test in staging first
8. `utm.source` holds 51 canonical values; sources named after salespeople are empty by design.

## Agent Routing

| Phase | Model | Why |
|-------|-------|-----|
| Connection + generation | opus | Complex KPI calculations, brand compliance |
| Publishing + emails | opus | Multi-step Odoo operations |

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common | CRM data extraction and website publishing |
| dynapps-brand | common | Brand tokens for HTML styling |

## Consumed by
| Skill | Suite | When |
|-------|-------|------|
| crm-send-emails | sales | Email delivery of generated report |
| crm-debug | sales | Troubleshooting report issues |

## References

| File | Content |
|------|---------|
| [references/kpi-definitions.md](references/kpi-definitions.md) | 21+ KPI definitions and calculation formulas |
| [references/setup_guide.md](references/setup_guide.md) | Environment variables, config, brand compliance, module architecture |
