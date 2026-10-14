# Add-on Detection Signals

Guide for automatically detecting which add-ons apply, based on the intake data (`xx_data_collection`).

For each add-on, the **signals** (keywords, phrases, patterns) that indicate the add-on is needed are listed.

> **Note on language:** the "Signals" column is kept in **Spanish** on purpose — these are literal
> keywords matched against the intake document (`toma de datos`), which is written in Spanish for
> Spanish-speaking clients (`es` is this system's reference market/locale). Translating them would
> break the pattern match. Add-on names carry an English gloss in parentheses.

---

## 0. General Configuration

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Multi-compañía (Multi-company) | "grupo de empresas", "filial", "varias sociedades", "inter-company", "consolidación", >1 tax ID |
| Multi-idioma (Multi-language) | "internacional", "exportación", "clientes extranjeros", "inglés", "francés", "portugués", languages mentioned |
| 30-60 usuarios (30-60 users) | User count between 30-60 |
| 60-100 usuarios (60-100 users) | User count between 60-100 |
| +100 usuarios (100+ users) | User count >100 |

## 1. CRM

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| 4-10 comerciales (4-10 sales reps) | "equipo comercial" + count between 4-10, "varios comerciales", "multi-equipo" |
| +10 comerciales (10+ sales reps) | >10 sales reps mentioned, "red comercial", "delegaciones comerciales" |
| Lead scoring | "cualificación", "scoring", "leads automáticos", "nurturing", "embudo" |
| Automatizaciones (Automations) | "emails automáticos", "alertas comerciales", "seguimiento automático" |
| Migración oportunidades (Opportunity migration) | "pipeline actual", "histórico comercial", "CRM anterior", "Salesforce", "HubSpot" |
| ⚙️ Prospección avanzada (Advanced prospecting) | "prospección", "enriquecimiento", "búsqueda empresas" |

## 2. Sales

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Suscripciones (Subscriptions) | "recurrente", "suscripción", "contrato", "mensualidad", "cuota", "renovación", "SaaS" |
| Plantillas adicionales (Additional templates) | "varios tipos de presupuesto", "servicios y productos", "nacional y exportación" |
| Migración productos (Product migration) | "catálogo actual", "productos en ERP anterior", "migrar productos" |
| Migración clientes (Customer migration) | "base de clientes", "fichas de cliente", "migrar clientes" |
| Migración tarifas (Price-list migration) | "tarifas actuales", "listas de precios", "descuentos por cliente" |
| ⚙️ Flujo aprobación (Approval workflow) | "aprobación", "autorización", "descuento máximo", "margen mínimo" |
| ⚙️ Kits | "kit", "pack", "compuesto", "bundle", "lote de productos" |
| ⚙️ Flujos tipo pedido (Order-type flows) | "consignación", "muestra", "venta directa vs distribuidor", "tipos de venta" |
| ⚙️ Fidelización (Loyalty) | "puntos", "fidelización", "programa de puntos", "descuento por volumen acumulado" |

## 3. Purchasing

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Reabastecimiento (Replenishment) | "stock mínimo", "reposición automática", "punto de pedido", "reabastecimiento" |
| Acuerdos marco (Framework agreements) | "contrato de compra", "precio negociado", "acuerdo marco", "tarifa anual" |
| Dropshipping | "dropshipping", "envío directo", "proveedor envía al cliente" |
| Migración tarifas compra (Purchase price-list migration) | "precios de compra", "tarifas proveedor actuales" |
| ⚙️ Gestión avanzada (Advanced management) | "evaluación proveedores", "plazos entrega", "alertas compra" |

## 4. Inventory

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Multi-almacén (Multi-warehouse) | "almacenes", "delegaciones", "nave", >1 location mentioned, "transferencias internas" |
| Trazabilidad lotes (Lot traceability) | "lote", "trazabilidad", "lotes de fabricación", "recall" |
| Trazabilidad nº serie (Serial-number traceability) | "número de serie", "unitario", "garantía por unidad", "serial" |
| Fechas caducidad (Expiry dates) | "caducidad", "perecedero", "fecha consumo", "FEFO", "alimentación" |
| Rutas avanzadas (Advanced routes) | "pick-pack-ship", "cross-docking", "tránsito", "zona de picking" |
| Códigos de barras (Barcodes) | "escáner", "código de barras", "pistola", "PDA almacén", "operarios" |
| Reabastecimiento automático (Automatic replenishment) | "reposición entre almacenes", "stock mínimo por almacén" |
| Migración stock (Stock migration) | "inventario actual", "stock valorado", "migrar stock" |
| ⚙️ Generador nº serie/lote (Serial/lot number generator) | "nomenclatura lotes", "generación automática", "formato serie" |
| ⚙️ Stock en proyectos (Stock in projects) | "materiales por proyecto", "consumo proyecto", "obras" |
| ⚙️ Marcas y etiquetas (Brands & tags) | "marcas", "etiquetas producto", "multi-marca" |

## 5. Invoicing

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| ⚙️ Generador PDFs (PDF generator) | "plantilla factura personalizada", "formato propio", "PDF a medida" |
| ⚙️ Márgenes y costes (Margins & costs) | "margen", "coste landed", "margen mínimo", "control de costes" |

## 6. Accounting

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| SII | "SII", "suministro inmediato", "AEAT", turnover >€6M, "registro de facturas electrónico" |
| SEPA | "domiciliación", "SEPA", "mandato", "cobro bancario", "recibo domiciliado" |
| Multi-divisa (Multi-currency) | "divisa", "dólar", "libra", "moneda extranjera", "tipo de cambio" |
| Contabilidad analítica (Analytic accounting) | "centro de coste", "analítica", "departamental", "por proyecto contable" |
| Presupuestos contables (Accounting budgets) | "presupuesto por departamento", "control desviaciones", "budget" |
| Intrastat | "intrastat", "349", "intracomunitario", "UE" |
| ⚙️ Mejoras SII (SII enhancements) | "errores SII", "dashboard SII", "reintentos" (requires SII active) |

## 7. Project

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Facturación proyecto (Project invoicing) | "facturar horas", "facturar al cliente por proyecto", "hitos facturables" |
| Planificación recursos (Resource planning) | "carga de trabajo", "disponibilidad", "planificación equipo" |
| ⚙️ Stock en proyectos (Stock in projects) | "materiales", "stock por proyecto", "consumo materiales" |

## 8. HR

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Ausencias (Time off) | "vacaciones", "ausencias", "permisos", "calendario equipo", "bajas" |
| Gastos (Expenses) | "gastos empleado", "notas de gasto", "dietas", "kilometraje" |
| Evaluaciones (Reviews) | "evaluación", "desempeño", "objetivos", "feedback", "revisión anual" |
| Recruiting | "selección", "candidatos", "ofertas empleo", "recruiting", "contratación" |
| Flota (Fleet) | "vehículos", "flota", "coches empresa", "renting", "leasing vehículos" |
| Formación empleados (Employee training) | "formación interna", "cursos", "plan de formación" |
| 50-100 empleados (50-100 employees) | Employee count between 50-100 |
| +100 empleados (100+ employees) | Employee count >100 |
| Migración saldos vacaciones (Leave-balance migration) | "saldos vacaciones", "días pendientes", "migrar vacaciones" |
| ⚙️ Portal empleado (Employee portal) | "portal empleado", "autoservicio", "consulta nómina" |

## 9. Manufacturing (MRP)

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Multi-centro (Multi work-center) | >1 "centro de trabajo", "líneas de producción", "plantas" |
| Órdenes de trabajo (Work orders) | "operaciones", "work orders", "secuencia fabricación", "tiempos por operación" |
| Subcontratación (Subcontracting) | "subcontratación", "fabricación externa", "maquila", "proveedor fabrica" |
| MPS | "plan maestro", "previsión demanda", "planificación producción", "MPS" |
| Control calidad (Quality control) | "calidad", "control", "certificados", "ISO", "checks", "inspección" |
| PLM | "versiones", "ECO", "ciclo de vida", "cambios de ingeniería" |
| Mantenimiento (Maintenance) | "mantenimiento", "preventivo", "correctivo", "GMAO", "máquinas" |
| Despiece multi-nivel (Multi-level BoM) | "semielaborado", "producto intermedio", "multi-nivel", "explosión" |
| Migración LdM (BoM migration) | "listas de materiales", "BOM actual", "escandallo" |
| Migración rutas (Route migration) | "centros de trabajo actuales", "rutas fabricación" |

## 10. Website / eCommerce

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Páginas adicionales (Additional pages) | >5 pages mentioned, "muchas secciones" |
| eCommerce | "tienda online", "ecommerce", "venta online", "carrito", "B2C" |
| Multi-idioma web (Multi-language site) | "web en inglés", "web multi-idioma", "internacional" |
| Diseño personalizado (Custom design) | "diseño a medida", "branding", "marca propia", "no template estándar" |
| Chat en vivo (Live chat) | "chat", "livechat", "atención en vivo" |
| Migración contenidos (Content migration) | "web actual", "migrar contenido web", "WordPress actual" |
| ⚙️ Google Merchant | "Google Shopping", "Google Merchant", "catálogo Google" |

## 11. Helpdesk

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Equipos adicionales (Additional teams) | >1 "equipo de soporte", "técnico y comercial", "postventa" |
| Partes horas tickets (Time logging on tickets) | "registrar tiempo soporte", "horas por ticket" |
| Facturación soporte (Support invoicing) | "facturar soporte", "soporte facturable" |
| Automatizaciones (Automations) | "asignación automática", "escalado", "SLA automático" |
| Chat en vivo (Live chat) | "livechat", "chat soporte" |

## 12. Email Marketing

| Add-on | Signals in the intake data (Spanish, matched verbatim) |
|-------------|-------------------------|
| Automatización (Automation) | "secuencia emails", "nurturing", "drip", "automatización marketing" |
| Segmentación avanzada (Advanced segmentation) | "segmentos", "comportamiento", "dinámico" |
| Plantillas adicionales (Additional templates) | >3 email types mentioned |
| SMS marketing | "SMS", "mensajes texto", "campañas SMS" |
| ⚙️ Diseño avanzado (Advanced design) | "diseño profesional email", "newsletter corporativa" |
| ⚙️ Correo saliente mejorado (Enhanced outbound mail) | "seguimiento correo", "reintentos", "logs email" |

---

## Dynamic complements (off-catalogue)

The catalogue above is **fixed**. When the deal context calls for a complement that is **not** in it,
the agent can create one: it lands in the workbook, and from there it flows into the web HTML and the
PDF with **no engine change** (`extract_app_details` / `calc_app_price_split` / `app_feature_items`
already read whatever extras rows exist).

### How it works — reserved free slots

`generate_spreadsheet_template.py --free-slots N` (default `N = FREE_EXTRA_SLOTS = 3`) reserves **N
empty rows per app INSIDE the extras SUM range**, each pre-seeded with the per-row `=Ax*Cx` formula
and grouped under the `Personalizaciones` sub-category. An empty row evaluates to 0, so the subtotal,
the app total, the 10% go-live support and the grand total are **unchanged until a slot is filled**.

Rows are **never inserted at runtime** — formulas use absolute refs (`=ROUND((F10+F15+…)*pct,1)`,
`='Operaciones'!D3`); see the contract on `fill_spreadsheet.delete_rows_from_sheet`. Same idiom as
the existing "max 10 analysis sessions / 5 custom dev items" slot limits.

### Authoring — `budget_json.apps.<id>.extras_custom`

```json
"apps": {
  "crm": {
    "activa": true,
    "extras_activos": ["Lead scoring y nurturing"],
    "extras_custom": [
      {"name": "Portal de obra a medida", "horas": 6, "cantidad": 1}
    ]
  }
}
```

| Field | Meaning |
|---|---|
| `name` (`nombre`) | Complement name. **This is the KEY** of `app_overrides.<APP>.features`, so it MUST be in **Spanish** — the web template's JS branches on these keys. The localized label goes in the object form `{"title": …, "expl": …}`. |
| `horas` (`hours`) | Estimated hours. Price = `horas × implementation rate`, rounded with the engine's own money rounding (`Locale.round_step`, es = nearest 10). |
| `unit_price` (`precio_ud`) | Closed unit price, when the rate must not be applied. Skips the hours calculation; the band still applies. |
| `cantidad` (`qty`) | Units (column A). Default 1. Line total = `cantidad × unit price`. |
| `confirmado` | `true` = the sales rep has explicitly accepted an out-of-band price. |

**The rate comes from the SPREADSHEET**, not from the pack: `'Interna Comercial'!B5` (implementation
€/h) via `proposal_data.soporte_rates`. So a per-deal rate adjustment by the rep is respected
automatically.

### Blocking guards (a model-invented number reaching a client is the PR4021 bug class)

| Guard | Behaviour |
|---|---|
| **Price band** | Unit price outside `[min, max]` → `WARNING` on stderr **and an error**, unless the entry carries `"confirmado": true` (then: warning only, so it stays in the log). Configurable per market in `locales/<code>/pricing.json → custom_complement_band.min` / `.max` (es default `250` / `5000`; code defaults in `fill_spreadsheet.DEFAULT_BAND_MIN/MAX` if a pack omits them). The band is a **policy parameter**, so it lives in its own key next to `complement_prices`, never inside it: `complement_prices` is the registry of real complements and `validate_locale.py` warns about any id there that is not a first-class complement. |
| **Substring collision** | Add-on matching is by **substring**, so a new name may not equal, contain, or be contained in any existing complement or base-feature name of that app (nor in another new complement of the same run) → error telling the operator to rename. |
| **Slot exhaustion** | More complements than free slots → **error**, never a silent drop. Regenerate the template with `--free-slots N` or retrofit (below). |
| **Inactive app** | `extras_custom` on an app with `activa: false` → error (the complement would vanish when the section is cleaned). |
| **Non-Spanish key** | Flagged with a `WARNING` on stderr (heuristic: it cannot be authoritative — real keys carry anglicisms such as `Lead scoring`, `Dropshipping`, `SMS marketing`). |

### Iterating a deal — the fill is IDEMPOTENT

STEP 3 downloads the deal's **current** `spreadsheet_data`, fills it and uploads it again, so from the
second iteration onwards the complement is **already inside the workbook**. `fill_spreadsheet` handles
that by marking the name cell of a custom row with its own style (`CUSTOM_NAME_STYLE`, compared by
resolved **props** and not by style id — o-spreadsheet remaps ids on export):

| Iterating | Behaviour |
|---|---|
| Same complement again | Its own row is **reclaimed by name** and rewritten in place. It never collides with itself, the slot is not consumed twice, and re-running with the same budget yields a **byte-identical** workbook and identical `order_totals.py` output — indefinitely, not just twice. |
| Price or quantity changed | Same row, recomputed from the sheet's **current** B5 rate. |
| Complement **dropped** from the budget | The row is **cleared** (name, quantity and unit price gone, the `=Ax*Cx` formula left intact) and the slot returns to free — byte-identical to a freshly generated one. The extras subtotal, the app total, the 10% go-live support and the grand total all drop accordingly. A complement the rep deleted must not survive in the workbook. |
| Renamed | The old row is recovered as a free slot and reused by the new name. |

The guards are **not** relaxed to achieve this: only a name that matches **exactly** (normalized) claims
an existing row. A genuinely different name that collides by substring with a catalogue extra, a base
feature or another custom row still errors, on the second pass as on the first. Slot exhaustion counts
free + recovered + own rows, so 6 complements against 5 slots still errors and iteration 2 never
exhausts spuriously.

Every failure **raises `ComplementError`, exits non-zero and writes no output file** — a complement is
never silently dropped or zeroed.

### Retrofit of already-deployed workbooks

`python3 scripts/insert_free_slots.py <workbook.json> <out.json> --locale <code> [--free-slots 5]
[--dry-run]` — offline, idempotent, style-preserving, **no Odoo contact** (a person redeploys).

In a deployed workbook the app blocks are compact (only ONE blank row between the last extra and
`Subtotal Extras`), and shifting rows is forbidden, so the script **rebuilds each app sheet with the
template builder** and swaps it in, keeping the sheet name, its id and its app order, then rewrites
the cross-sheet refs in `Interna Comercial`. Consequence: manual edits inside the app sheets are
regenerated from the pack — this is a retrofit for **base** workbooks, not for a deal already worked
on. The manifest lists every base price that changes.

### Promotion to the fixed catalogue

Every invented complement is recorded in a JSON registry (`$QB_COMPLEMENT_REGISTRY`, default
`~/.dynapps/qb_complement_candidates.json`; `--registry` / `--deal-ref` on `fill_spreadsheet.py`).

```
python3 scripts/complement_candidates.py report [--min-deals 2] [--json]
```

surfaces the complements seen in **more than one deal** as candidates for promotion into the fixed
catalogue, with their price spread. **Proposal only** — the script never promotes anything and never
edits a pack; a person decides name, price, detection signals and translations.
