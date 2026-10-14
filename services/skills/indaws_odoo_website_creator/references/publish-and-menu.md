# Publish Lifecycle, Menus, SEO & Multi-website

What happens after the page arch and `website.page` record exist: publishing,
SEO metadata, optional menu entry, multi-website scoping, and collision handling.

---

## 1. Publish lifecycle — pages are created UNPUBLISHED

The skill **always** creates the page with `is_published=false`, like the
mailing-creator leaves mailings in draft. Publishing is an **explicit user action**.

```bash
# Created unpublished (default):
#   website.page { ..., "is_published": false }

# Publishing later (explicit, on user confirmation):
./update_record.sh website.page '[<page_id>]' '{"is_published": true}'

# Unpublish / take down:
./update_record.sh website.page '[<page_id>]' '{"is_published": false}'
```

- An unpublished page is reachable in **edit mode** by a logged-in editor but
  returns 404 to anonymous visitors.
- Never auto-publish. Confirm with the user first, then flip the flag.

---

## 2. SEO fields on `website.page`

Set these at create time (or via a follow-up `write`). They drive the `<title>`,
meta description, keywords, and indexing of the public page:

| Field | Purpose |
|---|---|
| `website_meta_title` | `<title>` / SEO title. |
| `website_meta_description` | Meta description shown in search snippets. |
| `website_meta_keywords` | Meta keywords (legacy; low SEO weight but supported). |
| `website_indexed` | Whether search engines may index the page (robots). |

```bash
./update_record.sh website.page '[<page_id>]' '{
  "website_meta_title": "Custom Odoo Services | My Company",
  "website_meta_description": "We implement Odoo tailored to your business. Request a demo.",
  "website_meta_keywords": "Odoo, ERP, implementation",
  "website_indexed": true
}'
```

These field names are stable across v18/v19. If a write rejects one,
**verify on the instance**.

---

## 3. Optional menu entry — `website.menu`

To make the page reachable from the site navigation, create a `website.menu` row
pointing at it:

| Field | Purpose | Notes |
|---|---|---|
| `name` | Menu label shown in the navbar. | e.g. `"Services"`. |
| `url` | The link target. | Usually the page's `url` (e.g. `/services`). |
| `parent_id` | Parent menu (for sub-menus / dropdowns). | omit/false for a top-level item. |
| `sequence` | Order among siblings. | lower = earlier. |
| `website_id` | Which website the menu belongs to. | see §4. |
| `page_id` | Direct link to the `website.page` record. | field name **verify on instance**; if absent, link via `url`. |

```bash
./create_record.sh website.menu '{
  "name": "Services",
  "url": "/services",
  "parent_id": false,
  "sequence": 20,
  "website_id": <website_id>
}'
```

Creating a menu is optional — a page works without one (reachable by URL). Offer
it; default to **not** adding a menu unless the user asks.

---

## 4. Multi-website — `website_id` matters

- If the instance has multiple websites, set `website_id` on the page (and menu)
  to the **correct** website. Resolve the target website id first
  (search `website`).
- If `website_id` is **null/false**, the page is **shared across all websites**.
  Only do this deliberately.
- Mismatched `website_id` between the page and its menu leads to a menu item that
  points at a page the visitor can't see on that site.

---

## 5. URL / key collision handling (idempotency)

Before creating, check that neither already exists:

- The page **`url`** (e.g. `/services`) — search `website.page` for the same
  `url` (scoped to the same `website_id`).
- The view **`key`** on `ir.ui.view` (e.g. `website.servicios`) — must be unique.

If either collides, do **not** silently overwrite. Offer the user:
1. a **suffix** (`/services-2`, key `website.services_2`), or
2. an explicit, confirmed **overwrite** of the existing page/view.

This keeps re-runs safe and avoids clobbering live content.

---

## 6. What to return to the user

After creating the (unpublished) page, return both URLs so the user can review
then publish:

- **Edit URL** (builder, requires login):
  `<ODOO_URL>/odoo/website` → open the page, or `<ODOO_URL><page_url>?enable_editor=1`
  (the exact edit-deep-link form can vary by version — **verify on instance**).
- **Public preview URL**: `<ODOO_URL><page_url>` (e.g. `https://my-odoo.com/services`)
  — note it 404s for anonymous visitors until `is_published` is set to `true`.
