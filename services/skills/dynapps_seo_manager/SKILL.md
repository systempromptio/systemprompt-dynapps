---
name: dynapps-seo-manager
version: 2.0.0
effort: medium
model: sonnet
description: >-
  SEO management for www.dynapps.es (Spain market of the Dynapps multi-region group site):
  audit, keyword strategy, rank tracking, technical/on-page/content optimization, competitor gap,
  and hreflang/multi-region checks. AUDIT-ONLY: produces prioritized recommendations for the web
  team (the new CMS has no API execution layer). Data via DataForSEO (Python scripts) and Google
  Analytics/GSC. Produces Dynapps-branded HTML reports and weekly improvement digests.
  Triggers: "auditoría SEO", "audita dynapps.es", "estrategia de keywords", "rank tracking",
  "posiciones de keywords", "optimización SEO", "meta descriptions", "gap de competidores SEO",
  "informe SEO semanal", "SEO audit", "/seo-audit".
---

# SEO Manager — Dynapps Spain

End-to-end SEO for www.dynapps.es: audit → strategy → weekly improvement → prioritized recommendations.
All content stays Spanish-primary, brand-compliant (zero emojis, "tu" form), data-driven.
**Audit-only**: the skill proposes; the web team executes on the CMS.

## Workflow

A full SEO audit runs in five steps; each points to its reference/script. Operational detail
(scripts, weekly cycle, templates, recommendations protocol, competitors, data files) in
[references/audit-workflow.md](references/audit-workflow.md).

1. **Keywords** — Build/refresh the keyword map and scoring. Pull rankings with `scripts/rank_tracker.py --keywords target --save`. Strategy and priority formula in [references/keyword-strategy.md](references/keyword-strategy.md).
2. **Technical** — Crawl Core Web Vitals, crawlability, HTTPS, speed, mobile, hreflang: `scripts/seo_audit.py --technical` + `scripts/run_technical_audit.py` (fetches the live sitemap and filters to www.dynapps.es). Checklist + thresholds in [references/audit-checklist.md](references/audit-checklist.md).
3. **Content / on-page** — Titles, metas, headings, images, thin content, freshness, cannibalization: `scripts/site_analyzer.py --audit full`. Blog rules in [references/blog-content-guide.md](references/blog-content-guide.md) (partially legacy — validate against the new CMS).
4. **Competitors** — `scripts/seo_audit.py --competitors` against the primary set (processcontrol.es, tecnativa.com, studio73.es, …). Shared keywords, ranking shifts, content/backlink gaps. List in [references/audit-workflow.md](references/audit-workflow.md); flag new Labs SERP competitors for review.
5. **Report** — `scripts/generate_html_reports.py --type audit|weekly`. Prioritize actions (Impact × Effort) and deliver the recommendations document to the web team; log the batch in `data/changelog.json`. Weekly template in [references/weekly-seo-report.md](references/weekly-seo-report.md).

DataForSEO env vars and query catalog: [references/dataforseo-api.md](references/dataforseo-api.md).

## GOTCHAS

- **DataForSEO is metered and paid** — every script call costs credits and has rate limits; scripts cache responses 24h (`_fetched_at`). Never loop calls or re-run `--full` casually; use `--force` only when data is truly stale.
- **Rank tracking needs history** — a single pull is a snapshot, not a trend. Movements (subidas/bajadas) only mean something against `data/rank_history.json` (52 weeks). Always `--save` and `--compare`, never report position changes from one fetch. History restarted July 2026 with the dynapps.es migration — early trends will be sparse.
- **GSC/Analytics data lags** — Search Console is delayed ~2-3 days and finalizes later; never present "today's" GSC numbers as real-time, and reconcile against DataForSEO rather than mixing sources.
- **Technical vs content audits answer different questions** — a green technical crawl (CWV, HTTPS, speed) does not mean the page ranks; thin/duplicate content fails despite perfect technicals. Run both; don't substitute one for the other.
- **Keyword cannibalization** — two dynapps.es pages targeting the same intent split authority and suppress both. Before adding a target keyword, check no existing page already owns it (consolidate/redirect instead of competing with yourself).
- **Multi-region group site** — the sitemap carries hreflang alternates for dynapps.be/.nl/.fr/.ch; audits cover **www.dynapps.es only** (the crawler filters the sitemap). Validate hreflang consistency and self-reference, but never audit or propose changes on other group domains. Some tags (canonical/hreflang) may be JS-injected — raw fetches under-report them; confirm with the DataForSEO browser-rendered crawl before flagging.
- **The site runs on a new CMS with no API integration** — there is NO execution layer (the old Odoo write path via odoo-pilot is retired; see [references/archive-odoo-seo-fields.md](references/archive-odoo-seo-fields.md)). Every finding becomes a recommendation (page, element, current value, proposed value, rationale) for manual execution by the web team.

