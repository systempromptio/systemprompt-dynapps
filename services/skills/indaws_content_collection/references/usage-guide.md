# Usage Guide

## Search Patterns

### By Keyword

Search across all content for a keyword:
```bash
grep -i "inventory" data/*.json
grep -i "manufacturing" data/cases.json
```

### By Module

Find all content related to a specific module:
```bash
# Cases using the module
grep "indaws_inventory" data/cases.json

# Videos about the topic
grep -i "inventory" data/videos.json
```

### By Client

Find case study for a specific client:
```bash
grep -i "client_name" data/cases.json
```

---

## Refresh Workflow

To update the content collection:

```bash
# Set environment variables
export YOUTUBE_API_KEY="<your-youtube-key>"  # example placeholder
export ODOO_URL="https://indaws.es"
export ODOO_DB="indaws_production"
export ODOO_KEY="<your-odoo-key>"  # example placeholder

# Run collection (from skill directory)
cd skills/content-collection
python scripts/collect_all.py

# Or run individual collectors
python scripts/collect_youtube.py
python scripts/collect_cases.py
python scripts/collect_modules.py
python scripts/build_indexes.py

# Re-package the skill
python /path/to/skill-creator/scripts/package_skill.py .
```

### Collection Scripts

| Script | Source | Requirements |
|--------|--------|--------------|
| `collect_youtube.py` | YouTube Data API v3 | `YOUTUBE_API_KEY`, `google-api-python-client` |
| `collect_cases.py` | Odoo Blog | `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, odoo-pilot |
| `collect_modules.py` | Odoo Products | `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, odoo-pilot |
| `collect_sitemap.py` | Website sitemap.xml | None (public URL) |
| `build_indexes.py` | Local JSON files | None |
| `collect_all.py` | All sources | All above |

---

## Integration with Other Skills

### For Consumer Skills (e.g., blog-creator)

Reference this skill in your SKILL.md:

```markdown
## Required Skills

- **content-collection**: For case studies, module specs, and videos.
  - Read `references/cases-index.md` to browse case studies
  - Read `references/modules-index.md` to browse modules
  - Load full records from `data/*.json` as needed
```

### With dynapps-brand

When generating content that references this collection:
1. Find relevant content using indexes
2. Load full records from JSON
3. Apply dynapps-brand voice and visual guidelines

---

## Assets

### Thumbnails (`assets/thumbnails/`)

YouTube video thumbnails stored as JPG files named by video ID:
- `VIDEO_ID.jpg` - High quality thumbnail (typically 480x360)

Use in HTML/markdown:
```html
<img src="assets/thumbnails/VIDEO_ID.jpg" alt="Video title">
```
