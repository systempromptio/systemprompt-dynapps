# Performance — fast pages without losing design or builder compatibility

Checklist applied at step 10 (pre-handoff) and when authoring new patterns. Everything here is
builder-safe: standard HTML attributes and injected CSS/JS only — nothing `build_page_arch` strips.
Optimize delivery and scheduling, **never** by degrading the design (see the accordion rule below).

## 1. Images — the #1 lever

See `image-assets.md` §2c for the full rules. Summary:

- Upload optimized files (WebP / q80 JPEG; ≤200 KB content, ≤500 KB hero). `/web/image` serves
  API uploads as-is — it won't fix a heavy original.
- Serve resized variants when the original outsizes the slot: `/web/image/<id>/600x0`.
- `loading="lazy" decoding="async"` on everything below the fold, inside hidden `.tab-pane`s and
  off-screen carousel items. **Never** on the LCP/hero image (`fetchpriority="high"` there instead).
- Decorative images: `fetchpriority="low"`.
- Reserve space (`aspect-ratio` wrapper / sized column) → no CLS.

## 2. Animations — compositor-only, and stop when unseen

- Animate **`transform` and `opacity` only** (compositor; no layout/paint per frame). Exception:
  `s_iw_hacc` animates `flex-grow` because cards genuinely resize — that IS the design, its cost is
  contained to the row; do NOT replace it with `transform:scale` (distorts content).
- **Infinite animations must pause off-screen** (`s_iw_vmarquee`, `s_iw_globe`): each template ships
  a tiny IntersectionObserver that toggles `.iw-idle` → `animation-play-state:paused`. Identical on
  screen; zero cost scrolled away.
- `will-change` only where and while the effect can occur (e.g. inside the hover-capable media query,
  or on the single permanently-animating track) — a blanket `will-change` on N cards pins N compositor
  layers in GPU memory for nothing.
- Every animated pattern carries `@media (prefers-reduced-motion: reduce)` — function preserved
  (swap/reveal become instant), animation dropped.

## 3. JS — observe, don't poll

- **IntersectionObserver, never scroll/resize listeners** for scroll-driven behavior (all patterns
  comply). One-shot reveals `unobserve` after firing (`iw-rup`).
- One **delegated** listener per behavior (`document`-level, keyed on the hook class) serves every
  instance on the site — never one listener per element for repeatable blocks.
- Prefer **native mechanics over custom JS**: Bootstrap tabs (`data-bs-toggle`), CSS scroll-snap —
  already loaded on every Odoo frontend, zero added bytes.
- Every injected block is **inert if unused** (`if(!els.length) return`) so one site-wide footer
  block never taxes pages that don't use the pattern.

## 4. CSS/JS delivery

- Injected pattern CSS/JS goes in **once, site-wide** (`custom_code_footer` or `ir.asset` — see
  `css-injection-ladder.md`), so it's cached across pages. Don't re-inject per page; don't duplicate
  a block that's already in the footer (check first — GOTCHA 15 merge rule).
- Tier 3 (`ir.asset` in `web.assets_frontend`) gets minified and bundled by Odoo — prefer it once
  the injected code stops being trivial.
- Keep pattern snippets as small as they are — the whole interactive set is ~2 KB. If custom code
  grows past a screen, question the design (or it's module territory).

## 5. Verify (step 10)

Three quick checks before handing back:

1. **Weight/order** — heaviest image ≤ its slot + budget? LCP image eager, everything hidden or
   below-fold lazy? (`DevTools → Network → Img`, or Lighthouse.)
2. **Motion** — infinite animations pause off-screen? Reduced-motion honored? No scroll listeners?
3. **Stability** — no CLS: every image/visual slot reserves space; fonts/placeholders don't reflow.

Lighthouse (mobile) on the preview URL is the reference measurement; the budget mindset: a pattern
page should stay comfortably in the green on Performance without sacrificing any visual behavior.