## NEVER

- Execute or attempt any website change — audit-only; deliver recommendations instead.
- Audit or target a domain other than www.dynapps.es (other group domains belong to other teams).
- Propose a URL change without its redirect, or propose deleting a page instead of redirecting.
- Propose content that breaks dynapps-brand (emojis, hashtags, "usted", non es-ES variants).
- Report ranking movements or GSC figures as real-time without acknowledging history/lag.

## ALWAYS

- Back every recommendation with data (DataForSEO + GSC/Analytics).
- `--save` rank pulls so history accrues; compare against `data/rank_history.json`.
- Log every audit run and delivered recommendation batch in `data/changelog.json`; verify application in the next run and measure impact.
- Validate content recommendations against `dynapps-brand` before delivering.
- Prioritize actions with Impact × Effort and surface the top 5 in each report.
- Recommend batching (~20 changes/week) and flag homepage changes for explicit approval.

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| `dynapps-brand` | common-skills | Validating content recommendations against brand voice, palette, typography |
| `content-collection` | common-skills | Inventory of existing cases, Core modules, videos, sitemap for interlinking and dedup |
| `competitive-analysis` | ai-claude-marketing | Deep competitor positioning beyond SEO metrics (battlecards, messaging) |

## Consumed by

| Skill | Suite | Why |
|-------|-------|-----|
| `performance-analytics` | ai-claude-marketing | Pulls SEO metrics (rankings, traffic, technical scores) for marketing performance reports |
| `competitive-analysis` | ai-claude-marketing | Requests SEO competitor data: domain authority, keyword rankings, backlink profiles |
| `content-creation` | ai-claude-marketing | Uses keyword targets and on-page recommendations when drafting web/blog content |

## Agent Routing

Runs entirely on **sonnet** (medium effort); no sub-agents. DataForSEO/script calls are blocking
data-gathering steps; delegate brand validation to `dynapps-brand` as a blocking step before
finalizing any recommendation batch.

## REFERENCES

| Reference | When to read |
|-----------|-------------|
| [references/audit-workflow.md](references/audit-workflow.md) | Operational detail: scripts, weekly cycle, templates, recommendations protocol, competitors, data files |
| [references/audit-checklist.md](references/audit-checklist.md) | Step 2 — full SEO checklist, thresholds, audit frequency |
| [references/keyword-strategy.md](references/keyword-strategy.md) | Step 1 — keyword categories, map, scoring formula |
| [references/dataforseo-api.md](references/dataforseo-api.md) | DataForSEO auth, query catalog, cost/limits |
| [references/blog-content-guide.md](references/blog-content-guide.md) | Step 3 — blog post creation rules (LEGACY in parts: describes the old Odoo blog) |
| [references/weekly-seo-report.md](references/weekly-seo-report.md) | Step 5 — Tuesday report template |
| [references/archive-odoo-seo-fields.md](references/archive-odoo-seo-fields.md) | LEGACY — Odoo field mapping from the pre-migration site (kept in case an execution layer returns) |
| `scripts/` | `seo_audit.py`, `rank_tracker.py`, `site_analyzer.py`, `run_technical_audit.py`, `generate_html_reports.py` |
| `schemas/placeholders.json` | Template variable schema for HTML report generation |
| `data/` | Cached rankings, history, audit results, changelog, generated reports; `data/archive-indaws/` holds pre-migration indaws.es data |

## Success Criteria

- Audit covers all five steps (keywords, technical, content, competitors, report) with data-backed findings.
- Recommendations are prioritized (Impact × Effort) with a clear top 5, each with page/element/current/proposed/rationale.
- Every delivered batch is brand-compliant, logged in `data/changelog.json`, and verified for application in the next run.
- Rank movements are reported against history, not single snapshots; GSC lag is acknowledged.
- Hreflang/multi-region consistency validated without touching other group domains.
