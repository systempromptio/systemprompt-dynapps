# Snippet Catalog — Odoo native `s_*` website snippets

A **map** of Odoo's built-in website building blocks. This is *not* a copy of
`odoo-frontend`'s `iw-*` component HTML — `odoo-frontend` owns the actual markup
generation. This file tells you **what each native snippet is, when to reach for
it, and how to wrap your generated content** so the Website Builder recognizes it
section by section.

> Source of truth: `odoo-frontend/references/odoo-editor-rules.md` + Odoo docs.
> v19-specific grid behavior is marked **"verify per version"**.

---

## 1. How a snippet is structured

Every snippet is a `<section>` carrying the snippet class, optional Odoo spacing,
and the editor metadata attributes, then a Bootstrap container/row/col tree:

```html
<section class="s_text_block pt40 pb40" data-snippet="s_text_block" data-name="Text">
  <div class="container">
    <div class="row">
      <div class="col-lg-6">
        <!-- generated content -->
      </div>
      <div class="col-lg-6">
        <!-- generated content -->
      </div>
    </div>
  </div>
</section>
```

| Attribute | Purpose | Required? |
|---|---|---|
| `class="s_xxx ..."` | Identifies the snippet type to the builder + applies its base CSS. | Yes |
| `data-snippet="s_xxx"` | Tells the editor which palette block this is (selectable/draggable). | Yes (for editability) |
| `data-name="Human Name"` | Label shown in the editor's block tree. | Recommended |
| `pt40 pb40` (or similar) | Odoo vertical spacing (see §3). | Optional |
| `.container` / `.container-fluid` | Bootstrap centering vs full-width. | Yes inside the section |

Without `data-snippet` + the `s_*` class, the section renders but the builder
treats it as opaque HTML rather than an editable block.

---

## 2. The common `s_*` snippets — what & when

| Snippet class | What it is | Use it when… |
|---|---|---|
| `s_banner` | Full-width hero banner with background + heading + CTA. | The page's opening hero / above-the-fold statement. |
| `s_cover` | Full-viewport cover (similar to banner, taller, often centered). | A landing page that opens with one big focal message. |
| `s_text_block` | Generic centered text section (title + paragraphs). | Plain prose: intro, mission, terms, any text-only block. |
| `s_text_image` | Text on the left, image on the right. | Explaining a feature/product alongside a visual (text-first). |
| `s_image_text` | Image on the left, text on the right. | Same as above but image-first (alternate rhythm down the page). |
| `s_three_columns` | Three equal columns (icon/title/text cards). | Three parallel items: services, steps, value props. |
| `s_features` | Feature grid (icons + short descriptions). | Listing product/service features in a responsive grid. |
| `s_numbers` | Big statistic counters (e.g. "+500 clientes"). | Showcasing metrics / proof points. |
| `s_call_to_action` | Banner with headline + prominent button. | Mid-page or end-of-page conversion prompt. |
| `s_references` | Client logos / reference strip. | Social proof via brand logos. |
| `s_comparisons` | Comparison / pricing table columns. | Pricing tiers or plan/feature comparisons. |
| `s_quotes_carousel` | Rotating testimonials carousel. | Customer quotes / testimonials. |
| `s_company_team` | Team member cards (photo + name + role). | "Meet the team" / about-us people section. |
| `s_website_form` | A form whose submission creates a record (e.g. `crm.lead`). | Contact / lead-capture forms. See `website-forms-to-leads.md`. |

> Choose the snippet whose **semantics** match the content, then let
> `odoo-frontend` fill the inner container/row/col with the styled markup.
> When in doubt, `s_text_block` (text) or `s_features` (grid) are safe generic
> fallbacks.

---

## 3. Odoo spacing classes

Odoo replaces ad-hoc margins with a fixed vertical-rhythm scale applied on the
`<section>`. Top padding `ptN`, bottom padding `pbN`:

```
pt8  pt16 pt24 pt32 pt40 pt48 pt64 pt80 pt96
pb8  pb16 pb24 pb32 pb40 pb48 pb64 pb80 pb96
```

- These map to pixel values and survive the editor.
- `pt40 pb40` is a sensible default for a standard content section.
- Use larger steps (`pt64`/`pt96`) for hero/cover sections, smaller for tight
  stacks. Do **not** invent values outside the scale (e.g. `pt50`).
- Bootstrap spacing utilities (`p-*`, `m-*`, `px-*`, `py-*`, `mt-*`, `mb-*`) are
  also safe and are used *inside* the section for fine-grained spacing.

---

## 4. Responsiveness — the Bootstrap 5 grid

Odoo is Bootstrap 5 native. Responsiveness comes from the **12-column grid**, not
from absolute CSS. A `.row` holds `.col-*` children; breakpoints control when
columns stack:

| Class prefix | Breakpoint (min-width) | Typical use |
|---|---|---|
| `col-` | always (xs, all widths) | Base column count. |
| `col-sm-` | ≥576px | Small phones landscape / large phones. |
| `col-md-` | ≥768px | Tablets. |
| `col-lg-` | ≥992px | Laptops/desktops. |
| `col-xl-` | ≥1200px | Wide desktops. |

```html
<div class="row">
  <!-- 1 col on mobile, 2 on tablet, 3 on desktop -->
  <div class="col-12 col-md-6 col-lg-4">…</div>
  <div class="col-12 col-md-6 col-lg-4">…</div>
  <div class="col-12 col-md-6 col-lg-4">…</div>
</div>
```

Rules of thumb:
- Always give a base `col-12` (or `col-`) so blocks stack cleanly on mobile, then
  widen at `md`/`lg`.
- Column numbers in a row should sum to 12 (or wrap intentionally).
- Use responsive display utilities for show/hide: `d-none d-lg-block`,
  `d-flex flex-column flex-md-row`, `gap-*`.
- Never rely on `position: fixed`/`sticky` or pixel widths for layout — they break
  in the editor and aren't responsive.

---

## 5. v19 optional grid mode — verify per version

Odoo 19 adds an **optional grid mode** for free column positioning. When a column
layout is converted to grid mode the markup gains:

- `o_grid` on the row-equivalent container,
- `o_grid_item` on each positioned child (with grid placement metadata).

Notes:
- This is **opt-in**. The standard `container` → `row` → `col-*` Bootstrap layout
  remains fully valid and is the **portable** choice across v18 and v19.
- Use grid mode only when free positioning is explicitly required; otherwise stick
  to the responsive grid above.
- Exact attribute/class names and placement metadata for grid mode are
  version-specific — **verify per version** via `odoo-version-api-reference`
  (`references/diffs/v19-diff.md`) before relying on them.
