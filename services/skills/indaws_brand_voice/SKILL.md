---
name: indaws-brand-voice
version: 1.0.3
effort: low
model: sonnet
description: >-
  [SUSTITUIDA — no usar] Doctrina obsoleta que contradice Odoo Knowledge (space 12).
  Voz y vocabulario: `es-lint` y `dynapps-brand`; calendario y contenido: `social-calendar`.
---

# Brand Voice — Indaws

Framework de voz de marca para todo el contenido de marketing Indaws.
Complementa `dynapps-brand` (common-skills) — no lo duplica.

## Workflow

1. **Load dynapps-brand** (common-skills) — fuente principal de colores, tipografia, logos e identidad verbal completa (`dynapps-brand/references/verbal-identity.md`, `dynapps-brand/references/social-media.md`, `dynapps-brand/references/brand-system.md`)
2. **Check vocabulary table** (`references/vocabulary.md`) — verificar terminos correctos vs prohibidos antes de escribir cualquier pieza
3. **Calibrate tone by channel** (`references/voice-attributes.md`) — consultar los 7 atributos de voz y la matrix de adaptacion por canal
4. **Run compliance checklist** (`references/compliance-checklist.md`) — pasar el checklist completo antes de publicar cualquier contenido oficial
5. **Delegate writing** — si se genera contenido, delegar a content-creation, linkedin-content-manager o email-campaign-builder segun canal

## GOTCHAS

1. **Cero emojis, absolutamente** — ni en subject lines de email, ni en CTAs, ni en alt text. No hay excepciones. Un solo emoji invalida la pieza.
2. **Nunca "usted"** — ni en propuestas formales, ni en comunicacion con directivos. El tuteo es norma absoluta de Indaws en todos los canales y registros.
3. **Authority 8/10 pero nunca arrogante** — hablamos con autoridad de Gold Partner pero sin despreciar alternativas ni hacer afirmaciones que no podemos respaldar con datos o casos reales.
4. **"Problema" es vocabulario prohibido** — siempre "reto". Del mismo modo: "coste" → "inversion", "soporte" → "acompanamiento", "mantenimiento" → "evolucion continua". Verificar SIEMPRE la tabla completa.
5. **Cero hashtags en LinkedIn** — es una regla de marca, no una preferencia. LinkedIn no lleva hashtags en contenido Indaws.
6. **Los nombres de marca son invariables** — "Indaws" con I mayuscula, "Odoo" con O mayuscula, "Indaws Core" completo. Nunca "ODOO", nunca "indaws", nunca "Core" solo.
7. **La voz es constante, el tono varia** — no confundir los 7 atributos de voz (constantes) con las adaptaciones por canal (variables). El registro interno es mas informal pero sigue siendo Indaws.

## ALWAYS

- Leer `dynapps-brand` (common-skills) antes de cualquier operacion de voz de marca
- Verificar vocabulario contra `references/vocabulary.md` en cada pieza
- Tutear al lector en todos los canales y registros
- Respaldar afirmaciones de autoridad con datos o casos reales
- Aplicar variante peninsular es-ES (acentos, signos de apertura, variante lexica)

## NEVER

- Usar emojis en ningun contenido oficial
- Usar hashtags en LinkedIn ni en ningun canal oficial
- Usar "usted" o "vosotros"
- Usar terminos prohibidos: coste, problema, soporte, mantenimiento, tecnicos, presupuesto, instalacion, herramienta, plantilla, crisis
- Hacer afirmaciones superlativas absolutas ("el mejor", "garantizamos resultados")
- Mencionar competidores en LinkedIn o redes sociales
- Generar contenido final sin pasar el compliance checklist

## Delegates to

| Skill | Suite | When |
|-------|-------|------|
| dynapps-brand | common-skills | Fuente principal de identidad de marca (SIEMPRE cargar primero) |
| content-creation | ai-claude-marketing | Redaccion de blog posts, landing pages, case studies |
| linkedin-content-manager | ai-claude-marketing | Produccion y publicacion de posts LinkedIn |
| email-campaign-builder | ai-claude-marketing | Redaccion de campanas de email en Odoo |

## Consumed by

| Skill | Suite | Why |
|-------|-------|-----|
| content-creation | ai-claude-marketing | Calibrar tono y vocabulario en cada pieza de contenido |
| campaign-planning | ai-claude-marketing | Coherencia de voz a lo largo de la campana |
| linkedin-content-manager | ai-claude-marketing | Adaptaciones especificas LinkedIn (tono 5/10, sin hashtags) |
| email-campaign-builder | ai-claude-marketing | Adaptaciones email (complexity 4, un solo CTA) |
| case-study-generator | ai-claude-marketing | Vocabulario de exito y tono orientado a valor |
| visual-brand-strategist | marketing | Revision de voz de los nombres de paleta y textos del moodboard |

## REFERENCES

| File | Content |
|------|---------|
| `references/voice-attributes.md` | 7 atributos de voz con escalas 1-10, matrix de adaptacion por canal, ejemplos antes/despues |
| `references/vocabulary.md` | Tabla de vocabulario Indaws, nombres de producto, terminos de industria, terminos prohibidos, reglas de mencion de competidores, reglas es-ES |
| `references/compliance-checklist.md` | Checklist de reglas inquebrantables (emojis, hashtags, tratamiento) y protocolo de revision en 7 pasos |
| `common:references/shared/` | Patrones de conexion y protocolo compartidos del ecosistema |
