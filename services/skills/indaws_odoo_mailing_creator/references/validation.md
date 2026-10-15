# Validation (no browser)

Layered validation from cheap/automatic to manual. Steps 1–4 are scripted gates; step 5 is human. **No Playwright/Selenium/headless browser, no automated screenshots** (project rule).

## Table of contents
- [1. MJML build validation](#1-mjml-build-validation)
- [2. Static fragment validation](#2-static-fragment-validation)
- [3. Weight / Gmail clipping](#3-weight--gmail-clipping)
- [4. Optional: caniemail compatibility lint](#4-optional-caniemail-compatibility-lint)
- [5. Human visual QA checklist](#5-human-visual-qa-checklist)
- [6. Client engine cheat-sheet](#6-client-engine-cheat-sheet)

---

## 1. MJML build validation
`compile.cjs` runs MJML with `validationLevel:'strict'` — it fails on bad component nesting (e.g. `mj-text` directly under `mj-section`). Free first gate.

## 2. Static fragment validation
`validate.cjs <fragment.odoo.html>` (exit 0 required). It checks:
- weight vs limit (see below), forbidden tags (`script/iframe/object/embed/form/video/audio`),
- the `o_layout` wrapper is present, the file is a fragment (no `<html>/<head>/<body>`),
- images have `alt` and absolute `https` src,
- `[if mso]` comments are balanced, tables are balanced,
- no `{{ }}` placeholders (must be plain UPPERCASE words),
- inline `font-family` is present (so the design font beats Odoo's Arial rule).

## 3. Weight / Gmail clipping
Gmail clips the HTML around **~102 KB** ("[Message clipped]"), breaking open-tracking and CTAs below the cut. Only HTML/CSS bytes count (not image bytes, but long UTM URLs and data-URIs do). `validate.cjs` errors above `--max-kb` (default 100) and warns above `--warn-kb` (default 80, margin because Odoo adds its head). To reduce: keep minify on, drop unused CSS, shorten links, avoid data-URI images.

## 4. Optional: caniemail compatibility lint
For deeper CSS-support checking against real client data (without a browser), `doiuse-email` lints HTML+CSS against the "Can I email" dataset. Not installed by default — if added, pin it in `package.json` and `requirements.md`, then run it targeting `['outlook.*','gmail.*','apple-mail.*','*.ios']`. Treat `n` on structural features without a fallback as blocking; treat partial `a` (with fallback) as a warning. The dataset JSON is at `https://www.caniemail.com/api/data.json` (cacheable).

## 5. Human visual QA checklist
The only allowed visual validation. Deliver this markdown to the user and have them send themselves the test (step 10 of the workflow):

```
Visual QA — <slug>
- [ ] Outlook desktop (Word engine): layout intact, backgrounds show (VML), font falls back cleanly, no broken tables
- [ ] Gmail web + Gmail app: no clipping ("view entire message"), <style> applied, dark mode not broken
- [ ] Apple Mail macOS + iOS: webfonts, dark mode correct, top spacing acceptable
- [ ] New Outlook / OWA (Chromium): modern rendering OK
- [ ] Dark mode (all three engines): text legible, logo visible (not black-on-black), no grotesque inversion
- [ ] Images OFF: alt texts readable, layout holds
- [ ] Mobile: single-column, CTAs tappable (>=44px)
- [ ] Links/UTM/unsubscribe/preheader/subject/sender correct
```

## 6. Client engine cheat-sheet
- **Outlook desktop (Word engine)** — no flex/grid/border-radius/box-shadow/gradient/`@media`; needs tables, fixed widths, VML, `[if mso]`. Dominant in B2B corporate. Design the baseline for this; treat modern CSS as progressive enhancement.
- **New Outlook / OWA (Chromium)** — modern CSS OK.
- **Apple Mail (WebKit)** — most permissive; respects `prefers-color-scheme` and `meta color-scheme`.
- **Gmail** — own engine; rewrites HTML, ignores `prefers-color-scheme`, clips at ~102 KB, may drop invalid `<style>`.
