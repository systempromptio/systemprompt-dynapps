# Data Schemas

JSON structure for each data file in `data/`.

## videos.json

```json
{
  "id": "youtube_video_id",
  "title": "Video Title",
  "description": "Full description...",
  "publish_date": "2024-01-15",
  "duration": "PT12M30S",
  "url": "https://youtube.com/watch?v=...",
  "thumbnail": "assets/thumbnails/VIDEO_ID.jpg",
  "topics": ["odoo", "inventory", "manufacturing"]
}
```

## cases.json

```json
{
  "id": 123,
  "title": "Client Name - Project Title",
  "client": "Client Name",
  "industry": "Manufacturing",
  "url": "https://indaws.es/blog/...",
  "publish_date": "2024-02-20",
  "summary": "Brief summary...",
  "content": "Full blog post content...",
  "modules_used": ["indaws_inventory", "indaws_mrp"]
}
```

## modules.json

```json
{
  "id": 456,
  "name": "Indaws Inventory",
  "technical_name": "indaws_inventory",
  "code": "IND-INV",
  "description": "Extended inventory management...",
  "features": ["Feature 1", "Feature 2"],
  "price": 1500.00,
  "url": "https://indaws.es/shop/...",
  "related_cases": [123, 125],
  "related_videos": ["VIDEO_ID_1", "VIDEO_ID_2"]
}
```

## sitemap.json

```json
{
  "url": "https://www.indaws.es/blog/blog-indaws-1/articulo-ejemplo-123",
  "category": "blog",
  "title_hint": "Articulo Ejemplo",
  "lastmod": "2026-01-15",
  "priority": 0.5,
  "changefreq": ""
}
```

Categories: `blog`, `case_study`, `blog_other`, `page`, `service`, `slide`, `shop`, `job`, `event`, `forum`, `helpdesk`

## Cross-References

The `build_indexes.py` script automatically creates cross-references:

- **Modules** have `related_cases` and `related_videos` arrays
- Links are built by matching module names/keywords in case content and video topics
- Use these to find all content about a specific module

Example: To find all content about Indaws Inventory:
1. Read module from `data/modules.json` where `technical_name = "indaws_inventory"`
2. Use `related_cases` array to fetch case studies
3. Use `related_videos` array to fetch video details
