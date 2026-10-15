---
name: indaws-content-creation
version: 1.0.3
effort: medium
model: sonnet
description: >-
  [SUSTITUIDA — no usar] Doctrina obsoleta que contradice Odoo Knowledge (space 12).
  Voz y vocabulario: `es-lint` y `dynapps-brand`; calendario y contenido: `social-calendar`.
---

# content-creation

Base de conocimiento para creacion de contenido de marketing en todos los canales Indaws.

## Workflow

1. Check `dynapps-brand` for voice, color, and typography spec before writing
2. Consult `content-collection` for existing pieces — avoid duplicating topics
3. Select content type template from references (blog, LinkedIn, email, landing page)
4. Write following vocabulary rules in `references/content-vocabulary.md`
5. Apply SEO checklist from `references/blog-post-guide.md` for any web content
6. Final brand compliance review: no emojis, no hashtags, correct es-ES, "tu" not "usted"

## GOTCHAS

- **Emojis creep in**: Any emoji in official content violates brand. Zero tolerance, including subject lines and LinkedIn posts.
- **LinkedIn hashtags**: Never add hashtags to LinkedIn posts — Indaws policy is zero hashtags, no exceptions.
- **"Usted" instead of "tu"**: Content must always use "tu" treatment. "Usted"/"su" reads as cold and misaligns with brand voice.
- **Skipping SEO on blog posts**: Every blog post needs title tag, meta description, H1 keyword, slug, and internal links — do not publish without running the checklist in `references/blog-post-guide.md`.
- **Writing for a Latin-American audience**: Use es-ES variants — "ordenador" not "computadora", "movil" not "celular". Indaws operates in Spain.
- **Links in LinkedIn post body**: Links in the post body kill organic reach. All links go in the first comment, never the post itself.
- **Generic CTAs**: "Click aqui", "Enviar", "Mas informacion" are banned. CTAs must be specific and action-oriented — see `references/headline-formulas.md`.
- **Autopromocion directa**: Content must deliver value first. Promotional framing without substance is off-brand.

## NEVER

- Add emojis to any official content
- Use hashtags in LinkedIn posts
- Address the reader as "usted" or "vosotros"
- Put links in the LinkedIn post body
- Publish web content without SEO checklist
- Write content in English for Spanish-speaking audience
- Use "computadora", "celular", or other LATAM variants
- Use generic CTAs ("click aqui", "enviar", "mas informacion")

## ALWAYS

- Read `dynapps-brand` before starting any content piece
- Check `content-collection` to avoid duplicate topics
- Use "tu" treatment throughout
- Apply es-ES spelling and punctuation (acentos, ¿¡)
- Use Indaws vocabulary: inversion, reto, acompanamiento, evolucion continua, especialistas, propuesta
- Include a specific, funnel-appropriate CTA in every piece
- Run SEO on-page checklist for all web content before publishing

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| dynapps-brand | common-skills | Voice attributes, color tokens, typography spec |
| content-collection | common-skills | Existing content inventory, success stories, avoid duplicates |
| linkedin-content-manager | ai-claude-marketing | Full LinkedIn post creation, visual briefs, scheduling |
| seo-manager | ai-claude-marketing | Keyword research, volume data, rank tracking |
| email-campaign-builder | ai-claude-marketing | Email HTML templates, Odoo field setup, automation flows |

## Consumed by

| Skill | Suite | Why |
|-------|-------|-----|
| campaign-planning | ai-claude-marketing | Pulls content production templates for campaign calendars |
| case-study-generator | ai-claude-marketing | Uses case study format and vocabulary rules |
| brand-voice | marketing | Redaccion de blog posts, landing pages y case studies on-brand |

## REFERENCES

- `references/blog-post-guide.md` — Blog structure, Odoo field mapping, SEO checklist, keyword strategy, intro examples, press release format
- `references/linkedin-quick-ref.md` — LinkedIn quick rules, funnel color mapping
- `references/email-templates.md` — Newsletter structure, email writing guide, Odoo personalization fields
- `references/landing-page-guide.md` — Landing page structure, Odoo Website Builder notes, case study format, repurposing matrix
- `references/content-vocabulary.md` — Indaws terminology table, language rules, brand voice attributes, quality checklist
- `references/headline-formulas.md` — 8 headline formulas, CTAs by funnel stage, CTA design rules, CTAs to avoid
- `common:references/shared/` — Shared brand tokens and cross-suite reference data
