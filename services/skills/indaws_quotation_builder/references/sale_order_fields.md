# sale.order Fields for an Implementation Quotation

Complete reference for the models, fields and relations of the `sale.order` that get written when
the quotation lands in Odoo.

> **IMPORTANT**: only fields of the new quotation version (`x_studio_new = True`) are documented.
> The legacy fields are excluded.
>
> **Language**: field *values* that the client reads (`x_studio_resumen`, `x_studio_concepto`,
> pain points, `x_studio_toma_de_datos`) are written in the client's language — default Spanish.
> Field *names* and selection *keys* are technical identifiers and are never translated. The
> `context: {"lang": "es_ES"}` on product searches is a database requirement, not a language
> choice.

---

## Relationship Diagram

```
sale.order (Quotation)
├── x_studio_fases_proyecto ──(O2M)──> x_sale_order_line_864da (Phases)
│   └── x_studio_concepts_ids ──(O2M)──> x_sale_order_line_36b23 (Concepts)
├── x_studio_conceptos_proyecto ──(O2M)──> x_sale_order_line_36b23 (Concepts)
│   └── x_studio_fase_id ──(M2O)──> x_sale_order_line_864da (Phase)
├── x_studio_painpoint_ids ──(O2M)──> x_sale_order_line_3c439 (Pain Points)
├── x_studio_services_ids ──(O2M)──> x_sale_order_services (Apps/Services)
│   └── x_studio_product_id ──(M2O)──> product.template
├── x_studio_indaws_core_ids ──(O2M)──> x_sale_order_line_04801 (Indaws Core)
│   └── x_studio_product_id ──(M2O)──> product.template
├── x_studio_experts_ids ──(M2M)──> x_hr_employee_line_1e36b (Experts)
│   ├── x_hr_employee_id ──(M2O)──> hr.employee
│   └── x_studio_product_id ──(M2O)──> product.product (Area/Role)
├── x_studio_success_cases_ids ──(M2M)──> blog.post (Success cases)
├── order_line ──(O2M)──> sale.order.line (Standard lines)
└── sale_order_template_id ──(M2O)──> sale.order.template (Template)
```

---

## One2many Models

| sale.order field | Child model | Link field back to sale.order |
|---|---|---|
| `x_studio_services_ids` | `x_sale_order_services` | `x_studio_order_id` |
| `x_studio_painpoint_ids` | `x_sale_order_line_3c439` | `x_sale_order_id` |
| `x_studio_fases_proyecto` | `x_sale_order_line_864da` | `x_sale_order_id` |
| `x_studio_conceptos_proyecto` | `x_sale_order_line_36b23` | `x_sale_order_id` |
| `x_studio_indaws_core_ids` | `x_sale_order_line_04801` | `x_sale_order_id` |

---

## 1. sale.order (Quotation)

### Configuration fields

| Field | Type | Description | Values / notes |
|-------|------|-------------|----------------|
| `name` | char | Quotation reference | e.g. "PR3488" |
| `x_studio_new` | boolean | Enable the new quotation version | **Always True** |
| `sale_order_template_id` | many2one → sale.order.template | Template | **Always ID 1** ("Propuesta de Implantación Odoo") |
| `opportunity_id` | many2one → crm.lead | CRM opportunity | Required |

### Project tab

| Field | Type | Description |
|-------|------|-------------|
| `x_studio_resumen` | html | Company/project summary. Bootstrap 5. |
| `x_studio_services_ids` | one2many → x_sale_order_services | Odoo applications included |
| `x_studio_demo_url` | char (url) | Link to the demo recording |
| `x_studio_success_cases_ids` | many2many → blog.post | Success cases (blog_id = 16) |
| `x_studio_experts_ids` | many2many → x_hr_employee_line_1e36b | Expert team |
| `x_studio_painpoint_ids` | one2many → x_sale_order_line_3c439 | Client pain points |

### Proposal tab

| Field | Type | Description |
|-------|------|-------------|
| `x_studio_fases_proyecto` | one2many → x_sale_order_line_864da | Project phases |
| `x_studio_conceptos_proyecto` | one2many → x_sale_order_line_36b23 | Concepts broken down per phase |
| `x_studio_indaws_core_ids` | one2many → x_sale_order_line_04801 | Indaws Core products |
| `x_studio_server_type` | selection | Server type |
| `x_studio_server_cost` | monetary | Server cost (visible when indaws-cloud) |
| `x_studio_maintenance_amount` | monetary | Monthly maintenance amount |
| `x_studio_license_amount` | monetary | Monthly licence amount |
| `x_studio_migration` | char | Future migration estimate (e.g. "3750-5000") |
| `x_studio_toma_de_datos` | html | Client-facing structured data collection (Step 10.5) |

