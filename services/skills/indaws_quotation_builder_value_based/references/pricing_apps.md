# Pricing per Application — Skill Reference

Source: `pricing_apps.md` v3.0 (2026-04-09). **INTERNAL USE ONLY.**

> **Note on language:** the add-on names below are kept in **Spanish** (with an English gloss in
> parentheses) because they are the literal feature names used in the Spanish spreadsheet (`es` is
> this system's byte-parity reference locale) and match `app_overrides.<APP>.features` keys
> verbatim regardless of the proposal's output language (see `references/localization.md`).

## General Rules

- **Reference** prices. The sales rep can adjust **±15%**. More than 15% requires management approval.
- The add-on catalogue below is **fixed**, but it is not a ceiling: an add-on the deal needs and the
  catalogue lacks can be created per deal — see **Dynamic complements** at the end of this file.
- Add-ons marked with ⚙️ include the **Dynapps Core** module (absorbed into the price, not visible to the client).
- **General Configuration** always applies (every project).
- The client sees: a per-app card with a closed price = `base + Σ selected add-ons`.
- The client does NOT see: hours, the Core breakdown, internal margins.

## Pricing Table

### 0. General Configuration — Base: €500

| Add-on | Price | Signal |
|-------------|--------|-------|
| Multi-compañía (Multi-company) | +€250/additional company | Group of companies |
| Multi-idioma (Multi-language) | +€250 | International clients/suppliers |
| 30-60 usuarios (30-60 users) | +€250 | Medium-sized team |
| 60-100 usuarios (60-100 users) | +€500 | Large team |
| +100 usuarios (100+ users) | +€750 | Large company |

### 1. CRM — Base: €750

Includes: pipeline ≤3 sales reps, custom fields, activities, contacts, reporting, ⚙️ Projects on opportunities. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| 4-10 comerciales (4-10 sales reps) | +€500 | Multi-team |
| +10 comerciales (10+ sales reps) | +€1,000 | Large sales team |
| Lead scoring y nurturing (Lead scoring & nurturing) | +€500 | Automatic qualification |
| Automatizaciones comerciales (Sales automations) | +€250 | Automatic emails, alerts |
| Migración oportunidades (Opportunity migration) | +€500 | Prior historical pipeline |
| ⚙️ Prospección avanzada (Advanced prospecting) | +€500 | Lead enrichment |

### 2. Sales — Base: €1,500

Includes: quotes, orders, multi-price-list, product catalog, customer portal, reporting, ⚙️ Multiple discounts, ⚙️ Quote versions, ⚙️ Quote enhancements, ⚙️ Per-customer product codes, ⚙️ Sales documents. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Suscripciones / Recurrencia (Subscriptions / Recurring) | +€750 | Recurring contracts |
| Plantillas adicionales (Additional templates) | +€250 | >1 quote template |
| Migración productos (Product migration) | +€250 | Catalog from a previous system |
| Migración clientes (Customer migration) | +€250 | Prior customer records |
| Migración tarifas (Price-list migration) | +€500 | Prior price lists |
| ⚙️ Flujo de aprobación (Approval workflow) | +€500 | Approval by amount/discount |
| ⚙️ Kits (productos compuestos) (Kits — composite products) | +€500 | Packs/kits with a breakdown |
| ⚙️ Flujos por tipo de pedido (Flows by order type) | +€500 | Different sales flows |
| ⚙️ Fidelización (Loyalty) | +€500 | B2B points program |

### 3. Purchasing — Base: €1,000

Includes: purchase orders, multi-supplier, RFQ, invoice control, supplier migration, reporting, ⚙️ Purchase-sales integration. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Reglas de reabastecimiento (Replenishment rules) | +€500 | Automatic minimum-stock replenishment |
| Acuerdos marco (Framework agreements) | +€500 | Supplier contracts |
| Dropshipping | +€500 | Direct supplier→client shipping |
| Migración tarifas compra (Purchase price-list migration) | +€250 | Prior purchase prices |
| ⚙️ Gestión avanzada compras (Advanced purchasing management) | +€750 | Supplier evaluation, alerts |

### 4. Inventory — Base: €1,250

Includes: 1 warehouse, movements, products, product migration, periodic inventory, reporting, ⚙️ Valued delivery notes, ⚙️ Advanced stock management. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Multi-almacén (Multi-warehouse) | +€250/warehouse | >1 physical location |
| Trazabilidad por lotes (Lot traceability) | +€500 | Batch/lot tracking |
| Trazabilidad por nº de serie (Serial-number traceability) | +€500 | Unit-level tracking |
| Fechas de caducidad (Expiry dates) | +€250 | Perishable products |
| Rutas logísticas avanzadas (Advanced logistics routes) | +€500 | Pick-pack-ship, cross-docking |
| Códigos de barras (Barcodes) | +€500 | Scanner-based operations |
| Reabastecimiento automático (Automatic replenishment) | +€500 | Replenishment between warehouses |
| Migración stock actual (Current stock migration) | +€500 | Inventory with lots/serials |
| ⚙️ Generador nº serie/lote (Serial/lot number generator) | +€250 | Automatic naming |
| ⚙️ Stock en proyectos (Stock in projects) | +€500 | Materials linked to projects |
| ⚙️ Marcas y etiquetas (Brands & tags) | +€250 | Advanced categorization |

### 5. Invoicing — Base: €1,500

Includes: customer/supplier invoicing, full flow, collections/payments, reporting, ⚙️ Invoicing utilities. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| ⚙️ Generador de PDFs (PDF generator) | +€500 | Custom print templates |
| ⚙️ Márgenes y costes avanzados (Advanced margins & costs) | +€500 | Margin control, landed costs |

### 6. Accounting — Base: €2,000

Includes: Spanish chart of accounts, bank reconciliation, fixed assets, closing entries, collections/payments, chart-of-accounts + opening-entry migration, reporting. 3 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| SII | +€500 | Turnover >€6M or opted-in |
| Domiciliación SEPA (SEPA direct debit) | +€750 | SEPA direct-debit collections (⚙️ Core included) |
| Multi-divisa (Multi-currency) | +€250 | Multi-currency operations |
| Contabilidad analítica (Analytic accounting) | +€500 | Cost centers |
| Presupuestos contables (Accounting budgets) | +€500 | Variance control |
| Intrastat / modelo 349 (Intrastat / form 349) | +€250 | Intra-EU operations |
| ⚙️ Mejoras SII (SII enhancements) | +€500 | SII dashboard, retries (requires SII) |

### 7. Project — Base: €1,500

Includes: multi-project, tasks/subtasks, timesheets, collaboration, reporting, ⚙️ Per-project clock-in. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Facturación por proyecto (Per-project invoicing) | +€500 | Invoicing hours/milestones to the client |
| Planificación de recursos (Resource planning) | +€500 | Workload, availability |
| ⚙️ Stock en proyectos (Stock in projects) | +€500 | Materials per project |

### 8. HR — Base: €1,000

Includes: employees, departments, contracts, clock-in, employee migration. 1 training session.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Ausencias y vacaciones (Time off & leave) | +€500 | Time-off request/approval |
| Gastos (Expenses) | +€500 | Employee expenses |
| Evaluaciones / Desempeño (Reviews / Performance) | +€500 | Reviews, 360° feedback |
| Recruiting | +€500 | Hiring processes |
| Flota de vehículos (Vehicle fleet) | +€500 | Company vehicles |
| Formación de empleados (Employee training) | +€250 | Internal courses |
| 50-100 empleados (50-100 employees) | +€250 | Medium headcount |
| +100 empleados (100+ employees) | +€500 | Large headcount |
| Migración saldos vacaciones (Leave-balance migration) | +€250 | Prior pending balances |
| ⚙️ Portal de empleado (Employee portal) | +€500 | Employee self-service portal |

### 9. Manufacturing (MRP) — Base: €2,000

Includes: bills of materials, 1 work center, manufacturing orders, material consumption, reporting. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Multi-centro de trabajo (Multi work-center) | +€250/center | >1 production center/line |
| Órdenes de trabajo (Work orders) | +€500 | Per-center operations |
| Subcontratación (Subcontracting) | +€750 | Manufacturing at suppliers |
| Planificación producción (MPS) (Production planning — MPS) | +€750 | Master plan, demand forecast |
| Control de calidad (Quality control) | +€750 | Checks, alerts, certificates |
| PLM | +€500 | BoM versions, ECOs |
| Mantenimiento (Maintenance) | +€500 | Preventive and corrective |
| Despiece / Multi-nivel (Multi-level BoM) | +€500 | Multi-level BoM |
| Migración LdM (BoM migration) | +€500 | Prior bills of materials |
| Migración rutas/centros (Routes/centers migration) | +€250 | Prior centers and routes |

### 10. Website / eCommerce — Base: €1,500

Includes: 5-page website, blog, contact form, customer portal, basic SEO, responsive, ⚙️ eCommerce enhancements. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Páginas adicionales (Additional pages) | +€100/page | >5 static pages |
| Tienda online (eCommerce) (Online store — eCommerce) | +€1,500 | Catalog, cart, checkout |
| Multi-idioma web (Multi-language site) | +€500 | Site in several languages |
| Diseño personalizado (Custom design) | +€750 | Bespoke design |
| Chat en vivo (Livechat) (Live chat) | +€250 | Visitor chat support |
| Migración contenidos (Content migration) | +€500 | Prior website content |
| ⚙️ Google Merchant | +€250 | Google Shopping sync |

### 11. Helpdesk — Base: €1,000

Includes: 1 support team, tickets, customer portal, knowledge base, reporting. 2 training sessions.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Equipos adicionales (Additional teams) | +€250/team | >1 support team |
| Partes de horas en tickets (Time logging on tickets) | +€250 | Time tracking per ticket |
| Facturación de soporte (Support invoicing) | +€500 | Invoicing support hours |
| Automatizaciones (Automations) | +€250 | Auto-assignment, escalation |
| Chat en vivo (Live chat) | +€250 | Livechat with tickets |

### 12. Email Marketing — Base: €750

Includes: email campaigns, contact lists, 3 templates, reporting. 1 training session.

| Add-on | Price | Signal |
|-------------|--------|-------|
| Automatización de marketing (Marketing automation) | +€750 | Sequences, triggers |
| Segmentación avanzada (Advanced segmentation) | +€250 | Dynamic segments |
| Plantillas adicionales (Additional templates) | +€250 | >3 email templates |
| SMS marketing | +€500 | SMS campaigns |
| ⚙️ Diseño avanzado email (Advanced email design) | +€500 | Professional templates |
| ⚙️ Correo saliente mejorado (Enhanced outbound mail) | +€250 | Tracking, retries |

## Base-Price Summary Table

| # | App | Base |
|---|-----|------|
| 0 | General Configuration | €500 |
| 1 | CRM | €750 |
| 2 | Sales | €1,500 |
| 3 | Purchasing | €1,000 |
| 4 | Inventory | €1,250 |
| 5 | Invoicing | €1,500 |
| 6 | Accounting | €2,000 |
| 7 | Project | €1,500 |
| 8 | HR | €1,000 |
| 9 | Manufacturing (MRP) | €2,000 |
| 10 | Website / eCommerce | €1,500 |
| 11 | Helpdesk | €1,000 |
| 12 | Email Marketing | €750 |

## Functional analysis and custom development

Formulas, internal rates (€80/h analysis, €75/h development) and the hours table by development type: see **`analysis_pricing.md`** (single source of truth).

## Maintenance

| Item | Price |
|----------|--------|
| Base | €320/month (5h included) |
| Extra hour | €64/h |
| Includes | Functional questions, bug fixes, development warranty |
| Excludes | Training, new development, Core, Studio, out-of-scope apps |

## Dynamic complements (priced per deal)

An add-on the deal genuinely needs but the catalogue above does not carry is created per deal, in one
of the **5 free slots per app** the workbook reserves (`generate_spreadsheet_template.py
--free-slots 5`; retrofit deployed base workbooks with `scripts/insert_free_slots.py`). So the ceiling
per app is **catalogue + 5**.

**Price = estimated hours × the implementation rate READ FROM THE SPREADSHEET** (`'Interna
Comercial'!B5`, default €75/h — `proposal_data.soporte_rates`), rounded with the engine's own money
rounding (nearest 10 for `es`). The sheet is the source of truth, so a per-deal rate adjustment by the
rep is applied automatically. A closed `unit_price` may be given instead, skipping the rate.

**Sanity band:** a unit price outside `custom_complement_band.min` … `custom_complement_band.max`
(es: €250 … €5,000) warns on stderr and is **rejected** unless the sales rep confirms it explicitly
(`"confirmado": true`). Same spirit as the ±15% adjustment rule: an off-band number reaching a client
is a management decision, not an agent's.

Complements that show up in more than one deal are surfaced as promotion candidates by
`scripts/complement_candidates.py report`. Authoring detail, guards and the `extras_custom` schema:
**`references/addon_details.md` §Dynamic complements**.
