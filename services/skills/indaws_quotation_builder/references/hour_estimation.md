# Implementation Hour Estimation (GOVERNING MODEL)

Estimation system based on weekly velocities and duration per application.

> This is the **governing** estimation model for this skill. `timeline_rules.md` and
> `classification_rules.md` describe an older model kept for legacy quotations. Never mix the two
> within one quotation (SKILL.md `## Critical` 4).

---

## Principle: Two Parallel Tracks

Every implementation project has two tracks running in parallel:

| Track | What it covers | Velocity | Who |
|-------|----------------|----------|-----|
| **Functional** | Configuration, implementation, meetings, training, migration, support | 12h/week | Consulting team |
| **Development** | Custom developments, integrations, Studio changes | 25h/week | Development team |

**Fundamental rule**: the project's total duration is the maximum of the two tracks. Functional
implementation cannot finish before the developments do.

---

## Reference Velocities

### Functional: 12h/week
A sustainable pace for the consulting team with one client. Includes working sessions with the
client, configuration between sessions, preparation and follow-up.

### Development: 25h/week
The pace of a developer dedicated to the project. Includes coding, internal testing, reviews and
fixes.

### Project management: 1h/week
Always fixed, from kick-off to go-live.

---

## Duration Criteria per Application

Implementation is planned application by application, **sequentially** by default.

### Initial Configuration
**Always 1 week (10h)** — companies, users, languages, currencies; taxes, sequences, base master
data; technical environment setup.

### CRM
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic CRM: standard pipeline, no automation, no developments |
| 2 | CRM with assignment rules, sales teams, or some automation |
| 3 | Complex CRM: multiple pipelines, integrations, associated developments |

**Complexity signals**: multiple sales teams, automatic assignment rules, web/form integration,
lead scoring, custom developments.

### Sales
| Weeks | Criterion |
|-------|-----------|
| 2 | Standard sales: quotations, orders, basic price lists |
| 3 | Complex pricing, multiple price lists, volume discounts, commissions |
| 4 | Multi-company sales, intercompany, complex approval processes, EDI |

**Complexity signals**: multiple price lists, tiered discounts, approval process, intercompany
sales, customer portal, special order types.

### Purchase
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic purchasing: simple orders, few suppliers |
| 2 | Purchase agreements, multiple suppliers per product, reordering rules |
| 3 | Purchasing with approval processes, intercompany purchasing, supplier EDI |

**Complexity signals**: framework agreements, automatic reordering rules, multi-level approvals,
dropshipping, subcontracting.

### Inventory
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic inventory: 1 warehouse, simple receipts/deliveries, no traceability |
| 2 | Multi-warehouse OR traceability (lots/serials) OR multi-step operations |
| 3 | Advanced picking, barcodes, complex routes, cross-docking |
| 4 | All of the above + multiple internal locations, complex replenishment rules |

**Complexity signals**: more than 1 warehouse, lot/serial traceability, picking operations,
scanner/barcode use, custom warehouse routes.

### Manufacturing
| Weeks | Criterion |
|-------|-----------|
| 2 | Basic manufacturing: simple BOMs, manual orders |
| 3 | MRP planning, work centres, basic quality control |
| 4 | Subcontracting, advanced routings, capacity planning |
| 5–6 | Full MRP II, multiple centres, integrated maintenance, PLM |

**Complexity signals**: multi-level BOMs, subcontracting operations, work centres with capacity,
quality control, capacity planning, maintenance.

### Accounting
| Weeks | Criterion |
|-------|-----------|
| 2 | Standard accounting: invoicing, receipts/payments, basic chart of accounts |
| 3 | Multi-company, fixed assets, advanced analytic accounting |
| 4 | Consolidation, complex localization, advanced bank integration |

**Complexity signals**: multi-company, fixed assets, budgets, advanced bank reconciliation,
multiple journals, multi-axis analytic accounting.

### Project
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic project management: tasks, deadlines, team |
| 2 | Timesheets, per-project invoicing, advanced planning |

### HR
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic HR: employees, departments, time off |
| 2 | Payroll, expenses, appraisals, recruitment |
| 3 | Full HR: all of the above + fleet + approvals |

### Helpdesk
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic helpdesk: tickets, teams, simple SLA |
| 2 | Multiple teams, complex SLAs, customer portal, knowledge base |

