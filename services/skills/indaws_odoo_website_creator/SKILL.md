---
name: indaws-odoo-website-creator
version: 1.4.5
effort: medium
model: sonnet
description: >-
  Creates and publishes website PAGES in Odoo (v18/v19) via API, no UI. Modes: brief, local HTML,
  reproduce an image section-by-section, duplicate a page, and FORM (s_website_form→crm.lead).
  Wraps content in the native structure (website.layout + #wrap.oe_structure + s_* snippets +
  Bootstrap 5 grid); always UNPUBLISHED. Triggers: "create a website page in odoo", "upload this
  HTML as a page", "reproduce this image as a page", "duplicate page X", "add a form that creates
  leads".
---

# Odoo Website Creator

Publishes website **pages** on any Odoo instance (v18/v19) from a brief, HTML, an image, or by
duplicating a page; also adds lead-generating forms. Brand-agnostic; every page comes out
**unpublished**. Owns the publication pipeline (`website.page`+`ir.ui.view`, `#wrap.oe_structure`,
snippets, CSS/images, `s_website_form`→`crm.lead`, publish/menu/SEO). Does NOT generate the HTML
(→ `odoo-frontend`) nor palette/JS snippets (→ module).

**Brand policy:** Modes B/D (pre-made HTML / duplicate) are brand-neutral; Modes A/C delegate HTML to
`odoo-frontend`, which applies the Dynapps brand by default — say so if the user wants a neutral page.

**Output language — ask, don't assume** (step 2; default `website.default_lang_id`): for several
languages author the base then add each as a **translation on the same records**, never a page per
language. See `references/multilanguage-output.md`.

## When to use

Triggers: "create/publish a page in odoo", "upload this HTML as a page", "reproduce this
image/screenshot as a page", "duplicate page X", "add a form that creates leads", or the user pastes
HTML/a mockup/a `/odoo/...` page URL.

NOT for: generating the HTML/design (→ `odoo-frontend`) · mailings (→ `odoo-mailing-creator`) · blog ·
a reusable palette snippet or interactive-JS component (→ module; see `references/custom-components.md`).

## Workflow [SONNET]

0. **Connection** — ask once: odoo-pilot or MCP. Never ask for credentials. See
   `references/connection-modes.md` and `references/shared/odoo-connection.md`.
1. **Version** — detect `server_version`. v18/v19 = preloaded flow; other versions = web search
   (`references/version-notes.md`).
