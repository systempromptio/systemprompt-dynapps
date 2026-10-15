# Builder-Safe Transformations

The website analog of the mailing skill's "email-safe transformations". Turns arbitrary HTML
(a design file, or HTML generated from a brief/image) into a QWeb `arch` that renders, is
**100% responsive**, and stays **editable block-by-block** in Odoo's Website Builder.

`scripts/build_page_arch.py` applies the deterministic parts; the rest is judgment guided here.

## The transformation table

| # | Transformation | Why |
|---|----------------|-----|
| 1 | Strip `<!DOCTYPE>`, `<html>`, `<head>`, `<body>` — keep only the fragment. | The arch is a fragment inside `website.layout`. |
| 2 | Wrap the whole in `<t t-call="website.layout"><div id="wrap" class="oe_structure">…</div></t>`. | Header/footer + editable drop zone. Without `#wrap.oe_structure` the page is not editable. |
| 3 | Make each top-level block a `<section class="s_… pt40 pb40" data-snippet="s_…" data-name="…">`. | Builder selects/moves/deletes per block. Map to the nearest native `s_*` (see `snippet-catalog.md`); for bespoke blocks use a `s_`-prefixed custom name (see `custom-components.md`). |
| 4 | Ensure each section uses the Bootstrap 5 grid: `container` → `row` → `col-*` (with `col-md-*`/`col-lg-*` breakpoints). | Responsiveness is delivered by the grid, not absolute CSS. |
| 5 | Remove `<script>` and JS event attributes (`onclick`, `onload`, …). | Stripped by the editor/sanitizer; security. Interactive JS needs a module. |
| 6 | Remove bare `<iframe>` (videos/maps) → use the Bootstrap `ratio` wrapper for YouTube, or flag for the media dialog. | Bare iframes are restricted; `ratio ratio-16x9` is responsive and safe. |
| 7 | Remove `position:fixed` / `position:sticky`. | Work in frontend but break editor mode. |
| 8 | Extract `<style>` blocks and external CSS — do NOT leave `<style>` in editable content. | `<style>`/`@import` inside snippet content is stripped. Re-home via the CSS injection ladder. |
| 9 | Extract local/data-URI images → upload to `ir.attachment`, rewrite `src` to `/web/image/<id>`. | Keeps the arch small; images served by Odoo. See `image-assets.md`. |
| 10 | Force `img-fluid` + `alt` on every `<img>`. | Responsive images + accessibility. |
| 11 | Drop `data-oe-*` attributes from the source. | Reserved by Odoo; collisions corrupt editor behavior. |
| 12 | Keep only editor-friendly markup (Bootstrap utilities + your own classes); avoid deeply nested exotic selectors. | The editor may restructure the DOM; simple structure survives. |
| 13 | Escape for XML/QWeb: `&` → `&amp;` in text/attributes; ensure the arch is valid XML. | The arch is parsed as XML; an unescaped `&` breaks the view. |

## Allowed building blocks (quick list)

- **Block:** `div, section, p, h1–h6, ul, ol, li, table, blockquote, figure, hr, br, pre`
- **Inline:** `span, a, strong, em, u, s, small, sub, sup, img, br`
- **Bootstrap 5:** grid (`container`, `row`, `col-*`, `offset-*`), spacing (`p-*`, `m-*`, `pt-*`…),
  flex (`d-flex`, `flex-*`, `align-items-*`, `justify-content-*`, `gap-*`), text (`text-center`,
  `fw-bold`, `text-uppercase`), display (`d-none`, `d-md-block`), sizing (`w-100`, `h-100`),
  border/radius, `ratio ratio-16x9`, `img-fluid`.
- **Odoo spacing:** `pt8…pt96`, `pb8…pb96` (preserved by the editor).

## Mapping content → snippets (the "Odoo-ification")

Walk the source top to bottom and map each visual section to the nearest native snippet:

| Visual section | Snippet class |
|---|---|
| Full-width hero / banner | `s_banner` (or `s_cover`) |
| Plain text block | `s_text_block` |
| Text + image side by side | `s_text_image` / `s_image_text` |
| 3 columns / feature grid | `s_three_columns` / `s_features` |
| Stats / numbers | `s_numbers` |
| Testimonials | `s_quotes_carousel` |
| Logos / references | `s_references` |
| Pricing / comparison | `s_comparisons` |
| Call to action | `s_call_to_action` |
| Lead form | `s_website_form` (see `website-forms-to-leads.md`) |

When nothing fits, emit a custom embedded section (Tier N1) — see `custom-components.md`.

## What `build_page_arch.py` does vs. what you do

- **Script (deterministic):** strip scaffolding; remove `<script>`/`on*`/`position:fixed|sticky`;
  add `img-fluid`+`alt`; drop `data-oe-*`; XML-escape; wrap in `website.layout` + `#wrap.oe_structure`;
  optionally wrap loose top-level blocks in `<section data-snippet data-name>`.
- **You (judgment):** choose the right `s_*` per section; restructure to a clean grid; decide
  which styles become utility classes vs. `ir.asset` CSS.

## Verify after building

The arch must: parse as XML, contain exactly one `#wrap.oe_structure`, contain `t-call="website.layout"`,
contain no `<script>`/`<style>`, and have at least one `<section data-snippet>`.
`build_page_arch.py --check <file>` reports these.
