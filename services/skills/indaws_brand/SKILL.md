---
name: indaws-brand
version: 1.0.2
effort: low
model: sonnet
disable-model-invocation: true
description: >
  [OBSOLETO — usar dynapps-brand] Guia de marca histórica Indaws. La marca migró a Dynapps:
  dynapps-brand es ahora la skill única de marca (visual + verbal). Se mantiene solo por
  compatibilidad/histórico; no usar para contenido nuevo.
  Triggers: "marca indaws (legacy)", "identidad indaws histórica".
---

# ⚠️ OBSOLETO — Indaws Brand Guidelines (legacy)

> **Esta skill está obsoleta.** La marca migró de Indaws a **Dynapps**. Usa **`dynapps-brand`**,
> que ahora cubre identidad **visual + verbal** (la voz se absorbió ahí: `verbal-identity.md`,
> `social-media.md`, `content-review-checklist.md`). Este fichero se conserva solo como histórico
> y no debe usarse para contenido nuevo.

## Overview

This skill enforces the Indaws brand identity across all document types. Two pillars:

1. **Verbal Identity**: How Indaws speaks (tone, vocabulary, writing style)
2. **Visual Identity**: How Indaws looks (colors, typography, logos, imagery)

All visual tokens (colors, typography, gradients, logo rules) are defined in `assets/brand-tokens.json`. Other skills that need Indaws brand values MUST read from that file instead of hardcoding hex values.

---

## Document Type Routing

Read the appropriate reference before generating each document type:

| Task | Reference to Read |
|---|---|
| Writing content (any type) | [references/verbal-identity.md](references/verbal-identity.md) |
| Visual design decisions | [references/visual-identity.md](references/visual-identity.md) |
| HTML documents / web pages | [references/html-documents.md](references/html-documents.md) |
| PDF reports / proposals | [references/pdf-reports.md](references/pdf-reports.md) |
| PDF generation (technical) | `pdf-renderer` skill (Playwright) |
| Presentations / slides | [references/presentations.md](references/presentations.md) |
| Social media / marketing | [references/social-media.md](references/social-media.md) |
| Asset inventory (logos, fonts) | [references/assets-catalog.md](references/assets-catalog.md) |
| Pre-delivery checklist | [references/content-review-checklist.md](references/content-review-checklist.md) |

For most tasks, read **verbal-identity.md** (for copy) AND the format-specific reference (for layout/structure).

---

## Hard Rules

These are inviolable. No exceptions, no creative interpretations:

1. **ZERO emojis** in any output
2. **Always "tu"** (informal Spanish), never "usted"
3. **Logo placement**: only 4 corners or absolute center
4. **Value vocabulary**: never use cost/expense/problem language (see verbal-identity.md)
5. **Only brand colors**: no grays, no pure black, no off-palette colors
6. **Only Montserrat** (or Verdana fallback)
7. **Gradients**: always dark to light
8. **Logo minimum sizes**: 70px digital / 24mm print
9. **Never say "no lo hacemos"**: always reframe as expert recommendation
10. **No quick tips**: argue every recommendation with reasoning
11. **Ortografia es_ES completa**: toda salida debe usar acentos, enes y dieresis correctos del espanol de Espana. Sin excepciones.

---

## Gotchas

Patrones de fallo reales detectados en produccion:

1. **Acentos desaparecen si los ejemplos no los tienen**: Claude replica el estilo ortografico del contexto que lee. Si las referencias omiten tildes, el output tambien las omitira. Por eso todas las referencias de este skill deben estar correctamente acentuadas. **CRITICO para HTML/PDF**: Cuando se genera un documento HTML largo, es muy frecuente que los acentos se pierdan progresivamente. Escribir SIEMPRE con acentos desde la primera linea. Palabras que fallan con frecuencia: analisis, valoracion, informacion, configuracion, seccion, descripcion, modulo, metodo, categoria, periodo, diagnostico, fabricacion, produccion, correccion, resolucion, inversion, antiguedad, rotacion, conciliacion, transito, automatico, especifico, tambien, segun, mas, ademas.

2. **Tablas se cortan entre paginas en PDF**: SIEMPRE anadir `thead { display: table-header-group; }` y `tr { page-break-inside: avoid; }`.

3. **Margenes laterales excesivos en PDF**: El `@page { margin: 25mm }` ya aporta margen suficiente. El padding adicional del `.page` debe ser minimo (~1.2rem lateral).

