# Interactive Patterns (no module) — indexed by BEHAVIOR

Reusable, **brand/content-agnostic** interactive behaviors you can drop into any page without building
a module. Pick by **behavior**; the visual content is a *slot* you fill (cards, images, a phone
mockup, a chart, plain text — anything). The behavior comes from a small CSS+JS snippet injected ONCE
site-wide (`website.custom_code_footer` or `ir.asset` — see `css-injection-ladder.md`); the JS uses
`querySelectorAll` / event delegation, so it serves every instance of the pattern on the site.

> **These are PATTERNS, not page instances.** Fill the slots with whatever the page needs. Do NOT
> save a page's filled-in version (a specific phone, a client's copy) back into the skill — that's an
> instance; keep it in the project/client repo. Only the agnostic pattern belongs here.

## Choose by behavior

| You want… | Hook class | Template | Behavior source |
|-----------|------------|----------|-----------------|
| A row of items that scrolls horizontally (swipe/trackpad/scrollbar), N visible, optional arrows | `s_iw_hscroll` | `templates/pattern_horizontal_scroll.html` | inline CSS (scroll-snap) + tiny JS for arrows |
| Scroll "pins" a stage and swaps content **and** a visual in place as you scroll (scrollytelling) | `s_iw_scrollswap` | `templates/pattern_scroll_pinned_swap.html` | injected CSS (sticky) + IntersectionObserver JS |
| A column (or row) of items that auto-scrolls in an endless loop (marquee / infinite scroll) | `s_iw_vmarquee` | `templates/pattern_vertical_marquee.html` | injected CSS only (duplicate the set + `translateY(-50%)`) |
| A decorative circular/sphere image shown as a **half dome** with a gentle spin/oscillation (globe, planet) | `s_iw_globe` | `templates/pattern_globe_dome.html` | injected CSS keyframes; **mask half** via `overflow:hidden` wrapper (height≈width/2) + **oscillate ±30°** (not full 360°) so a flat 2D render never reveals its blank face. True 3D globe = WebGL module |
| Pick an option from a list → a panel updates in place (tabs / selector, active item highlighted) | `s_iw_tabs` | `templates/pattern_tabs_panel.html` | **native Bootstrap tabs** (`data-bs-toggle="tab"`), no custom JS; injected CSS for the active-state look |
| Flip a 2-3 mode **segmented switch** (pill bar) → a full-width panel swaps (Monthly/Yearly, Picking/Shipping) | `s_iw_segtoggle` | `templates/pattern_segmented_toggle.html` | **native Bootstrap tabs** (horizontal pill variant of `s_iw_tabs`), no custom JS; injected CSS for the active pill. Panels may be identical now and diverge later |
| A row of narrow cards where **hover OR click expands** one (horizontal accordion); others shrink | `s_iw_hacc` | `templates/pattern_horizontal_accordion.html` | injected CSS (flex-grow anim) + tiny JS (`.active` on hover+click, revert to selected on mouseleave). Empty body = reserved slot; degrades to a vertical stack <768px |
| Items (cards/rows) that **fade + slide up one-by-one as you scroll** (reveal on scroll); optional **fixed column** beside them | `iw-rup` + `iw-sticky-col` | `templates/pattern_reveal_on_scroll.html` | injected CSS + one IntersectionObserver (`.iw-in` per element, `unobserve` after). `.iw-sticky-col` = injected `position:sticky` (never inline — build strips it). Inert with no `.iw-rup`; respects reduced-motion |
| A card that **grows + gains shadow on hover** (expand on hover, Apple-store feel); often inside a carousel | `iw-hx` | `templates/pattern_hover_expand.html` | injected CSS only (`:hover` scale + box-shadow). Desktop-only + reduced-motion aware; if inside `s_iw_hscroll`, pad the track so the scaled card isn't clipped. Inert with no `.iw-hx` |
| A bespoke static block that isn't a native snippet | your own `s_…` class | `templates/custom_component_section.html` | none (Tier N1) |
| Site-wide custom CSS/JS without a module | — | — | `css-injection-ladder.md` |

## Rules common to all patterns

- **Visual is agnostic.** The "phone" in any example is just one way to fill the visual slot — swap it
  for an `<img class="img-fluid">`, a card, a chart, etc. Never bake a specific visual into the pattern.
- **`position:sticky`/`fixed` go in INJECTED CSS, never inline** — `build_page_arch` strips inline
  sticky/fixed. It works in Odoo because `#wrapwrap` uses `overflow-x:clip` (not `hidden`).
- **Interactive JS** rides in Custom Code / `ir.asset` (no module). The JS is generic (keys on the hook
  class), so inject it once and reuse the pattern many times.
- **When a pattern needs editor options or a drag-drop palette entry → that's a module** (Tier N3, see
  `custom-components.md`). Detect and hand off; don't fake it via API.
- **Responsive contract** (see `responsive.md`): every pattern works at 390px — hover is never the
  only way to reach content (tap must work), tap targets ≥44px, fixed heights use `min(Npx,XXvh)`,
  and viewport-height stages carry a `100dvh` fallback for mobile Safari. Verify each instance in
  the mobile viewport before handing back.
- **Performance contract** (see `performance.md`): animate `transform`/`opacity` only (exception:
  `s_iw_hacc`'s flex-grow — that resize IS the design); infinite animations pause off-screen via the
  `.iw-idle` observer each template ships; `will-change` only where/while the effect can occur;
  IntersectionObserver, never scroll listeners; every block inert if unused; images inside hidden
  panes / off-screen items get `loading="lazy" decoding="async"`; all patterns respect
  `prefers-reduced-motion`.
- **`s_iw_scrollswap` (pin+swap) is ALL-DEVICE — keep the pin on every width; do NOT un-pin on mobile.**
  Making it fit small screens is done by SHRINKING the visual per breakpoint (`.iw-visual` max-width
  ~150→210→280), never by dropping the pin: any `@media` that sets `.iw-sticky-stage`/`.iw-step` to
  `static` breaks the responsive layout. The stage column must stretch (no `align-items-start` on the
  row) or the sticky has no travel. It works because `#wrapwrap` uses `overflow-x:clip`, so
  `position:sticky` holds at all widths. Still verify at `<768px` in the browser — validated all-device
  on a real home. See `pattern_scroll_pinned_swap.html`.

Each template's header documents the exact **contract** (classes / `data-*`) and the **CSS/JS to inject**.
Add new patterns here the same way: name the behavior, define a neutral hook class + contract, give the
inject snippet — keep it free of any specific page's content.
