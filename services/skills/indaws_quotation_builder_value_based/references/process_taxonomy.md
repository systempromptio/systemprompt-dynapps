# Process taxonomy — E2E process view (P2P / O2C / R2R + residual)

Literature analysis behind the proposal's **process view** (`module_view: process`):
which end-to-end process frameworks exist, where the boundaries are, and how each
Odoo app in `shared/app_registry.json` maps to ONE process (`apps[*].process`).
The client sees their operation as three chained flows plus a residual group —
before the module breakdown — instead of a flat app list.

**Changing the mapping = editing `app_registry.json`, never code.** Labels are
localized via `strings.sections.proc_<key>_name/_tag` (process names themselves —
Procure-to-Pay, Order-to-Cash, Record-to-Report — are consulting canon and stay
in English in every locale; only taglines translate).

## 1. Frameworks reviewed

| Framework | Taxonomy | Notes |
|---|---|---|
| **APQC PCF** (v7.x, 13 categories) | Operating processes 1–6 (Vision, Develop Products, Market & Sell, Deliver Physical, Deliver Services, Customer Service) + Management & Support Services 7–13 (Human Capital, IT, Finance, Assets, Risk, External Relations, Capabilities) | E2E mega-processes cut ACROSS categories: O2C ≈ 3+6+parts of 9; P2P ≈ 4.2+parts of 9. "Management & Support Services" is the canonical label for the residual bucket. |
| **SAP Signavio / S/4HANA** | Seven cross-industry E2Es: Idea-to-Market, Source-to-Pay, Plan-to-Fulfill, Lead-to-Cash, Recruit-to-Retire, Acquire-to-Decommission, + Finance & Governance | The maximalist list. Lead-to-Cash extends O2C upstream into CRM/marketing; Source-to-Pay extends P2P upstream into sourcing/contracting. |
| **Big-4 / Gartner ERP practice** | The classic short names as workstream headers: **O2C, P2P, R2R, H2R** (+ Plan-to-Produce when manufacturing is heavy) | This is the proposal-deck standard and what mid-market clients recognise. We follow it. |
| **Odoo official docs** | Functional grouping (Finance, Sales, Websites, Supply Chain, Manufacturing, HR, Marketing, Services, Productivity) — NOT E2E | Odoo itself markets "Procure-to-Pay workflow" webinars; partners map Q2C = CRM/Sales/Invoicing and P2P = Purchase/Inventory-in/Vendor bills. |

Other E2Es in the literature not used as first-class groups here (folded per §3):
Hire-to-Retire (HR), Plan-to-Produce/Plan-to-Fulfill (manufacturing+warehouse),
Issue-to-Resolution (helpdesk/field service), Acquire-to-Retire (fixed assets),
Project-to-Result / Service-to-Cash (PSA), Quote-to-Cash, Meter-to-Cash (utility
variant).

## 2. Boundary decisions (the ones that matter)

- **Customer invoicing → O2C.** Invoice issuance, AR and cash application are O2C;
  **R2R starts at the GL** (subledger posting, close, consolidation, statements).
  This is the consistent convention across NetSuite/SAP/consulting sources — so
  the Odoo app *Facturación* is O2C, not R2R.
- **Vendor bills/AP → P2P** (3-way match + payment); R2R takes over at GL posting.
- **Inventory/warehouse**: strictly its own supply-chain flow (Plan-to-Fulfill).
  Convention in a lean grouping: goods **receipt** is P2P, goods **issue** is O2C.
  In a 4-group mid-market proposal we fold warehouse into the P2P group and say so
  in the tagline ("aprovisionamiento, stock y producción").
- **Manufacturing → Plan-to-Produce**, never O2C. Folded into the P2P group
  (SAP-style "Source-to-Pay & Supply Chain" extension). If the Inbuilt /
  construction vertical grows a manufacturing-heavy pipeline, promote a 4th core
  process `plan_to_produce` in the registry instead of overloading P2P.
- **CRM/marketing**: outside classic O2C (which starts at the confirmed order);
  SAP's Lead-to-Cash and Odoo practice pull them in upstream. We follow
  Lead-to-Cash pragmatically: CRM, email marketing and eCommerce sit in the O2C
  group — one customer-facing flow, no separate "Lead-to-Order" header.
- **Helpdesk → Issue-to-Resolution**, customarily appended to the customer-facing
  group in proposals → O2C.
- **HR → Hire-to-Retire**, always its own E2E in the literature → our residual
  group, which is named after it.
- **Projects/PSA → Project-to-Result**; merged into O2C when projects are
  billable. Our Proyecto app is used both ways → residual group (transversal).

## 3. The mapping (registry `apps[*].process`)

Order of the flow graphic (client requirement): **P2P → O2C → R2R**, residual last.

