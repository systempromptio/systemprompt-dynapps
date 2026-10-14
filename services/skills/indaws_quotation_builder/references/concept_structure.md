# Concept Structure per Phase

The standard structure of the concepts that make up each phase of an implementation quotation.

> **Language.** Concept titles and bullets are written in the **client's** language (default
> Spanish) — the client reads them in the PDF. The English below is the structural reference.

## Concept HTML Format

Every concept uses the `x_studio_concepto` field (HTML) in this mandatory format:

```html
<p><strong>Concept title</strong></p>
<ul>
<li>Point 1: concrete, project-specific description</li>
<li>Point 2: another task or deliverable included</li>
<li>Point 3: etc.</li>
</ul>
```

**General rules**:

- Bold title inside `<p><strong>…</strong></p>`
- Bullet list with `<ul><li>…</li></ul>` (4–8 bullets per concept)
- Bullets must be **concrete and specific to the client's project**, never generic
- Development concepts must describe what will be built (functionality visible to the user)
- Every concept needs BOTH `x_name` (char) and `x_studio_concepto` (HTML) — an empty HTML renders
  a blank concept in the Odoo UI

**Per-concept rules**:

- **Data migration**: state that the import comes from Excel files. **Always include as the last
  bullet** that exporting data to Excel or CSV from the current system is the client's
  responsibility.
- **Meetings and training**: training is for the **key users** of each department; never say "all
  users" and never state the total user count.
- **Go-live support**: never say "all users" or state the total user count. Support is generic,
  covering the first weeks of real use.

---

## Analysis Phase

An analysis phase has **a single main concept**, whose description names the apps being analysed.

### Structure

| Concept | Type | Hours | Price |
|---------|------|-------|-------|
| Project analysis (detailed description) | Funcional | Calculated | hours × €80/h |

If the analysis is **on-site**, add:

| Concept | Type | Hours | Price |
|---------|------|-------|-------|
| On-site visit travel costs | Funcional | 0 | €320–650 (by distance) |

### Analysis Hour Calculation

**Governing model** — see `hour_estimation.md` and the formulas in `workflow_detail.md`:
- Online: `(n_apps × 1.5h × 2) + (1h × n_developments) + 5h preparation + 4h presentation`
- On-site: `(n_days × 8h × 2 consultants) + (1h × n_developments) + 5h preparation + 4h presentation`

**Legacy model** — flat hours by classification (`classification_rules.md`): Simple **25h**,
Medium **40h**, Complex **60h**. Only for quotations already built that way.

---

## Phase 0: Rescue Pack (optional)

Applies only when the client **already runs Odoo** from another partner. Duration: 1 week.

> **BLOCKING prerequisite:** requires access to **the source code** (git, zip, shared folder or
> another route) **AND to the database** (dump/backup or direct access). The Rescue Pack is the
> code audit plus the X-ray of the real database; without **both** it cannot be delivered. Confirm
> this before quoting it.

| Concept | Type | Hours | Price |
|---------|------|-------|-------|
| Rescue pack | Funcional | 0 | €995 (price of product ID 330) |

> When the Rescue Pack is included, also add a `sale.order.line` with product ID 330, quantity 1
> and the product-record price.

