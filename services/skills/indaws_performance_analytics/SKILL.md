---
name: indaws-performance-analytics
version: 1.0.3
effort: medium
model: sonnet
description: >
  [SUSTITUIDA — no usar] Doctrina obsoleta que contradice Odoo Knowledge (space 12).
  Voz y vocabulario: `es-lint` y `dynapps-brand`; calendario y contenido: `social-calendar`.
---

# Performance Analytics — Indaws

Track, analyze, report, and optimize marketing performance across all Indaws channels.
All reports follow Indaws brand voice: professional "tu", zero emojis, data-driven insights.

## Workflow

1. Identify report type and time period (weekly digest, monthly, quarterly, campaign, event)
2. Pull data from each channel source (LinkedIn CSV export, Odoo via odoo-pilot, SEO via seo-manager, Google Analytics)
3. Apply benchmarks from `references/benchmarks.md` — adjust for Spanish B2B seasonality
4. Build report from template in `references/report-templates.md`, formatting per dynapps-brand
5. Add trend analysis — flag deviations >20% from expected trend, classify root cause
6. Prioritize optimization recommendations using Impact vs Effort matrix from `references/optimization-framework.md`

## GOTCHAS

- **LinkedIn Analytics export has a 7-day minimum delay** — data exported today reflects up to 7 days ago; never report "this week" LinkedIn data as real-time
- **Odoo email stats exclude bounced emails before counting opens** — delivered = sent minus bounces; open rate denominator is delivered, not sent; mixing these inflates apparent open rates
- **Google Analytics sessions are not the same as users** — one user can generate multiple sessions; use "users" for reach metrics and "sessions" for volume metrics
- **August and December dips are normal for Spanish B2B** — never flag these as problems without YoY comparison; seasonality calendar is in `references/benchmarks.md`
- **Attributing pipeline to last touch undervalues brand content** — Odoo uses first-touch attribution by default; content that generates awareness but not the final conversion gets zero credit; always note this limitation in pipeline reports
- **Email open rates are unreliable since iOS 15 (2021)** — Apple Mail Privacy Protection inflates opens; use click rate and click-to-open rate as primary email engagement metrics
- **LinkedIn follower counts lag by 24-48 hours** — daily follower snapshots are not real-time; use weekly cadence for follower growth tracking

## NEVER

- NEVER report LinkedIn "this week" data as current — always note the 7-day export delay
- NEVER use sessions and users interchangeably in Google Analytics reports
- NEVER attribute August/December performance drops to campaign failure without checking YoY
- NEVER write Odoo API calls directly — always use odoo-pilot scripts (see `references/shared/odoo-connection.md`)
- NEVER present open rate as the primary email success metric — lead with click rate
- NEVER report pipeline attribution without noting Odoo uses first-touch model

## ALWAYS

- ALWAYS apply Spanish B2B seasonality context when interpreting month-over-month changes
- ALWAYS pull pipeline data via odoo-pilot with `context: {lang: es_ES}`
- ALWAYS use YoY comparison for any period that overlaps August or December
- ALWAYS note attribution model limitations in any pipeline or revenue report
- ALWAYS format reports per dynapps-brand guidelines (professional "tu", zero emojis, zero hashtags)
- ALWAYS include a prioritized recommendations section in monthly and quarterly reports

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| odoo-pilot | common-skills | Pull CRM leads, email campaign stats, event data, pipeline metrics from Odoo |
| seo-manager | ai-claude-marketing | Pull organic keyword rankings, Core Web Vitals, backlink data from DataForSEO |
| dynapps-brand | common-skills | Apply formatting, tone, and vocabulary compliance to any report output |

## Consumed by

| Skill | Suite | Why |
|-------|-------|-----|
| campaign-planning | ai-claude-marketing | Uses performance history and benchmarks to set campaign KPI targets |
| competitive-analysis | ai-claude-marketing | Uses channel benchmarks and share-of-voice data for competitive positioning |

## REFERENCES

- `references/kpi-definitions.md` — Full KPI tables for all channels with definitions, benchmarks, data sources
- `references/report-templates.md` — Weekly digest, monthly, quarterly, campaign, and event report templates
- `references/benchmarks.md` — Spanish B2B market benchmarks, seasonality calendar, comparison period guide
- `references/attribution-guide.md` — UTM tracking conventions, Odoo attribution fields, multi-touch considerations
- `references/optimization-framework.md` — A/B testing rules, optimization levers by funnel stage, Impact vs Effort matrix
- `references/shared/odoo-connection.md` — Odoo connection setup and odoo-pilot script usage
