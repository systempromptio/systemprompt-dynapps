# Project Classification Rules (LEGACY MODEL)

Algorithm for classifying an Odoo project as **Simple**, **Medium** or **Complex** from the CRM
opportunity's collected data.

> **Legacy model.** This classification sets flat analysis hours (25/40/60h) and feeds
> `timeline_rules.md` and the per-level hours in `app_implementation_hours.md`. The governing model
> for new work is `hour_estimation.md` (weekly velocities). Use this file only when editing a
> quotation already built on the legacy model — never mix the two (SKILL.md `## Critical` 4).

---

## Criteria per Classification

### Simple project (analysis: 25h)

**All** of the following must hold:

- < 10 users
- 1–4 basic applications (CRM, Sales, Invoicing, Purchase, basic Accounting)
- Does NOT include Manufacturing
- Does NOT include Inventory with multi-warehouse, advanced routes or traceability
- No significant custom developments (0–1 minor ones)
- No complex integrations with external systems
- Simple or non-existent migration

### Medium project (analysis: 40h)

Meets **at least one** of the following without reaching Complex:

- 10–50 users
- 4–8 applications
- May include Inventory of moderate complexity (1–2 warehouses, standard routes)
- May include basic Manufacturing (simple BOMs, no advanced planning)
- 1–3 small/medium custom developments
- Occasional integrations (1–2 external systems)
- Moderate data migration (several models from Excel/CSV)

### Complex project (analysis: 60h)

Meets **at least one** of the following:

- \> 50 users
- Manufacturing with multi-level BOMs / advanced planning / complex work centres
- Inventory with multi-warehouse (>2) + complex routes + full traceability
- \> 3 significant custom developments
- Multi-company
- Multiple integrations (>2 external systems)
- Complex migration with accounting and/or operational history
- eCommerce with a large catalogue + gateways + logistics

**Accumulation rule**: two or more Medium-level indicators together escalate automatically to
Complex.

---

## Decision Algorithm

```
1. Any Complex indicator?            -> Complex
2. Two or more Medium indicators?    -> Complex (accumulation rule)
3. Any Medium indicator?             -> Medium
4. ALL Simple criteria met?          -> Simple
5. In doubt                          -> ask the user
```

---

## Key Indicators to Look For in the Collected Data

Search in the source language of the data (usually Spanish); the English glosses below are for
orientation.

### High-complexity indicators
- manufacturing, production, BOM, bill of materials, work centres
- multi-company, several companies, intercompany
- multi-warehouse, several warehouses, traceability, lots, serials
- "integration with", API, gateway, EDI, "connection to"
- full migration, accounting history, "X years of data"
- eCommerce, "online shop with…"

### Medium-complexity indicators
- inventory, warehouse (without multi-warehouse)
- basic manufacturing, simple BOMs
- custom report, custom field, minor development
- "connect to 1 system", "import data from Excel"
- app count between 4 and 8

### Simplicity indicators
- basic, standard, simple, "we only need"
- few apps mentioned (CRM, Sales, Invoicing)
- small company, few users
- "we have no previous system", "we're starting from scratch"

---

## Ambiguous Indicators

If the collected data holds indicators that do not clearly fit one category, **ALWAYS** ask the
user before classifying:

- User count not specified
- Manufacturing mentioned without complexity detail
- "Integrations" mentioned without saying how many or which
- Migration mentioned without detail on the source
- Apps mentioned generically ("all the modules", "whatever we need")

**Question format** (asked in the user's language):

```
I found the following ambiguous indicators:
- [indicator 1]: [why it is ambiguous]
- [indicator 2]: [why it is ambiguous]

My tentative classification is [X]. Can you clarify these points?
```