2. **Input mode** — A brief · B local HTML · C image · D duplicate · +Form. Also capture the
   **output language(s)** for the page content here (default: the site's default language). See
   `references/input-modes.md`.
3. **Generate HTML (A/C)** — delegate to `odoo-frontend` (Bootstrap 5), telling it the target output
   language so the copy is written in it, and require the **mobile-first contract** in the delegation
   (`references/responsive.md`: col-12 base grid, `min()` caps instead of fixed px, stack-on-mobile
   flex rows). `odoo-frontend` applies the Dynapps brand identity by default; if the page must be
   brand-neutral, say so explicitly in the delegation (see Brand policy above). Heavy creative
   work → opus subagent.
4. **Builder-safe transform (B/C)** — `scripts/build_page_arch.py`: strips scaffolding, wraps each
   block in `<section class="s_…" data-snippet data-name>`, fixes the grid, removes forbidden
   patterns (`<script>`, `on*`, `<iframe>`, `position:fixed/sticky`), extracts CSS/images, wraps in
   `website.layout` + `#wrap.oe_structure`, and emits `responsive:` warnings for mobile-overflow
   traps (bare max-width, fixed/min widths, 100vw/100vh, unwrapped tables — fix them, see
   `references/responsive.md`). See `references/builder-safe-transformations.md`,
   `references/snippet-catalog.md`.
5. **Version deltas** — if applicable (grid, theme SCSS), invoke `odoo-version-api-reference`.
6. **Assets + CSS** — images → `scripts/upload_assets.py` (`ir.attachment`, `/web/image/<id>`,
   `img-fluid`); CSS without a module → the ladder in `references/css-injection-ladder.md`
   (`scripts/inject_css.py`). See `references/image-assets.md`, `references/custom-components.md`,
   `references/layout-recipes.md`.
7. **Form (optional)** — `scripts/ensure_website_crm.py` + `scripts/build_lead_form.py`
   (`s_website_form`→`crm.lead`). See `references/website-forms-to-leads.md`.
8. **Arch** — `templates/page_arch_skeleton.xml` + sections (+form). See
   `references/odoo-website-fundamentals.md`.
9. **Create page UNPUBLISHED** — `scripts/create_website_page.py`: `ir.ui.view` (qweb, unique key) +
   `website.page` (`is_published=false`, SEO) + optional `website.menu`. See
   `references/publish-and-menu.md`. If several output languages were requested, create the
   base-language page first, then add each extra language as a translation on the **same**
   `ir.ui.view`/`website.page` (write under the `lang` context) — not as a new page. See
   `references/multilanguage-output.md`.
10. **Verify and return** — arch not empty, snippets present, no URL/key collision. Return the edit
    URL and the preview URL. Stays UNPUBLISHED. **Brand check** (3 yes/no questions before handing
    back, not a new step): (1) is every color sourced from the confirmed palette/theme tokens except
    for deliberate one-offs? (2) if any Tier‑1 inline hex was used, was it a genuine one-off and not
    a recurring brand color that belongs in the theme instead (cross-ref
    `references/theme-customization.md`)? (3) if the page is meant to be brand-neutral, does it carry
    no leaked brand-specific residue? **Performance check** (3 more, same style — see
    `references/performance.md`): (1) images optimized/resized to their slot, below-fold + hidden-pane
    images `loading="lazy"`, LCP image eager? (2) infinite animations pause off-screen and
    reduced-motion is honored? (3) no CLS — every image/visual slot reserves its space?
    **Responsive check** (see `references/responsive.md`): (1) previewed at 390px — no sideways
    scroll, nothing clipped (remember `#wrapwrap` clips overflow invisibly: LOOK for cut content)?
    (2) `build_page_arch.py --check` reports zero unaccepted `responsive:` warnings? (3) touch-usable
    — targets ≥44px, no hover-gated content, interactions work by tap?
11. **Batch (optional)** — N pages: parallel or sequential? If parallel, N `Agent()` calls in one
    message.

## GOTCHAS

Full set (20 failure patterns) in [references/gotchas.md](references/gotchas.md). Top ones:

1. Without `#wrap.oe_structure` inside `website.layout` the page is **not editable**; `<style>`/`<script>` in editable content is **stripped** (use the CSS ladder).
2. **Publish explicitly** (`is_published=false` + return edit URL); check URL/`key` collisions; the builder **rewrites the whole `arch` on Save** — never write arch via API while the page is open.
3. **Restored/prod DB**: don't overwrite theme/footer blindly (recolors whole site / wipes client code) — read state, skip theme, merge footer between markers.

## NEVER

Ask for credentials (delegate to odoo-pilot/MCP) · publish (always `is_published=false`) · generate
the HTML yourself when `odoo-frontend` is available · hardcode a brand (brand-agnostic; Dynapps only
via `dynapps-brand` on request) · skip the `website.layout`+`#wrap.oe_structure` wrapper · use
`<script>`/`on*`/`<iframe>`/`position:fixed|sticky` in content · build a module for an avoidable
"custom component" · store a real client's actual content inside the skill (only brand-agnostic
PATTERNS; see `references/interactive-patterns.md`).

## ALWAYS

Verify the version before pushing · wrap in `website.layout`+`#wrap.oe_structure` · upload images to
`ir.attachment` and rewrite `src` to `/web/image/<id>`+`img-fluid` · optimize image weight/loading
(`references/performance.md`: lazy below fold + hidden panes, eager LCP, resize oversized originals) ·
inject CSS via the ladder (never an editable `<style>`) · verify `website_crm` before adding a form ·
verify the mobile viewport (390px) and clear the `responsive:` lint warnings before handing back ·
create UNPUBLISHED and return edit+preview URLs · check for URL/key collisions.

## Delegates to

| Skill | When |
|-------|------|
| odoo-pilot | Auth + CRUD + module install (when not using MCP) |
| odoo-frontend | Editor-compatible HTML (modes A/C) — applies the Dynapps brand by default unless told to stay neutral |
| odoo-version-api-reference | website/web_editor deltas v18↔v19 (Step 5) |
| dynapps-brand | Brand tokens — only when the user wants Dynapps applied |

## Consumed by

None yet — standalone page publisher.

## REFERENCES

`references/`: odoo-website-fundamentals · builder-safe-transformations · snippet-catalog ·
custom-components · interactive-patterns · css-injection-ladder · layout-recipes · theme-customization ·
website-forms-to-leads · image-assets · publish-and-menu · connection-modes · version-notes ·
input-modes · multilanguage-output · performance · responsive. Shared: `references/shared/odoo-connection.md`.

## SCRIPTS

`scripts/`: build_page_arch · create_website_page · build_lead_form · ensure_website_crm ·
upload_assets · inject_css · duplicate_page · extract_page_id.

## TEMPLATES

`templates/`: page_arch_skeleton.xml · lead_form_block.html · custom_component_section.html ·
pattern_horizontal_scroll.html · pattern_scroll_pinned_swap.html · pattern_vertical_marquee.html ·
pattern_globe_dome.html · pattern_tabs_panel.html · pattern_segmented_toggle.html ·
pattern_horizontal_accordion.html · pattern_reveal_on_scroll.html · pattern_hover_expand.html.
