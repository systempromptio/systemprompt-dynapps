# CSS Injection Ladder — custom CSS without a module

How to add custom CSS to a website page **without scaffolding an Odoo module**.
Three tiers, simplest/safest first. The skill tries them in order and verifies
which one works on the target instance.

> Why a ladder at all: see §"Why inline `<style>` gets stripped" below. Short
> answer — `<style>` tags inside editable snippet content are removed by the
> editor, so custom CSS must live *outside* the page arch.

---

## Tier 1 — Utility classes + per-element `style` attributes (no external CSS)

Covers the **majority** of cases. No external CSS file, nothing to register.

- Prefer Bootstrap 5 utilities and Odoo spacing classes (`pt40`, `d-flex`,
  `text-center`, `rounded`, `bg-*`, `gap-*`, `col-lg-*`).
- For one-off colors/sizes the editor preserves the inline `style` attribute on
  an element:

```html
<section class="s_text_block pt40 pb40" style="background-color:#0d1b3e;">
  <div class="container">
    <h2 class="text-white" style="letter-spacing:.02em;">Title</h2>
  </div>
</section>
```

- The `style` **attribute** on an element survives the editor; a `<style>`
  **element** does not. Keep selectors out of the page entirely.
- This is the only tier needed for most "N1" embedded components.

**When to stop here:** the styling is local to a few elements on one page and can
be expressed with utilities + inline `style`.

---

## Tier 2 — Per-page / per-site Custom Code

Odoo can inject raw `<head>` / before-`</body>` code per website and per page,
without touching content. This is where to put a small `<style>` block (or a
`<link>`) that should not live inside the editable arch.

| Scope | Model | Typical fields | Notes |
|---|---|---|---|
| Per page | `website.page` | head-injection field + body-injection field | Field names like `website_meta_*` are stable; the custom-code field names are **verify on instance**. |
| Whole site | `website` | site-level head/body custom-code fields | Field names **verify on instance** (commonly named around `custom_code_head` / `custom_code_footer`, but confirm before writing). |

Update via odoo-pilot:

```bash
# Per-page <head> injection (CONFIRM the exact field name on the instance first)
./update_record.sh website.page '[<page_id>]' \
  '{"<head_custom_code_field>": "<style>.my-hero{background:#0d1b3e}</style>"}'
```

- Scope is exactly one page (or the whole site) — good for page-specific tweaks.
- The injected `<style>` lives in the document `<head>`, outside editable content,
  so the editor does not strip it.
- **Verify the exact field names** on the target instance (search `website.page`
  / `website` fields) instead of assuming; mark them "verify on instance".

**When to use:** one page (or one site) needs a handful of CSS rules that don't
fit Tier 1, and you don't need reuse via the asset pipeline.

---

## Tier 3 — `ir.asset` + `ir.attachment` (frontend bundle, reusable, no module)

The "correct bundle" route: store the CSS in an attachment served by URL, then
register an `ir.asset` that appends it to the frontend bundle. Reusable site-wide,
still **no module scaffolding**.

### Step 1 — create the CSS attachment

```bash
# Encode the CSS file to base64, then create the attachment:
CSS_B64=$(base64 -i custom.css | tr -d '\n')
./create_record.sh ir.attachment '{
  "name": "custom_page.css",
  "type": "binary",
  "datas": "'"$CSS_B64"'",
  "mimetype": "text/css",
  "public": true
}'
# → returns <attachment_id>; the file is served at /web/content/<attachment_id>
```

### Step 2 — register the asset in the frontend bundle

```bash
./create_record.sh ir.asset '{
  "name": "custom_page_css",
  "bundle": "web.assets_frontend",
  "path": "/web/content/<attachment_id>",
  "directive": "append",
  "active": true
}'
```

| `ir.asset` field | Meaning | Example |
|---|---|---|
| `name` | Human-readable identifier. | `"custom_page_css"` |
| `bundle` | Which asset bundle to extend. | `"web.assets_frontend"` (public site) |
| `path` | URL/path of the asset to add. | `/web/content/<attachment_id>` |
| `directive` | How to add it. | `"append"` (also `prepend`, `before`, `after`, `replace`, `remove`) |
| `target` | Reference path for `before`/`after`/`replace`. | only needed for those directives |
| `active` | Enable/disable without deleting. | `true` |

- After creating the asset, the frontend bundle must regenerate (a fresh page
  load / cache bust). Verify the CSS actually loads on the live page.
- Reusable across every page on the site — best when several pages share styling.

**When to use:** the CSS is shared across pages, or large enough that you want it
as a real, cacheable, version-controllable asset.

---

## Why inline `<style>` inside editable content gets stripped

The Website Builder sanitizes editable snippet content on save. `<style>` and
`<script>` elements (and `on*` handlers, raw `<iframe>`, `@import` in body) are
removed because the editor only round-trips a safe subset of HTML and reorganizes
the DOM. A `<style>` block placed inside `#wrap.oe_structure` will therefore
vanish on the next edit/save. The ladder keeps CSS **outside** the editable arch:
inline `style` attributes (Tier 1) survive, and Tiers 2–3 inject into `<head>` /
the bundle where the editor never touches it.

---

## Decision guide

| Situation | Tier |
|---|---|
| A few elements, one page, simple colors/spacing. | **Tier 1** (utilities + `style`). |
| Page- or site-specific rules that need real CSS selectors, no reuse. | **Tier 2** (Custom Code). |
| CSS shared across many pages, or large/maintainable stylesheet. | **Tier 3** (`ir.asset` + `ir.attachment`). |
| Draggable palette snippets, SCSS theme overrides, interactive JS. | Out of scope — needs a module; detect and hand off to the dev suite. |