4. **Generar PDF sin consultar pdf-renderer**: Siempre consultar el skill `pdf-renderer` antes de generar cualquier PDF.

5. **Bloques de codigo largos rompen la paginacion**: Los `<pre>` con mas de 25-30 lineas se cortan entre paginas. Dividir en chunks de ~25 lineas con `<pre>` separados, cada uno con `page-break-inside: avoid`.

6. **Diagramas ASCII y codigo real usan el mismo estilo**: Diagramas de flujo necesitan fondo claro (clase `diagram-block`: fondo lila, borde Blue Lilac). Codigo ejecutable va con fondo oscuro (clase `code-block`: fondo Blue Space, borde Warm Yellow). No mezclar.

7. **Metadata del documento en linea**: En informes tecnicos, usar el componente `.metadata` grid del `base.html`, no texto en negrita en parrafo.

8. **Font-size de codigo demasiado grande en PDF**: Los bloques `<pre>` en PDF deben usar 7-7.5pt (0.65rem), no el tamano de body (10pt).

9. **Logo olvidado**: El logo de Indaws DEBE aparecer en TODOS los documentos sin excepción. Antes de entregar cualquier HTML, PDF, presentación o email, verificar visualmente que el logo está presente en la posición correcta (una de las 4 esquinas o centro absoluto). Si falta, añadirlo antes de continuar. Este es el error más frecuente y nunca debe volver a ocurrir.

10. **Acentos y eñes omitidos en español**: Todo el texto en español DEBE usar ortografía completa es_ES: tildes (á, é, í, ó, ú), eñe (ñ), diéresis (ü), signos de apertura (¿, ¡). Nunca escribir "analisis", "tambien", "segun", "informacion", "mas", "ademas", "seccion", "descripcion", "modulo", "automatico", "configuracion". Si detectas que estás omitiendo tildes, reescribe el párrafo completo antes de seguir.

11. **Diseño AI Flop prohibido**: No usar patrones de diseño genérico de IA: listas de cards con borde de color lateral, iconos de emoji como decoración, fondos planos blancos con accent color, grids simétricas sin jerarquía visual, separadores de línea horizontal sin propósito. En su lugar: Bento Box (celdas de tamaño variable con jerarquía visual clara), Glassmorphism (fondos semitransparentes con backdrop-filter blur), gradientes de marca, superposición de elementos. El diseño debe parecer obra de un diseñador senior, no generado por IA.

12. **Cards con borde de color prohibidos**: Nunca usar el patrón `border-left: 4px solid <color>` ni `border-top: 3px solid <color>` en cards o paneles. Es el patrón visual más reconocible del "diseño AI Flop". Usar en su lugar: glassmorphism (`background: rgba(...); backdrop-filter: blur(...)`, sombra sutil), o cards con fondo de color sólido de marca sin bordes decorativos de color.

13. **Énfasis textual: usar azul, nunca amarillo**: Cuando se necesita resaltar texto (negrita, highlight, color de acento), usar el azul de marca (Blue Space `#1B2A4A` o Blue Lilac `#6B7FD4`), nunca el amarillo/Warm Yellow. El amarillo Warm Yellow es solo para elementos decorativos gráficos (líneas, separadores), no para texto enfatizado.

Para gotchas técnicas de generación PDF (page breaks, fonts, images), consultar `pdf-renderer/references/playwright-pdf-guide.md`.

## Delegates to
| Skill | Suite | When |
|-------|-------|------|
| pdf-renderer | common | PDF output generation |

## Consumed by
| Skill | Suite | For what |
|-------|-------|----------|
| email-campaign-builder | common | Brand tokens for email campaigns |
| linkedin-content-manager | common | Brand voice compliance |
| seo-manager | common | Brand consistency in SEO content |
| event-planner | common | Brand materials for events |
| odoo-frontend | common | Brand tokens/colors for HTML |
| kick-off-presentation | consultant | Branded presentations |
| All 5 marketing skills | marketing | Brand compliance |
| competitive-intelligence | sales | Brand positioning |

For shared ecosystem patterns, see `common:references/shared/`.

## Workflow
1. Determine document type (HTML, PDF, presentation, email, social)
2. Read the appropriate reference for that type
3. Load brand tokens from `assets/brand-tokens.json`
4. Apply visual identity (colors, fonts, logos)
5. Apply verbal identity (tone, vocabulary, structure)
6. Validate against `references/content-review-checklist.md`