#### x_studio_server_type values

| Value | Label |
|-------|-------|
| `indaws-cloud` | Indaws Cloud |
| `odoo-online` | Odoo Online |
| `odoo-sh` | Odoo.sh |
| `on-premise` | On Premise |

---

## 2. x_sale_order_line_864da (Project Phases)

The quotation's phases. Each phase has a type, a duration and an hourly rate.

| Field | Type | Description | Notes |
|-------|------|-------------|-------|
| `x_name` | char | Phase name | e.g. "Fase 1: Análisis inicial" |
| `x_sale_order_id` | many2one → sale.order | Parent quotation | |
| `x_studio_sequence` | integer | Order | |
| `x_studio_tipo` | selection | Phase type | `Analisis` / `Implantacion` |
| `x_studio_duration` | char | Estimated duration | e.g. "3-4 semanas (1 mes)" |
| `x_studio_horas` | float | Total hours (computed) | Sum of its concepts |
| `x_studio_precio_hora` | float | Hourly rate | Analysis €80/h, Implementation €75/h |
| `x_studio_precio` | float | Total price (computed) | Sum of its concepts |
| `x_studio_concepts_ids` | one2many → x_sale_order_line_36b23 | This phase's concepts | |

---

## 3. x_sale_order_line_36b23 (Project Concepts)

Detail lines within each phase. Each concept carries hours and a price.

| Field | Type | Description | Notes |
|-------|------|-------------|-------|
| `x_name` | char | Short concept name | **REQUIRED** |
| `x_studio_concepto` | html | Concept description | **REQUIRED** — `<p><strong>…</strong></p><ul>…</ul>` |
| `x_sale_order_id` | many2one → sale.order | Parent quotation | |
| `x_studio_sequence` | integer | Order within the phase | |
| `x_studio_fase_id` | many2one → x_sale_order_line_864da | Parent phase | Domain: x_sale_order_id |
| `x_studio_tipo` | selection | Concept type | `Funcional` / `Desarrollo` |
| `x_studio_horas` | float | Concept hours | |
| `x_studio_precio` | float | Concept price | hours × the phase's hourly rate |

> **CRITICAL**: every concept MUST carry both `x_name` (char) and `x_studio_concepto` (HTML). If
> `x_studio_concepto` is left empty, the concept renders with no text in the Odoo interface.

---

## 4. x_sale_order_line_3c439 (Pain Points)

The client's pain points with the proposed solution.

| Field | Type | Description |
|-------|------|-------------|
| `x_name` | char | Pain point name |
| `x_sale_order_id` | many2one → sale.order | Parent quotation |
| `x_studio_sequence` | integer | Order |
| `x_studio_description` | char | Description of the problem |
| `x_studio_solution` | char | Solution name |
| `x_studio_solution_description` | char | Description of the solution |

Rules: exactly 3 pain points, drawn from the collected data (real problems), written in
business-oriented language and in an empathetic register — challenges and areas for improvement,
never failures or accusations.

---

## 5. x_sale_order_services (Apps/Services)

The Odoo applications included in the quotation.

| Field | Type | Description | Notes |
|-------|------|-------------|-------|
| `x_name` | char | Service name | e.g. "Ventas" |
| `x_studio_order_id` | many2one → sale.order | Parent quotation | |
| `x_studio_sequence` | integer | Order | |
| `x_studio_product_id` | many2one → product.template | The app's product | Category "Plantilla presupuestos" |
| `x_studio_incluido_presupuesto` | boolean | Included in the quotation | Always True for active apps |
| `x_studio_descripcion_larga` | html | Long description of the app | |
| `x_studio_precio` | float | Price | |
| `x_studio_num_semanas` | float | Number of weeks | |

---

## 6. x_sale_order_line_04801 (Indaws Core)

Indaws Core products included in the quotation.

| Field | Type | Description | Notes |
|-------|------|-------------|-------|
| `x_name` | char | Module name | **REQUIRED** (NOT NULL) |
| `x_sale_order_id` | many2one → sale.order | Parent quotation | |
| `x_studio_sequence` | integer | Order | |
| `x_studio_product_id` | many2one → product.template | Product | Category ID 5 |
| `x_studio_price` | float | Price | From the product catalogue |

---

## 7. x_hr_employee_line_1e36b (Expert Team)

Consultants assigned to the project. Shown on the client portal.

| Field | Type | Description |
|-------|------|-------------|
| `x_name` | char | Name with area, e.g. "Jesús Lloret (Fabricación)" |
| `x_hr_employee_id` | many2one → hr.employee | Employee |
| `x_studio_sequence` | integer | Order |
| `x_studio_product_id` | many2one → product.product | The expert's area/role |
| `x_studio_description` | char | Manual description |
| `x_studio_ai_description` | char | AI-generated description |

