---
name: indaws-competitive-intelligence
description: >
  Investiga competidores y genera battlecards accionables para el equipo comercial.
  Analiza otros partners de Odoo en Espana, ERPs competidores (SAP, Dynamics, Sage,
  Holded, A3ERP) y soluciones de nicho. Genera fichas con overview, producto,
  diferenciacion, talk tracks, preguntas trampa y datos win/loss de Odoo.
  Usar cuando el usuario pida: competencia, battlecard, competitive intelligence,
  comparar con [competidor], como ganamos a [competidor].
version: 1.0.1
model: sonnet
effort: medium
---

# Competitive Intelligence

Investigacion de competidores y generacion de battlecards para el mercado espanol de implantacion ERP/Odoo. Cubre partners Odoo, ERPs enterprise/mid-market, cloud/startup y soluciones verticales.

---

## Prerequisitos

- **odoo-pilot**: Para datos win/loss del CRM
- **Credenciales Odoo**: Variables `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, `ODOO_USER`
- See references/shared/odoo-connection.md for connection pattern.

---

## Tipos de Competidores

| Categoria | Ejemplos | Frecuencia |
|-----------|----------|-----------|
| **Partners Odoo Espana** | Otros Gold/Silver partners | Alta |
| **ERPs enterprise** | SAP Business One, Dynamics 365 | Media |
| **ERPs mid-market** | Sage X3, Sage 200, A3ERP | Media |
| **ERPs cloud/startup** | Holded, Factorial, Personio | Alta |
| **Soluciones verticales** | Software especifico de sector | Baja |

---

## Workflow

> Detalle completo de cada paso en `references/workflow_detail.md`

1. **Identificar competidor** - Preguntar: competidor, contexto del deal, que necesita (battlecard completa o respuesta puntual). DETENTE y espera respuesta.
2. **Investigar** - Consultar CRM via odoo-pilot (leads ganados/perdidos vs competidor) + web publica (producto, precios, reviews G2/Capterra) + `references/competitor_landscape.md`.
3. **Generar battlecard** - Template de 7 secciones: overview, producto, diferenciacion, talk tracks, preguntas trampa, win/loss insights, cuando ganamos/perdemos.
4. **Guardar** (opcional) - Si el usuario lo pide, persistir en `references/competitor_landscape.md`.

---

## RULES

1. **NUNCA** inventar datos del competidor - solo informacion publica verificable.
2. **NUNCA** difamar al competidor - ser objetivo y factual.
3. Datos win/loss deben venir de Odoo CRM (no inventados). Si no hay datos, indicar "Sin datos historicos".
4. Talk tracks positivos (hablar de Indaws, no atacar). Preguntas trampa sutiles (no agresivas).
5. Debilidades del competidor deben ser verificables (no opiniones).
6. Actualizar battlecards trimestralmente como minimo.

---

## GOTCHAS

1. CRM puede no tener tag del competidor - buscar tambien en `description` con `ilike`.
2. Leads perdidos requieren `active=False` en el dominio.
3. No todos los competidores tienen presencia publica suficiente - indicar gaps.
4. Partners Odoo cambian de nivel (Gold/Silver) frecuentemente - verificar en odoo.com.
5. Precios de ERPs enterprise rara vez son publicos - usar rangos aproximados con disclaimer.
6. Reviews en G2/Capterra pueden estar sesgadas - cruzar multiples fuentes.
7. Datos win/loss pueden contener informacion sensible del cliente - no compartir externamente.

---

## Agent Routing

| Fase | Modelo | Justificacion |
|------|--------|---------------|
| Investigacion web + CRM | sonnet | Busqueda y agregacion de datos |
| Generacion battlecard | sonnet | Template filling con datos recopilados |
| Analisis win/loss | sonnet | Patrones en datos estructurados |

---

## Delegates to

| Skill | Suite | When | Agent()? |
|-------|-------|------|----------|
| odoo-pilot | common | CRM/partner data queries | No |

## Consumed by

| Skill | Suite | Purpose |
|-------|-------|---------|
| call-prep | sales | Competitive context for meetings |
| presales workflows | sales | Market positioning data |

---

## References

| File | Contents |
|------|----------|
| `references/competitor_landscape.md` | Base de datos de competidores conocidos |
| `references/workflow_detail.md` | Pasos detallados, template battlecard, CRM queries, success criteria |