> **Client currently on Odoo.sh:** the Rescue Pack + migration to Indaws/Dynapps servers has two
> objective urgency levers (source of truth:
> `references/shared/odoo-lifecycle-licensing.md`): (1) their version leaves Odoo.sh on a
> fixed date (a hard deadline) and (2) a +25% surcharge on Enterprise renewal for versions outside
> the top three (official Odoo policy; confirm the client's renewal date). Use both to justify the
> Rescue Pack and the self-hosting proposal.

---

## Implementation Phase

The standard concept structure for an implementation phase. All concepts are type `Funcional`
except developments.

### Standard Concepts (in this order)

| # | Concept | Type | Description |
|---|---------|------|-------------|
| 1 | Technical configuration | Funcional | Initial setup: companies, users, languages, currencies, taxes, sequences |
| 2 | Internal project management | Funcional | 1h/week of coordination and follow-up through to go-live |
| 3 | Data migration | Funcional | Import of master data from Excel files: customers, products, chart of accounts, opening stock. Exporting the data is the client's responsibility |
| 4 | Module implementation | Funcional | Configuration, functional parameterisation and standard formats of the apps |
| 5 | Meetings and training | Funcional | Sessions with the client: key-user training, process validation, follow-up meetings |
| 6 | Go-live support | Funcional | Accompaniment through go-live and the first weeks of real use |

### Optional Concepts

| Concept | Type | When to include |
|---------|------|-----------------|
| Custom development: [name] | **Desarrollo** | One concept per specific development |
| Odoo Studio changes | **Desarrollo** | A single line grouping every Studio change |
| Odoo vXX → vYY migration | Funcional | If the client is migrating from an earlier version |

### Hour Distribution (70/30 split)

Functional hours (from the weekly-velocity model, see `hour_estimation.md`) distribute as:

| Concept | Calculation |
|---------|-------------|
| Technical configuration | 10h (fixed, 1 week) |
| Project management | 1h × total project weeks |
| Data migration | Included in the implementation hours |
| **Module implementation** | **70% of the functional hours** (excl. config, management, support) |
| **Meetings and training** | **30% of the functional hours** (excl. config, management, support) |
| Go-live support | 24–48h by complexity (2–4 weeks) |
| Developments | Own hours, outside the 70/30 split. **They arrive with the AI factor already applied from the technical estimate: do not re-apply it here.** Functional lines (implementation, meetings, training, management) carry no factor. Criteria: delegate to the `odoo-effort-estimator` skill |

**Note**: "Print formats" is NOT a separate concept. Standard formats sit inside "Module
implementation". Bespoke custom formats are a `Desarrollo` concept.

---

## Multiple Analysis Phases

On large projects, analysis phases are interleaved with implementations:

```
Phase 1: Initial analysis
Phase 2: Implementation (main apps)
Phase 3: Analysis II (secondary apps)
Phase 4: Implementation (secondary apps)
Phase 5: Analysis III (if needed)
Phase 6: Implementation (remaining modules)
```

Each additional analysis phase covers the apps of the following implementation phase.

---

## Concept Price Calculation

- `concept_price = hours × phase_hourly_rate`
- Analysis: €80/h
- Implementation: €75/h
- A phase's total price is the sum of its concepts' prices

---

## Example: project with 3 apps + developments (12-week duration)

**Functional calendar**: Config (1 wk) + Sales (2 wk) + Inventory (2 wk) + Accounting (2 wk) +
Migration (1 wk) + Support (2 wk) = 10 functional weeks
**Developments**: 200h / 25h per week = 8 weeks → project = max(10, 8) = 10 weeks

**Functional hours** = 10 weeks × 12h/week = 120h
- Initial configuration: 10h (fixed)
- Go-live support: 24h (fixed)
- Hours to distribute: 120h − 10h − 24h = 86h
  - Implementation (70%): 60h
  - Meetings and training (30%): 26h

| Seq | Concept | Hours | Price (€75/h) |
|-----|---------|-------|---------------|
| 0 | Technical configuration | 10 | €750 |
| 1 | Internal project management | 10 | €750 |
| 2 | Data migration | 10 | €750 |
| 3 | Module implementation | 50 | €3,750 |
| 4 | Meetings and training | 26 | €1,950 |
| 5 | Development: [name 1] | 120 | €9,000 |
| 6 | Development: [name 2] | 80 | €6,000 |
| 7 | Go-live support | 24 | €1,800 |
| **TOTAL** | | **330** | **€24,750** |

### Example concept HTML (bullet format)

Shown here in Spanish, because that is the language the client reads:

```html
<!-- Functional concept: implementation -->
<p><strong>Implantación de módulos</strong></p>
<ul>
<li>Ventas: pedidos, tarifas por cliente, descuentos en cascada</li>
<li>Inventario: almacenes, ubicaciones, trazabilidad por lotes</li>
<li>Contabilidad: plan contable adaptado, conciliación bancaria, cobros/pagos</li>
<li>Formatos de impresión estándar de todos los documentos</li>
</ul>

<!-- Functional concept: migration (always ends with the client's responsibility) -->
<p><strong>Migración de datos</strong></p>
<ul>
<li>Preparación de plantillas de importación en formato Excel</li>
<li>Importación de clientes, productos, plan contable</li>
<li>Importación de tarifas de venta y compra</li>
<li>Validación y conciliación de datos migrados</li>
<li>La exportación de datos a ficheros Excel o CSV desde el sistema actual es responsabilidad del cliente</li>
</ul>

<!-- Functional concept: training (always "key users", never all of them) -->
<p><strong>Reuniones y formación</strong></p>
<ul>
<li>Formación a los usuarios clave de cada departamento</li>
<li>Validación de procesos con los responsables de cada área</li>
<li>Reuniones de seguimiento semanal con el equipo del proyecto</li>
<li>Formaciones de refuerzo previas a la salida a producción</li>
</ul>

<!-- Development concept -->
<p><strong>Desarrollo: Integración con [sistema externo]</strong></p>
<ul>
<li>Envío automático de pedidos de compra al sistema externo</li>
<li>Recepción de albaranes de entrada en Odoo</li>
<li>Sincronización bidireccional de estados</li>
<li>Comunicación mediante ficheros/API</li>
</ul>
```