| Process (key) | Header | Odoo apps |
|---|---|---|
| `p2p` | Procure-to-Pay | Compras, Inventario, Fabricación (MRP) |
| `o2c` | Order-to-Cash | CRM, Ventas, Facturación, Website/eCommerce, Marketing por email, Helpdesk |
| `r2r` | Record-to-Report | Contabilidad |
| `transversal` | Personas y soporte (residual) | RRHH (Hire-to-Retire), Proyecto, Dynapps Construcción, Configuraciones Generales* |

\* `config_general` carries `process: transversal` for completeness but never
appears in groups: `build_context` splits it out as the non-uncheckable included
row before grouping.

## 4. How the engine uses it

- `build_context` annotates each active app with `process`, picks the view
  (`extra_data.module_view` overrides; auto = `process` iff ≥ `PROCESS_VIEW_MIN_APPS`
  (6) scoped apps AND ≥ 2 processes represented, else `flat`), stable-sorts
  `c["apps"]` by `processes._order` and exposes `c["process_groups"]`.
- **Flat view emits NOTHING new** — no CSS, no markers — so small deals and the
  es golden fixtures stay byte-identical (parity gate intact; only the `ctx.json`
  goldens were re-frozen for the two additive keys).
- Web: `process_flow_web` renders the **Odoo metro map** in the MINDMAP slot
  (`mindmap_section_web` returns it with `sid="mapa"` in process view; the
  mindmap only renders in flat view). Hover lights up any station (icon+name),
  including unlit ones. `registry.flows` defines the FULL Odoo Enterprise network (6 lines:
  `mkt` Market-to-Lead feeding into CRM, `l2c` with smart-naming rules, `p2p`
  with Calidad/Mantenimiento, `r2r`, `h2r` with the whole employee journey,
  `serv`); curated non-sellable apps are `flows.ghosts` entries (`~id` slots,
  official icons in assets/odoo-icons, labels via strings.sections.metro_app_<id>;
  POS/Suscripciones/PLM/productivity apps deliberately left off for readability).
  v4 topology: no milestones, no terminus; interchanges CRM (mkt→l2c) and
  Ventas (l2c→serv, a VERTICAL line — a sale spawns a project — drawn first so
  it tunnels under P2P/H2R at crossings); every money line (l2c via
  Facturación, p2p direct, h2r via Gastos (Nóminas removed: never sold in ES)) converges into the
  **Contabilidad hub** (r2r line, no drawn track), showing Odoo's integration.
  Paths are pointer-transparent so crossings never block station clicks.
  The network is ALWAYS drawn entirely — `_metro_model` assigns each station a state: `on` (in scope,
  selected), `fase2` (in scope, not preselected; dashed), `ghost` (not in this
  deal; greyed, resolved from the registry + locale catalog, not clickable). A
  line is LIT iff ≥1 station is `on`; unlit lines/milestones grey out. **No
  prices on the map** (processes, not money). `_metro_layout` holds the fixed
  geometry (`_METRO_GEOM`) server-side; `_METRO_JS` draws and syncs two-way
  with the single-list checkboxes via `data-app` (B3 keys). The L1 name
  resolves from the first LIT naming rule (click → lead → quote → order;
  localized via `strings.sections.metro_line_l2c_<variant>`, English canon in
  `_METRO_EN_CANON`). Process headers in the app cards and price list stay
  (`_PROC_HEAD_CSS`; header `<li>` rows carry no checkbox → invisible to the
  recalc JS).
- PDF: `_pdf_procflow_band` renders the same model/layout as a **static SVG**
  (no JS, no prices) with legend chips (unlit lines dimmed), before the app
  pages; plus process subheads in the Inversión apps block (sentence case since
  v5 — NO CAPS rule, letter-spacing keeps the micro-label look). predemo:
  same map, non-interactive.
- **Sale-order spreadsheet templates are NOT affected**: the co-signed sheet keeps
  its internal functional categories (Comercial/Operaciones/Finanzas/…, a
  different axis) and `generate_spreadsheet_template.py` output is byte-identical
  modulo random sheet UUIDs — no re-deploy of template ids 1/72/16/71 needed.

## Sources

- APQC PCF: <https://www.apqc.org/resource-library/resource-collection/apqcs-process-classification-framework-pcf-cross-industry-and>
- SAP Signavio E2E processes: <https://community.sap.com/t5/technology-blog-posts-by-sap/end-to-end-processes-and-modular-processes-in-sap-signavio-process-insights/ba-p/13694038>
- SAP Learning, core E2E business processes: <https://learning.sap.com/courses/introducing-sap-s-core-business-processes-sap-for-industries-and-the-sap-partner-network/understanding-sap-s-end-to-end-business-processes>
- O2C vs R2R boundary: <https://www.netsuite.com/portal/resource/articles/accounting/order-to-cash-otc-o2c.shtml> · <https://www.mhcautomation.com/blog/p2p-vs-r2r-vs-q2c-vs-o2c/>
- Odoo app grouping (official docs): <https://www.odoo.com/documentation/18.0/applications.html>
