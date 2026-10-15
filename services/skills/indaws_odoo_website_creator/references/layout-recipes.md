# Layout recipes (builder-safe, no module)

Layout recipes proven in production (rehearsal on a copy of master) for the
edit-HTML → `build_page_arch.py` → push cycle. All with Bootstrap 5 + inline styles that **survive**
`build_page_arch.py` (which only strips `<script>`/`<style>`/`on*`/`position:fixed|sticky`/`data-oe-*`).

## 1. Icons: inline SVG (no Font Awesome, no module)

`build_page_arch.py` **preserves** `<svg>`, `<path>`, `<rect>`, `<circle>`, `<line>` with their attributes
(`viewBox`, `stroke-width`, etc.). It is the most reliable way to drop in a custom icon (magnifying glass, box…)
that matches a Figma, without depending on the instance's FA set or on a module.

```html
<a href="#" class="btn px-4 rounded-pill d-inline-flex align-items-center gap-2"
   style="background-color:#1D1C3B; border-color:#1D1C3B; color:#ffffff;">
  <span>Merchandising</span>
  <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24"
       fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"
       stroke-linejoin="round" aria-hidden="true">
    <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h6"></path>
    <circle cx="16" cy="14" r="3.4"></circle>
    <line x1="18.6" y1="16.6" x2="21" y2="19"></line>
  </svg>
</a>
```
- `stroke="currentColor"` → the icon inherits the button's `color` (white on navy, navy on outline).
- Close the tags (`<path ...></path>`) — the arch is XML; `--check` validates it.

## 2. Identical square cells (photos + placeholder) with `.ratio.ratio-1x1`

Typical problem: mixing `<img>` (with intrinsic size) and a placeholder `<div>` in the same grid
makes them **not measure the same** (the slot comes out larger/smaller). Solution: wrap **each** cell —
image or placeholder— in `.ratio.ratio-1x1`. Bootstrap forces the square box and the child fills it.

```html
<div class="ratio ratio-1x1"><img src="/web/image/123" class="rounded-4" style="object-fit:cover;"/></div>
<div class="ratio ratio-1x1"><div class="wc-ph rounded-4"></div></div>   <!-- same-size placeholder -->
```

## 3. Staggered (masonry) columns without fixed pixels

To shift a column half a cell down (responsive staggered effect), add a
**spacer with `aspect-ratio`** at the start of the column, instead of `margin-top` in px:

```html
<div class="col-6"><div class="d-flex flex-column gap-2"> ...celdas... </div></div>
<div class="col-6"><div class="d-flex flex-column gap-2">
  <div style="aspect-ratio:2/1;"></div>   <!-- = half a cell; scales with width -->
  ...celdas...
</div></div>
```
`aspect-ratio:2/1` → height = half the cell width. The stagger holds on any screen.

## 4. Controlling a grid's size without fighting the columns

Instead of shrinking the Bootstrap column (`col-lg-5`→`4`…), cap the inner grid's width and
align it. Each photo ≈ `max-width` ÷ number of columns − gutter:

```html
<div class="col-lg-5">
  <div class="row g-1 ms-auto" style="max-width:350px;">  <!-- ~170px per photo (2 cols) -->
```
`ms-auto` sticks it to the right; `mx-auto` centers it. Change only the `max-width` to scale everything.

## 5. Designing on the client's REAL theme (inherited palette)

On a restored DB, the theme classes (`text-primary`, `btn-primary`, `text-secondary`,
`bg-*`) resolve to the **client's palette**, which may be "inverted" relative to the Figma
(e.g. `o-color-1`=teal, `o-color-2`=navy). If you use the tokens, the title/button come out with the
color swapped. **Solution:** for one-off section colors, **hardcode the brand hex**
inline (`style="color:#1D1C3B"`) instead of the token — NEVER rewrite the global palette (it would break the
rest of the site, see GOTCHA #15). Always verify in the browser; don't assume the token's color.

## 6. Images = public `ir.attachment`; the ids are NOT portable

Upload each image as `ir.attachment` `{public:true, type:'binary', datas:<b64>}` and reference it via
`/web/image/<id>`. Idempotent: search by `name` before creating. **Deploy warning:** attachment ids are
**instance-specific** — the deploy-to-master script must **re-upload the
images and re-map the `/web/image/<id>`** in the arch (just as it does with the newsletter's `data-list-id`).

## 7. "Label + value" rows (pills, pricing table): STACK on mobile

A two-pill row (narrow label on the left + wide value on the right) built as `d-flex`
**overflows on mobile**: a fixed `min-width` on the label + `flex-grow-1` on the value do NOT wrap,
so a long value ("Depending on quantities and customization") runs off the screen (real failure in the
"Technology platform" table of a real Software page). The default `d-flex` is `flex-row` without
wrap at ALL widths.

**Fix — `flex-column` on mobile, `flex-md-row` at `md+`**, and make shrink/grow only apply from `md`:

```html
<div class="d-flex flex-column flex-md-row align-items-md-center gap-2 gap-md-3 mb-3">
  <div class="bg-white rounded-pill text-center py-2 px-3 flex-md-shrink-0" style="min-width:170px;">Production</div>
  <div class="bg-white rounded-pill text-center py-2 px-3 flex-md-grow-1">Depending on quantities and customization</div>
</div>
```
- Mobile (`<768px`): stacked, each pill at full width (the default `flex-column` with stretch) → nothing overflows; the `min-width` is easily met.
- `md+`: back to a row — `flex-md-shrink-0` pins the label to its `min-width`, `flex-md-grow-1` stretches the value.
- Key: use `flex-md-shrink-0`/`flex-md-grow-1` (with breakpoint), NOT `flex-shrink-0`/`flex-grow-1`
  (which apply at all widths and reintroduce the overflow). General rule: any `d-flex` row
  with fixed widths needs an explicit plan for `<768px` — stack or `flex-wrap`. ALWAYS verify
  the section in the mobile viewport.
