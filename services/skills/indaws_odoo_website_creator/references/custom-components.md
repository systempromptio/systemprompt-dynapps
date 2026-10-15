# Custom Components Without a Module — Feasibility Study

Can you ship a bespoke component without developing an Odoo module? **Partly yes.** The honest
answer is three tiers. Tiers N1 and N2 are fully doable via API (this skill's scope). N3 needs a
module — this skill detects it and hands off.

## The three tiers

| Tier | What it is | Module? | How |
|------|------------|---------|-----|
| **N1 — Embedded component** | A bespoke block (custom layout, card, hero) placed on a page. | **No** | Editor-safe HTML (Bootstrap 5 + your own classes) written directly into the page `arch`, wrapped as `<section class="s_… " data-snippet="s_…" data-name="…">` so it is selectable/movable/editable in the builder. Template: `templates/custom_component_section.html`. |
| **N2 — Reusable CSS / light styling** | Shared visual styling across pages. | **No (data only)** | `ir.attachment` (the CSS) + `ir.asset` record pointing to `web.assets_frontend`, OR per-page/site Custom Code. No scaffolding, no deploy. See `css-injection-ladder.md`. |
| **N3 — Palette snippet / interactive JS** | A reusable drag-and-drop snippet in the editor palette, with custom options, or a component with interactive JavaScript. | **Yes — module required** | Templates inheriting `website.snippets`, option components (in v19 the `BaseOptionComponent` architecture), assets registered at install, and for JS the v19 `Interaction`/Colibri framework. **Out of scope for pure-API.** |

## N1 in detail — the embedded custom component

This is the workhorse for "I want something that isn't a stock Odoo snippet." Rules that make it
behave like a native block:

1. Wrap as a `<section>` with a **unique `s_`-prefixed class**, `data-snippet` (same name), and a
   human `data-name`. The builder then treats it as a first-class block.
2. Build the internals with the **Bootstrap 5 grid** (`container`/`row`/`col-*`) → responsive by
   construction, no media queries needed.
3. Style with **utility classes + per-element `style`** attributes (N1-only), or shared CSS via
   **N2** when several pages reuse it.
4. **Never** put `<style>`/`<script>` inside the block (stripped in editable content) and **never**
   use `data-oe-*` (reserved by Odoo).

What N1 gives you: a custom-looking, responsive, editable-text block. What it does **not** give:
an entry in the snippet **palette**, custom snippet **options** (the right-panel controls), or
interactive JS. Those are N3.

## N2 in detail — shared CSS without a module

You do not need a module to add site-wide CSS. Two routes (full ladder in `css-injection-ladder.md`):
- **Custom Code**: inject `<style>` into the site/page head via the `website` / `website.page`
  Custom Code fields (exact field names: verify on instance).
- **`ir.asset` + `ir.attachment`**: store the CSS as an attachment served by URL and register an
  `ir.asset` (`bundle='web.assets_frontend'`, `directive='append'`) — reusable everywhere.
  Use `scripts/inject_css.py`.

JS via `ir.asset` is technically possible the same way, but **interactive component JS that hooks
into the editor/runtime belongs to N3** (it needs the proper framework registration). Keep N2 to
styling and trivial, self-contained scripts only — and even then, prefer a module if it grows.

## N3 — when to stop and hand off

Stop and tell the user a **module** is required when they ask for any of:
- A snippet that appears in the editor's **drag-and-drop palette** for reuse across pages.
- Custom snippet **options** (color/layout controls in the right panel).
- **Interactive behavior** driven by JavaScript (sliders with custom logic, live filters,
  fetch-driven widgets). In v19 this means the `Interaction`/Colibri framework + `public.interactions`
  registry; in v18 the legacy `publicWidget`.

Handoff message (English, to the user):

> That requires an Odoo module (reusable palette snippet or interactive JS), which cannot be done via
> API alone. Do you want me to hand it off to the development suite (`odoo-module-upgrader` /
> developer) to build it as a module?

Do **not** attempt to fake N3 via API (e.g., injecting editor JS through `ir.asset`): it is fragile,
breaks across versions, and fights the builder. Detect, explain, hand off.

## Decision guide

```
Need a bespoke block on a page?            → N1 (embedded section). No module.
Need that block's styling reused sitewide? → N2 (ir.asset / Custom Code). No module.
Need it in the snippet palette / options?  → N3. Module. Hand off.
Need interactive JS behavior?              → N3. Module. Hand off.
```
