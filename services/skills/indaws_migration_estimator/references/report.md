# Informe PDF de migración — estructura canónica

El entregable de Fase 1 es un **informe PDF** (Dynapps brand) que Dynapps presenta a
su cliente final. Se genera con la skill `dynapps-brand` (+ `pdf-renderer`).

Estructura de referencia = PDFs **Geosquare** y **Luminex** (formato actual). El
formato antiguo "Code Refactoring / 1A-1B" (Newstairs) está **deprecado**: usar las
4 secciones de abajo.

## Portada
- Logo dynapps, "ODOO MIGRATION ESTIMATE".
- Título: `<Cliente>` · `Odoo vX → vY`.
- Subtítulo: "Direct 1:1 technical migration".
- Meta-fila: CLIENT · DATE · MODULES ANALYSED · TOTAL ESTIMATE · STRATEGY (Direct 1:1).
- Footer: "Dynapps — Odoo Migration Services · Confidential".

## 01 — Executive Summary ("Protected budget at a glance")
Las **4 cards numeradas** (los "puntos 1 2 3 4") + card TOTAL:

| # | Card | Subtexto | Regla |
|---|---|---|---|
| 1 | **Code Migration** | "N modules · 1:1 migration" | variable (ver `calibration.md`) |
| 2 | **Database Upgrade** | "~X GB · K iterations" | variable, floor 20h |
| 3 | **Unit Testing** | "Capped allocation" | **10 h** fijo |
| 4 | **Project Management** | "Coordination & follow-up" | **10 h** fijo |
| — | **TOTAL PROJECT** | "Code · database · testing · PM" | suma de 1+2+3+4 |

Más: cards SOURCE VERSION / TARGET VERSION (default v19.0) / STRATEGY (Direct 1:1).

## 02 — Code Migration (tabla por módulo)
Encabezado: "N in-scope modules, 1:1 strategy". Indicar reparto in-house/vendor y
que **los módulos OCA se excluyen** cuando existe port nativo v19 o equivalente en
core. **Excepción:** un OCA sin port v19 ni equivalente en core entra al desglose
como módulo a migrar.
Columnas: `Module · Functionality · Key vX→vY Changes` (o `Type · Tier`) · `Hours`.
Una fila por módulo 🔴 Custom. Cierre con **Subtotal · Code Migration = bucket 1**.

## 03 — Database, Testing & Management
Tres bloques/cards:
- **Database upgrade**: "X GB, K iterations (Test 1 / Test 2 / Rehearsal / Production)".
  Nota: excluye remediación de calidad de datos (desconocida hasta el primer test). → bucket 2.
- **Unit testing**: "Capped, prioritised allocation (heaviest/critical first)". → **10 h**.
- **Project management**: "Coordination, planning, follow-up, hand-offs, release". → **10 h**.

## 04 — Critical Points / Findings ("What drives the risk")
Bullets derivados del análisis estático. Patrones típicos:
- **Salto de N versiones** y breaking changes cruzados (attrs/states, name_get,
  tree→list, groups_id→group_ids, category_id→privilege_id, OWL Interaction v19…).
- **Módulos más pesados** (con sus horas y por qué).
- **Reescrituras OWL obligatorias** (JS legacy incompatible v19).
- **Raw SQL** a refactorizar a ORM.
- **Integraciones externas / auth sensibles** (validación contra sistemas vivos).
- **Drift en módulos OCA excluidos** (si parcheados localmente, vuelven a scope).

### Decisiones pendientes — OCA / terceros a portar (OBLIGATORIO si los hay)
Por cada módulo OCA o de terceros que **no tiene port v19 ni equivalente en core** y
que por tanto **habría que portar** (caso 3 del árbol de `calibration.md`): listarlo
aquí explícitamente con sus **horas estimadas**, marcado como **decisión de Dynapps/
cliente** (portar o descartar). Formato sugerido: mini-tabla `Módulo · Por qué no hay
alternativa v19 · Horas si se porta · Recomendación`. Estas horas **ya están sumadas
en el bucket 1 / total** del presupuesto; el PDF deja claro que son **opcionales sujetas
a decisión**, para que el cliente pueda quitarlas si decide no portar ese módulo.
- Distinguir de los OCA/terceros **excluidos** (casos 1 y 2: existe port v19 o ya es
  nativo) → esos NO se listan como decisión, solo se mencionan como "cubiertos por
  port nativo/OCA v19".

## Plantilla y destinatario
- Plantilla: Dynapps Brand tal cual (estos PDFs son la referencia visual).
- Destinatario de referencia: **Janik** (Dynapps); lo confirma el usuario.
- El PDF se **adjunta** al sale.order (ver `quotation.md`); **no se envía por correo
  desde Odoo** (regla dura del suite).

## Importes: SOLO horas en el PDF
El informe muestra **únicamente horas** (por bucket, por módulo, total). **Nunca** € ni
"@ 45 €/h" en portada, cards, tablas ni footer. El precio va solo en el `sale.order`
(ver `business-rules.md` §Importes y `quotation.md`).

## Render del PDF — SE LLAMA A `dynapps-brand` (NO re-implementar aquí)
El PDF de marca lo genera **`dynapps-brand`**, que es el SSOT (plantilla + scripts + QA).
migration-estimator **solo rellena contenido** y llama a sus scripts. NO clones el CSS
ni los scripts de render aquí (así se propagaron los artefactos al no haber fuente única).

1. **Plantilla**: copia `dynapps-brand/assets/templates/migration-report-template.html`
   y sustituye los `{{placeholders}}` + filas/bloques `repeat` con el contenido del
   informe (4 secciones de arriba). No reescribas su CSS.
2. **Render + footer band**:
   ```bash
   node <dynapps-brand>/scripts/gen.js report.html <outdir> "© 2026 Dynapps · <Cliente> · Odoo <s> → <t> · Confidential"
   python3 <dynapps-brand>/scripts/stamp.py <outdir>/content.pdf <outdir>/band.pdf report.pdf
   ```
3. **QA obligatorio** antes de adjuntar/crear el pedido:
   `python3 <dynapps-brand>/scripts/verify.py report.pdf` → debe dar `RESULT: OK`.
   Incluye el chequeo de **márgenes laterales**: falla si el contenido está a <7 mm de un
   borde o desplazado a la derecha. Causa típica: un `code` o identificador largo sin
   partir (`relative_uom_id`, `stock.move.line…`) en una tarjeta `.find` o celda. La
   plantilla ya lo evita (minmax(0,1fr) + overflow-wrap); si falla, no metas CSS propio:
   acorta/parte el texto o actualiza `dynapps-brand`.

Detalle de marca, las **2 reglas críticas de CSS** (full-bleed=100vh; sin border en
contenedores que se parten) y la mecánica de paginación: `dynapps-brand/references/pdf-reports.md`
y el SSOT del motor `pdf-renderer/references/playwright-pdf-guide.md`.
