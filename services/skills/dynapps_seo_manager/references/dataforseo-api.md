# DataForSEO API Reference

## Authentication

Basic HTTP auth with login and password.

```bash
export DATAFORSEO_LOGIN="victor@indaws.es"
export DATAFORSEO_KEY="<stored in environment>"
```

All requests use:
```bash
curl -s "https://api.dataforseo.com/v3/[endpoint]" \
  -H "Content-Type: application/json" \
  -u "$DATAFORSEO_LOGIN:$DATAFORSEO_KEY" \
  -d '[{...}]'
```

## Location & Language Codes

| Market | Location Code | Language Code |
|--------|--------------|---------------|
| Spain | 2724 | es |
| Mexico | 2484 | es |
| Global | — | es |

---

## Endpoints Used

### 1. SERP — Live Keyword Rankings

Check where a keyword ranks, who else ranks.

```bash
POST /v3/serp/google/organic/live/advanced
[{
  "keyword": "odoo partner españa",
  "location_code": 2724,
  "language_code": "es",
  "device": "desktop",
  "depth": 100
}]
```

**Use**: Weekly rank tracking, competitor position monitoring.
**Cost**: 1 task = ~$0.002

### 2. Keyword Data — Search Volume

Get search volume, CPC, competition for keywords.

```bash
POST /v3/keywords_data/google_ads/search_volume/live
[{
  "keywords": ["odoo partner españa", "implementador odoo", "consultoria erp"],
  "location_code": 2724,
  "language_code": "es"
}]
```

**Use**: Keyword research, strategy updates.
**Cost**: 1 task per batch = ~$0.05

### 3. DataForSEO Labs — Related Keywords

Find related keywords for a seed keyword.

```bash
POST /v3/dataforseo_labs/google/related_keywords/live
[{
  "keyword": "odoo",
  "location_code": 2724,
  "language_code": "es",
  "depth": 2,
  "limit": 50
}]
```

**Use**: Keyword discovery, content ideas.

### 4. DataForSEO Labs — Competitors Domain

Find organic competitors for a domain.

```bash
POST /v3/dataforseo_labs/google/competitors_domain/live
[{
  "target": "dynapps.es",
  "location_code": 2724,
  "language_code": "es",
  "limit": 20
}]
```

**Use**: Competitor discovery, benchmark updates.

### 5. DataForSEO Labs — Domain Ranked Keywords

All keywords a domain ranks for.

```bash
POST /v3/dataforseo_labs/google/ranked_keywords/live
[{
  "target": "dynapps.es",
  "location_code": 2724,
  "language_code": "es",
  "limit": 500,
  "order_by": ["keyword_data.keyword_info.search_volume,desc"]
}]
```

**Use**: Full keyword inventory, tracking, cannibalization detection.

### 6. DataForSEO Labs — Keywords For Site

Similar to ranked keywords but organized differently.

```bash
POST /v3/dataforseo_labs/google/keywords_for_site/live
[{
  "target": "dynapps.es",
  "location_code": 2724,
  "language_code": "es",
  "limit": 200
}]
```

### 7. On-Page — Site Audit

Automated crawl and technical analysis.

```bash
# Start task
POST /v3/on_page/task_post
[{
  "target": "dynapps.es",
  "max_crawl_pages": 500,
  "load_resources": true,
  "enable_javascript": true,
  "enable_browser_rendering": true,
  "check_spell": false
}]

# Get results (after crawl completes)
POST /v3/on_page/summary/[task_id]
POST /v3/on_page/pages/[task_id]
POST /v3/on_page/duplicate_tags/[task_id]
POST /v3/on_page/duplicate_content/[task_id]
POST /v3/on_page/links/[task_id]
```

**Use**: Technical audit, on-page issues detection.
**Cost**: ~$0.01 per 10 pages crawled.

### 8. Backlinks — Profile Analysis

```bash
POST /v3/backlinks/summary/live
[{
  "target": "dynapps.es",
  "internal_list_limit": 10,
  "include_subdomains": true
}]
```

```bash
POST /v3/backlinks/backlinks/live
[{
  "target": "dynapps.es",
  "mode": "as_is",
  "limit": 100,
  "order_by": ["rank,desc"]
}]
```

**Use**: Backlink profile audit, new/lost link tracking.

### 9. SERP — Competitors

Find who competes for the same SERPs.

```bash
POST /v3/dataforseo_labs/google/serp_competitors/live
[{
  "keywords": ["odoo partner", "implementador odoo", "consultoria erp"],
  "location_code": 2724,
  "language_code": "es",
  "limit": 20
}]
```

---

## Rate Limits & Costs

- **Rate**: 2000 requests/minute (generous)
- **Billing**: Pay per task, varies by endpoint
- **Typical weekly cost**: ~$5-15 for monitoring + occasional research
- **Full audit cost**: ~$20-30 (crawl + all analyses)

## Best Practices

1. **Batch requests** — send multiple keywords in one call where possible
2. **Cache results** — store in data/ files, don't re-fetch unnecessarily
3. **Use filters** — limit results to reduce cost and processing
4. **Track costs** — log API calls and monitor spending
5. **Respect crawl politeness** — don't run full crawl more than monthly

## Error Handling

| Status Code | Meaning | Action |
|-------------|---------|--------|
| 20000 | OK | Process results |
| 40000 | Bad request | Check parameters |
| 40100 | Unauthorized | Check credentials |
| 40200 | Payment required | Check balance |
| 50000 | Internal error | Retry after delay |
