# Image Assets — uploading and referencing images on website pages

How images get from a local file (or data-URI) into an Odoo website page. Pattern:
**upload to `ir.attachment` → reference by `/web/image/<id>` in the page arch**.

> Web-page rule (NOT the email rule): website images use the `img-fluid` class and
> are served via `/web/image/`. Do **not** put inline `width`/`height`/`style`
> sizing on web images — that is the *email* convention. The mailing-creator uses
> absolute URLs + inline `width`/`style`; web pages use `/web/image/` +
> `img-fluid`. Same upload mechanism, different `src` and sizing rules.

---

## 1. Upload a local / data-URI image to `ir.attachment`

Base64-encode the file and create the attachment via odoo-pilot:

```bash
IMG_B64=$(base64 -i hero.png | tr -d '\n')
./create_record.sh ir.attachment '{
  "name": "hero.png",
  "type": "binary",
  "datas": "'"$IMG_B64"'",
  "mimetype": "image/png",
  "public": true
}'
# → returns <attachment_id>
```

| `ir.attachment` field | Value | Why |
|---|---|---|
| `name` | Original filename, e.g. `"hero.png"`. | Identifies the file. |
| `type` | `"binary"`. | Stored data (vs `"url"`). |
| `datas` | Base64 of the file bytes (no newlines). | The image payload. |
| `mimetype` | `"image/png"`, `"image/jpeg"`, `"image/webp"`, `"image/svg+xml"`. | Correct content-type when served. |
| `public` | `true`. | So the image is readable by anonymous site visitors. |
| `res_model` / `res_id` | Optional association (see §4). | Ties the file to the page. |

A data-URI image (`data:image/png;base64,…`) is already base64 — strip the
`data:…;base64,` prefix and use the remainder directly as `datas`.

---

## 2. Reference the image in the page arch

Two equivalent URL forms once you have the attachment id:

```html
<!-- By attachment id -->
<img src="/web/image/<attachment_id>" class="img-fluid" alt="Team photo"/>

<!-- By model/id/field (works for any binary field, e.g. a record's image) -->
<img src="/web/image/<model>/<id>/<field>" class="img-fluid" alt="Producto"/>
```

Always:
- add the `img-fluid` class → responsive (scales to its container, never overflows);
- add a meaningful `alt` (accessibility + SEO);
- do **not** add inline `width`/`height`/`style` for sizing — let `img-fluid` +
  the Bootstrap grid column control the size (use `col-*` widths, `rounded`,
  `w-100`, etc. via classes instead).

---

## 2b. When you DO need to cap size — use `min(100%, Npx)`, never a bare `max-width`

`img-fluid` + the grid column is the default and handles most cases. But some layouts genuinely need
to cap an image (or an image *grid*) at a pixel width — a hero photo cluster, a phone mock, a badge
strip that shouldn't stretch full-column. The trap: a **bare** `max-width:460px` (or a fixed
`width:440px`) looks right on desktop but **overflows / clips on narrow phones**, because 460px is
wider than the viewport and the image no longer shrinks.

**Fix: cap with `min(100%, Npx)`** so the image never exceeds `Npx` *and* never exceeds its container:

```html
<!-- overflows on a 390px phone -->  <img src="/web/image/123" style="max-width:460px;"/>
<!-- responsive: ≤460px AND ≤100% -->  <img src="/web/image/123" class="img-fluid" style="max-width:min(100%,460px);"/>
```