### Website / eCommerce
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic corporate site with Odoo Website |
| 2 | Simple eCommerce: catalogue, cart, payment gateway |
| 3 | Advanced eCommerce: multiple web price lists, variants, marketplace, shipping |

### Point of Sale
| Weeks | Criterion |
|-------|-----------|
| 1 | Basic POS: 1–2 terminals, simple products |
| 2 | Multiple stores, loyalty programmes, restaurant |

### Integrations
**Always at least 1 week per integration.**
- Simple integrations (standard REST API): 1 week
- Complex integrations (EDI, two-way sync): 2 weeks
- Legacy systems or proprietary formats: 2–3 weeks

### Simple Apps (1 week each)
Contacts (as a standalone implementation), Calendar, Notes, Discuss, Approvals, Surveys, Sign,
Documents.

---

## Estimation Algorithm

### Step 1: Functional duration

```
1. For each app in the project:
   - Assess complexity against the criteria above
   - Assign implementation weeks

2. Sum the weeks (sequential by default):
   total_functional_weeks = initial_config + sum(weeks_per_app) + golive_support

3. Functional hours:
   functional_hours = total_functional_weeks × 12h/week
```

### Step 2: Development duration

```
1. Sum every development's hours (testing included):
   total_development_hours = sum(development_hours_i)

2. Development weeks:
   development_weeks = ceil(total_development_hours / 25)
```

### Step 3: Project duration

```
project_duration_weeks = max(total_functional_weeks, development_weeks)
```

If the development weeks exceed the functional ones, the functional implementation has to be
extended (add support/accompaniment).

### Step 4: Project management

```
project_management_hours = project_duration_weeks × 1h/week
```

### Step 5: Go-live support

**2 weeks** (24h) for standard projects, **3–4 weeks** (36–48h) for complex ones.

### Step 6: Split the functional hours into concepts

The total functional hours (excluding project management and developments) split as:

```
implementation_hours = functional_hours × 0.70
meetings_training_hours = functional_hours × 0.30
```

**Note**: data migration sits inside the implementation hours.

---

## Presenting the Project Calendar

Before generating the quotation, present an **implementation calendar** to the user, in their
language.

### Per-app planning table

```
| Week   | Functional track (12h/wk) | Development track (25h/wk) |
|--------|---------------------------|----------------------------|
| W1     | Initial configuration     | Development D1, D2…        |
| W2     | CRM                       | (continues)                |
| W3-W4  | Sales                     | Development D3…            |
| W5-W6  | Inventory                 | (continues)                |
| ...    | ...                       | ...                        |
| WN-1   | Data migration            | Final testing              |
| WN     | Go-live support           | —                          |
```

### Hour summary

```
FUNCTIONAL TRACK:
- Initial configuration:      10h (1 wk)
- [App 1]:                    XXh (N wk)
- [App 2]:                    XXh (N wk)
- ...
- Go-live support:            24h (2 wk)
- FUNCTIONAL SUBTOTAL:        XXXh

  -> 70% Implementation:      XXXh
  -> 30% Meetings/Training:   XXXh

DEVELOPMENT TRACK:
- [Development 1]:            XXh
- [Development 2]:            XXh
- ...
- DEVELOPMENT SUBTOTAL:       XXXh

PROJECT MANAGEMENT:
- 1h/week × N weeks:          Xh

PROJECT TOTAL:                XXXh
```

The user must **validate or adjust** this calendar before anything continues.

**Adjustments that can be offered**:

- **Parallelise apps** to shorten the duration (needs more consultants)
- **Extend the duration** if the client needs more time to absorb the change
- **Reorder apps** by the client's priority

---

## Important Rules

1. **Sequential by default**: apps are implemented one after another. Parallelising to shorten is
   possible but requires the user's agreement.
2. **Development runs in parallel**: developments always run alongside the functional
   implementation.
3. **Implementation >= Development**: the functional implementation cannot be shorter than the
   developments.
4. **Print formats**: NOT a separate concept. Standard formats sit inside implementation; custom
   formats are developments.
5. **Data migration**: sits inside the implementation hours (not a separate concept with its own
   hours).
6. **70/30**: the functional hours (excluding project management and developments) split 70%
   implementation / 30% meetings and training.
7. **Project management**: always a fixed 1h/week, independent of the 70/30 split.
8. **Go-live support**: always included at the end, 2–4 weeks depending on complexity.