To find available experts:

```bash
search_records.sh x_hr_employee_line_1e36b '[]' '["id","x_name","x_hr_employee_id","x_studio_product_id","x_studio_ai_description"]' 200
```

> Deduplicate by `x_hr_employee_id` — one employee may hold several records, one per skill. Keep
> the one whose area is most relevant. Maximum 5 experts.

---

## 8. blog.post (Success Cases)

Corporate blog articles used as success cases. Filter on `blog_id = 16`.

Relevant fields: `id`, `name`, `subtitle`, `teaser`, `cover_properties`.

```bash
search_records.sh blog.post '[["blog_id","=",16]]' '["id","name","subtitle","teaser"]' 100
```

---

## 9. sale.order.line (Standard Sale Lines)

### Fields

| Field | Type | Description |
|-------|------|-------------|
| `product_id` | many2one → product.product | **REQUIRED** — product (the variant, NOT the template) |
| `name` | text | **REQUIRED** — line description |
| `product_uom_qty` | float | **REQUIRED** — quantity (always 1) |
| `price_unit` | float | **REQUIRED** — unit price = the phase's total price |
| `product_uom_id` | many2one → uom.uom | The product's unit of measure |
| `tax_ids` | many2many → account.tax | Taxes. Syntax: `[[6, 0, [<ids>]]]`. Tax ID 2 = 21% VAT Services |

### Logic by quotation type

