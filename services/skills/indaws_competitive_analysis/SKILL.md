---
name: indaws-competitive-analysis
version: 1.0.1
effort: medium
model: sonnet
description: >-
  Delivers competitive intelligence on Odoo partners in Spain: battlecards, positioning maps,
  messaging comparisons, and content gap analysis. Use when the user needs to research a competitor,
  benchmark Indaws positioning, or identify content opportunities.
  Triggers: "analiza la competencia", "battlecard de X", "mapa de posicionamiento",
  "quiénes son nuestros competidores", "gap de contenido", "competitive analysis",
  "cómo nos diferenciamos de", "qué hace X partner", "positioning comparison",
  "comparativa competidores".
---

# Competitive Analysis — Indaws

Systematic competitive intelligence for the Odoo partner ecosystem in Spain.

## Workflow

1. **Define research scope** — Clarify which competitor(s), what deliverable (battlecard / positioning map / content gap / messaging matrix / full review), and time horizon (ad-hoc vs. quarterly cycle). Read `references/competitor-profiles.md` to check if fresh data already exists (< 90 days).
2. **Gather primary sources** — Pull data from Odoo Partner Directory, competitor websites, LinkedIn pages, Google My Business, Glassdoor, and Odoo Apps Store. For SEO metrics (domain authority, organic traffic, keyword rankings) delegate to **seo-manager**. For traffic benchmarks delegate to **performance-analytics**. Full source list in `references/research-methodology.md`.
3. **Build messaging matrix** — Map each competitor across positioning statement, tagline, key differentiators, target audience, tone of voice, CTA, and social proof. Use Odoo-specific dimensions (tier, methodology, verticals, geographic coverage, international backing). Full matrix template and positioning maps in `references/messaging-frameworks.md`.
4. **Run content gap analysis** — Inventory competitor content output (blog, LinkedIn, case studies, events, video, downloads, App Store modules). Identify high-priority gaps (covered by 2+ competitors but not Indaws), differentiation opportunities, and blue-ocean topics. Score with the Content Opportunity Formula. Full process in `references/content-gap-analysis.md`.
5. **Create battlecards** — One battlecard per major competitor using the template in `references/content-gap-analysis.md`. Include threat level, their pitch (verbatim), strengths/weaknesses, our differentiators, objection handling, win/lose themes, and key intelligence. Validate positioning pillars against `dynapps-brand` guidelines.
6. **Update competitor profiles** — Record all new findings in `references/competitor-profiles.md` with date stamps. Flag any profile older than 90 days as stale. Trigger quarterly deep review (January, April, July, October) per `references/research-methodology.md`.

## GOTCHAS

1. **Treating Odoo.com as a competitor** — Odoo B.V. is a vendor and partner ecosystem owner, not a competing implementor. Never include them in battlecards or positioning comparisons. Their partners are the competitors.
2. **Conflating Dynapps partnership tier with Indaws tier** — Indaws is the legal entity with Gold status in Spain. Dynapps (Belgium parent) has its own tier. Never present Dynapps's overall group credentials as Indaws-specific wins without grounding.
3. **Using stale data** — Partner websites, LinkedIn follower counts, and team sizes change rapidly. Always date-stamp observations and refuse to present data older than 90 days as current. Check profile last-updated dates before delivering any output.
4. **Treating market position as fixed** — Partnership tiers change (Gold ↔ Silver), companies merge or close, new entrants appear. Never assume last quarter's map is still valid without re-checking the Odoo Partner Directory.
5. **Disparaging competitors** — Battlecard weaknesses must be observable and factual ("no case studies published since 2022") not editorial ("low quality team"). Battlecards are internal only; disparaging language creates legal and reputational risk if leaked.
6. **Confusing SEO visibility with market share** — A competitor can have high organic traffic and low deal volume, or vice versa. Domain authority is a proxy, not ground truth. Always triangulate with LinkedIn activity, Glassdoor headcount, and CRM deal data.

## NEVER

- Include Odoo B.V. in competitor comparisons — they are the platform vendor.
- Share battlecards externally — they are strictly internal sales tools.
- Present data older than 90 days as current without re-verification.
- Make up competitor data — if a field is unknown, mark it as `[unknown — needs verification]`.
- Use disparaging or unprovable language about competitors.

## ALWAYS

- Date-stamp every observation in `references/competitor-profiles.md`.
- Validate final positioning outputs against `dynapps-brand` guidelines (Montserrat, Indaws palette, "tu" voice, zero emojis).
- Include a threat level (High / Medium / Low) in every battlecard.
- Cross-reference SEO data from seo-manager before presenting traffic-related competitor claims.
- Keep Indaws positioning statement internally consistent: Gold partner + Dynapps group + Comunidad Valenciana focus + PYME specialization.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| `dynapps-brand` | common-skills | Validating that positioning outputs and messaging comply with Indaws brand voice, palette, and style rules |
| `seo-manager` | ai-claude-marketing | Fetching SEO competitor data: domain authority, keyword rankings, organic traffic estimates, backlink profiles |
| `performance-analytics` | ai-claude-marketing | Retrieving traffic benchmarks and channel performance data to contextualize competitor visibility |

## Consumed by

| Skill | Suite | Why |
|-------|-------|-----|
| `campaign-planning` | ai-claude-marketing | Uses battlecards and positioning maps to define differentiation strategy in campaign briefs |
| `seo-manager` | ai-claude-marketing | Pulls competitor positioning for SEO competitor-gap analysis |

## Agent Routing

Run entirely on **sonnet** (medium effort); no sub-agents needed — delegate SEO data gathering to `seo-manager` and traffic benchmarks to `performance-analytics` as blocking steps before finalizing any output that includes competitor visibility data.

## REFERENCES

| Reference | When to read |
|-----------|-------------|
| [references/competitor-profiles.md](references/competitor-profiles.md) | Always — check for existing data before starting research |
| [references/research-methodology.md](references/research-methodology.md) | Step 2 — full source list, quarterly/monthly cadence, ad-hoc triggers |
| [references/messaging-frameworks.md](references/messaging-frameworks.md) | Step 3 — messaging matrix template, Odoo-specific dimensions, positioning maps |
| [references/content-gap-analysis.md](references/content-gap-analysis.md) | Steps 4-5 — content inventory, gap scoring formula, battlecard template |
| `common:references/shared/` | Shared coding standards and Odoo connection patterns (when generating Odoo CRM reports) |
