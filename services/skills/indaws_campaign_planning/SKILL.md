---
name: indaws-campaign-planning
version: 1.0.3
effort: medium
model: sonnet
description: >
  [SUSTITUIDA — no usar] Doctrina obsoleta que contradice Odoo Knowledge (space 12).
  Voz y vocabulario: `es-lint` y `dynapps-brand`; calendario y contenido: `social-calendar`.
---

# Campaign Planning — Indaws

Planificacion, ejecucion y medicion de campanas de marketing para Indaws.
Solo canales propios. Sin publicidad de pago.

---

## Workflow

1. **Define SMART objective** — choose campaign type (awareness, lead gen, event, product launch, nurturing, SEO) and write a measurable goal with deadline. Use `references/campaign-framework.md` objective examples.
2. **Select audience persona** — identify primary buyer persona (Decision Maker, Influencer Tecnico, Usuario Final, Partner Referral) and tailor value proposition. See `references/campaign-framework.md` personas table.
3. **Choose channels** — use Channel Selection Matrix in `references/channel-guide.md` to pick primary and support channels. Never propose paid ads.
4. **Build content calendar** — populate weekly calendar template from `references/content-calendar.md`. Establish quarterly themes, monthly detail, weekly execution cadence. Calendar must be approved before any content production starts.
5. **Set KPIs** — assign targets per campaign type from `references/kpi-guide.md`. Configure measurement dashboard, record baseline, and set UTMs before launch.
6. **Execute and measure** — delegate execution to specialist skills (see Delegates to). Track KPIs at midpoint and produce closing report comparing results vs objectives.

---

## GOTCHAS

- **Paid ads creep**: Users sometimes ask to boost a LinkedIn post or run Google Ads. Indaws does NOT do paid advertising. Redirect to organic amplification strategies.
- **No measurement plan before launch**: Campaigns launched without UTMs, baseline data, or a dashboard will have unmeasurable results. Block launch until measurement is configured.
- **Content production before calendar approval**: Producing assets before the calendar is finalized wastes effort when priorities shift. Always finalize and approve the calendar first.
- **Skipping buyer persona**: Selecting channels without first identifying the audience leads to wrong channel mix (e.g., nurturing emails sent to Decision Makers who only attend events).
- **Single-channel campaigns**: Relying on one channel (e.g., email only) underperforms. Every campaign needs at least a primary + one support channel.
- **ROI measured too early**: B2B sales cycles are 90-180 days. Measuring ROI at 30 days will always look bad. Set review dates at 90 and 180 days.
- **Event promotion without lead follow-up plan**: Events generate leads only if there is a post-event email sequence and CRM pipeline stage defined before the event.

---

## NEVER

- Propose paid advertising (Google Ads, LinkedIn Ads, Meta Ads, sponsored content).
- Produce content assets before the content calendar is approved.
- Launch a campaign without defined KPIs and a measurement setup.
- Use emojis or hashtags in any Indaws-facing content (brand rule).
- Create emails in Odoo in any state other than DRAFT.

---

## ALWAYS

- Apply `dynapps-brand` compliance to all campaign materials.
- Define a SMART objective before selecting channels or planning content.
- Use the Channel Selection Matrix before recommending channels.
- Set a closing report date (with results vs objectives) at campaign kickoff.
- Confirm all emails are created as DRAFT in Odoo before handing off to email-campaign-builder.

---

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| `dynapps-brand` | common-skills | All materials need brand compliance check |
| `content-creation` | ai-claude-marketing | Blog posts, landing pages, content briefs |
| `linkedin-content-manager` | ai-claude-marketing | LinkedIn post creation and scheduling |
| `email-campaign-builder` | ai-claude-marketing | Email sequences and automation in Odoo |
| `event-planner` | ai-claude-marketing | Event logistics, Odoo setup, promotion, follow-up |
| `seo-manager` | ai-claude-marketing | Keyword research, SEO optimization, rank tracking |
| `content-collection` | common-skills | Inventory and reuse of existing content assets |

---

## Consumed by

| Skill | Suite | Role |
|-------|-------|------|
| `performance-analytics` | ai-claude-marketing | Measures campaign results against KPI targets; produces closing reports |

---

## REFERENCES

Shared base: `common:references/shared/`

| File | Content |
|------|---------|
| `references/campaign-framework.md` | Campaign framework, SMART objectives, buyer personas, value propositions, brief template, launch checklist, campaign types quick reference |
| `references/channel-guide.md` | Channel selection matrix, LinkedIn, email, events, blog, landing pages — parameters, cadence, and when to use each |
| `references/content-calendar.md` | Weekly calendar template, planning process (quarterly/monthly/weekly), cadence per channel, seasonal calendar, content mix targets |
| `references/budget-guide.md` | Budget by spend category, budget by campaign type, ROI calculation formula |
| `references/kpi-guide.md` | KPIs by campaign type (awareness, lead gen, event, product launch, nurturing, SEO), measurement setup checklist |
