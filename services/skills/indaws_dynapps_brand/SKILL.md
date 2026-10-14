---
name: indaws-dynapps-brand
version: 1.0.6
effort: low
model: sonnet
description: >
  Sistema de identidad de marca Dynapps (visual + verbal: voz, tono, vocabulario de valor)
  para producir documentos, paginas, emails, redes y presentaciones on-brand.
  Triggers (ES/EN): "colores dynapps"/"dynapps colors", "brand dynapps"/"dynapps brand",
  "logo dynapps"/"dynapps logo", "documento dynapps"/"dynapps document",
  "presentacion dynapps"/"dynapps presentation", "HTML dynapps"/"dynapps HTML",
  "diseño dynapps"/"dynapps design", "paleta dynapps"/"dynapps palette",
  "tono dynapps"/"voz de marca", "contenido dynapps"/"dynapps content", "redes dynapps".
---

# Dynapps Visual Identity & Brand Design System

Dynapps is an Odoo implementation partner. Mission: *"We digitalise your business processes in Odoo."* Brand values: **Professional. Reliable. Flexible.**
This skill governs **both visual design and verbal identity** (voice, tone, value vocabulary).

**Read `references/brand-system.md` at start** for visuals (colors, CSS variables, typography, logo rules, background modes). For any **text** (proposals, emails, blog, social, reports) also read `references/verbal-identity.md` (voice, "tú", value dictionary, soft-no) and check `references/content-review-checklist.md` before delivering.

## Workflow

1. **Load brand system**: Read `references/brand-system.md` (colors, gradients, typography, logos, patterns).
2. **Identify format**: HTML page / PPTX / DOCX/PDF / marketing asset. For office formats use the matching skill alongside this one (`.pptx` → `pptx`, `.docx` → `docx`, `.pdf` → `pdf`).
3. **Apply format-specific rules**: HTML → dark hero + light body; PPTX → light-first sandwich; DOCX/PDF → always light. See `references/html-documents.md` for HTML detail.
4. **Review against checklist**: Use the Visual Review Checklist in `references/brand-system.md`.
5. **Deliver**: On-brand output with correct logo, colors, typography.

## GOTCHAS

1. **Extended palette for digital, primary palette for print** — never mix them. Digital CSS uses `--midnight: #050543`, not `#010114`.
2. **Logo: lowercase "dynapps" in wordmark, capital "Dynapps" in text** — never invert.
3. **Gradient at 45deg only (Violet→Carmine→Orange, midpoint 52%)** — never use as full-page background.
4. **Dark backgrounds are high-impact moments only** — max 1 accent slide per deck, 1 spotlight section per HTML page.
5. **Readex Pro must be loaded** — never fall back to system fonts without the Google Fonts preconnect.
6. **Logo variant must match background** — black logo on light, white logo on dark. Using the wrong variant is the most common brand error.
7. **Glass nav is always dark** (`rgba(5,5,67,0.88)`) — even on light-body pages.
8. **PDF page-breaks are not a brand concern** — multi-page tables, repeating `tfoot` totals,
   footer pinning, orphan headings and specificity bugs are solved once in
   `pdf-renderer/references/playwright-pdf-guide.md` ("Multi-page report patterns"). Follow it;
   `references/pdf-reports.md` only adds the Dynapps brand layer (light-only, fonts, logo, cover).
9. **Los acentos desaparecen si los ejemplos no los tienen** — Claude replica el estilo
   ortográfico del contexto que lee. **CRÍTICO en HTML/PDF largos**: las tildes se pierden
   progresivamente según avanza el documento. Escribir SIEMPRE con acentos desde la primera
   línea. Lista completa de palabras que más fallan en `references/verbal-identity.md` §Ortografía.

## NEVER / ALWAYS

**NEVER:** Apply gradient to wordmark. Use primary palette values for digital work. Use dark background for body/content sections (only title, closing, accent). Omit font loading. **Omitir tildes, eñes, diéresis o signos de apertura `¿ ¡` en texto español** — si detectas que estás omitiendo tildes, reescribe el párrafo completo antes de seguir.
**ALWAYS:** Use extended palette CSS variables for digital. Apply correct logo variant per background. Highlight one keyword per headline in accent color. Include gradient accent line in nav/footer. **Ortografía es_ES completa (á é í ó ú · ñ · ü · ¿ ¡) en toda salida en español, sin excepciones** — ver `references/verbal-identity.md` §Ortografía.

## Delegates to

| Skill | Suite | For what |
|-------|-------|----------|
| artifact-design | local | HTML artifact creation (use dynapps-brand colors/tokens) |

## Consumed by

| Skill | Suite | For what |
|-------|-------|----------|
| email-campaign-builder | common | On-brand email HTML |
| odoo-module-documenter | common | On-brand documentation pages |
| onboarding | common | On-brand onboarding materials |
| migration-estimator | migration-lab | Brand style for the migration-estimate PDF |

See `references/shared/coding-standards.md` for code quality rules when generating HTML/CSS artifacts.

## References

- `references/brand-system.md` — **READ FIRST (visual)** — complete color system, CSS variables, typography, logo rules, background patterns, visual review checklist
- `references/verbal-identity.md` — **voice/tone** — archetype, "tú", value dictionary, soft-no, writing examples (read for any text content)
- `references/social-media.md` — per-platform voice/content/hashtag rules (visuals defer to brand-system)
- `references/content-review-checklist.md` — pre-delivery voice + visual checklist
- `references/html-documents.md` — HTML-specific patterns: nav, hero, sections, footer, scroll reveal
- `references/pdf-reports.md` — Dynapps PDF brand layer (light-only, fonts, logo, cover/footer) + **canonical report template/scripts + critical CSS gotchas**
- `assets/templates/migration-report-template.html` — canonical branded report template (cover + footer band + 4-section content), filled by consumer skills (e.g. migration-lab). Do NOT clone CSS elsewhere.
- `scripts/gen.js` + `scripts/stamp.py` — render content + brand footer band, stamp it flush to every page. `scripts/verify.py` — mandatory post-render QA (cover full-bleed + no lateral hairlines).
- `pdf-renderer/references/playwright-pdf-guide.md` — **shared SSOT** for PDF page-breaks, multi-page tables, footer pinning (consumed by all document skills)
- `assets/logos/` — SVG logo files (4 variants: Horizontal/Vertical × Black/White)
- `assets/icons/` — 8 service-pillar icons (Boost, Build, Extra, Improve, Integrate, Manage, Personalise & Develop, Streamline)
- `assets/fonts/` — Readex Pro TTF files (6 weights: 200-700)
- `assets/examples/` — `dynapps_website_strategy.html`, `dynapps_marketing_onboarding.html` — proven delivered examples
