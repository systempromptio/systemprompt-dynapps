---
name: indaws-odoo-frontend
version: 1.0.2
effort: medium
model: sonnet
description: >
  Genera HTML compatible con Odoo Editor (Bootstrap 5 + clases iw-*) para paginas web, blogs,
  emails, slides y eLearning siguiendo la identidad Indaws. NOT modulos Python → odoo-module-creator.
  Triggers: "pagina web odoo", "blog post html", "email marketing template", "slide html odoo",
  "contenido portal html", "convertir pptx a html", "landing page odoo".
---

# Odoo Frontend

Create HTML content for Odoo using Bootstrap 5 and the Indaws design system (`iw-*` classes). All output must be compatible with Odoo Editor and follow the dynapps-brand guidelines.

**Before starting any task, read the `dynapps-brand` skill** to load brand colors, typography, voice rules, and the zero-emoji policy.

## Quick Reference

**Brand tokens**: Para los valores exactos de los tokens de marca, delega en la skill `dynapps-brand`. Do NOT hardcode hex values.

| Token | CSS Var | File |
|-------|---------|------|
| Warm Yellow | `--indaws-warm-yellow` | `assets/css/indaws-frontend.css` |
| Blue Lilac | `--indaws-blue-lilac` | |
| Blue Space | `--indaws-blue-space` | |
| Light Sky | `--indaws-light-sky` | |
| Font | Montserrat (400, 600, 700) | Google Fonts |

## Workflow

### Phase 0: Detect Mode

| User wants... | Mode | Action |
|---------------|------|--------|
| New website/landing page | A: New Page | Phase 1 |
| New blog post | A: Blog | Phase 1 |
| New email template | A: Email | Phase 1 |
| New HTML presentation | A: Presentation | Phase 1 (with presentation flow) |
| Convert a .pptx file | B: PPT Conversion | Phase 4 |
| Enhance existing HTML | C: Enhancement | Read existing file, enhance with iw-* classes |
| Quick component | D: Component | Read `references/components-catalog.md`, produce directly |

### Phase 1: Content Discovery

Read `references/content-discovery.md` for the full AskUserQuestion script (purpose, content readiness, images, audience, presentation-specific questions).

### Phase 2: Style Selection (Presentations Only)

Read `references/style-selection.md` for mood-to-style mapping and preview generation.

### Phase 3: Generate Content

**Read `references/odoo-editor-rules.md` first.** Key rules:
1. Bootstrap 5 grid (`container`, `row`, `col-*`) for all layouts
2. `iw-*` classes from `assets/css/indaws-frontend.css` for styling
3. Wrap in `<section>` tags with Odoo snippet classes (`s_text_block`, etc.)
4. No `<script>` tags (stripped by editor), no `data-oe-*` attributes
5. No inline `<style>` in emails (use inline styles); images use `img-fluid`

**By content type:**

- **Page / Landing page**: Read `references/self-contained-output.md` for mandatory format (inlined CSS, no DOCTYPE wrapper, XML-safe URLs, image placeholders). Read `references/landing-page-creation.md` for section ordering, CTA placement, SEO guidelines.
- **Email**: Read `references/email-creation.md`. Table-based layout, max 600px, inline styles only, bulletproof VML buttons, Montserrat with Verdana fallback. Brand compliance: read `dynapps-brand` verbal + visual identity first.
- **Blog**: Read `references/blog-creation.md`. BLUF structure, 1500+ words, TOC from H2s, interlinking via `content-collection` sitemap, YouTube embed, FAQ + Schema JSON-LD. Brand compliance: read `dynapps-brand` verbal + visual identity first.
- **Presentation**: Read `references/presentations.md` for slide types, navigation JS, structure.

### Phase 4: PPT Conversion

1. Extract content with `python-pptx` (text, images, slide order, notes)
2. Present summary to user for confirmation
3. Run Phase 2 (Style Selection)
4. Map each slide to closest type from `references/presentations.md`
5. Preserve images and speaker notes (as `<!-- NOTES: ... -->`)

### Phase 5: Preview & Delivery

Read `references/delivery-workflow.md` for the complete flow: preview generation, user review loop, image manifest creation, delivery summary, and publishing via odoo-pilot (blog, email, landing page).

## Key Gotchas

- **Never hardcode hex colors** -- delega siempre en la skill `dynapps-brand` para los tokens de marca
- **Never use generic placeholder URLs** (placeholder.com, lorempixel) -- use inline SVG with brand colors
- **Emails: inline styles only** -- Gmail strips `<style>` blocks, Outlook uses Word renderer
- **Self-contained pages: no iw-reveal animations** -- requires JS which Odoo strips
- **XML-safe URLs** -- use `&amp;` not `&` in QWeb template content
- **Spanish text** -- preserve all accents and diacritics (never strip)
- **CSS loading for non-page content**: read `references/css-loading.md`

## GOTCHAS
1. **Odoo Editor strips non-standard HTML** — only Bootstrap 5 classes and `iw-*` custom classes survive.
2. **Background images need absolute URLs in email** — relative paths break in mail clients.
3. **Glassmorphism requires backdrop-filter** — not supported in all email clients. Provide solid fallback.
4. **Font loading fails in PDF** — use pdf-renderer skill which handles Playwright font embedding.
5. **Dark theme variables don't exist in Odoo Editor** — dark mode must use explicit alternate color classes.

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| dynapps-brand | common | Brand tokens/colors for all HTML output |
| content-collection | common | Casos de exito, videos y sitemap para interlinking SEO |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| report-docx-template | consultant | HTML content for reports |
| Content creation | marketing | HTML content for campaigns |

Para los tokens de marca, delega en la skill `dynapps-brand`. See `common:references/shared/` for shared patterns.

## References

### assets/
- `css/indaws-frontend.css` -- Complete design system (tokens, typography, colors, sections, cards, buttons, badges, features, stats, testimonials, tables, pricing, timeline, dividers, callouts, images, animations, email utilities, bento grid, responsive breakpoints)
- `js/iw-reveal.js` -- Intersection Observer for scroll-triggered entrance animations

### references/
- `odoo-editor-rules.md` -- Allowed HTML, Bootstrap grid, snippet structure, forbidden patterns
- `components-catalog.md` -- Copy-paste HTML: heroes, grids, cards, stats, testimonials, pricing, CTA
- `content-discovery.md` -- Phase 1 AskUserQuestion scripts (purpose, content, images, audience)
- `style-selection.md` -- Phase 2 mood-to-style mapping and preview generation
- `self-contained-output.md` -- Mandatory format for website pages (inlined CSS, QWeb rules)
- `blog-creation.md` -- BLUF structure, SEO, interlinking, Schema JSON-LD, odoo-pilot publishing
- `email-creation.md` -- Table layout, inline styles, VML buttons, odoo-pilot publishing
- `landing-page-creation.md` -- Page types, persuasion flow, CTA strategy, odoo-pilot publishing
- `presentations.md` -- Slide types, navigation JS, PPT conversion guide
- `preview-workflow.md` -- Browser preview system, wrapper template, iteration flow
- `image-workflow.md` -- Unsplash API, SVG placeholders, Odoo Media Library, image manifest
- `delivery-workflow.md` -- Preview, review loop, manifest, publishing options
- `css-loading.md` -- Loading iw-* classes in Odoo (customization panel or module asset bundle)