**With Rescue Pack** (client already runs Odoo): a line with product ID 330 ("Paquete de
rescate"), quantity 1, the product-record price.

**Without Rescue Pack — analysis only**: one line with the product "Análisis de proyecto"
(template ID 124), quantity 1, price = the analysis phase's `x_studio_precio`. Do not include
other phases or Core.

**Without Rescue Pack — full project**: one line per phase with its matching product and price
(each phase's `x_studio_precio`), plus one line per Core package with its product and price.

### Reference products

| Product | product.template ID | product.product ID | UoM | Tax |
|---|---|---|---|---|
| Implantación ERP Odoo | 25 | 25 | Ud (6) | 21% VAT Services (2) |
| Análisis de proyecto | 124 | 124 | Units (1) | 21% VAT Services (2) |
| Paquete de rescate (Rescue pack) | 330 | 330 | — | — |
| INBUILT (construction vertical) | 228 | 228 | — | — |

**IMPORTANT**: `sale.order.line.product_id` is Many2one → `product.product` (NOT
product.template). For products without variants the IDs usually coincide, but always verify by
searching `product.product` on `product_tmpl_id`.

---

## 10. Standard Prices

| Item | Price |
|------|-------|
| Analysis hour | €80/h |
| Implementation hour | €75/h |
| Odoo Enterprise licence | €27.90/month (reference, Custom yearly, list price, Spain) |
| Indaws Cloud server | €1,080/year |
| Rescue Pack | €995 |

### Maintenance — tiered (DEFAULT)

| Project total | Monthly amount |
|---|---|
| < €12,000 | €250/month |
| €12,000 – 30,000 | €350/month |
| > €30,000 | €460/month |

### Maintenance — scalable (OVERRIDE)

Only when the consultant explicitly asks for it:
- Base: €320/month with 5h included
- Additional hour: €64/h
- Formula: `320 + (extra_hours × 64)`

### Product categories

| Category | ID | Description |
|----------|----|-------------|
| Indaws Core | 5 | In-house horizontal modules |
| inBUILT | 6 | Construction vertical |

---

## Operation Sequence — 5 stages: A → B → C → D → E

### Stage A: scalar fields + One2many without dependencies

```bash
update_record.sh sale.order '[<id>]' '{
  "sale_order_template_id": <template_id>,
  "x_studio_new": true,
  "x_studio_resumen": "<p>...</p>",
  "x_studio_server_type": "<hosting>",
  "x_studio_server_cost": <server_cost>,
  "x_studio_maintenance_amount": <250|350|460>,
  "x_studio_license_amount": <monthly_licence>,
  "x_studio_migration": "<range>",
  "x_studio_demo_url": "<demo_url>",
  "x_studio_services_ids": [[0,0,{...}], ...],
  "x_studio_painpoint_ids": [[0,0,{...}], ...],
  "x_studio_fases_proyecto": [[0,0,{...}], ...],
  "x_studio_indaws_core_ids": [[0,0,{...}], ...],
  "x_studio_experts_ids": [[6, 0, [<expert_ids>]]],
  "x_studio_success_cases_ids": [[6, 0, [<blog_post_ids>]]]
}'
```

### Stage B: get the IDs of the phases just created

```bash
search_records.sh x_sale_order_line_864da '[["x_sale_order_id","=",<sale_order_id>]]' '["id","x_name","x_studio_tipo"]'
```

Identify each phase's ID (Rescue Pack, Analysis, Implementation).

### Stage C: create the concepts with their phase IDs

```bash
update_record.sh sale.order '[<id>]' '{
  "x_studio_conceptos_proyecto": [[0,0,{"x_studio_fase_id":<phase_id>,"x_name":"...","x_studio_concepto":"<p>...</p>",...}], ...]
}'
```

**REMINDER**: every concept MUST carry both `x_name` and `x_studio_concepto` (HTML).

### Stage D: create the order lines (sale.order.line)

Logic by scenario:

**With Rescue Pack**: a line with product 330, qty 1, the product-record price.

**Analysis only**: one line with "Análisis de proyecto" (ID 124), qty 1, price = the analysis
phase total.

**Full project**: one line per phase + one line per Core package.

```json
[0, 0, {
  "product_id": "<product_product_id>",
  "name": "<product name>",
  "product_uom_qty": 1,
  "price_unit": "<total price of the phase or package>",
  "product_uom_id": "<the product's uom_id>",
  "tax_ids": [[6, 0, [2]]]
}]
```

For Core packages, find the `product.product` from the template:

```bash
search_records.sh product.product '[["product_tmpl_id","in",[<core_template_ids>]]]' '["id","name","product_tmpl_id","uom_id"]' 100
```

**IMPORTANT**: if order lines already exist, delete them first with `[2, <id>, 0]` or ask the
user. Appending silently double-counts the project.

### Stage E: write "Toma de Datos" as HTML

Write the rewritten, structured data collection into `x_studio_toma_de_datos` (HTML) on the
sale.order, in the client's language.

**Content** (5 sections):

1. **General information**: company data, sector, users, locations
2. **Project objectives**: motivation, pain points, phased approach
3. **Functional scope**: detail per module/app with key decisions and requirements
4. **Customisations and developments**: integrations, developments identified
5. **Data migration**: data to migrate, format, volume, source system

**Exclude**: commercial information, additional internal notes, unresolved `[ASK]` fields, source
references, contact details.

**Format**: clean HTML with no inline styles. Semantic tags: `<h2>`, `<h3>`, `<table>`,
`<ul>/<li>`, `<p>`, `<strong>`.

```bash
update_record.sh sale.order '[<sale_order_id>]' '{"x_studio_toma_de_datos": "<html_content>"}'
```

---

## Mandatory Prior Searches

**CRITICAL — LANGUAGE**: ALWAYS pass `context: {"lang": "es_ES"}` on product searches.

Real example of the failure:

- product.template ID 119: base name = "Compras (copia)" → in Spanish = "Inventario"
- product.template ID 120: base name = "Inventario (copia)" → in Spanish = "Gestión de proyectos"

Without the Spanish context, mapping by name binds the wrong products.

### Searches needed

1. **Quotation template**: `search_records.sh sale.order.template '[["name","ilike","Propuesta de Implantación Odoo"]]' '["id","name"]' 1`
2. **App products** (`x_studio_services_ids`): `product.template` with `[["categ_id.name","ilike","Plantilla Presupuestos"]]` and context `{"lang": "es_ES"}`
3. **Core products** (`x_studio_indaws_core_ids`): `product.template` with `[["categ_id.name","ilike","CORE"]]` and context `{"lang": "es_ES"}`
4. **Order-line products**: `product.template` "Implantación ERP" and "Análisis de proyecto" with context `{"lang": "es_ES"}`, then `product.product` via `product_tmpl_id`
5. **Rescue Pack product**: `search_records.sh product.product '[["id","=",330]]' '["id","name","list_price"]' 1`
6. **Experts**: `search_records.sh x_hr_employee_line_1e36b '[]' '["id","x_name","x_hr_employee_id","x_studio_product_id","x_studio_ai_description"]' 200`
7. **Success cases**: `search_records.sh blog.post '[["blog_id","=",16]]' '["id","name","subtitle","teaser"]' 100`

### COMMON ERRORS

1. Searching `product.product` instead of `product.template` for `x_studio` fields → wrong IDs
2. Searching without `context: {"lang": "es_ES"}` → base-language names do not match → wrong mapping
3. Using a `product.template` ID in `sale.order.line.product_id` → the field expects a `product.product` ID
4. Forgetting `x_name` on Core packages → NOT NULL constraint error
5. Forgetting `x_studio_concepto` on concepts → a concept with no text in the interface
6. Appending order lines to a quotation that already has them → the project is double-counted
