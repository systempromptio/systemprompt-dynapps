> **LEGACY (julio 2026):** Ejecución vía Odoo del antiguo www.indaws.es. El sitio actual www.dynapps.es usa otro CMS sin capa de ejecución API — el skill es audit-only. Se conserva por si vuelve una capa de ejecución.

# Odoo SEO Fields — Model & Field Reference

## Website Pages

### Model: `website.page`

| Field | Type | SEO Use |
|-------|------|---------|
| `name` | Char | Page name (internal) |
| `url` | Char | Page URL path |
| `website_meta_title` | Char | SEO title tag (overrides name) |
| `website_meta_description` | Text | Meta description |
| `website_meta_keywords` | Char | Meta keywords (low SEO value but available) |
| `seo_name` | Char | URL slug |
| `is_published` | Boolean | Published status |
| `website_indexed` | Boolean | Whether page should be indexed |

### Changing a page title
```bash
# Read current
./scripts/search_records.sh website.page '[["url","=","/que-es-odoo"]]' '["name","website_meta_title","website_meta_description"]'

# Update
./scripts/update_record.sh website.page '[PAGE_ID]' '{"website_meta_title":"Nuevo titulo SEO | Indaws","website_meta_description":"Nueva meta description optimizada"}'
```

---

## Blog Posts

### Model: `blog.post`

| Field | Type | SEO Use |
|-------|------|---------|
| `name` | Char | Post title (H1 and default title tag) |
| `subtitle` | Char | Subtitle |
| `content` | Html | Post body content |
| `website_meta_title` | Char | SEO title (overrides name if set) |
| `website_meta_description` | Text | Meta description |
| `seo_name` | Char | URL slug |
| `tag_ids` | Many2many | Blog tags |
| `blog_id` | Many2one | Parent blog |
| `is_published` | Boolean | Published status |
| `published_date` | Datetime | Publication date |
| `visits` | Integer | Visit counter |

### Optimizing a blog post
```bash
# Find the post
./scripts/search_records.sh blog.post '[["seo_name","like","caso-exito"]]' '["name","website_meta_title","website_meta_description","visits"]' 10

# Update meta tags
./scripts/update_record.sh blog.post '[POST_ID]' '{"website_meta_title":"Caso de Exito: [Cliente] con Odoo | Indaws","website_meta_description":"Descubre como [Cliente] transformo su gestion con Odoo. Resultados: [dato concreto]. Acompanamiento experto de Indaws."}'

# Update content (be careful with HTML)
./scripts/update_record.sh blog.post '[POST_ID]' '{"content":"<p>Nuevo contenido HTML aqui...</p>"}'
```

---

## Blog (container)

### Model: `blog.blog`

| Field | Type | SEO Use |
|-------|------|---------|
| `name` | Char | Blog name |
| `subtitle` | Char | Blog subtitle |
| `website_meta_title` | Char | Blog listing page title |
| `website_meta_description` | Text | Blog listing meta desc |

---

## Redirects

### Model: `website.rewrite` (Odoo 19)

| Field | Type | Use |
|-------|------|-----|
| `name` | Char | Description |
| `url_from` | Char | Source URL |
| `url_to` | Char | Destination URL |
| `redirect_type` | Selection | '301' (permanent) or '302' (temporary) |

### Creating a 301 redirect
```bash
./scripts/create_record.sh website.rewrite '{"name":"SEO redirect: old-slug to new-slug","url_from":"/blog/old-slug","url_to":"/blog/new-slug","redirect_type":"301"}'
```

**ALWAYS create a redirect when changing a URL/slug.**

---

## Images / Attachments

### Model: `ir.attachment`

| Field | Type | SEO Use |
|-------|------|---------|
| `name` | Char | File name |
| `description` | Text | Used as alt text in website |
| `alt_text` | Char | Alt text (Odoo 19) |
| `mimetype` | Char | Image type |
| `url` | Char | Public URL |

### Updating image alt text
```bash
# Find images without alt text
./scripts/search_records.sh ir.attachment '[["mimetype","like","image/"],["alt_text","=",false],["res_model","=","blog.post"]]' '["name","url"]' 50

# Update alt text
./scripts/update_record.sh ir.attachment '[ATTACHMENT_ID]' '{"alt_text":"Implementacion Odoo en empresa de alimentacion ecologica"}'
```

---

## Menu / Navigation

### Model: `website.menu`

| Field | Type | SEO Use |
|-------|------|---------|
| `name` | Char | Menu item text |
| `url` | Char | Link URL |
| `parent_id` | Many2one | Parent menu |
| `sequence` | Integer | Order |

---

## Change Protocol

Before any change:

```bash
# 1. Read current value
CURRENT=$(./scripts/search_records.sh [model] '[["id","=",ID]]' '["field_to_change"]')

# 2. Log to changelog
echo "{\"date\":\"$(date -Iseconds)\",\"model\":\"[model]\",\"id\":ID,\"field\":\"field_to_change\",\"before\":$CURRENT,\"after\":\"new_value\",\"reason\":\"SEO optimization\"}" >> data/changelog.json

# 3. Make the change
./scripts/update_record.sh [model] '[ID]' '{"field_to_change":"new_value"}'

# 4. Verify
./scripts/search_records.sh [model] '[["id","=",ID]]' '["field_to_change"]'
```

---

## Common Bulk Operations

### Find all pages missing meta descriptions
```bash
./scripts/search_records.sh website.page '[["website_meta_description","=",false],["is_published","=",true]]' '["name","url"]' 100
```

### Find all blog posts missing meta descriptions
```bash
./scripts/search_records.sh blog.post '[["website_meta_description","=",false],["is_published","=",true]]' '["name","seo_name"]' 100
```

### Find duplicate title tags
```bash
./scripts/search_records.sh blog.post '[["is_published","=",true]]' '["name","website_meta_title","seo_name"]' 500
# Post-process in Python to find duplicates
```

### Find thin content (need to check content length)
```bash
./scripts/search_records.sh blog.post '[["is_published","=",true]]' '["name","content","seo_name"]' 500
# Post-process: strip HTML, count words, flag < 300
```