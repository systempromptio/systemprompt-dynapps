# Odoo Ecosystem Content Sources

Curated sources for content discovery. Scan regularly for trending topics, news, and inspiration.

## Tier 1 — Official Odoo

| Source | URL | Content Type | Scan Frequency |
|--------|-----|--------------|----------------|
| Odoo Blog | https://www.odoo.com/blog | Product updates, features, tips | Weekly |
| Odoo Release Notes | https://www.odoo.com/odoo-18-release-notes | Version changes, new modules | Per release |
| Odoo Twitter/X | https://x.com/Odoo | Announcements, tips, events | Daily |
| Odoo LinkedIn | https://www.linkedin.com/company/odoo/ | Company news, partner spotlights | Weekly |
| Odoo YouTube | https://www.youtube.com/@OdooOfficial | Tutorials, webinars, OXP talks | Weekly |
| Odoo GitHub | https://github.com/odoo/odoo | Code changes, new features | Weekly |
| Odoo Forum | https://www.odoo.com/forum | Community questions, pain points | Weekly |

## Tier 2 — Community (OCA)

| Source | URL | Content Type |
|--------|-----|--------------|
| OCA GitHub | https://github.com/OCA | Community modules, PRs, new repos |
| OCA Website | https://odoo-community.org/ | Projects, working groups, events |
| OCA Twitter/X | https://x.com/aboreal | Community updates |

## Tier 3 — Partners and Ecosystem

| Source | URL | Notes |
|--------|-----|-------|
| Odoo Partners LinkedIn | Search "Odoo Gold Partner" | Competitor content, trends |
| Cybrosys Blog | https://www.cybrosys.com/blog/ | Technical tutorials (large volume) |
| Hibou Blog | https://www.hibou.io/blog | US partner, good technical content |
| Odoo Mates YouTube | https://www.youtube.com/@OdooMates | Technical walkthroughs |

## Tier 4 — Industry / ERP General

| Source | URL | Content Type |
|--------|-----|--------------|
| Gartner ERP | https://www.gartner.com/reviews/market/cloud-erp | Market analysis, reviews |
| G2 Odoo Reviews | https://www.g2.com/products/odoo/reviews | User sentiment, pain points |
| r/Odoo Reddit | https://www.reddit.com/r/Odoo/ | Community discussions, complaints, praise |
| ERP News | https://erpnews.com/ | Industry news, comparisons |

## Tier 5 — Indaws Internal

| Source | Location | Content Type |
|--------|----------|--------------|
| Case studies | content-collection skill → cases index | Client transformations |
| YouTube channel | content-collection skill → videos index | Podcast, tutorials, pills |
| Indaws Core modules | content-collection skill → modules index | Product features |
| Indaws blog | https://www.indaws.es/blog | Published articles |
| Indaws events | Team input / calendar | Past and upcoming events |
| Team photos | Marketing team / shared drive | Behind-the-scenes visuals |

## Content Angles per Source

When scanning sources, look for these angles:

### From Odoo Official
- New feature → "Odoo pill" post explaining business value
- Version release → Opinion post on impact for Spanish companies
- Event announcement → Promo or live-coverage opportunity

### From OCA / Community
- Popular new module → Technical pill or carousel
- Community debate → Opinion post taking a stance

### From Competitors / Partners
- Trending topic other partners post about → Create better version
- Gap: topics competitors ignore → Opportunity to lead

### From Indaws Internal
- New case study → Case study post (always high priority)
- Podcast episode → Promo post with best quote
- Team event → Culture post with real photos
- Module launch → Educational carousel

## Discovery Script Usage

The `scripts/discover_content.py` script automates scanning Tier 1-4 sources:

```bash
python scripts/discover_content.py --sources all --days 7
python scripts/discover_content.py --sources odoo,oca --days 3
python scripts/discover_content.py --sources reddit,twitter --days 1
```

Output: JSON list of content ideas with title, source URL, relevance score, and suggested post type.
