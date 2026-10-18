# SEO Audit Workflow — operational detail

Consolidated operational reference for SKILL.md. Methodology by audit type lives in
[audit-checklist.md](audit-checklist.md); keyword scoring in [keyword-strategy.md](keyword-strategy.md);
DataForSEO queries/limits in [dataforseo-api.md](dataforseo-api.md); legacy Odoo field mapping in
[archive-odoo-seo-fields.md](archive-odoo-seo-fields.md) (pre-migration, kept for history).

## Lifecycle

```
AUDIT (deep, quarterly) → STRATEGY (keyword map, content plan) → WEEKLY CYCLE
WEEKLY CYCLE (Tuesday report):
  1. Monitor  — rank_tracker.py, traffic, issues
  2. Analyze  — opportunities + problems
  3. Propose  — prioritized recommendations (deliverable for the web team)
  4. Measure  — before/after once the team ships changes; log in data/changelog.json
```

## Scripts

| Script | Purpose | Usage |
|--------|---------|-------|
| `scripts/seo_audit.py` | Full audit data collection | `--full`, `--technical`, `--backlinks`, `--competitors`, `--keywords`, `--force` |
| `scripts/rank_tracker.py` | Track keyword positions | `--keywords target\|all`, `--compare`, `--save` |
| `scripts/site_analyzer.py` | On-page analysis | `--audit full\|quick`, `--url [URL]`, `--sitemap [file]` |
| `scripts/run_technical_audit.py` | Deep technical crawl: fetches the live sitemap (`https://www.dynapps.es/sitemap.xml`), filters to www.dynapps.es, validates hreflang | optional `--cached` to reuse `data/sitemap_urls.json` |
| `scripts/generate_html_reports.py` | Visual reports | `--type weekly\|audit\|keywords\|all`, `--output` |

Env vars: `DATAFORSEO_LOGIN`, `DATAFORSEO_KEY`. Scripts cache API responses 24h (`_fetched_at` timestamp); `--force` bypasses.

## Running a full audit

```bash
python scripts/seo_audit.py --full
python scripts/run_technical_audit.py
python scripts/generate_html_reports.py --type audit
```

## Weekly cycle

```bash
# Monday night — data collection
python scripts/rank_tracker.py --keywords target --save
python scripts/seo_audit.py --keywords
# Tuesday — report + delivery
python scripts/generate_html_reports.py --type weekly
```

Report contents: dashboard (metrics vs prev week), top 5 prioritized actions, keyword movements
(subidas/bajadas/nuevas TOP 10), technical issues (incl. hreflang inconsistencies), competitor
movements. Template in [weekly-seo-report.md](weekly-seo-report.md). Send summary to Victor,
deliver the recommendations document to the web team, log the batch in `data/changelog.json`.

## Priority framework

Score = Impact (1-5) × Effort (1-5 inverse).

| Priority | Criteria | Example |
|----------|----------|---------|
| Quick wins | High impact, low effort | Fix missing meta descriptions |
| Strategic | High impact, high effort | Create pillar content |
| Low-hanging | Medium impact, low effort | Add alt tags to images |
| Backlog | Low impact, high effort | Restructure URLs |

Keyword-level scoring formula in [keyword-strategy.md](keyword-strategy.md).

## HTML report templates

Dynapps-branded dashboards (Readex Pro, Midnight Denim / Unicorn Violet / Sunset Tangerine palette).
Variables use `{{VARIABLE}}` syntax, populated by `generate_html_reports.py`.

| Template | Output |
|----------|--------|
| `templates/weekly-seo-report.html` | Bento grid: score, keywords, actions |
| `templates/audit-report.html` | Ring scores, check lists, issues table |
| `templates/keyword-tracker.html` | Filterable rankings with trends |

## Recommendations delivery (audit-only mode)

www.dynapps.es runs on a CMS without API integration: **the skill never executes changes**.
Every finding is delivered as a recommendation for manual execution by the web team.

Each recommendation includes: page URL, element, current value, proposed value, rationale
(data-backed). Delivery protocol: compile batch → validate against dynapps-brand → deliver to
the web team → append the batch to `data/changelog.json` (`"type": "recommendations"`) →
verify application in the next audit run → measure ranking impact.

Safety guidance to attach to every batch: never change URLs without redirects; never delete
pages (redirect instead); homepage changes flagged for explicit approval; batch ~20 changes/week
to isolate impact.

> Legacy: the old Odoo site allowed direct writes via odoo-pilot (`website.page`, `blog.post`,
> `ir.attachment`). That protocol is preserved in [archive-odoo-seo-fields.md](archive-odoo-seo-fields.md)
> in case an API execution layer returns.

## Content creation

Always read [blog-content-guide.md](blog-content-guide.md) first (parts describe the legacy Odoo
blog — validate structure/URLs/CTAs against the new CMS). Pre-creation checklist: search existing
posts on the topic (update, don't duplicate); mandatory internal links (use content-collection
sitemap); zero emojis; "tu" form not "usted".

## Primary competitors (Spain)

| Domain | Notes |
|--------|-------|
| processcontrol.es | Gold Partner |
| tecnativa.com | Strong technical content |
| studio73.es | Active blog |
| nanobytes.es | — |
| puntsistemes.es | — |
| qubiq.es | — |
| factorlibre.com | — |

Track per competitor: shared keywords, ranking changes, new content, backlink growth.
Every audit also surfaces new SERP competitors from DataForSEO Labs (`serp_competitors.json`)
and flags candidates for Victor to review.

## Data files

| File | Purpose |
|------|---------|
| `data/ranked_keywords.json` | Current keyword rankings |
| `data/rank_history.json` | Historical rankings (52 weeks; restarted July 2026) |
| `data/backlinks_summary.json` | Backlink profile metrics |
| `data/backlinks_top.json` | Top referring pages |
| `data/sitemap_urls.json` | Latest www.dynapps.es sitemap snapshot (auto-refreshed) |
| `data/technical_audit_full.json` | Full technical audit results |
| `data/audit_report.md` | Generated audit report (markdown) |
| `data/changelog.json` | Log of audit runs and recommendation batches |
| `data/reports/*.html` | Generated HTML reports |
| `data/archive-indaws/` | Pre-migration indaws.es data (July 2026) |
