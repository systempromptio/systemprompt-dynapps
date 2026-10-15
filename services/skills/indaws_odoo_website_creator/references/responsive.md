# Responsive — every page must work on a phone

The guarantee: anything this skill publishes renders correctly at **390px** (phone), **768px**
(tablet) and desktop, with no horizontal overflow, no clipped content and touch-usable controls.
Enforced three ways: the mobile-first authoring contract below, the automatic lint in
`build_page_arch.py` (runs on every build and `--check`), and the Responsive check in step 10.

## What Odoo already gives you (don't fight it, don't bypass it)

- `website.layout` ships the `viewport` meta — one more reason the wrapper is mandatory.
- `#wrapwrap` uses `overflow-x:clip` → small overflows don't create a sideways scrollbar, they get
  **clipped invisibly**. That masks the symptom, not the bug: clipped content is lost content.
- Bootstrap 5 with RFS: `display-*` headings scale down on small screens by themselves.
- The Website Builder's device toggle writes `d-none d-md-block`-style classes — they survive the
  build and are the sanctioned way to show/hide per device (used by `pattern_globe_dome`).

## Mobile-first grid contract

- Structure every section as `container > row > col-*`. Columns are **full-width on mobile by
  default** (any `col-md-*`/`col-lg-*` is 100% below its breakpoint) — so the mobile layout is the
  *default*, and breakpoints only *upgrade* it. Never author desktop-first.
- Control stacking order with `order-*` when mobile order ≠ desktop order (e.g. visual above text:
  `order-1 order-md-2`, as `pattern_scroll_pinned_swap` does).
- Spacing: `g-*` gutters and responsive spacers (`p-4 p-lg-5`, `gap-2 gap-md-3`) — not fixed px.

## The overflow traps (the lint catches all of these)

| Trap | Why it breaks a 390px phone | Fix |
|---|---|---|
| Bare `max-width:460px` on an `<img>` / fixed `width:440px` | Intrinsic size wider than the viewport → clipped (a bare cap on an auto-width text block is fine) | `min(100%,460px)` (image-assets.md §2b) |
| `min-width:NNNpx` + `flex-shrink-0` in a `d-flex` row | The row can't compress → runs off | Stack on mobile: `flex-column flex-md-row`, breakpointed `flex-md-shrink-0` (layout-recipes.md §7) |
| `100vw` | Includes the scrollbar → always overflows | `100%` / `w-100` |
| `100vh` | Mobile Safari counts the collapsed URL bar → taller than the visible screen | Add a fallback pair: `min-height:calc(100vh - X); min-height:calc(100dvh - X);` |
| `<table>` unwrapped | Tables don't shrink below content width | `.table-responsive` or an `overflow-x:auto` wrapper |
| `white-space:nowrap` on long text | Can't wrap → overflows | Only on short labels, never body copy |
| Fixed heights (`height:540px`, `min-height:520px`) | Dominates or overflows a short screen | `min(540px,70vh)` / `min(520px,66vw)` — `min()`/`clamp()` inline survive the build |

## Touch

- Tap targets **≥ 44×44px** (the carousel arrows are exactly 44px — the floor, not a suggestion).
- **Hover is an enhancement, never a gate**: content must be reachable without hover.
  `iw-hx` (hover-expand) is decorative-only ✓; `s_iw_hacc` also activates on click/tap ✓. A new
  pattern that reveals content on hover MUST also handle click.
- Native scrolling beats JS: scroll-snap tracks (`s_iw_hscroll`) swipe correctly by construction.

## Verify (step 10) — in the browser, not by reading the arch

1. Preview at **390px** (DevTools device toolbar or the builder's mobile toggle): no sideways
   scroll, nothing clipped, nothing overlapping. `#wrapwrap`'s clip hides overflow — so *look* for
   cut-off pills/images/tables, don't just scroll.
2. Every interactive pattern works by touch (tap = click) and the page is readable without hover.
3. `build_page_arch.py --check <arch>` reports zero `responsive:` warnings — or each remaining one
   is consciously accepted (e.g. a `nowrap` on a 2-word label).

Fixed-width DESIGNS from a Figma/screenshot (mode C) are desktop artboards: reproduce the look,
but translate fixed px into the contract above — the client approved a design, not an overflow.
