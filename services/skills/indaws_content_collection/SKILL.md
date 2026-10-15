---
name: indaws-content-collection
version: 2.0.2
effort: low
model: sonnet
description: >
  Catalogo pre-empaquetado de contenido Dynapps: casos de exito, modulos [CORE], videos YouTube y
  sitemap completo para interlinking SEO. Usar cuando se necesiten referencias a exitos de cliente,
  funcionalidades Core o URLs internas. NOT generacion de contenido → odoo-frontend / dynapps-brand.
  Triggers: "casos de exito dynapps", "videos dynapps", "sitemap dynapps", "referencias internas SEO",
  "modulos core dynapps", "interlinking blog", "contenido dynapps". (Antes marca Indaws.)
---

# Content Collection

Pre-packaged catalog of Dynapps content from four sources:

| Source | Content | Data File |
|--------|---------|-----------|
| YouTube Channel | `@dynapps-es` videos + thumbnails | `data/videos.json` |
| Marketing site cases | Client success stories (`www.dynapps.es/cases`) | `data/cases.json` |
| Odoo Products | Dynapps `[CORE]` module specifications (`portal.dynapps.es`) | `data/modules.json` |
| Website Sitemaps | All public URLs (portal + www) categorized by type | `data/sitemap.json` |

Data is collected by running `scripts/collect_all.py` and stored in the skill package.

---

## Content Routing

| Need | Read This |
|------|-----------|
| Find a case study | [references/cases-index.md](references/cases-index.md) |
| Find a module spec | [references/modules-index.md](references/modules-index.md) |
| Find a video | [references/videos-index.md](references/videos-index.md) |
| Find a page for interlinking | [references/sitemap-index.md](references/sitemap-index.md) |
| Get full record details | Load specific item from `data/*.json` |

---

## Key Rules

- Data is pre-packaged (not fetched at runtime) — run `collect_all.py` to refresh
- Index files (`references/*-index.md`) are auto-generated; do not edit manually
- Cross-references (`related_cases`, `related_videos`) are rebuilt each time indexes are generated
- Use `grep -i "keyword" data/*.json` to search across all content
- Modules have `related_cases` and `related_videos` arrays for cross-content discovery

---

## GOTCHAS
1. **JSON data files are NOT real-time** — data is refreshed periodically. Don't assume latest blog posts/cases are included.
2. **sitemap.json is large** — never read the whole file. Use grep/search to find specific URLs.
3. **`[CORE]` is the ONLY reliable module identifier** — modules are filtered by the `[CORE]` marker in the product name, NOT by category (they span several categories and some have none). `technical_name` is a slug derived from the name; use it, not `name`.
4. **Two web properties** — the Odoo (`portal.dynapps.es`) hosts shop/blog/slides; the marketing site (`www.dynapps.es`) hosts service pages, blog and `/cases/`. The sitemap collector merges both.
5. **Cases are scraped HTML, not Odoo** — `collect_cases.py` scrapes `www.dynapps.es/cases` (enumerated from the sitemap). Client/sector/metrics live in narrative prose; `modules_used` are detected functional areas, and `publish_date` is usually empty.
6. **Video thumbnails are YouTube URLs** — they may be blocked by firewalls. Always provide text fallback.

---

## Delegates to
None — this is a data catalog skill.

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| linkedin-content-manager | common | Case studies for LinkedIn posts |
| odoo-module-creator | developer | Module naming references |
| content-creation | marketing | Content sourcing |
| competitive-analysis | marketing | Case study references |
| linkedin | marketing | Content inspiration |

For shared ecosystem patterns, see `common:references/shared/`.

## REFERENCES

| File | Content |
|------|---------|
| [references/cases-index.md](references/cases-index.md) | Browseable index of all case studies |
| [references/modules-index.md](references/modules-index.md) | Browseable index of all Dynapps [CORE] modules |
| [references/videos-index.md](references/videos-index.md) | Browseable index of all YouTube videos |
| [references/sitemap-index.md](references/sitemap-index.md) | Browseable index of all website URLs by category |
| [references/youtube-casos-exito.md](references/youtube-casos-exito.md) | Success story videos playlist and embed patterns |
| [references/data-schemas.md](references/data-schemas.md) | JSON schemas for all data files + cross-reference docs |
| [references/usage-guide.md](references/usage-guide.md) | Search patterns, refresh workflow, integration guide, assets |