Same idea for a **grid** of images: cap the *row* wrapper, not each cell — `style="max-width:min(100%,350px);"`
on the inner `.row`, then `ms-auto`/`mx-auto` to align (see `layout-recipes.md` §4). And to make an
image fully fill a fixed-height half (e.g. a newsletter block's left column): `class="w-100 h-100"` +
`style="object-fit:cover;min-height:400px;"` — `object-fit:cover` survives `build_page_arch`.

> This is the ONE sanctioned use of inline `max-width`/`object-fit` on a web image. It's not the email
> convention (fixed `width`); it's a *responsive cap*. Still add `img-fluid` + `alt`. Always check the
> mobile breakpoint in the browser — a cap that looks fine on desktop is exactly where mobile breaks.

---

## 2c. Performance — weight, delivery and loading order

Images are almost always the #1 page-weight item. Three levers, in order of impact:

**1. Upload light files.** `/web/image/<id>` serves the stored binary as-is — Odoo does NOT
re-compress an API upload (the automatic WebP conversion only happens in the editor's media
dialog). So optimize BEFORE uploading: prefer **WebP** (or quality-80 JPEG for photos, SVG for
logos/icons); target **≤ 200 KB** for content images and **≤ 500 KB** for a full-bleed hero.
Never upload a 4000px original for a 400px slot.

**2. Serve resized variants.** The `/web/image` controller resizes on the fly and caches the
result — append the box size to the URL:

```html
<!-- full-size original -->        <img src="/web/image/123" class="img-fluid" alt="…"/>
<!-- resized server-side ≤600px --> <img src="/web/image/123/600x0" class="img-fluid" alt="…"/>
```

`<width>x<height>` fits the image inside that box keeping aspect ratio; `0` = unconstrained side
(`600x0` = max 600px wide). Use it whenever the original is much larger than the rendered slot
(card thumbnails, logo strips, list items). Stable across v18/v19 — verify once per instance.

**3. Load in the right order.**

| Image | Attributes |
|---|---|
| Hero / LCP image (first viewport) | eager (no `loading`), consider `fetchpriority="high"` |
| Below the fold, hidden `.tab-pane`, off-screen carousel items | `loading="lazy" decoding="async"` |
| Decorative (globe, background flourish) | `decoding="async" fetchpriority="low"` (+ `loading="lazy"` if below fold) |

These are standard HTML attributes — the Website Builder preserves them and `build_page_arch`
doesn't strip them. **Never lazy-load the LCP image**: that directly worsens the LCP score.

**CLS**: every image/visual slot must reserve its space — `aspect-ratio` on the wrapper (the
pattern templates already do this) or a sized grid column. A page that reflows as images arrive
fails Core Web Vitals even if it loads fast.

---

## 3. Rewriting `src` from local paths to Odoo URLs

When "Odoo-ifying" a reference HTML or a brief-generated page, every local /
data-URI image must be rewritten:

1. Collect each unique local path or data-URI in the source markup.
2. Upload each once → get its `<attachment_id>` (build a `local_path → URL` map;
   reuse the same attachment for repeated paths — don't upload duplicates).
3. Replace each `src` with `/web/image/<attachment_id>`.
4. Ensure the resulting `<img>` has `img-fluid` + `alt`.

This keeps the page arch small (no giant base64 blobs inline) and lets Odoo serve,
cache, and resize the images.

---

## 4. Association to the page (`res_model` / `res_id`)

Attachments can be associated to the record they belong to so they're tracked and
cleaned up with it:

- Associate to the page's view: `res_model="ir.ui.view"`, `res_id=<view_id>`.
- Or to the `website.page`: `res_model="website.page"`, `res_id=<page_id>`.

This is optional for rendering (the `/web/image/<id>` URL works regardless once
`public=true`), but recommended for housekeeping. The exact convention can vary by
instance — **verify on the target instance** if association matters for cleanup.

---

## 5. Helper script

`scripts/upload_assets.py` automates the batch: it takes a list of local image
paths, uploads each to `ir.attachment` (`type="binary"`, `public=true`, correct
`mimetype`), and emits a `local_path → /web/image/<id>` map that the arch builder
uses for `src` rewriting. It shells out to odoo-pilot's `create_record.sh` and
**never prompts for credentials** (those are exported via odoo-pilot first). When
running over MCP instead, call the MCP `create_record` tool with the same
arguments rather than the script.
