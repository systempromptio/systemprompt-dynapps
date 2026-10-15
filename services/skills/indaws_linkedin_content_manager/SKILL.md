---
name: indaws-linkedin-content-manager
version: 1.1.3
effort: medium
model: sonnet
description: >-
  [SUSTITUIDA — no usar] Doctrina obsoleta que contradice Odoo Knowledge (space 12).
  Voz y vocabulario: `es-lint` y `dynapps-brand`; calendario y contenido: `social-calendar`.
---

# LinkedIn Content Manager — Indaws

Gestión completa del LinkedIn de Indaws: del descubrimiento al contenido listo para publicar.

## Agent Routing

| Task | Route |
|------|-------|
| Tono / brand-voice de un post | `brand-voice` skill (delegate) |
| Redacción larga / blog cruzado | `content-creation` skill (delegate) |
| Casos, vídeos, módulos para inspirar contenido | `content-collection` skill |
| Métricas de página, reporte de rendimiento | `performance-analytics` skill |
| Visuales (carrusel, infografía, quote card) | `~~design` connector (Canva/Figma), brief en el post |
| Leer datos de CRM/eventos para ideas | `odoo-pilot` (JSON-RPC) |

## Workflow

1. **DISCOVER** (lunes AM) — `python scripts/discover_content.py --sources all --days 7 --output text` → 10-15 ideas raw. Prioridad: interno (casos, podcast "IAhora qué?", eventos, módulos Core) > ecosistema Odoo > tendencias. Fuentes curadas en `references/odoo-sources.md`.
2. **CURATE** — quedarte con las 5 mejores ideas de la semana.
3. **WEEKLY REPORT** — rellenar `templates/weekly-linkedin-report.html` con 5 posts + briefs. Mix funnel obligatorio en `references/funnel-strategy.md`; calendario en `references/content-calendar.md` / `templates/content-calendar.html`.
4. **CREATE** — escribir copy + visual brief por post. Estructura: Hook (2 líneas) → Cuerpo (80-300 palabras) → CTA (1 línea) + primer comentario (enlaces) + visual brief. Formatos, hooks y tamaños en `references/post-formats.md`; ejemplos completos por tipo en `references/post-examples.md`.
5. **REVIEW** — pasar por `brand-voice`; previsualizar con `templates/post-preview-card.html`.
6. **PUBLISH** — manual. El equipo publica desde el reporte (ver GOTCHAS).
7. **MEASURE** — métricas por post → ajustes próxima semana. Targets y plantilla en `references/weekly-report.md`.

Antes de escribir: leer `dynapps-brand/references/social-media.md` y `verbal-identity.md`.

## Funnel (resumen)

| Stage | Cadencia | Objetivo | Color |
|-------|----------|----------|-------|
| TOFU | 2/sem | Alcance | `#8AC2DB` |
| MOFU | 2/sem | Confianza | `#6B68FA` |
| BOFU | 1/sem | Conversión | `#E5B92B` |

Mix semanal: Mar TOFU · Mié MOFU · Jue TOFU · Vie MOFU · Lun sig. BOFU. Detalle en `references/funnel-strategy.md`.

## Métricas por fase

| Fase | Principal | Target |
|------|-----------|--------|
| TOFU | Impresiones | >1000/post |
| MOFU | Engagement rate | >3% |
| BOFU | Click-through | >1% CTR |

## GOTCHAS

- **No hay API de publicación abierta para páginas de empresa**: LinkedIn no permite auto-post fiable a la página. Salida = borrador/reporte; el equipo publica manualmente. NUNCA afirmar publicación automática.
- **Enlaces en el cuerpo penalizan el alcance** (2-3x menos): poner el enlace en el primer comentario, no en el post.
- **Hashtags y emojis = CERO** en contenido Indaws. Romper esto rompe brand-voice.
- **Métricas de página ≠ perfil**: este skill trata la página de empresa; los benchmarks de perfil personal no aplican.
- **BOFU nunca antes de 2 posts TOFU/MOFU** esa semana, ni 2 posts del mismo tipo consecutivos.
- **Longitud**: hook crítico en las 2 primeras líneas (corte "ver más"); cuerpo 80-300 palabras según tipo.
- **Vocabulario de valor**: nunca "coste", "problema", "gasto" → "inversión", "reto", "oportunidad".

## NEVER

- Afirmar o implicar publicación automática a LinkedIn (no existe integración).
- Usar hashtags, emojis o cuentas personales.
- Escribir contenido sin respetar brand-voice ni el vocabulario de valor.
- Poner enlaces en el cuerpo del post.

## ALWAYS

- Idioma español, voz "tú" profesional, tono Indaws.
- Leer `dynapps-brand` antes de redactar.
- Enlaces en el primer comentario.
- Salida como borrador/reporte para publicación manual.

## Delegates to

- `brand-voice` — calibración de tono y compliance del copy.
- `content-creation` — redacción larga y best practices de escritura.

## Consumed by

- `campaign-planning` — calendario LinkedIn dentro de la estrategia de campaña.
- `performance-analytics` — datos de rendimiento LinkedIn para reportes consolidados.

## REFERENCES

- `references/funnel-strategy.md` — TOFU/MOFU/BOFU detallado.
- `references/post-formats.md` — hooks, longitudes, tamaños, especificaciones.
- `references/post-examples.md` — ejemplos completos por tipo + series de funnel.
- `references/content-calendar.md` — calendario y workflow.
- `references/weekly-report.md` — plantilla del reporte semanal + targets.
- `references/odoo-sources.md` — fuentes curadas del ecosistema Odoo.
- `scripts/discover_content.py` — discovery automatizado de fuentes.
- `schemas/placeholders.json` — placeholders de los templates.
- `templates/weekly-linkedin-report.html` · `content-calendar.html` · `post-preview-card.html` — visuales HTML.
- `common:references/shared/` — N/A para publicación (sin API). Si lees datos de Odoo CRM/eventos para ideas, usa `odoo-pilot`.

## Success Criteria

- 5 ideas curadas mapeadas a funnel con mix semanal válido.
- Cada post: hook + cuerpo + CTA + primer comentario + visual brief.
- Cero hashtags/emojis, español, vocabulario de valor, brand-voice OK.
- Entregado como borrador/reporte para publicación manual.
