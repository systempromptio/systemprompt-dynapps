# Implementation Hours per Odoo Application

Reference table of estimated pure implementation hours per Odoo application, segmented by the
project's complexity level.

> This table belongs to the **legacy** classification model (Simple/Medium/Complex, see
> `classification_rules.md`). The governing model estimates in weeks per app — see
> `hour_estimation.md`. Use one model per quotation (SKILL.md `## Critical` 4); this table is the
> app catalogue for either, but its per-level hours only apply to the legacy model.

---

## What These Hours Include

- Functional configuration of the application
- Basic testing of the configured flows
- Functional adjustments after client feedback
- Resolution of configuration issues

## What They Do NOT Include

These are calculated separately:

- Initial configuration (a fixed line)
- Meetings and training with the client
- Data migration
- Custom developments (>8h get their own line)
- Go-live support
- Internal project management

---

## Hours per Application

| Application | Simple | Medium | Complex | Notes |
|---|---|---|---|---|
| CRM | 8 | 14 | 24 | Custom pipeline, sales teams, automation, lead scoring |
| Sales | 10 | 18 | 30 | Product catalogue, multiple price lists, quotation templates, approvals |
| Purchase | 8 | 14 | 24 | Purchase flow, supplier management, approvals, framework agreements |
| Inventory | 12 | 24 | 48 | Warehouses, routes, reordering rules, traceability, valuation |
| Invoicing | 8 | 14 | 22 | Invoicing flow, payment methods, reminders, reports |
| Accounting | 16 | 28 | 44 | Chart of accounts, taxes, tax reporting, SEPA, bank reconciliation, closings |
| HR | 8 | 16 | 28 | Employees, contracts, time off, appraisals, payroll |
| Manufacturing | — | 32 | 64 | BOMs, manufacturing orders, work centres, planning, quality |
| Project | 6 | 12 | 22 | Project types, stages, timesheets, per-project invoicing |
| Helpdesk | 6 | 10 | 18 | Support teams, SLAs, ticket types, customer portal |
| Point of Sale | 8 | 16 | 28 | Terminals, payment methods, products, sessions, reports |
| eCommerce | 12 | 24 | 44 | Web catalogue, payment gateways, shipping methods, shop design |
| Website | 8 | 16 | 28 | Pages, menus, design, forms, basic SEO |
| Field Service | 6 | 12 | 20 | Worksheets, field planning, materials |
| Sign | 3 | 5 | 8 | Signature templates, signer roles, flows |
| eLearning | 4 | 8 | 14 | Courses, multimedia content, certifications, assessments |
| Fleet | 4 | 8 | 12 | Vehicles, services, contracts, costs |
| Email Marketing | 4 | 8 | 14 | Contact lists, email templates, campaigns, metrics |
| Quality | 6 | 10 | 18 | Control points, quality alerts, production checks |
| Planning | 4 | 8 | 14 | Roles, shifts, resource planning, conflicts |
| Mobile app | 4 | 8 | 14 | Configuration, permissions, compatible modules |

---

## Usage Rules

### Fine-adjustment rule

The table's hours are the **base reference**. An app can be moved one level up or down when the
collected data justifies it:

- Data describes an app as **"basic"** inside a Complex project → use the Medium value for that app
- Data describes an app as **"advanced"** inside a Simple project → use the Medium value for that app

**Mandatory**: justify every adjustment when presenting to the user.

**Example**:
```
Inventory: 24h (Medium instead of Complex because the collected data says
"a single warehouse without traceability", even though the project is Complex
because of the user count and advanced manufacturing)
```

### Small-development rule

Custom developments estimated at **<=8h** do NOT get a separate quotation line. They are added to
the corresponding application's hours.

**Example**:
```
Sales: 22h (18h base + 4h development: custom field for minimum margin)
```

Only developments **>8h** get their own line with a descriptive name:
```
Development: Integration with the previous ERP (24h)
Development: Custom supplier portal (16h)
```

### Manufacturing in a Simple project

Manufacturing **does not apply** to Simple projects (marked "—" in the table). If the collected
data mentions manufacturing, the project is automatically Medium or Complex.

---

## Typical Reference Ranges

To sanity-check that the total app implementation is reasonable:

| Project type | App implementation hour range |
|---|---|
| Simple (3–4 apps) | 30–60h |
| Medium (5–7 apps) | 80–160h |
| Complex (7+ apps) | 160–350h |

If the calculated total falls outside these ranges, revisit the adjustments applied.
